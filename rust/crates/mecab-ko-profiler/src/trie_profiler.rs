//! Memory profiling for Trie data structures.
//!
//! This module provides specialized profiling for trie-based data structures
//! used in dictionary implementations (FST, Double-Array Trie).

use crate::allocator::{MemoryGuard, MemorySnapshot};
use crate::stats::{ComponentStats, StatsCollector};
use std::collections::HashMap;

/// Profiler for Trie data structures.
#[derive(Debug)]
pub struct TrieProfiler {
    collector: StatsCollector,
    measurements: HashMap<String, MemorySnapshot>,
}

impl TrieProfiler {
    /// Creates a new trie profiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            collector: StatsCollector::new(),
            measurements: HashMap::new(),
        }
    }

    /// Profiles a trie construction operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use mecab_ko_profiler::trie_profiler::TrieProfiler;
    ///
    /// let mut profiler = TrieProfiler::new();
    /// profiler.profile_construction("fst_build", || {
    ///     // Build your FST here
    /// });
    /// ```
    pub fn profile_construction<F, R>(&mut self, name: impl Into<String>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let name = name.into();
        let _guard = MemoryGuard::new(&name);

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.measurements.insert(name.clone(), diff);
        self.collector.add_component(name, diff);

        result
    }

    /// Profiles trie node allocation patterns.
    pub fn profile_node_allocation<F, T>(&mut self, count: usize, f: F)
    where
        F: Fn(usize) -> T,
    {
        let _guard = MemoryGuard::new("node_allocation");

        let start = crate::allocator::snapshot();

        for i in 0..count {
            let _ = f(i);
        }

        let end = crate::allocator::snapshot();
        let diff = end.diff(&start);

        self.collector.add_component("trie_nodes", diff);
    }

    /// Profiles trie memory overhead.
    ///
    /// Compares the actual memory usage against the theoretical minimum
    /// based on node count and entry size.
    #[must_use]
    pub fn analyze_overhead(&self, node_count: usize, entry_size: usize) -> OverheadAnalysis {
        let total_measured = self
            .measurements
            .values()
            .map(|s| s.current_usage)
            .sum::<u64>();

        let theoretical_minimum = (node_count * entry_size) as u64;
        let overhead = total_measured.saturating_sub(theoretical_minimum);
        let overhead_ratio = if theoretical_minimum > 0 {
            overhead as f64 / theoretical_minimum as f64
        } else {
            0.0
        };

        OverheadAnalysis {
            theoretical_minimum,
            actual_usage: total_measured,
            overhead,
            overhead_ratio,
            node_count,
            entry_size,
        }
    }

    /// Profiles a trie lookup operation.
    pub fn profile_lookup<F, R>(&mut self, key_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new(format!("lookup_{key_count}_keys"));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.collector
            .add_component(format!("lookup_{key_count}"), diff);

        result
    }

    /// Gets the component statistics for a specific measurement.
    #[must_use]
    pub fn get_stats(&self, name: &str) -> Option<ComponentStats> {
        self.measurements
            .get(name)
            .map(|snapshot| ComponentStats::from_snapshot(name, *snapshot))
    }

    /// Finalizes profiling and returns detailed statistics.
    #[must_use]
    pub fn finish(self) -> crate::stats::DetailedStats {
        self.collector.finish()
    }
}

impl Default for TrieProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Analysis of trie memory overhead.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverheadAnalysis {
    /// Theoretical minimum memory usage.
    pub theoretical_minimum: u64,
    /// Actual measured memory usage.
    pub actual_usage: u64,
    /// Overhead in bytes.
    pub overhead: u64,
    /// Overhead as a ratio of theoretical minimum.
    pub overhead_ratio: f64,
    /// Number of nodes in the trie.
    pub node_count: usize,
    /// Size of each entry in bytes.
    pub entry_size: usize,
}

impl OverheadAnalysis {
    /// Gets the overhead as a percentage.
    #[must_use]
    pub fn overhead_percentage(&self) -> f64 {
        self.overhead_ratio * 100.0
    }

    /// Checks if the overhead is within acceptable limits.
    #[must_use]
    pub fn is_acceptable(&self, max_overhead_ratio: f64) -> bool {
        self.overhead_ratio <= max_overhead_ratio
    }
}

