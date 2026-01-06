//! Memory profiling for dictionary data structures.
//!
//! This module provides specialized profiling for `MeCab` dictionary components
//! including lexicon, connection costs, and feature data.

use crate::allocator::{MemoryGuard, MemorySnapshot};
use crate::stats::{ComponentStats, StatsCollector};
use std::collections::HashMap;

/// Profiler for dictionary data structures.
#[derive(Debug)]
pub struct DictProfiler {
    collector: StatsCollector,
    measurements: HashMap<String, MemorySnapshot>,
}

impl DictProfiler {
    /// Creates a new dictionary profiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            collector: StatsCollector::new(),
            measurements: HashMap::new(),
        }
    }

    /// Profiles dictionary loading operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use mecab_ko_profiler::dict_profiler::DictProfiler;
    ///
    /// let mut profiler = DictProfiler::new();
    /// profiler.profile_load("mecab-ko-dic", || {
    ///     // Load dictionary here
    /// });
    /// ```
    pub fn profile_load<F, R>(&mut self, dict_name: impl Into<String>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let name = dict_name.into();
        let _guard = MemoryGuard::new(format!("load_{name}"));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.measurements.insert(name.clone(), diff);
        self.collector.add_component(format!("dict_{name}"), diff);

        result
    }

    /// Profiles lexicon construction.
    pub fn profile_lexicon<F, R>(&mut self, entry_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new(format!("lexicon_{entry_count}_entries"));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.collector.add_component("lexicon", diff);

        result
    }

    /// Profiles connection cost matrix loading.
    pub fn profile_connection_costs<F, R>(&mut self, matrix_size: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new(format!("connection_costs_{matrix_size}"));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.collector.add_component("connection_costs", diff);

        result
    }

    /// Profiles feature data loading.
    pub fn profile_features<F, R>(&mut self, feature_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new(format!("features_{feature_count}"));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.collector.add_component("features", diff);

        result
    }

    /// Analyzes dictionary memory distribution.
    #[must_use]
    pub fn analyze_distribution(&self) -> DictMemoryDistribution {
        let lexicon = self.get_component_usage("lexicon");
        let connection_costs = self.get_component_usage("connection_costs");
        let features = self.get_component_usage("features");

        let total = lexicon + connection_costs + features;

        DictMemoryDistribution {
            lexicon,
            connection_costs,
            features,
            total,
        }
    }

    /// Profiles dictionary lookup operation.
    pub fn profile_lookup<F, R>(&mut self, query: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new(format!("lookup_{}", query.len()));

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.collector
            .add_component(format!("lookup_len_{}", query.len()), diff);

        result
    }

    /// Analyzes compression efficiency if dictionary uses compression.
    #[must_use]
    pub fn analyze_compression(
        &self,
        compressed_size: u64,
        uncompressed_size: u64,
    ) -> CompressionAnalysis {
        let ratio = if uncompressed_size > 0 {
            compressed_size as f64 / uncompressed_size as f64
        } else {
            0.0
        };

        let savings = uncompressed_size.saturating_sub(compressed_size);
        let savings_percentage = if uncompressed_size > 0 {
            (savings as f64 / uncompressed_size as f64) * 100.0
        } else {
            0.0
        };

        CompressionAnalysis {
            compressed_size,
            uncompressed_size,
            ratio,
            savings,
            savings_percentage,
        }
    }

    fn get_component_usage(&self, component: &str) -> u64 {
        self.measurements
            .get(component)
            .map_or(0, |s| s.current_usage)
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

impl Default for DictProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory distribution across dictionary components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DictMemoryDistribution {
    /// Memory used by lexicon.
    pub lexicon: u64,
    /// Memory used by connection costs.
    pub connection_costs: u64,
    /// Memory used by feature data.
    pub features: u64,
    /// Total memory usage.
    pub total: u64,
}

impl DictMemoryDistribution {
    /// Gets the percentage of memory used by each component.
    #[must_use]
    pub fn percentages(&self) -> (f64, f64, f64) {
        if self.total == 0 {
            return (0.0, 0.0, 0.0);
        }

        let lexicon_pct = (self.lexicon as f64 / self.total as f64) * 100.0;
        let costs_pct = (self.connection_costs as f64 / self.total as f64) * 100.0;
        let features_pct = (self.features as f64 / self.total as f64) * 100.0;

        (lexicon_pct, costs_pct, features_pct)
    }

    /// Gets the largest component.
    #[must_use]
    pub fn largest_component(&self) -> &str {
        let max = self.lexicon.max(self.connection_costs).max(self.features);

        if max == self.lexicon {
            "lexicon"
        } else if max == self.connection_costs {
            "connection_costs"
        } else {
            "features"
        }
    }
}

/// Compression efficiency analysis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompressionAnalysis {
    /// Compressed size in bytes.
    pub compressed_size: u64,
    /// Uncompressed size in bytes.
    pub uncompressed_size: u64,
    /// Compression ratio.
    pub ratio: f64,
    /// Bytes saved by compression.
    pub savings: u64,
    /// Savings as a percentage.
    pub savings_percentage: f64,
}

