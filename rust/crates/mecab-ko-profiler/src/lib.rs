//! Memory profiling tools for MeCab-Ko components.
//!
//! This crate provides comprehensive memory profiling capabilities for analyzing
//! the memory usage patterns of MeCab-Ko morphological analysis components.
//!
//! # Features
//!
//! - **Custom Allocator**: Track all allocations and deallocations with minimal overhead
//! - **Component Profilers**: Specialized profilers for Trie, Dictionary, and Tokenizer
//! - **Detailed Statistics**: Collect allocation patterns, memory distribution, and efficiency metrics
//! - **Multiple Output Formats**: Generate reports in JSON or human-readable text
//! - **CLI Tool**: Command-line interface for profiling MeCab-Ko operations
//!
//! # Examples
//!
//! ## Basic Memory Tracking
//!
//! ```rust
//! use mecab_ko_profiler::allocator::{MemoryGuard, snapshot};
//!
//! // Track memory usage in a scope
//! {
//!     let _guard = MemoryGuard::new("my_operation");
//!
//!     // Your code here
//!     let data = vec![0u8; 1024];
//!
//!     // Memory statistics will be printed when guard is dropped
//! }
//!
//! // Get a snapshot of current memory usage
//! let snap = snapshot();
//! println!("Current usage: {} bytes", snap.current_usage);
//! ```
//!
//! ## Profiling Dictionary Loading
//!
//! ```rust,no_run
//! use mecab_ko_profiler::dict_profiler::DictProfiler;
//! use mecab_ko_profiler::reporter::{ProfilingReport, ReportFormat};
//!
//! let mut profiler = DictProfiler::new();
//!
//! // Profile dictionary loading
//! profiler.profile_load("mecab-ko-dic", || {
//!     // Load your dictionary here
//! });
//!
//! // Generate and display report
//! let stats = profiler.finish();
//! let report = ProfilingReport::new(stats);
//! println!("{}", report.to_text());
//! ```
//!
//! ## Profiling Tokenization
//!
//! ```rust
//! use mecab_ko_profiler::tokenizer_profiler::TokenizerProfiler;
//!
//! let mut profiler = TokenizerProfiler::new();
//!
//! let text = "한국어 형태소 분석";
//! profiler.profile_tokenize(text, || {
//!     // Tokenize the text here
//! });
//!
//! // Analyze scaling behavior
//! let scaling = profiler.analyze_scaling(&[10, 100, 1000]);
//! if scaling.is_linear() {
//!     println!("Memory usage scales linearly with input size");
//! }
//! ```
//!
//! # CLI Usage
//!
//! The `mecab-profile` binary provides a command-line interface:
//!
//! ```bash
//! # Profile dictionary loading
//! mecab-profile dict --dict-path /path/to/dict --output report.json
//!
//! # Profile tokenization
//! mecab-profile tokenize --text "분석할 텍스트" --format text
//!
//! # Generate memory report
//! mecab-profile report --input profile.json --output report.txt
//! ```

#![warn(missing_docs)]
#![allow(unsafe_code)] // Required for custom allocator implementation
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_precision_loss)] // Acceptable for profiling statistics
#![allow(clippy::cast_possible_wrap)] // Acceptable for memory diff reporting
#![allow(clippy::cast_sign_loss)] // Acceptable for memory calculations

pub mod allocator;
pub mod reporter;
pub mod stats;

#[cfg(feature = "profilers")]
pub mod dict_profiler;
#[cfg(feature = "profilers")]
pub mod tokenizer_profiler;
#[cfg(feature = "profilers")]
pub mod trie_profiler;

// Re-export commonly used types
pub use allocator::{MemoryGuard, MemorySnapshot, TrackingAllocator};
pub use reporter::{ProfilingReport, ReportFormat};
pub use stats::{ComponentStats, DetailedStats, StatsCollector};

#[cfg(feature = "profilers")]
pub use dict_profiler::DictProfiler;
#[cfg(feature = "profilers")]
pub use tokenizer_profiler::TokenizerProfiler;
#[cfg(feature = "profilers")]
pub use trie_profiler::TrieProfiler;

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::allocator::{get_stats, reset_stats, snapshot, MemoryGuard, MemorySnapshot};
    pub use crate::reporter::{ProfilingReport, ReportFormat};
    pub use crate::stats::{ComponentStats, DetailedStats, StatsCollector};

    #[cfg(feature = "profilers")]
    pub use crate::dict_profiler::DictProfiler;
    #[cfg(feature = "profilers")]
    pub use crate::tokenizer_profiler::TokenizerProfiler;
    #[cfg(feature = "profilers")]
    pub use crate::trie_profiler::TrieProfiler;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires global allocator to be installed"]
    fn test_basic_tracking() {
        allocator::reset_stats();

        {
            let _guard = MemoryGuard::new("test");
            let _v = vec![0u8; 1024];

            let snap = allocator::snapshot();
            assert!(snap.allocations > 0);
        }
    }

    #[test]
    fn test_stats_collector() {
        let mut collector = StatsCollector::new();

        let snapshot = MemorySnapshot {
            allocations: 10,
            deallocations: 5,
            total_allocated: 1000,
            total_deallocated: 500,
            current_usage: 500,
            peak_usage: 800,
        };

        collector.add_component("test", snapshot);

        let stats = collector.finish();
        assert_eq!(stats.components.len(), 1);
        assert_eq!(stats.overall.current_usage, 500);
    }
}
