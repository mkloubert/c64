// Cobra64 - A concept for a modern Python-like compiler creating C64 binaries
// Copyright (C) 2026  Marcel Joachim Kloubert <marcel@kloubert.dev>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Cobra64 Compiler CLI
//!
//! A modern compiler for the Commodore 64 with Python-like syntax.

use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

use cobra64::error::{format_error, format_warning};
use cobra64::output::{format_from_extension, write_output, OutputFormat};
use cobra64::runner::{find_vice, SourceWatcher, ViceRunner};

/// Cobra64 - A modern compiler for the Commodore 64
#[derive(Parser, Debug)]
#[command(name = "cobra64")]
#[command(author = "Cobra64 Team")]
#[command(version)]
#[command(about = "A modern compiler for the Commodore 64 with Python-like syntax")]
#[command(long_about = r#"
Cobra64 compiles source files written in a Python-like language into
executable programs for the Commodore 64 home computer.

The output can be either:
  - PRG files (.prg) - Raw C64 program files
  - D64 files (.d64) - Disk images for use with emulators

Example usage:
  cobra64 hello.cb64 -o hello.prg
  cobra64 game.cb64 utils.cb64 -o game.d64

Run in VICE emulator:
  cobra64 hello.cb64 --run
  cobra64 hello.cb64 -r
  cobra64 hello.cb64 -o hello.prg --run

Watch mode with hot-reload:
  cobra64 game.cb64 --watch
  cobra64 game.cb64 -w
  cobra64 game.cb64 -o game.prg --watch
"#)]
struct Cli {
    /// Source files to compile (.cb64)
    #[arg(required = true)]
    source_files: Vec<PathBuf>,

    /// Output file (.prg or .d64). Required unless --run or --watch is used.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Run compiled program in VICE emulator after compilation
    #[arg(short, long)]
    run: bool,

    /// Watch source files and hot-reload on changes (implies --run)
    #[arg(short, long)]
    watch: bool,

    /// Path to VICE emulator binary (auto-detected if not specified)
    #[arg(long)]
    vice_path: Option<PathBuf>,

    /// Remote monitor port for VICE communication
    #[arg(long, default_value = "6510")]
    vice_port: u16,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Determine output path and format
    // If --run or --watch is used without -o, create a temp file
    let (output_path, format, is_temp_output) = match &cli.output {
        Some(path) => {
            let fmt = match format_from_extension(path) {
                Some(f) => f,
                None => {
                    eprintln!("Error: Unknown output format. Use .prg or .d64 extension.");
                    return ExitCode::from(2);
                }
            };
            (path.clone(), fmt, false)
        }
        None => {
            if cli.run || cli.watch {
                // Create temp file path based on source file name
                let source_stem = cli.source_files[0]
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("program");
                let temp_path = std::env::temp_dir().join(format!("{}.prg", source_stem));
                (temp_path, OutputFormat::Prg, true)
            } else {
                eprintln!("Error: Output file is required. Use -o <file.prg> or -o <file.d64>");
                eprintln!("       Or use --run to compile and run without saving.");
                return ExitCode::from(2);
            }
        }
    };

    if cli.verbose {
        println!("Cobra64 Compiler v{}", cobra64::VERSION);
        println!("Output format: {:?}", format);
        if is_temp_output {
            println!("Output: {} (temporary)", output_path.display());
        } else {
            println!("Output: {}", output_path.display());
        }
        println!("Source files:");
        for file in &cli.source_files {
            println!("  - {}", file.display());
        }
        println!();
    }

