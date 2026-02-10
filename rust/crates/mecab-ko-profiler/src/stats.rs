//! Advanced memory statistics collection and analysis.
//!
//! This module provides detailed memory usage statistics including
//! allocation patterns, memory fragmentation, and component-level breakdowns.

use crate::allocator::MemorySnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Detailed memory statistics with component breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedStats {
    /// Component-level statistics.
    pub components: HashMap<String, ComponentStats>,
    /// Overall statistics.
    pub overall: OverallStats,
    /// Allocation size histogram.
    pub size_histogram: SizeHistogram,
    /// Time-series data points.
    pub timeline: Vec<TimePoint>,
}

impl DetailedStats {
    /// Creates a new detailed statistics collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            overall: OverallStats::default(),
            size_histogram: SizeHistogram::new(),
            timeline: Vec::new(),
        }
    }

    /// Adds a component's statistics.
    pub fn add_component(&mut self, name: impl Into<String>, stats: ComponentStats) {
        self.components.insert(name.into(), stats);
    }

    /// Records a time point in the timeline.
    pub fn record_timepoint(&mut self, label: impl Into<String>, snapshot: MemorySnapshot) {
        self.timeline.push(TimePoint {
            label: label.into(),
            timestamp: Instant::now(),
            snapshot,
        });
    }

    /// Computes overall statistics from component data.
    pub fn compute_overall(&mut self) {
        let mut total_allocated = 0u64;
        let mut total_deallocated = 0u64;
        let mut current_usage = 0u64;
        let mut peak_usage = 0u64;

        for stats in self.components.values() {
            total_allocated += stats.total_allocated;
            total_deallocated += stats.total_deallocated;
            current_usage += stats.current_usage;
            peak_usage = peak_usage.max(stats.peak_usage);
        }

        self.overall = OverallStats {
            total_allocated,
            total_deallocated,
            current_usage,
            peak_usage,
            component_count: self.components.len(),
        };
    }

    /// Gets the top N components by memory usage.
    #[must_use]
    pub fn top_components(&self, n: usize) -> Vec<(&str, &ComponentStats)> {
        let mut components: Vec<_> = self
            .components
            .iter()
            .map(|(name, stats)| (name.as_str(), stats))
            .collect();

        components.sort_by(|a, b| b.1.current_usage.cmp(&a.1.current_usage));
        components.truncate(n);
        components
    }
}

impl Default for DetailedStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a specific component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentStats {
    /// Component name.
    pub name: String,
    /// Number of allocations.
    pub allocations: usize,
    /// Number of deallocations.
    pub deallocations: usize,
    /// Total bytes allocated.
    pub total_allocated: u64,
    /// Total bytes deallocated.
    pub total_deallocated: u64,
    /// Current memory usage.
    pub current_usage: u64,
    /// Peak memory usage.
    pub peak_usage: u64,
    /// Average allocation size.
    pub avg_allocation_size: u64,
    /// Time spent on allocations.
    pub allocation_time: Duration,
}

impl ComponentStats {
    /// Creates statistics from a memory snapshot.
    #[must_use]
    pub fn from_snapshot(name: impl Into<String>, snapshot: MemorySnapshot) -> Self {
        let avg_allocation_size = if snapshot.allocations > 0 {
            snapshot.total_allocated / snapshot.allocations as u64
        } else {
            0
        };

        Self {
            name: name.into(),
            allocations: snapshot.allocations,
            deallocations: snapshot.deallocations,
            total_allocated: snapshot.total_allocated,
            total_deallocated: snapshot.total_deallocated,
            current_usage: snapshot.current_usage,
            peak_usage: snapshot.peak_usage,
            avg_allocation_size,
            allocation_time: Duration::default(),
        }
    }

    /// Computes the allocation efficiency (ratio of current to peak).
    #[must_use]
    pub fn efficiency(&self) -> f64 {
        if self.peak_usage > 0 {
            self.current_usage as f64 / self.peak_usage as f64
        } else {
            0.0
        }
    }

