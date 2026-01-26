#!/usr/bin/env bash
# Profiling script for mecab-ko-elasticsearch

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$CRATE_DIR/profiling"

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== MeCab-Ko Elasticsearch Profiling ===${NC}"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check for required tools
check_tool() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${YELLOW}Warning: $1 not found. Install with: cargo install $2${NC}"
        return 1
    fi
    return 0
}

echo -e "\n${GREEN}Checking required tools...${NC}"
FLAMEGRAPH_AVAILABLE=false
CARGO_BENCH_AVAILABLE=true

if check_tool flamegraph flamegraph; then
    FLAMEGRAPH_AVAILABLE=true
fi

# Run benchmarks
echo -e "\n${GREEN}Running benchmarks...${NC}"
cd "$CRATE_DIR"

echo -e "${BLUE}Standard benchmarks${NC}"
cargo bench --bench analyzer_bench -- --save-baseline baseline

echo -e "${BLUE}Performance benchmarks${NC}"
cargo bench --bench performance_bench -- --save-baseline perf_baseline

# Generate flamegraph if available
if [ "$FLAMEGRAPH_AVAILABLE" = true ]; then
    echo -e "\n${GREEN}Generating flamegraph...${NC}"

    # Profile analyzer benchmark
    cargo flamegraph --bench analyzer_bench -o "$OUTPUT_DIR/analyzer_flamegraph.svg" -- --bench

    echo -e "${GREEN}Flamegraph saved to: $OUTPUT_DIR/analyzer_flamegraph.svg${NC}"
fi

# Run with different optimization levels
echo -e "\n${GREEN}Testing optimization levels...${NC}"

for opt_level in 2 3; do
    echo -e "${BLUE}Optimization level: $opt_level${NC}"
    RUSTFLAGS="-C opt-level=$opt_level" cargo bench --bench performance_bench -- \
        --save-baseline "opt_level_$opt_level" 2>&1 | tee "$OUTPUT_DIR/opt_level_$opt_level.log"
done

# Memory profiling (if valgrind is available)
if command -v valgrind &> /dev/null; then
    echo -e "\n${GREEN}Running memory profiling with valgrind...${NC}"

    # Build test binary
    cargo build --release --example basic_usage

    # Run with massif
    valgrind --tool=massif --massif-out-file="$OUTPUT_DIR/massif.out" \
        "$CRATE_DIR/../../target/release/examples/basic_usage" 2>&1 | \
        tee "$OUTPUT_DIR/massif.log"

    # Generate massif visualization if available
    if command -v ms_print &> /dev/null; then
        ms_print "$OUTPUT_DIR/massif.out" > "$OUTPUT_DIR/massif_report.txt"
        echo -e "${GREEN}Memory profile saved to: $OUTPUT_DIR/massif_report.txt${NC}"
    fi
fi

# CPU profiling with perf (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]] && command -v perf &> /dev/null; then
    echo -e "\n${GREEN}Running CPU profiling with perf...${NC}"

    # Record perf data
    sudo perf record -F 99 -g -- \
        cargo bench --bench performance_bench -- --profile-time=10

    # Generate perf report
    sudo perf report --stdio > "$OUTPUT_DIR/perf_report.txt"

    # Clean up
    sudo chown "$USER:$USER" perf.data
    mv perf.data "$OUTPUT_DIR/"

    echo -e "${GREEN}Perf data saved to: $OUTPUT_DIR/perf.data${NC}"
fi

# Generate comparison report
echo -e "\n${GREEN}Generating comparison report...${NC}"

cat > "$OUTPUT_DIR/README.md" << 'EOF'
# Profiling Results

## Overview

This directory contains profiling results for mecab-ko-elasticsearch.

## Files

- `analyzer_flamegraph.svg` - Flamegraph showing CPU time distribution
- `massif.out` / `massif_report.txt` - Memory usage analysis
- `perf.data` / `perf_report.txt` - CPU profiling data (Linux)
- `opt_level_*.log` - Benchmark results for different optimization levels

## Viewing Results

### Flamegraph
Open `analyzer_flamegraph.svg` in a web browser. Wider boxes indicate more CPU time.

### Massif Report
```bash
cat massif_report.txt | less
```

### Perf Report
```bash
perf report -i perf.data
```

## Benchmark Baselines

Baselines are stored in `target/criterion/`:
- `baseline` - Standard analyzer benchmarks
- `perf_baseline` - Comprehensive performance benchmarks
- `opt_level_*` - Optimization level comparisons

## Comparing Benchmarks

```bash
# Compare against baseline
cargo bench --bench performance_bench -- --baseline baseline

# Compare two baselines
critcmp baseline perf_baseline
```
EOF

echo -e "\n${GREEN}Profiling complete!${NC}"
echo -e "Results saved to: ${BLUE}$OUTPUT_DIR${NC}"
echo -e "\nNext steps:"
echo -e "  1. View flamegraph: xdg-open $OUTPUT_DIR/analyzer_flamegraph.svg"
echo -e "  2. Review benchmark results: ls -la target/criterion/"
echo -e "  3. Check profiling report: cat $OUTPUT_DIR/README.md"