/// Profiles FST (Finite State Transducer) construction and usage.
#[derive(Debug)]
pub struct FstProfiler {
    profiler: TrieProfiler,
}

impl FstProfiler {
    /// Creates a new FST profiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            profiler: TrieProfiler::new(),
        }
    }

    /// Profiles FST construction from key-value pairs.
    pub fn profile_build<F, R>(&mut self, entry_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.profiler
            .profile_construction(format!("fst_build_{entry_count}_entries"), f)
    }

    /// Profiles FST search operations.
    pub fn profile_search<F, R>(&mut self, query_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.profiler.profile_lookup(query_count, f)
    }

    /// Finalizes profiling and returns detailed statistics.
    #[must_use]
    pub fn finish(self) -> crate::stats::DetailedStats {
        self.profiler.finish()
    }
}

impl Default for FstProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Profiles Double-Array Trie construction and usage.
#[derive(Debug)]
pub struct DoubleArrayProfiler {
    profiler: TrieProfiler,
}

impl DoubleArrayProfiler {
    /// Creates a new double-array trie profiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            profiler: TrieProfiler::new(),
        }
    }

    /// Profiles double-array construction.
    pub fn profile_build<F, R>(&mut self, entry_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.profiler
            .profile_construction(format!("da_build_{entry_count}_entries"), f)
    }

    /// Profiles common prefix search.
    pub fn profile_common_prefix_search<F, R>(&mut self, query_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.profiler.profile_lookup(query_count, f)
    }

    /// Analyzes the array size and fill factor.
    #[must_use]
    pub fn analyze_fill_factor(&self, array_size: usize, used_nodes: usize) -> FillFactorAnalysis {
        let fill_factor = if array_size > 0 {
            used_nodes as f64 / array_size as f64
        } else {
            0.0
        };

        let wasted_space = (array_size - used_nodes) * std::mem::size_of::<u32>();

        FillFactorAnalysis {
            array_size,
            used_nodes,
            fill_factor,
            wasted_space,
        }
    }

    /// Finalizes profiling and returns detailed statistics.
    #[must_use]
    pub fn finish(self) -> crate::stats::DetailedStats {
        self.profiler.finish()
    }
}

impl Default for DoubleArrayProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Analysis of double-array trie fill factor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FillFactorAnalysis {
    /// Total array size.
    pub array_size: usize,
    /// Number of used nodes.
    pub used_nodes: usize,
    /// Fill factor (0.0 to 1.0).
    pub fill_factor: f64,
    /// Wasted space in bytes.
    pub wasted_space: usize,
}

impl FillFactorAnalysis {
    /// Gets the fill factor as a percentage.
    #[must_use]
    pub fn fill_percentage(&self) -> f64 {
        self.fill_factor * 100.0
    }

    /// Checks if the fill factor is within acceptable limits.
    #[must_use]
    pub fn is_acceptable(&self, min_fill_factor: f64) -> bool {
        self.fill_factor >= min_fill_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_profiler_construction() {
        let mut profiler = TrieProfiler::new();

        let result = profiler.profile_construction("test_trie", || {
            let v: Vec<u8> = vec![0; 1024];
            v.len()
        });

        assert_eq!(result, 1024);
        assert!(profiler.get_stats("test_trie").is_some());
    }

    #[test]
    fn test_overhead_analysis() {
        let analysis = OverheadAnalysis {
            theoretical_minimum: 1000,
            actual_usage: 1500,
            overhead: 500,
            overhead_ratio: 0.5,
            node_count: 100,
            entry_size: 10,
        };

        assert!((analysis.overhead_percentage() - 50.0).abs() < f64::EPSILON);
        assert!(analysis.is_acceptable(0.6));
        assert!(!analysis.is_acceptable(0.4));
    }

    #[test]
    fn test_fill_factor_analysis() {
        let analysis = FillFactorAnalysis {
            array_size: 1000,
            used_nodes: 750,
            fill_factor: 0.75,
            wasted_space: 1000,
        };

        assert!((analysis.fill_percentage() - 75.0).abs() < f64::EPSILON);
        assert!(analysis.is_acceptable(0.7));
        assert!(!analysis.is_acceptable(0.8));
    }
}
