//! jemalloc-based memory profiling for production environments.
//!
//! This module provides integration with jemalloc's built-in profiling
//! capabilities via `tikv-jemalloc-ctl`. It offers:
//!
//! - Real-time memory statistics
//! - Heap profiling and dump generation
//! - Memory fragmentation analysis
//! - Arena statistics
//!
//! # Feature Flag
//!
//! This module requires the `jemalloc` feature to be enabled:
//!
//! ```toml
//! [dependencies]
//! mecab-ko-profiler = { version = "0.6", features = ["jemalloc"] }
//! ```
//!
//! # Examples
//!
//! ```rust,ignore
//! use mecab_ko_profiler::jemalloc_profiler::JemallocProfiler;
//!
//! let profiler = JemallocProfiler::new();
//!
//! // Get current memory statistics
//! let stats = profiler.stats()?;
//! println!("Allocated: {} bytes", stats.allocated);
//! println!("Resident: {} bytes", stats.resident);
//!
//! // Take a heap dump
//! profiler.dump_heap("heap_profile.out")?;
//! ```

use serde::{Deserialize, Serialize};
use std::io;
use tikv_jemalloc_ctl::epoch;
use tikv_jemalloc_ctl::stats::{active, allocated, mapped, metadata, resident, retained};

/// jemalloc memory profiler.
#[derive(Debug)]
pub struct JemallocProfiler {
    _private: (),
}

impl JemallocProfiler {
    /// Creates a new jemalloc profiler.
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Refreshes jemalloc statistics.
    ///
    /// Call this before reading stats to ensure you get current values.
    ///
    /// # Errors
    ///
    /// Returns an error if the epoch advance fails.
    pub fn refresh(&self) -> Result<(), JemallocError> {
        epoch::advance().map_err(|e| JemallocError::Ctl(format!("{e:?}")))?;
        Ok(())
    }

    /// Gets current jemalloc memory statistics.
    ///
    /// # Errors
    ///
    /// Returns an error if reading stats fails.
    pub fn stats(&self) -> Result<JemallocStats, JemallocError> {
        self.refresh()?;

        let allocated_val = allocated::read().map_err(|e| JemallocError::Ctl(format!("{e:?}")))?;
        let active_val = active::read().map_err(|e| JemallocError::Ctl(format!("{e:?}")))?;
        let metadata_val = metadata::read().map_err(|e| JemallocError::Ctl(format!("{e:?}")))?;
        let resident_val = resident::read().map_err(|e| JemallocError::Ctl(format!("{e:?}")))?;
        let mapped_val = mapped::read().map_err(|e| JemallocError::Ctl(format!("{e:?}")))?;
        let retained_val = retained::read().map_err(|e| JemallocError::Ctl(format!("{e:?}")))?;

        Ok(JemallocStats {
            allocated: allocated_val,
            active: active_val,
            metadata: metadata_val,
            resident: resident_val,
            mapped: mapped_val,
            retained: retained_val,
        })
    }

    /// Calculates memory fragmentation ratio.
    ///
    /// Fragmentation = (active - allocated) / active
    /// A value close to 0 means low fragmentation.
    ///
    /// # Errors
    ///
    /// Returns an error if reading stats fails.
    pub fn fragmentation_ratio(&self) -> Result<f64, JemallocError> {
        let stats = self.stats()?;
        if stats.active == 0 {
            return Ok(0.0);
        }
        Ok((stats.active - stats.allocated) as f64 / stats.active as f64)
    }

    /// Calculates memory efficiency.
    ///
    /// Efficiency = allocated / resident
    /// A value close to 1 means high efficiency.
    ///
    /// # Errors
    ///
    /// Returns an error if reading stats fails.
    pub fn efficiency(&self) -> Result<f64, JemallocError> {
        let stats = self.stats()?;
        if stats.resident == 0 {
            return Ok(0.0);
        }
        Ok(stats.allocated as f64 / stats.resident as f64)
    }

    /// Dumps heap profile to a file.
    ///
    /// Requires jemalloc to be compiled with profiling support
    /// (`MALLOC_CONF=prof:true`).
    ///
    /// # Errors
    ///
    /// Returns an error if profiling is not enabled or dump fails.
    #[cfg(target_os = "linux")]
    pub fn dump_heap(&self, path: &str) -> Result<(), JemallocError> {
        use std::ffi::CString;
        use tikv_jemalloc_ctl::raw;

        let path_cstr = CString::new(path)
            .map_err(|e| JemallocError::Io(io::Error::new(io::ErrorKind::InvalidInput, e)))?;

        // prof.dump writes the heap profile to the specified path
        let mib = raw::name_to_mib(b"prof.dump\0")
            .map_err(|e| JemallocError::Ctl(format!("Failed to get MIB: {e}")))?;

        raw::write_mib(&mib, path_cstr.as_bytes_with_nul())
            .map_err(|e| JemallocError::Ctl(format!("Failed to dump heap: {e}")))?;

        Ok(())
    }

    /// Dumps heap profile to a file (stub for non-Linux).
    #[cfg(not(target_os = "linux"))]
    pub fn dump_heap(&self, _path: &str) -> Result<(), JemallocError> {
        Err(JemallocError::Ctl(
            "Heap profiling is only supported on Linux".to_string(),
        ))
    }

