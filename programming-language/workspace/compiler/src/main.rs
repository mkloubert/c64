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
use cobra64::output::{format_from_extension, write_output};

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
"#)]
struct Cli {
    /// Source files to compile (.cb64)
    #[arg(required = true)]
    source_files: Vec<PathBuf>,

    /// Output file (.prg or .d64)
    #[arg(short, long)]
    output: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Determine output format
    let format = match format_from_extension(&cli.output) {
        Some(f) => f,
        None => {
            eprintln!("Error: Unknown output format. Use .prg or .d64 extension.");
            return ExitCode::from(2);
        }
    };

    if cli.verbose {
        println!("Cobra64 Compiler v{}", cobra64::VERSION);
        println!("Output format: {:?}", format);
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

    // Get the primary filename for error messages
    let primary_filename = cli.source_files[0]
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("<input>");

    let (code, warnings) = match cobra64::compile_with_warnings(&source) {
        Ok((code, warnings)) => (code, warnings),
        Err(e) => {
            eprint!("{}", format_error(&e, &source, Some(primary_filename)));
            return ExitCode::from(1);
        }
    };

    // Print warnings (they don't prevent compilation)
    for warning in &warnings {
        eprint!("{}", format_warning(warning, &source, Some(primary_filename)));
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
        println!("Writing {}...", cli.output.display());
    }

    if let Err(e) = write_output(&code, &cli.output, format, program_name) {
        eprintln!("Error: Cannot write {}: {}", cli.output.display(), e);
        return ExitCode::from(1);
    }

    if cli.verbose {
        println!("Done!");
    } else {
        println!(
            "Compiled {} -> {}",
            cli.source_files
                .iter()
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
                .collect::<Vec<_>>()
                .join(", "),
            cli.output.display()
        );
    }

    ExitCode::SUCCESS
}
