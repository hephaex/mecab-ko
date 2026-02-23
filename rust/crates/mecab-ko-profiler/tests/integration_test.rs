//! Integration tests for mecab-ko-profiler.
//!
//! Some tests require a global allocator to track memory allocations.
//! To run these tests, use:
//!
//! ```bash
//! cargo test -p mecab-ko-profiler --features test-allocator
//! ```

use mecab_ko_profiler::prelude::*;

// Set up global allocator for tests that require memory tracking
#[cfg(feature = "test-allocator")]
#[global_allocator]
static GLOBAL: mecab_ko_profiler::TrackingAllocator =
    mecab_ko_profiler::TrackingAllocator::new(std::alloc::System);

#[test]
#[cfg_attr(
    not(feature = "test-allocator"),
    ignore = "Requires global allocator - run with --features test-allocator"
)]
fn test_basic_memory_tracking() {
    reset_stats();

    {
        let _guard = MemoryGuard::new("test");
        let _v = vec![0u8; 1024];

        let snap = snapshot();
        assert!(snap.allocations > 0);
        assert!(snap.current_usage >= 1024);
    }
}

#[test]
#[cfg_attr(
    not(feature = "test-allocator"),
    ignore = "Requires global allocator - run with --features test-allocator"
)]
fn test_snapshot_diff() {
    reset_stats();

    let snap1 = snapshot();

    {
        let _v = vec![0u8; 2048];
        let snap2 = snapshot();

        let diff = snap2.diff(&snap1);
        assert!(diff.allocations > 0);
        assert!(diff.total_allocated >= 2048);
    }
}

#[test]
#[cfg_attr(
    not(feature = "test-allocator"),
    ignore = "Requires global allocator - run with --features test-allocator"
)]
fn test_memory_guard_nesting() {
    reset_stats();

    {
        let _outer = MemoryGuard::new("outer");
        let _v1 = vec![0u8; 512];

        {
            let _inner = MemoryGuard::new("inner");
            let _v2 = vec![0u8; 256];

            let snap = snapshot();
            assert!(snap.current_usage >= 768);
        }
    }
}

#[test]
fn test_stats_collector() {
    let mut collector = StatsCollector::new();

    let snapshot1 = MemorySnapshot {
        allocations: 10,
        deallocations: 5,
        total_allocated: 1000,
        total_deallocated: 500,
        current_usage: 500,
        peak_usage: 800,
    };

    let snapshot2 = MemorySnapshot {
        allocations: 20,
        deallocations: 10,
        total_allocated: 2000,
        total_deallocated: 1000,
        current_usage: 1000,
        peak_usage: 1500,
    };

    collector.add_component("component1", snapshot1);
    collector.add_component("component2", snapshot2);

    let stats = collector.finish();

    assert_eq!(stats.components.len(), 2);
    assert_eq!(stats.overall.component_count, 2);
    assert_eq!(stats.overall.current_usage, 1500);
}

#[test]
fn test_profiling_report_json() {
    let mut collector = StatsCollector::new();

    let snapshot = MemorySnapshot {
        allocations: 100,
        deallocations: 50,
        total_allocated: 10000,
        total_deallocated: 5000,
        current_usage: 5000,
        peak_usage: 8000,
    };

    collector.add_component("test", snapshot);

    let stats = collector.finish();
    let report = ProfilingReport::new(stats);

    let json = report.to_json();
    assert!(json.is_ok());

    let json_str = json.unwrap();
    assert!(json_str.contains("test"));
    assert!(json_str.contains("allocations"));
}

#[test]
fn test_profiling_report_text() {
    let mut collector = StatsCollector::new();

    let snapshot = MemorySnapshot {
        allocations: 100,
        deallocations: 50,
        total_allocated: 10000,
        total_deallocated: 5000,
        current_usage: 5000,
        peak_usage: 8000,
    };

    collector.add_component("test", snapshot);

    let stats = collector.finish();
    let report = ProfilingReport::new(stats);

    let text = report.to_text();
    assert!(text.contains("Memory Profiling Report"));
    assert!(text.contains("test"));
    assert!(text.contains("Overall Statistics"));
}

#[cfg(feature = "profilers")]
mod profiler_tests {
    use mecab_ko_profiler::dict_profiler::DictProfiler;
    use mecab_ko_profiler::tokenizer_profiler::TokenizerProfiler;
    use mecab_ko_profiler::trie_profiler::TrieProfiler;

