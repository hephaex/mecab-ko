//! Basic usage example for mecab-ko-profiler.

use mecab_ko_profiler::prelude::*;

fn main() {
    println!("MeCab-Ko Memory Profiler - Basic Usage Example");
    println!("================================================\n");

    // Example 1: Simple memory tracking with guards
    println!("Example 1: Memory Guard");
    {
        let _guard = MemoryGuard::new("example_allocation");
        let _data = vec![0u8; 1024 * 1024]; // 1MB allocation
        println!("Allocated 1MB of data");
        // Guard will print stats when dropped
    }
    println!();

    // Example 2: Manual snapshot collection
    println!("Example 2: Manual Snapshots");
    let start = snapshot();
    println!("Start: {start:?}");

    let _test_data = vec![42u64; 10000];

    let end = snapshot();
    println!("End: {end:?}");

    let diff = end.diff(&start);
    println!("Diff: {diff:?}\n");

    // Example 3: Using StatsCollector
    println!("Example 3: Stats Collector");
    let mut collector = StatsCollector::new();

    let snap1 = MemorySnapshot {
        allocations: 100,
        deallocations: 50,
        total_allocated: 10000,
        total_deallocated: 5000,
        current_usage: 5000,
        peak_usage: 8000,
    };

    let snap2 = MemorySnapshot {
        allocations: 200,
        deallocations: 100,
        total_allocated: 20000,
        total_deallocated: 10000,
        current_usage: 10000,
        peak_usage: 15000,
    };

    collector.add_component("component1", snap1);
    collector.add_component("component2", snap2);

    let stats = collector.finish();
    let report = ProfilingReport::new(stats);

    println!("\nGenerated Report:");
    println!("{}", report.to_text());

    // Example 4: JSON export
    if let Ok(json) = report.to_json() {
        println!("\nJSON Export (first 200 chars):");
        println!("{}...", &json[..json.len().min(200)]);
    }

    println!("\nExample completed successfully!");
}