impl CompressionAnalysis {
    /// Checks if compression is effective.
    #[must_use]
    pub fn is_effective(&self, min_savings_pct: f64) -> bool {
        self.savings_percentage >= min_savings_pct
    }

    /// Gets the compression ratio as a human-readable string.
    #[must_use]
    pub fn ratio_string(&self) -> String {
        format!("{:.2}:1", 1.0 / self.ratio)
    }
}

/// Profiler for user dictionary operations.
#[derive(Debug)]
pub struct UserDictProfiler {
    dict_profiler: DictProfiler,
}

impl UserDictProfiler {
    /// Creates a new user dictionary profiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            dict_profiler: DictProfiler::new(),
        }
    }

    /// Profiles user dictionary addition.
    pub fn profile_add_entry<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = MemoryGuard::new("user_dict_add");

        let start = crate::allocator::snapshot();
        let result = f();
        let end = crate::allocator::snapshot();

        let diff = end.diff(&start);
        self.dict_profiler
            .collector
            .add_component("user_dict_add", diff);

        result
    }

    /// Profiles user dictionary compilation.
    pub fn profile_compile<F, R>(&mut self, entry_count: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.dict_profiler
            .profile_load(format!("user_dict_compile_{entry_count}"), f)
    }

    /// Finalizes profiling and returns detailed statistics.
    #[must_use]
    pub fn finish(self) -> crate::stats::DetailedStats {
        self.dict_profiler.finish()
    }
}

impl Default for UserDictProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires global allocator to be installed
    fn test_dict_profiler_load() {
        let mut profiler = DictProfiler::new();

        let result = profiler.profile_load("test_dict", || {
            let v: Vec<u8> = vec![0; 2048];
            v.len()
        });

        assert_eq!(result, 2048);
        assert!(profiler.get_stats("dict_test_dict").is_some());
    }

    #[test]
    fn test_dict_memory_distribution() {
        let dist = DictMemoryDistribution {
            lexicon: 1000,
            connection_costs: 500,
            features: 250,
            total: 1750,
        };

        let (lex_pct, costs_pct, features_pct) = dist.percentages();
        assert!((lex_pct - 57.14).abs() < 0.1);
        assert!((costs_pct - 28.57).abs() < 0.1);
        assert!((features_pct - 14.29).abs() < 0.1);

        assert_eq!(dist.largest_component(), "lexicon");
    }

    #[test]
    fn test_compression_analysis() {
        let analysis = CompressionAnalysis {
            compressed_size: 250,
            uncompressed_size: 1000,
            ratio: 0.25,
            savings: 750,
            savings_percentage: 75.0,
        };

        assert!(analysis.is_effective(50.0));
        assert!(!analysis.is_effective(80.0));
        assert_eq!(analysis.ratio_string(), "4.00:1");
    }
}
