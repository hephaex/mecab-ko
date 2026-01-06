//! Memory profiling for tokenizer operations.
//!
//! This module provides specialized profiling for morphological analysis
//! including lattice construction, Viterbi search, and result generation.

use crate::allocator::{MemoryGuard, MemorySnapshot};
use crate::stats::{ComponentStats, StatsCollector};
use std::collections::HashMap;

/// Profiler for tokenizer operations.
#[derive(Debug)]
pub struct TokenizerProfiler {
    collector: StatsCollector,
    measurements: HashMap<String, MemorySnapshot>,
}

impl TokenizerProfiler {
    /// Creates a new tokenizer profiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            collector: StatsCollector::new(),
            measurements: HashMap::new(),
        }
    }

    /// Profiles tokenizer initialization.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use mecab_ko_profiler::tokenizer_profiler::TokenizerProfiler;
    ///
    /// let mut profiler = TokenizerProfiler::new();
    /// profiler.profile_init(|| {
    ///     // Initialize tokenizer here
    /// });
    /// ```
    pub fn profile_init<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new("tokenizer_init");

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.measurements.insert("init".to_string(), diff);
        self.collector.add_component("tokenizer_init", diff);

        result
    }

    /// Profiles lattice construction for a text.
    pub fn profile_lattice<F, R>(&mut self, text_len: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new(format!("lattice_len_{text_len}"));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.measurements
            .insert(format!("lattice_{text_len}"), diff);
        self.collector
            .add_component(format!("lattice_len_{text_len}"), diff);

        result
    }

    /// Profiles Viterbi search operation.
    pub fn profile_viterbi<F, R>(&mut self, node_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new(format!("viterbi_{node_count}_nodes"));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.measurements
            .insert(format!("viterbi_{node_count}"), diff);
        self.collector
            .add_component(format!("viterbi_{node_count}_nodes"), diff);

        result
    }

    /// Profiles complete tokenization operation.
    pub fn profile_tokenize<F, R>(&mut self, text: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new(format!("tokenize_len_{}", text.len()));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.measurements
            .insert(format!("tokenize_{}", text.len()), diff);
        self.collector
            .add_component(format!("tokenize_len_{}", text.len()), diff);

        result
    }

    /// Profiles result formatting operation.
    pub fn profile_format_result<F, R>(&mut self, token_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new(format!("format_{token_count}_tokens"));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.collector
            .add_component(format!("format_{token_count}_tokens"), diff);

        result
    }

    /// Analyzes memory usage patterns across different text sizes.
    #[must_use]
    pub fn analyze_scaling(&self, text_sizes: &[usize]) -> ScalingAnalysis {
        let mut data_points = Vec::new();

        for &size in text_sizes {
            if let Some(snapshot) = self.measurements.get(&format!("tokenize_{size}")) {
                data_points.push((size, snapshot.current_usage));
            }
        }

        ScalingAnalysis { data_points }
    }

    /// Analyzes per-character memory usage.
    #[must_use]
    pub fn analyze_per_char_usage(&self, text_len: usize) -> Option<f64> {
        self.measurements
            .get(&format!("tokenize_{text_len}"))
            .map(|snapshot| {
                if text_len > 0 {
                    snapshot.current_usage as f64 / text_len as f64
                } else {
                    0.0
                }
            })
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

impl Default for TokenizerProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Analysis of memory usage scaling with input size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalingAnalysis {
    /// Data points: (`text_size`, `memory_usage`)
    pub data_points: Vec<(usize, u64)>,
}

impl ScalingAnalysis {
    /// Estimates the scaling factor (linear, quadratic, etc.).
    ///
    /// Returns the estimated complexity factor.
    #[must_use]
    pub fn estimate_complexity(&self) -> f64 {
        if self.data_points.len() < 2 {
            return 0.0;
        }

        // Simple linear regression to estimate O(n^k)
        // We compute k from two points: memory = c * size^k
        let (size1, mem1) = self.data_points[0];
        let (size2, mem2) = *self.data_points.last().unwrap_or(&(1, 1));

        if size1 == 0 || size2 == 0 || mem1 == 0 || mem2 == 0 {
            return 0.0;
        }

        ((mem2 as f64).ln() - (mem1 as f64).ln()) / ((size2 as f64).ln() - (size1 as f64).ln())
    }

    /// Checks if scaling is linear (O(n)).
    #[must_use]
    pub fn is_linear(&self) -> bool {
        let complexity = self.estimate_complexity();
        (complexity - 1.0).abs() < 0.2 // Within 20% of linear
    }

    /// Gets the average memory usage per input unit.
    #[must_use]
    pub fn avg_per_unit(&self) -> f64 {
        if self.data_points.is_empty() {
            return 0.0;
        }

        let total_ratio: f64 = self
            .data_points
            .iter()
            .filter(|(size, _)| *size > 0)
            .map(|(size, mem)| *mem as f64 / *size as f64)
            .sum();

        let valid_count = self
            .data_points
            .iter()
            .filter(|(size, _)| *size > 0)
            .count();

        if valid_count > 0 {
            total_ratio / valid_count as f64
        } else {
            0.0
        }
    }
}

/// Profiler for lattice-specific operations.
#[derive(Debug)]
pub struct LatticeProfiler {
    profiler: TokenizerProfiler,
}

impl LatticeProfiler {
    /// Creates a new lattice profiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            profiler: TokenizerProfiler::new(),
        }
    }

    /// Profiles node insertion into the lattice.
    pub fn profile_node_insertion<F, R>(&mut self, node_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new(format!("insert_{node_count}_nodes"));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.profiler
            .collector
            .add_component(format!("node_insertion_{node_count}"), diff);

        result
    }

    /// Profiles path calculation in the lattice.
    pub fn profile_path_calculation<F, R>(&mut self, path_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.profiler.profile_viterbi(path_count, f)
    }

    /// Analyzes lattice density (nodes per character).
    #[must_use]
    pub fn analyze_density(&self, text_len: usize, node_count: usize) -> LatticeMetrics {
        let density = if text_len > 0 {
            node_count as f64 / text_len as f64
        } else {
            0.0
        };

        let avg_node_size = std::mem::size_of::<usize>() * 4; // Estimated node size
        let estimated_memory = (node_count * avg_node_size) as u64;

        LatticeMetrics {
            text_len,
            node_count,
            density,
            estimated_memory,
        }
    }

    /// Finalizes profiling and returns detailed statistics.
    #[must_use]
    pub fn finish(self) -> crate::stats::DetailedStats {
        self.profiler.finish()
    }
}

