# mecab-ko-profiler

Memory profiling tools for MeCab-Ko components.

[![Crates.io](https://img.shields.io/crates/v/mecab-ko-profiler.svg)](https://crates.io/crates/mecab-ko-profiler)
[![Documentation](https://docs.rs/mecab-ko-profiler/badge.svg)](https://docs.rs/mecab-ko-profiler)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Overview

`mecab-ko-profiler` provides comprehensive memory profiling capabilities for analyzing the memory usage patterns of MeCab-Ko morphological analysis components. It includes:

- **Custom Tracking Allocator**: Monitor all allocations and deallocations with minimal overhead
- **Component-Specific Profilers**: Specialized profilers for Trie, Dictionary, and Tokenizer
- **Detailed Statistics**: Collection of allocation patterns, memory distribution, and efficiency metrics
- **Multiple Output Formats**: Generate reports in JSON or human-readable text
- **CLI Tool**: Command-line interface for profiling operations
- **Benchmark Integration**: Criterion-based benchmarks for performance analysis

## Features

- `default`: CLI and component profilers
- `cli`: Command-line tool support
- `profilers`: Component-specific profilers (Trie, Dict, Tokenizer)
- `flamegraph`: Flamegraph generation support (optional)
- `full`: All features enabled

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mecab-ko-profiler = "0.1"

# For development/testing
[dev-dependencies]
mecab-ko-profiler = { version = "0.1", features = ["full"] }
```

## Usage

### Basic Memory Tracking

```rust
use mecab_ko_profiler::prelude::*;

fn main() {
    // Track memory usage in a scope
    {
        let _guard = MemoryGuard::new("my_operation");

        // Your code here
        let data = vec![0u8; 1024];

        // Memory statistics will be printed when guard is dropped
    }

    // Get a snapshot of current memory usage
    let snap = snapshot();
    println!("Current usage: {} bytes", snap.current_usage);
    println!("Peak usage: {} bytes", snap.peak_usage);
}
```

### Profiling Dictionary Operations

```rust
use mecab_ko_profiler::prelude::*;

fn main() {
    let mut profiler = DictProfiler::new();

    // Profile lexicon loading
    profiler.profile_lexicon(10000, || {
        // Load lexicon with 10000 entries
        let lexicon: Vec<String> = (0..10000)
            .map(|i| format!("word_{}", i))
            .collect();
    });

    // Profile connection costs
    profiler.profile_connection_costs(1000, || {
        // Load connection cost matrix
        let matrix: Vec<i16> = vec![0; 1000 * 1000];
    });

    // Generate report
    let stats = profiler.finish();
    let report = ProfilingReport::new(stats);

    println!("{}", report.to_text());
}
```

### Profiling Tokenization

```rust
use mecab_ko_profiler::prelude::*;

fn main() {
    let mut profiler = TokenizerProfiler::new();

    let text = "한국어 형태소 분석 테스트";

    // Profile tokenization
    profiler.profile_tokenize(text, || {
        // Your tokenization logic here
        let tokens: Vec<String> = text
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
    });

    // Analyze per-character usage
    if let Some(per_char) = profiler.analyze_per_char_usage(text.len()) {
        println!("Memory per character: {:.2} bytes", per_char);
    }

    let stats = profiler.finish();
    let report = ProfilingReport::new(stats);
    println!("{}", report.to_text());
}
```

### Analyzing Memory Scaling

```rust
use mecab_ko_profiler::prelude::*;

fn main() {
    let mut profiler = TokenizerProfiler::new();

    let sizes = vec![10, 50, 100, 500, 1000];
    for size in &sizes {
        let text: String = "한".chars().cycle().take(*size).collect();
        profiler.profile_tokenize(&text, || {
            // Tokenize
        });
    }

    let scaling = profiler.analyze_scaling(&sizes);
    let complexity = scaling.estimate_complexity();

    if scaling.is_linear() {
        println!("Memory usage scales linearly (O(n))");
    } else {
        println!("Estimated complexity: O(n^{:.2})", complexity);
    }
}
```

### Profiling Trie Structures

```rust
use mecab_ko_profiler::trie_profiler::FstProfiler;

fn main() {
    let mut profiler = FstProfiler::new();

    // Profile FST construction
    profiler.profile_build(10000, || {
        let data: Vec<(String, u64)> = (0..10000)
            .map(|i| (format!("key_{:08}", i), i as u64))
            .collect();
        // Build FST from data
    });

    let stats = profiler.finish();
    println!("FST built with {} components", stats.components.len());
}
```

### Generating Reports

```rust
use mecab_ko_profiler::prelude::*;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut profiler = DictProfiler::new();

    // ... perform profiling ...

    let stats = profiler.finish();
    let report = ProfilingReport::new(stats);

    // JSON output
    let json = report.to_json()?;
    std::fs::write("profile.json", json)?;

    // Text output
    let text = report.to_text();
    std::fs::write("profile.txt", text)?;

    // Write to file with format
    let mut file = File::create("report.json")?;
    report.write_to(&mut file, ReportFormat::Json)?;

    Ok(())
}
```

## CLI Usage

The `mecab-profile` binary provides a command-line interface:

### Profile Dictionary Operations

```bash
# Profile dictionary with simulated entries
mecab-profile dict --entries 10000 --format text

# Profile dictionary and save to file
mecab-profile dict --entries 50000 --format json --output dict_profile.json
```

### Profile Tokenization

```bash
# Profile tokenization of text
mecab-profile tokenize --text "한국어 형태소 분석 테스트"

# Profile tokenization from file
mecab-profile tokenize --file input.txt --format json

# Analyze scaling behavior
mecab-profile tokenize --text "테스트" --scaling --format text
```

### Profile Trie Structures

```bash
# Profile FST construction
mecab-profile trie --entries 10000 --trie-type fst

# Profile Double-Array Trie
mecab-profile trie --entries 20000 --trie-type double-array --output trie_profile.json
```

### Generate Report from Profile Data

```bash
# Convert JSON profile to text report
mecab-profile report --input profile.json --format text --output report.txt
```

### Run Benchmarks

```bash
# Run benchmark with profiling
mecab-profile benchmark --name tokenize --iterations 1000
```

## Output Examples

### Text Report

```
═══════════════════════════════════════════════════════════
           MeCab-Ko Memory Profiling Report
═══════════════════════════════════════════════════════════

Generated: 2026-01-06T12:34:56Z
Version: 0.1.0

Overall Statistics:
───────────────────────────────────────────────────────────
  Total Allocated:  10.5 MiB
  Total Deallocated: 8.2 MiB
  Current Usage:     2.3 MiB
  Peak Usage:        4.1 MiB
  Components:        3
  Efficiency:        56.1%

Component Breakdown:
───────────────────────────────────────────────────────────
╭───────────────┬──────────────┬───────────┬──────────┬──────────┬────────────╮
│ Component     │ Allocations  │ Current   │ Peak     │ Avg Size │ Efficiency │
├───────────────┼──────────────┼───────────┼──────────┼──────────┼────────────┤
│ lexicon       │        5,234 │   1.5 MiB │  2.1 MiB │    298 B │      71.4% │
│ connection    │          500 │ 512.0 KiB │  1.0 MiB │  1.0 KiB │      50.0% │
│ features      │        8,192 │ 256.0 KiB │  1.0 MiB │     32 B │      25.0% │
╰───────────────┴──────────────┴───────────┴──────────┴──────────┴────────────╯

Top Memory Consumers:
───────────────────────────────────────────────────────────
1. lexicon - 1.5 MiB (peak: 2.1 MiB)
2. connection_costs - 512.0 KiB (peak: 1.0 MiB)
3. features - 256.0 KiB (peak: 1.0 MiB)

Analysis & Recommendations:
───────────────────────────────────────────────────────────
  Efficiency Score: 56.1%

  Recommendations:
    1. Consider using an arena allocator for 'features'
    2. Component 'features' has small average allocation size (32B). Consider pooling.

═══════════════════════════════════════════════════════════
```

### JSON Report

```json
{
  "metadata": {
    "timestamp": "2026-01-06T12:34:56Z",
    "version": "0.1.0",
    "platform": "linux"
  },
  "stats": {
    "components": {
      "lexicon": {
        "name": "lexicon",
        "allocations": 5234,
        "deallocations": 2617,
        "total_allocated": 1572864,
        "total_deallocated": 786432,
        "current_usage": 1572864,
        "peak_usage": 2097152,
        "avg_allocation_size": 298,
        "allocation_time": {"secs": 0, "nanos": 0}
      }
    },
    "overall": {
      "total_allocated": 11010048,
      "total_deallocated": 8601600,
      "current_usage": 2408448,
      "peak_usage": 4294967,
      "component_count": 3
    }
  },
  "analysis": {
    "efficiency_score": 56.1,
    "recommendations": [
      "Consider using an arena allocator for 'features'"
    ]
  }
}
```

## Benchmarks

Run benchmarks with:

```bash
cargo bench --package mecab-ko-profiler
```

This will generate detailed HTML reports in `target/criterion/`.

## Architecture

### Core Components

1. **Allocator** (`allocator.rs`): Custom tracking allocator with atomic counters
2. **Stats** (`stats.rs`): Statistics collection and analysis
3. **Reporter** (`reporter.rs`): Report generation in multiple formats

### Component Profilers

1. **TrieProfiler** (`trie_profiler.rs`): FST and Double-Array Trie profiling
2. **DictProfiler** (`dict_profiler.rs`): Dictionary operations profiling
3. **TokenizerProfiler** (`tokenizer_profiler.rs`): Tokenization and lattice profiling

## Performance Considerations

The tracking allocator adds minimal overhead:

- ~2-5ns per allocation/deallocation
- Atomic operations use relaxed memory ordering
- No locks or mutexes in the hot path
- Zero-cost when compiled out in release mode (optional)

## Contributing

Contributions are welcome! Please ensure:

1. All code passes `cargo clippy -- -D warnings`
2. Code is formatted with `cargo fmt`
3. Tests pass with `cargo test`
4. Benchmarks run successfully with `cargo bench`

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Related Projects

- [mecab-ko](../mecab-ko): Main MeCab-Ko library
- [mecab-ko-dict](../mecab-ko-dict): Dictionary management
- [mecab-ko-core](../mecab-ko-core): Core tokenization engine

## Resources

- [Documentation](https://docs.rs/mecab-ko-profiler)
- [Repository](https://github.com/hephaex/mecab-ko)
- [Issue Tracker](https://github.com/hephaex/mecab-ko/issues)
