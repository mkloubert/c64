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

//! Performance benchmarks for the Cobra64 compiler.
//!
//! Run with: cargo bench
//!
//! Results are saved to target/criterion/ with HTML reports.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;

// ============================================================================
// Benchmark Inputs
// ============================================================================

fn load_input(name: &str) -> String {
    let path = format!("benches/inputs/{}.cb64", name);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to load benchmark input: {}", path))
}

// ============================================================================
// Lexer Benchmarks
// ============================================================================

fn bench_lexer(c: &mut Criterion) {
    let small = load_input("small");
    let medium = load_input("medium");
    let large = load_input("large");

    let mut group = c.benchmark_group("lexer");

    // Throughput based on source code size
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_with_input(BenchmarkId::new("tokenize", "small"), &small, |b, src| {
        b.iter(|| cobra64::lexer::tokenize(black_box(src)))
    });

    group.throughput(Throughput::Bytes(medium.len() as u64));
    group.bench_with_input(BenchmarkId::new("tokenize", "medium"), &medium, |b, src| {
        b.iter(|| cobra64::lexer::tokenize(black_box(src)))
    });

    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_with_input(BenchmarkId::new("tokenize", "large"), &large, |b, src| {
        b.iter(|| cobra64::lexer::tokenize(black_box(src)))
    });

    group.finish();
}

// ============================================================================
// Parser Benchmarks
// ============================================================================

fn bench_parser(c: &mut Criterion) {
    let small = load_input("small");
    let medium = load_input("medium");
    let large = load_input("large");

    // Pre-tokenize for parser benchmarks
    let small_tokens = cobra64::lexer::tokenize(&small).unwrap();
    let medium_tokens = cobra64::lexer::tokenize(&medium).unwrap();
    let large_tokens = cobra64::lexer::tokenize(&large).unwrap();

    let mut group = c.benchmark_group("parser");

    group.throughput(Throughput::Elements(small_tokens.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "small"),
        &small_tokens,
        |b, tokens| b.iter(|| cobra64::parser::parse(black_box(tokens))),
    );

    group.throughput(Throughput::Elements(medium_tokens.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "medium"),
        &medium_tokens,
        |b, tokens| b.iter(|| cobra64::parser::parse(black_box(tokens))),
    );

    group.throughput(Throughput::Elements(large_tokens.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "large"),
        &large_tokens,
        |b, tokens| b.iter(|| cobra64::parser::parse(black_box(tokens))),
    );

    group.finish();
}

// ============================================================================
// Analyzer Benchmarks
// ============================================================================

fn bench_analyzer(c: &mut Criterion) {
    let small = load_input("small");
    let medium = load_input("medium");
    let large = load_input("large");

    // Pre-parse for analyzer benchmarks
    let small_tokens = cobra64::lexer::tokenize(&small).unwrap();
    let medium_tokens = cobra64::lexer::tokenize(&medium).unwrap();
    let large_tokens = cobra64::lexer::tokenize(&large).unwrap();

    let small_ast = cobra64::parser::parse(&small_tokens).unwrap();
    let medium_ast = cobra64::parser::parse(&medium_tokens).unwrap();
    let large_ast = cobra64::parser::parse(&large_tokens).unwrap();

    let mut group = c.benchmark_group("analyzer");

    group.bench_with_input(
        BenchmarkId::new("analyze", "small"),
        &small_ast,
        |b, ast| b.iter(|| cobra64::analyzer::analyze(black_box(ast))),
    );

    group.bench_with_input(
        BenchmarkId::new("analyze", "medium"),
        &medium_ast,
        |b, ast| b.iter(|| cobra64::analyzer::analyze(black_box(ast))),
    );

    group.bench_with_input(
        BenchmarkId::new("analyze", "large"),
        &large_ast,
        |b, ast| b.iter(|| cobra64::analyzer::analyze(black_box(ast))),
    );

    group.finish();
}

// ============================================================================
// Code Generation Benchmarks
// ============================================================================

fn bench_codegen(c: &mut Criterion) {
    let small = load_input("small");
    let medium = load_input("medium");
    let large = load_input("large");

    // Pre-analyze for codegen benchmarks (analysis modifies AST in place)
    let small_tokens = cobra64::lexer::tokenize(&small).unwrap();
    let medium_tokens = cobra64::lexer::tokenize(&medium).unwrap();
    let large_tokens = cobra64::lexer::tokenize(&large).unwrap();

    let small_ast = cobra64::parser::parse(&small_tokens).unwrap();
    let medium_ast = cobra64::parser::parse(&medium_tokens).unwrap();
    let large_ast = cobra64::parser::parse(&large_tokens).unwrap();

    // Analyze to populate type info
    let _ = cobra64::analyzer::analyze(&small_ast);
    let _ = cobra64::analyzer::analyze(&medium_ast);
    let _ = cobra64::analyzer::analyze(&large_ast);

    let mut group = c.benchmark_group("codegen");

    group.bench_with_input(
        BenchmarkId::new("generate", "small"),
        &small_ast,
        |b, ast| b.iter(|| cobra64::codegen::generate(black_box(ast))),
    );

    group.bench_with_input(
        BenchmarkId::new("generate", "medium"),
        &medium_ast,
        |b, ast| b.iter(|| cobra64::codegen::generate(black_box(ast))),
    );

    group.bench_with_input(
        BenchmarkId::new("generate", "large"),
        &large_ast,
        |b, ast| b.iter(|| cobra64::codegen::generate(black_box(ast))),
    );

    group.finish();
}