impl Default for LatticeProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics for lattice analysis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatticeMetrics {
    /// Input text length.
    pub text_len: usize,
    /// Number of nodes in lattice.
    pub node_count: usize,
    /// Node density (nodes per character).
    pub density: f64,
    /// Estimated memory usage.
    pub estimated_memory: u64,
}

impl LatticeMetrics {
    /// Checks if density is within acceptable range.
    #[must_use]
    pub fn is_acceptable_density(&self, max_density: f64) -> bool {
        self.density <= max_density
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_profiler_init() {
        let mut profiler = TokenizerProfiler::new();

        let result = profiler.profile_init(|| {
            let v: Vec<u8> = vec![0; 512];
            v.len()
        });

        assert_eq!(result, 512);
        assert!(profiler.get_stats("init").is_some());
    }

    #[test]
    fn test_scaling_analysis() {
        let analysis = ScalingAnalysis {
            data_points: vec![(10, 100), (20, 200), (30, 300), (40, 400)],
        };

        let complexity = analysis.estimate_complexity();
        assert!((complexity - 1.0).abs() < 0.1); // Should be linear

        assert!(analysis.is_linear());

        let avg = analysis.avg_per_unit();
        assert!((avg - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_lattice_metrics() {
        let metrics = LatticeMetrics {
            text_len: 100,
            node_count: 300,
            density: 3.0,
            estimated_memory: 4800,
        };

        assert!(metrics.is_acceptable_density(5.0));
        assert!(!metrics.is_acceptable_density(2.0));
    }

    #[test]
    fn test_scaling_analysis_complexity() {
        // Test quadratic scaling
        let quadratic = ScalingAnalysis {
            data_points: vec![(10, 100), (20, 400), (30, 900)],
        };

        let complexity = quadratic.estimate_complexity();
        assert!((complexity - 2.0).abs() < 0.2); // Should be close to quadratic
        assert!(!quadratic.is_linear());
    }
}