    /// Computes the fragmentation indicator.
    #[must_use]
    pub fn fragmentation_score(&self) -> f64 {
        let active_allocations = self.allocations.saturating_sub(self.deallocations);
        if active_allocations > 0 && self.current_usage > 0 {
            let avg_active_size = self.current_usage / active_allocations as u64;
            if avg_active_size > 0 {
                (self.avg_allocation_size as f64 - avg_active_size as f64).abs()
                    / avg_active_size as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

/// Overall memory statistics summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverallStats {
    /// Total bytes allocated across all components.
    pub total_allocated: u64,
    /// Total bytes deallocated across all components.
    pub total_deallocated: u64,
    /// Current total memory usage.
    pub current_usage: u64,
    /// Peak total memory usage.
    pub peak_usage: u64,
    /// Number of tracked components.
    pub component_count: usize,
}

impl OverallStats {
    /// Computes the overall memory efficiency.
    #[must_use]
    pub fn efficiency(&self) -> f64 {
        if self.peak_usage > 0 {
            self.current_usage as f64 / self.peak_usage as f64
        } else {
            0.0
        }
    }
}

/// Histogram of allocation sizes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeHistogram {
    /// Buckets: 0-64, 65-256, 257-1KB, 1KB-4KB, 4KB-64KB, 64KB-1MB, 1MB+
    pub buckets: Vec<HistogramBucket>,
}

impl SizeHistogram {
    /// Creates a new size histogram with standard buckets.
    #[must_use]
    pub fn new() -> Self {
        let buckets = vec![
            HistogramBucket::new(0, 64),
            HistogramBucket::new(65, 256),
            HistogramBucket::new(257, 1024),
            HistogramBucket::new(1025, 4096),
            HistogramBucket::new(4097, 65536),
            HistogramBucket::new(65537, 1_048_576),
            HistogramBucket::new(1_048_577, u64::MAX),
        ];

        Self { buckets }
    }

    /// Records an allocation of the given size.
    pub fn record(&mut self, size: u64) {
        for bucket in &mut self.buckets {
            if size >= bucket.min && size <= bucket.max {
                bucket.count += 1;
                bucket.total_bytes += size;
                break;
            }
        }
    }

    /// Gets the most common allocation size range.
    #[must_use]
    pub fn most_common_range(&self) -> Option<(u64, u64, u64)> {
        self.buckets
            .iter()
            .max_by_key(|b| b.count)
            .map(|b| (b.min, b.max, b.count))
    }
}

impl Default for SizeHistogram {
    fn default() -> Self {
        Self::new()
    }
}

/// A single histogram bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    /// Minimum size (inclusive).
    pub min: u64,
    /// Maximum size (inclusive).
    pub max: u64,
    /// Number of allocations in this range.
    pub count: u64,
    /// Total bytes in this range.
    pub total_bytes: u64,
}

impl HistogramBucket {
    /// Creates a new histogram bucket.
    #[must_use]
    pub const fn new(min: u64, max: u64) -> Self {
        Self {
            min,
            max,
            count: 0,
            total_bytes: 0,
        }
    }

    /// Gets the average allocation size in this bucket.
    #[must_use]
    pub const fn avg_size(&self) -> u64 {
        if self.count > 0 {
            self.total_bytes / self.count
        } else {
            0
        }
    }
}

/// A time-series data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePoint {
    /// Label for this time point.
    pub label: String,
    /// Timestamp when recorded (serialized as duration since epoch in seconds).
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    /// Memory snapshot at this point.
    pub snapshot: MemorySnapshot,
}

/// A builder for collecting detailed statistics.
#[derive(Debug)]
pub struct StatsCollector {
    stats: DetailedStats,
    start_time: Instant,
}

impl StatsCollector {
    /// Creates a new stats collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: DetailedStats::new(),
            start_time: Instant::now(),
        }
    }

    /// Adds a component with its memory snapshot.
    pub fn add_component(&mut self, name: impl Into<String>, snapshot: MemorySnapshot) {
        let name = name.into();
        let stats = ComponentStats::from_snapshot(&name, snapshot);
        self.stats.add_component(name, stats);
    }

    /// Records a timeline point.
    pub fn record_point(&mut self, label: impl Into<String>, snapshot: MemorySnapshot) {
        self.stats.record_timepoint(label, snapshot);
    }

    /// Finalizes and returns the detailed statistics.
    #[must_use]
    pub fn finish(mut self) -> DetailedStats {
        self.stats.compute_overall();
        self.stats
    }

    /// Gets the elapsed time since collection started.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_stats_efficiency() {
        let snapshot = MemorySnapshot {
            allocations: 10,
            deallocations: 5,
            total_allocated: 1000,
            total_deallocated: 500,
            current_usage: 500,
            peak_usage: 800,
        };

        let stats = ComponentStats::from_snapshot("test", snapshot);
        assert!((stats.efficiency() - 500.0 / 800.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_size_histogram() {
        let mut histogram = SizeHistogram::new();

        histogram.record(32); // 0-64
        histogram.record(128); // 65-256
        histogram.record(512); // 257-1024
        histogram.record(2048); // 1KB-4KB

        assert_eq!(histogram.buckets[0].count, 1);
        assert_eq!(histogram.buckets[1].count, 1);
        assert_eq!(histogram.buckets[2].count, 1);
        assert_eq!(histogram.buckets[3].count, 1);
    }

    #[test]
    fn test_detailed_stats() {
        let mut stats = DetailedStats::new();

        let snapshot1 = MemorySnapshot {
            allocations: 5,
            deallocations: 2,
            total_allocated: 500,
            total_deallocated: 200,
            current_usage: 300,
            peak_usage: 400,
        };

        let snapshot2 = MemorySnapshot {
            allocations: 3,
            deallocations: 1,
            total_allocated: 300,
            total_deallocated: 100,
            current_usage: 200,
            peak_usage: 250,
        };

        stats.add_component(
            "component1",
            ComponentStats::from_snapshot("component1", snapshot1),
        );
        stats.add_component(
            "component2",
            ComponentStats::from_snapshot("component2", snapshot2),
        );
        stats.compute_overall();

        assert_eq!(stats.overall.component_count, 2);
        assert_eq!(stats.overall.total_allocated, 800);
        assert_eq!(stats.overall.current_usage, 500);
    }
}