// ============================================================================
// End-to-End Compilation Benchmarks
// ============================================================================

fn bench_compile(c: &mut Criterion) {
    let small = load_input("small");
    let medium = load_input("medium");
    let large = load_input("large");

    let mut group = c.benchmark_group("compile");

    // Throughput based on lines of code
    let small_lines = small.lines().count() as u64;
    let medium_lines = medium.lines().count() as u64;
    let large_lines = large.lines().count() as u64;

    group.throughput(Throughput::Elements(small_lines));
    group.bench_with_input(BenchmarkId::new("full", "small"), &small, |b, src| {
        b.iter(|| cobra64::compile(black_box(src)))
    });

    group.throughput(Throughput::Elements(medium_lines));
    group.bench_with_input(BenchmarkId::new("full", "medium"), &medium, |b, src| {
        b.iter(|| cobra64::compile(black_box(src)))
    });

    group.throughput(Throughput::Elements(large_lines));
    group.bench_with_input(BenchmarkId::new("full", "large"), &large, |b, src| {
        b.iter(|| cobra64::compile(black_box(src)))
    });

    group.finish();
}

// ============================================================================
// Micro-Benchmarks
// ============================================================================

fn bench_micro(c: &mut Criterion) {
    let mut group = c.benchmark_group("micro");

    // Benchmark minimal program
    let minimal = "def main():\n    pass\n";
    group.bench_function("minimal_program", |b| {
        b.iter(|| cobra64::compile(black_box(minimal)))
    });

    // Benchmark hello world
    let hello = "def main():\n    println(\"HELLO\")\n";
    group.bench_function("hello_world", |b| {
        b.iter(|| cobra64::compile(black_box(hello)))
    });

    // Benchmark variable declaration
    let variable = "def main():\n    x: byte = 42\n";
    group.bench_function("single_variable", |b| {
        b.iter(|| cobra64::compile(black_box(variable)))
    });

    // Benchmark arithmetic
    let arithmetic = "def main():\n    x: byte = 1 + 2 * 3 - 4 / 2\n";
    group.bench_function("arithmetic_expr", |b| {
        b.iter(|| cobra64::compile(black_box(arithmetic)))
    });

    // Benchmark function call
    let function = "def foo() -> byte:\n    return 42\n\ndef main():\n    x: byte = foo()\n";
    group.bench_function("function_call", |b| {
        b.iter(|| cobra64::compile(black_box(function)))
    });

    // Benchmark while loop
    let loop_code = "def main():\n    i: byte = 0\n    while i < 10:\n        i = i + 1\n";
    group.bench_function("while_loop", |b| {
        b.iter(|| cobra64::compile(black_box(loop_code)))
    });

    // Benchmark if-else
    let ifelse = "def main():\n    x: byte = 5\n    if x > 3:\n        println(\"A\")\n    else:\n        println(\"B\")\n";
    group.bench_function("if_else", |b| {
        b.iter(|| cobra64::compile(black_box(ifelse)))
    });

    group.finish();
}

// ============================================================================
// Scaling Benchmarks
// ============================================================================

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    // Test how compilation time scales with number of variables
    for count in [1, 5, 10, 20, 50].iter() {
        let mut source = String::from("def main():\n");
        for i in 0..*count {
            source.push_str(&format!("    v{}: byte = {}\n", i, i % 256));
        }

        group.bench_with_input(BenchmarkId::new("variables", count), &source, |b, src| {
            b.iter(|| cobra64::compile(black_box(src)))
        });
    }

    // Test how compilation time scales with number of functions
    for count in [1, 5, 10, 20].iter() {
        let mut source = String::new();
        for i in 0..*count {
            source.push_str(&format!("def fn_{}():\n    pass\n\n", i));
        }
        source.push_str("def main():\n    pass\n");

        group.bench_with_input(BenchmarkId::new("functions", count), &source, |b, src| {
            b.iter(|| cobra64::compile(black_box(src)))
        });
    }

    group.finish();
}

// ============================================================================
// Main
// ============================================================================

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_analyzer,
    bench_codegen,
    bench_compile,
    bench_micro,
    bench_scaling,
);

criterion_main!(benches);