    #[test]
    #[cfg_attr(
        not(feature = "test-allocator"),
        ignore = "Requires global allocator - run with --features test-allocator"
    )]
    fn test_dict_profiler() {
        let mut profiler = DictProfiler::new();

        profiler.profile_lexicon(100, || {
            let _data: Vec<Vec<u8>> = (0..100).map(|i| format!("word_{i}").into_bytes()).collect();
        });

        let stats = profiler.finish();
        assert_eq!(stats.components.len(), 1);
        assert!(stats.components.contains_key("lexicon"));
    }

    #[test]
    #[cfg_attr(
        not(feature = "test-allocator"),
        ignore = "Requires global allocator - run with --features test-allocator"
    )]
    fn test_tokenizer_profiler() {
        let mut profiler = TokenizerProfiler::new();

        let text = "한국어 형태소 분석";
        profiler.profile_tokenize(text, || {
            let _tokens: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        });

        let stats = profiler.finish();
        assert!(!stats.components.is_empty());
    }

    #[test]
    #[cfg_attr(
        not(feature = "test-allocator"),
        ignore = "Requires global allocator - run with --features test-allocator"
    )]
    fn test_trie_profiler() {
        let mut profiler = TrieProfiler::new();

        profiler.profile_construction("test_trie", || {
            let _data: Vec<String> = (0..100).map(|i| format!("key_{i:04}")).collect();
        });

        let stats = profiler.finish();
        assert!(!stats.components.is_empty());
    }

    #[test]
    #[cfg_attr(
        not(feature = "test-allocator"),
        ignore = "Requires global allocator - run with --features test-allocator"
    )]
    fn test_scaling_analysis() {
        let mut profiler = TokenizerProfiler::new();

        // Use ASCII to ensure char count == byte length
        let sizes = vec![10, 20, 30, 40];
        for size in &sizes {
            let text: String = "a".repeat(*size);
            profiler.profile_tokenize(&text, || {
                let _tokens: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            });
        }

        let scaling = profiler.analyze_scaling(&sizes);
        assert_eq!(scaling.data_points.len(), sizes.len());
    }

    #[test]
    #[cfg_attr(
        not(feature = "test-allocator"),
        ignore = "Requires global allocator - run with --features test-allocator"
    )]
    fn test_dict_memory_distribution() {
        let mut profiler = DictProfiler::new();

        profiler.profile_lexicon(100, || {
            let _data: Vec<Vec<u8>> = (0..100).map(|i| format!("word_{i}").into_bytes()).collect();
        });

        profiler.profile_connection_costs(100, || {
            let _matrix: Vec<i16> = vec![0; 100 * 100];
        });

        profiler.profile_features(100, || {
            let _features: Vec<String> = (0..100).map(|i| format!("feature_{i}")).collect();
        });

        let dist = profiler.analyze_distribution();
        assert!(dist.total > 0);
    }
}

#[test]
fn test_component_stats() {
    let snapshot = MemorySnapshot {
        allocations: 10,
        deallocations: 5,
        total_allocated: 1000,
        total_deallocated: 500,
        current_usage: 500,
        peak_usage: 800,
    };

    let stats = ComponentStats::from_snapshot("test", snapshot);

    assert_eq!(stats.name, "test");
    assert_eq!(stats.allocations, 10);
    assert_eq!(stats.current_usage, 500);
    assert_eq!(stats.peak_usage, 800);

    let efficiency = stats.efficiency();
    assert!((efficiency - 0.625).abs() < 0.001);
}

#[test]
fn test_overall_stats() {
    let mut detailed = DetailedStats::new();

    let snap1 = MemorySnapshot {
        allocations: 10,
        deallocations: 5,
        total_allocated: 1000,
        total_deallocated: 500,
        current_usage: 500,
        peak_usage: 800,
    };

    let snap2 = MemorySnapshot {
        allocations: 5,
        deallocations: 2,
        total_allocated: 500,
        total_deallocated: 200,
        current_usage: 300,
        peak_usage: 400,
    };

    detailed.add_component("comp1", ComponentStats::from_snapshot("comp1", snap1));
    detailed.add_component("comp2", ComponentStats::from_snapshot("comp2", snap2));
    detailed.compute_overall();

    assert_eq!(detailed.overall.component_count, 2);
    assert_eq!(detailed.overall.total_allocated, 1500);
    assert_eq!(detailed.overall.current_usage, 800);
}

#[test]
fn test_size_histogram() {
    use mecab_ko_profiler::stats::SizeHistogram;

    let mut histogram = SizeHistogram::new();

    histogram.record(32);
    histogram.record(128);
    histogram.record(512);
    histogram.record(2048);
    histogram.record(100_000);

    // Verify histogram recorded entries correctly
    let total_count: u64 = histogram.buckets.iter().map(|b| b.count).sum();
    assert_eq!(total_count, 5);
}

#[test]
fn test_report_format_parsing() {
    assert_eq!(ReportFormat::parse("json"), Some(ReportFormat::Json));
    assert_eq!(ReportFormat::parse("text"), Some(ReportFormat::Text));
    assert_eq!(ReportFormat::parse("txt"), Some(ReportFormat::Text));
    assert_eq!(ReportFormat::parse("invalid"), None);
}

#[test]
#[cfg_attr(
    not(feature = "test-allocator"),
    ignore = "Requires global allocator - run with --features test-allocator"
)]
fn test_memory_reset() {
    reset_stats();

    let snap1 = snapshot();
    assert_eq!(snap1.allocations, 0);
    assert_eq!(snap1.current_usage, 0);

    {
        let _v = vec![0u8; 1024];
        let snap2 = snapshot();
        assert!(snap2.allocations > 0);
    }

    reset_stats();
    let snap3 = snapshot();
    // Note: After reset, counters are zero but memory may still be allocated
    assert_eq!(snap3.allocations, 0);
}
