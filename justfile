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

# Open benchmark report in the default browser
bench-report:
    #!/usr/bin/env bash
    REPORT_PATH="target/criterion/report/index.html"
    if [ ! -f "$REPORT_PATH" ]; then
        echo "Benchmark report not found. Run 'just bench' first."
        exit 1
    fi

    # Try to open the report with the appropriate command based on OS
    case "$(uname -s)" in
        Linux*)     xdg-open "$REPORT_PATH" ;;
        Darwin*)    open "$REPORT_PATH" ;;
        CYGWIN*|MINGW*|MSYS*)  start "$REPORT_PATH" ;;
        *)          echo "Unable to open report automatically. Please open $REPORT_PATH in your browser." ;;
    esac

# Clean all benchmark results
bench-clean:
    rm -rf target/criterion

# List all available benchmarks
bench-list:
    #!/usr/bin/env bash
    echo "Available benchmark groups:"
    find benches -name "*.rs" -exec grep -l "criterion_group" {} \; | xargs grep "criterion_group" | sed 's/.*criterion_group!(\([^,]*\).*/\1/'
