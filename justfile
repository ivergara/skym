# Default recipe to display help information
default:
    @just --list

# --------------------
# Benchmarking Recipes
# --------------------

# Run all benchmarks
bench:
    cargo bench

# Run benchmarks and save results as a baseline
bench-save BASELINE="original":
    cargo bench -- --save-baseline {{BASELINE}}

# Compare current benchmark results with a saved baseline
bench-compare BASELINE="original":
    cargo bench -- --baseline {{BASELINE}}