    /// Enables background thread for purging unused pages.
    ///
    /// # Errors
    ///
    /// Returns an error if setting fails.
    pub fn enable_background_threads(&self) -> Result<(), JemallocError> {
        tikv_jemalloc_ctl::background_thread::write(true)
            .map_err(|e| JemallocError::Ctl(format!("{e:?}")))?;
        Ok(())
    }

    /// Gets the number of arenas.
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
    pub fn arena_count(&self) -> Result<usize, JemallocError> {
        let narenas = tikv_jemalloc_ctl::arenas::narenas::read()
            .map_err(|e| JemallocError::Ctl(format!("{e:?}")))?;
        Ok(narenas as usize)
    }
}

impl Default for JemallocProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// jemalloc memory statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct JemallocStats {
    /// Total bytes allocated by the application.
    pub allocated: usize,
    /// Total bytes in active pages (may include fragmentation).
    pub active: usize,
    /// Total bytes used by jemalloc metadata.
    pub metadata: usize,
    /// Total bytes in resident (physical) memory.
    pub resident: usize,
    /// Total bytes in mapped memory.
    pub mapped: usize,
    /// Total bytes retained (not returned to OS).
    pub retained: usize,
}

impl JemallocStats {
    /// Calculates the overhead (metadata + retained).
    #[must_use]
    pub const fn overhead(&self) -> usize {
        self.metadata + self.retained
    }

    /// Calculates internal fragmentation.
    #[must_use]
    pub const fn internal_fragmentation(&self) -> usize {
        self.active.saturating_sub(self.allocated)
    }

    /// Calculates external fragmentation.
    #[must_use]
    pub const fn external_fragmentation(&self) -> usize {
        self.resident.saturating_sub(self.active)
    }
}

/// Errors from jemalloc operations.
#[derive(Debug, thiserror::Error)]
pub enum JemallocError {
    /// Error from jemalloc control interface.
    #[error("jemalloc ctl error: {0}")]
    Ctl(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

/// RAII guard for measuring memory changes.
pub struct JemallocGuard {
    label: String,
    start_stats: JemallocStats,
    profiler: JemallocProfiler,
}

impl JemallocGuard {
    /// Creates a new jemalloc measurement guard.
    ///
    /// # Errors
    ///
    /// Returns an error if reading initial stats fails.
    pub fn new(label: impl Into<String>) -> Result<Self, JemallocError> {
        let profiler = JemallocProfiler::new();
        let start_stats = profiler.stats()?;
        Ok(Self {
            label: label.into(),
            start_stats,
            profiler,
        })
    }

    /// Gets the memory change since guard creation.
    ///
    /// # Errors
    ///
    /// Returns an error if reading current stats fails.
    pub fn delta(&self) -> Result<JemallocDelta, JemallocError> {
        let current = self.profiler.stats()?;
        Ok(JemallocDelta {
            allocated: current.allocated as i64 - self.start_stats.allocated as i64,
            resident: current.resident as i64 - self.start_stats.resident as i64,
            active: current.active as i64 - self.start_stats.active as i64,
        })
    }

    /// Gets the label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Drop for JemallocGuard {
    fn drop(&mut self) {
        if let Ok(delta) = self.delta() {
            if delta.allocated != 0 || delta.resident != 0 {
                eprintln!(
                    "[{}] Memory delta: allocated={:+}, resident={:+}, active={:+}",
                    self.label, delta.allocated, delta.resident, delta.active
                );
            }
        }
    }
}

/// Memory delta between two points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct JemallocDelta {
    /// Change in allocated bytes.
    pub allocated: i64,
    /// Change in resident bytes.
    pub resident: i64,
    /// Change in active bytes.
    pub active: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jemalloc_profiler_creation() {
        let profiler = JemallocProfiler::new();
        // Just ensure it can be created
        assert!(profiler.stats().is_ok());
    }

    #[test]
    fn test_jemalloc_stats() {
        let profiler = JemallocProfiler::new();
        let stats = profiler.stats().unwrap();

        // Basic sanity checks
        assert!(stats.allocated > 0);
        assert!(stats.resident >= stats.allocated);
    }

    #[test]
    fn test_fragmentation_ratio() {
        let profiler = JemallocProfiler::new();
        let ratio = profiler.fragmentation_ratio().unwrap();

        // Fragmentation should be between 0 and 1
        assert!(ratio >= 0.0);
        assert!(ratio <= 1.0);
    }

    #[test]
    fn test_efficiency() {
        let profiler = JemallocProfiler::new();
        let efficiency = profiler.efficiency().unwrap();

        // Efficiency should be between 0 and 1
        assert!(efficiency >= 0.0);
        assert!(efficiency <= 1.0);
    }

    #[test]
    fn test_jemalloc_guard() {
        let guard = JemallocGuard::new("test").unwrap();

        // Allocate some memory
        let _v: Vec<u8> = vec![0; 1024 * 1024]; // 1MB

        let delta = guard.delta().unwrap();
        // The allocation should show up (though not necessarily exactly 1MB due to allocator overhead)
        assert!(delta.allocated > 0);
    }
}
