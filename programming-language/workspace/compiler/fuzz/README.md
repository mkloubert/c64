# Fuzzing the Cobra64 Compiler

This directory contains fuzz targets for testing the Cobra64 compiler with random inputs.

## Prerequisites

1. **Nightly Rust**: Fuzzing requires nightly Rust
   ```bash
   rustup install nightly
   ```

2. **cargo-fuzz**: Install the fuzzing tool
   ```bash
   cargo +nightly install cargo-fuzz
   ```

## Available Fuzz Targets

| Target | Description |
|--------|-------------|
| `fuzz_lexer` | Feeds random bytes to the lexer |
| `fuzz_parser` | Feeds tokenized input to the parser |
| `fuzz_compiler` | Tests the complete compilation pipeline |

## Running Fuzz Tests

### Basic Usage

```bash
# Run lexer fuzzer indefinitely
cargo +nightly fuzz run fuzz_lexer

# Run parser fuzzer for 60 seconds
cargo +nightly fuzz run fuzz_parser -- -max_total_time=60

# Run compiler fuzzer with specific options
cargo +nightly fuzz run fuzz_compiler -- -max_total_time=300 -jobs=4
```

### Useful Options

| Option | Description |
|--------|-------------|
| `-max_total_time=N` | Stop after N seconds |
| `-max_len=N` | Maximum input size in bytes |
| `-jobs=N` | Run N parallel fuzzing jobs |
| `-dict=FILE` | Use dictionary file for guided fuzzing |

### Checking Coverage

```bash
# Generate coverage report
cargo +nightly fuzz coverage fuzz_compiler

# View coverage in browser
cargo +nightly cov -- show target/coverage/fuzz_compiler --format=html
```

## Reproducing Crashes

When a crash is found, it's saved in `fuzz/artifacts/`:

```bash
# Reproduce a crash
cargo +nightly fuzz run fuzz_compiler fuzz/artifacts/fuzz_compiler/crash-xxx

# Minimize the crash case
cargo +nightly fuzz tmin fuzz_compiler fuzz/artifacts/fuzz_compiler/crash-xxx
```

## Adding a Seed Corpus

Add example inputs to help the fuzzer:

```bash
mkdir -p fuzz/corpus/fuzz_compiler
cp tests/fixtures/valid/*.cb64 fuzz/corpus/fuzz_compiler/
```

## Alternative: Property-Based Testing

If nightly Rust is not available, use proptest for property-based testing:

```bash
cargo test --test fuzz_tests
```

This provides similar random input testing but runs on stable Rust.

## Best Practices

1. **Run periodically**: Fuzz for at least 1 hour on each release
2. **Check CI results**: Integrate fuzzing into CI if possible
3. **Document crashes**: Add regression tests for any crashes found
4. **Use corpus**: Build up a corpus of interesting inputs over time
