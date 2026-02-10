//! Custom allocator for tracking memory allocations.
//!
//! This module provides a tracking allocator that wraps the system allocator
//! and records detailed statistics about memory allocations and deallocations.

use serde::{Deserialize, Serialize};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Thread-safe memory tracking statistics.
#[derive(Debug, Default)]
pub struct MemoryStats {
    /// Total number of allocations.
    pub allocations: AtomicUsize,
    /// Total number of deallocations.
    pub deallocations: AtomicUsize,
    /// Total bytes allocated.
    pub total_allocated: AtomicU64,
    /// Total bytes deallocated.
    pub total_deallocated: AtomicU64,
    /// Current bytes in use.
    pub current_usage: AtomicU64,
    /// Peak memory usage.
    pub peak_usage: AtomicU64,
}

impl MemoryStats {
    /// Creates a new `MemoryStats` instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            total_allocated: AtomicU64::new(0),
            total_deallocated: AtomicU64::new(0),
            current_usage: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
        }
    }

    /// Records an allocation.
    pub fn record_allocation(&self, size: usize) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        let size_u64 = size as u64;
        self.total_allocated.fetch_add(size_u64, Ordering::Relaxed);

        let current = self.current_usage.fetch_add(size_u64, Ordering::Relaxed) + size_u64;

        // Update peak usage if necessary
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_usage.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
    }

    /// Records a deallocation.
    pub fn record_deallocation(&self, size: usize) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        let size_u64 = size as u64;
        self.total_deallocated
            .fetch_add(size_u64, Ordering::Relaxed);
        self.current_usage.fetch_sub(size_u64, Ordering::Relaxed);
    }

    /// Resets all statistics to zero.
    pub fn reset(&self) {
        self.allocations.store(0, Ordering::Relaxed);
        self.deallocations.store(0, Ordering::Relaxed);
        self.total_allocated.store(0, Ordering::Relaxed);
        self.total_deallocated.store(0, Ordering::Relaxed);
        self.current_usage.store(0, Ordering::Relaxed);
        self.peak_usage.store(0, Ordering::Relaxed);
    }

    /// Gets a snapshot of current statistics.
    #[must_use]
    pub fn snapshot(&self) -> MemorySnapshot {
        MemorySnapshot {
            allocations: self.allocations.load(Ordering::Relaxed),
            deallocations: self.deallocations.load(Ordering::Relaxed),
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_deallocated: self.total_deallocated.load(Ordering::Relaxed),
            current_usage: self.current_usage.load(Ordering::Relaxed),
            peak_usage: self.peak_usage.load(Ordering::Relaxed),
        }
    }
}

/// A point-in-time snapshot of memory statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Total number of allocations.
    pub allocations: usize,
    /// Total number of deallocations.
    pub deallocations: usize,
    /// Total bytes allocated.
    pub total_allocated: u64,
    /// Total bytes deallocated.
    pub total_deallocated: u64,
    /// Current bytes in use.
    pub current_usage: u64,
    /// Peak memory usage.
    pub peak_usage: u64,
}

impl MemorySnapshot {
    /// Computes the difference between two snapshots.
    #[must_use]
    pub fn diff(&self, earlier: &Self) -> Self {
        Self {
            allocations: self.allocations.saturating_sub(earlier.allocations),
            deallocations: self.deallocations.saturating_sub(earlier.deallocations),
            total_allocated: self.total_allocated.saturating_sub(earlier.total_allocated),
            total_deallocated: self
                .total_deallocated
                .saturating_sub(earlier.total_deallocated),
            current_usage: self.current_usage,
            peak_usage: self.peak_usage.max(earlier.peak_usage),
        }
    }
}

/// Global memory statistics instance.
static MEMORY_STATS: MemoryStats = MemoryStats::new();

/// Custom tracking allocator.
pub struct TrackingAllocator<A: GlobalAlloc = System> {
    inner: A,
}

impl<A: GlobalAlloc> TrackingAllocator<A> {
    /// Creates a new tracking allocator wrapping the given allocator.
    #[must_use]
    pub const fn new(inner: A) -> Self {
        Self { inner }
    }
}

impl Default for TrackingAllocator<System> {
    fn default() -> Self {
        Self::new(System)
    }
}

// SAFETY: This implementation forwards all allocation requests to the inner
// allocator and records statistics. The inner allocator (System) is safe.
unsafe impl<A: GlobalAlloc> GlobalAlloc for TrackingAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            MEMORY_STATS.record_allocation(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        MEMORY_STATS.record_deallocation(layout.size());
        self.inner.dealloc(ptr, layout);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc_zeroed(layout);
        if !ptr.is_null() {
            MEMORY_STATS.record_allocation(layout.size());
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = self.inner.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            MEMORY_STATS.record_deallocation(layout.size());
            MEMORY_STATS.record_allocation(new_size);
        }
        new_ptr
    }
}

/// Gets the global memory statistics.
#[must_use]
pub fn get_stats() -> &'static MemoryStats {
    &MEMORY_STATS
}

/// Gets a snapshot of current memory statistics.
#[must_use]
pub fn snapshot() -> MemorySnapshot {
    MEMORY_STATS.snapshot()
}

/// Resets all memory statistics.
pub fn reset_stats() {
    MEMORY_STATS.reset();
}

/// A RAII guard for tracking memory usage within a scope.
pub struct MemoryGuard {
    start: MemorySnapshot,
    label: String,
}

impl MemoryGuard {
    /// Creates a new memory guard with the given label.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            start: snapshot(),
            label: label.into(),
        }
    }

    /// Gets the current memory usage since guard creation.
    #[must_use]
    pub fn current(&self) -> MemorySnapshot {
        snapshot().diff(&self.start)
    }

    /// Gets the label for this guard.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Drop for MemoryGuard {
    fn drop(&mut self) {
        let end = snapshot();
        let diff = end.diff(&self.start);

        // Only log if there's significant activity
        if diff.allocations > 0 || diff.deallocations > 0 {
            eprintln!(
                "[{}] Memory: {} allocations, {} deallocations, {} bytes net change",
                self.label, diff.allocations, diff.deallocations, diff.current_usage as i64
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_basic() {
        let stats = MemoryStats::new();

        stats.record_allocation(1024);
        assert_eq!(stats.allocations.load(Ordering::Relaxed), 1);
        assert_eq!(stats.total_allocated.load(Ordering::Relaxed), 1024);
        assert_eq!(stats.current_usage.load(Ordering::Relaxed), 1024);
        assert_eq!(stats.peak_usage.load(Ordering::Relaxed), 1024);

        stats.record_deallocation(512);
        assert_eq!(stats.deallocations.load(Ordering::Relaxed), 1);
        assert_eq!(stats.current_usage.load(Ordering::Relaxed), 512);
        assert_eq!(stats.peak_usage.load(Ordering::Relaxed), 1024);
    }

    #[test]
    fn test_snapshot_diff() {
        let stats = MemoryStats::new();

        let snap1 = stats.snapshot();
        stats.record_allocation(1024);
        stats.record_allocation(2048);
        let snap2 = stats.snapshot();

        let diff = snap2.diff(&snap1);
        assert_eq!(diff.allocations, 2);
        assert_eq!(diff.total_allocated, 3072);
    }

    #[test]
    #[ignore = "Requires global allocator to be installed"]
    fn test_memory_guard() {
        reset_stats();

        {
            let guard = MemoryGuard::new("test");
            let _v = vec![0u8; 1024];

            let current = guard.current();
            assert!(current.allocations > 0);
        }
    }
}