    // Read and concatenate source files
    let mut source = String::new();
    for path in &cli.source_files {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                if cli.verbose {
                    println!("Reading {}...", path.display());
                }
                source.push_str(&content);
                source.push('\n');
            }
            Err(e) => {
                eprintln!("Error: Cannot read {}: {}", path.display(), e);
                return ExitCode::from(3);
            }
        }
    }

    // Compile
    if cli.verbose {
        println!("Compiling...");
    }

    // Get the primary source path for include resolution
    let primary_source_path = &cli.source_files[0];

    // Get the primary filename for error messages
    let primary_filename = primary_source_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("<input>");

    let (code, warnings) =
        match cobra64::compile_with_path_and_warnings(&source, primary_source_path) {
            Ok((code, warnings)) => (code, warnings),
            Err(e) => {
                eprint!("{}", format_error(&e, &source, Some(primary_filename)));
                return ExitCode::from(1);
            }
        };

    // Print warnings (they don't prevent compilation)
    for warning in &warnings {
        eprint!(
            "{}",
            format_warning(warning, &source, Some(primary_filename))
        );
    }

    if cli.verbose {
        println!("Generated {} bytes of code", code.len());
    }

    // Get program name from first source file
    let program_name = cli.source_files[0]
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("PROGRAM");

    // Write output
    if cli.verbose {
        println!("Writing {}...", output_path.display());
    }

    if let Err(e) = write_output(&code, &output_path, format, program_name) {
        eprintln!("Error: Cannot write {}: {}", output_path.display(), e);
        return ExitCode::from(1);
    }

    if cli.verbose {
        println!("Done!");
    } else if !is_temp_output {
        println!(
            "Compiled {} -> {}",
            cli.source_files
                .iter()
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
                .collect::<Vec<_>>()
                .join(", "),
            output_path.display()
        );
    } else {
        println!(
            "Compiled {}",
            cli.source_files
                .iter()
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Run in VICE emulator if requested
    if cli.run || cli.watch {
        // Find VICE emulator
        let vice_path = match &cli.vice_path {
            Some(path) => {
                if !path.exists() {
                    eprintln!("Error: VICE path does not exist: {}", path.display());
                    return ExitCode::from(4);
                }
                path.clone()
            }
            None => match find_vice() {
                Some(path) => path,
                None => {
                    eprintln!("Error: VICE emulator not found.");
                    eprintln!();
                    eprintln!("Install VICE (x64sc) or specify the path with --vice-path:");
                    eprintln!("  macOS:   brew install vice");
                    eprintln!("  Ubuntu:  sudo apt install vice");
                    eprintln!("  Manual:  --vice-path /path/to/x64sc");
                    return ExitCode::from(4);
                }
            },
        };

        if cli.verbose {
            println!();
            println!("VICE path: {}", vice_path.display());
            println!("Monitor port: {}", cli.vice_port);
        }

        // Get absolute path for output file
        let abs_output_path = match output_path.canonicalize() {
            Ok(path) => path,
            Err(_) => output_path.clone(),
        };

        // Create and launch VICE
        let mut runner = ViceRunner::new(vice_path, cli.vice_port);

        if cli.verbose {
            println!("Launching VICE...");
        } else {
            println!("Running in VICE...");
        }

        if let Err(e) = runner.launch(&abs_output_path) {
            eprintln!("Error: Failed to start VICE: {}", e);
            return ExitCode::from(5);
        }

        // Watch mode: monitor files and hot-reload on changes
        if cli.watch {
            return run_watch_loop(cli, runner, format, abs_output_path);
        }

        // Non-watch mode: just wait for VICE to exit
        runner.wait();
    }

    ExitCode::SUCCESS
}

/// Run the watch loop for hot-reload functionality.
fn run_watch_loop(
    cli: Cli,
    mut runner: ViceRunner,
    format: OutputFormat,
    output_path: PathBuf,
) -> ExitCode {
    // Wait for VICE to be ready
    if cli.verbose {
        println!("Waiting for VICE to be ready...");
    }

    if let Err(e) = runner.wait_until_ready() {
        eprintln!("Warning: Could not connect to VICE monitor: {}", e);
        eprintln!("Hot-reload may not work. Continuing anyway...");
    }

    // Create file watcher
    let watcher = match SourceWatcher::new(&cli.source_files) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error: Failed to create file watcher: {}", e);
            return ExitCode::from(6);
        }
    };

    println!();
    println!("Watching for changes... (Press Ctrl+C to stop)");

    // Watch loop
    loop {
        // Wait for file change
        if let Err(e) = watcher.wait_for_change() {
            eprintln!("Watch error: {}", e);
            continue;
        }

        println!();
        if cli.verbose {
            println!("Change detected, recompiling...");
        } else {
            println!("Recompiling...");
        }

        // Re-read source files
        let mut source = String::new();
        let mut read_error = false;
        for path in &cli.source_files {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    source.push_str(&content);
                    source.push('\n');
                }
                Err(e) => {
                    eprintln!("Error: Cannot read {}: {}", path.display(), e);
                    read_error = true;
                    break;
                }
            }
        }

        if read_error {
            println!("Fix errors and save to retry.");
            continue;
        }

        // Get the primary source path for include resolution
        let primary_source_path = &cli.source_files[0];
        let primary_filename = primary_source_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<input>");

        // Compile
        let (code, warnings) =
            match cobra64::compile_with_path_and_warnings(&source, primary_source_path) {
                Ok((code, warnings)) => (code, warnings),
                Err(e) => {
                    eprint!("{}", format_error(&e, &source, Some(primary_filename)));
                    println!("Fix errors and save to retry.");
                    continue;
                }
            };

        // Print warnings
        for warning in &warnings {
            eprint!(
                "{}",
                format_warning(warning, &source, Some(primary_filename))
            );
        }

        if cli.verbose {
            println!("Generated {} bytes of code", code.len());
        }

        // Get program name from first source file
        let program_name = cli.source_files[0]
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("PROGRAM");

        // Write output
        if let Err(e) = write_output(&code, &output_path, format, program_name) {
            eprintln!("Error: Cannot write {}: {}", output_path.display(), e);
            println!("Fix errors and save to retry.");
            continue;
        }

        // Check if VICE is still running
        if !runner.is_running() {
            if cli.verbose {
                println!("VICE not running, relaunching...");
            } else {
                println!("Relaunching VICE...");
            }

            if let Err(e) = runner.launch(&output_path) {
                eprintln!("Error: Failed to start VICE: {}", e);
                println!("Fix errors and save to retry.");
                continue;
            }

            // Wait for VICE to be ready
            if let Err(e) = runner.wait_until_ready() {
                eprintln!("Warning: Could not connect to VICE monitor: {}", e);
            }
        } else {
            // Hot-reload via remote monitor
            if cli.verbose {
                println!("Reloading in VICE...");
            } else {
                println!("Reloading...");
            }

            if let Err(e) = runner.reload(&output_path) {
                eprintln!("Warning: Hot-reload failed: {}", e);
                eprintln!("Trying to relaunch VICE...");

                // Kill and relaunch
                let _ = runner.kill();
                if let Err(e) = runner.launch(&output_path) {
                    eprintln!("Error: Failed to restart VICE: {}", e);
                    continue;
                }
            }
        }

        println!(
            "Compiled {} -> {}",
            cli.source_files
                .iter()
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
                .collect::<Vec<_>>()
                .join(", "),
            output_path.display()
        );
        println!("Watching for changes...");
    }
}
