# skym

A Python package exposing fuzzy text matching based on the Rust creates `skim` and `fuzzy-matcher`.

## Benchmarking

This project includes a comprehensive benchmarking suite built with [Criterion.rs](https://github.com/bheisler/criterion.rs).

### Running Benchmarks

We use `just` as a task runner to simplify benchmark operations. Here are the common benchmarking commands:

### Basic Benchmarking

```bash
# Run all benchmarks
just bench

# Open the benchmark report in your browser
just bench-report
```

#### Comparing Implementations

When making optimizations, you can compare performance between different implementations:

```bash
# Run benchmarks and save as a baseline named "original"
just bench-save original

# Make your code changes, then run and compare with the baseline
just bench-compare original

# Generate a comparison table between two sets of results
just bench-table original new
```

### Managing Benchmark Results

```bash
# Clean all benchmark results
just bench-clean
```

### Understanding Benchmark Results

After running benchmarks, Criterion generates comprehensive reports in `target/criterion/`. The most useful files are:

1. **HTML Report**: Open `target/criterion/report/index.html` in your browser for a visual representation of benchmark results including:
   - Line graphs showing performance over time
   - Violin plots showing distribution of execution times
   - Detailed statistical analysis

2. **JSON Data**: Raw benchmark data is stored in JSON format under `target/criterion/<benchmark_name>/` for each benchmark:
   - `estimates.json`: Contains summary statistics (mean, median, etc.)
   - `sample.json`: Contains raw sample timing data
   - `baseline_estimates.json`: Available when comparing with a baseline
