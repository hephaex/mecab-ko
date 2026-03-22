//! String pooling utilities for memory-efficient string storage.
//!
//! This module provides a string pool that deduplicates strings, reducing
//! memory usage when many dictionary entries share the same surface forms
//! or features.
//!
//! # Examples
//!
//! ```rust
//! use mecab_ko_dict::string_pool::StringPool;
//!
//! let mut pool = StringPool::new();
//!
//! // Intern strings - identical strings return the same Arc
//! let s1 = pool.intern("안녕하세요");
//! let s2 = pool.intern("안녕하세요");
//!
//! // s1 and s2 point to the same allocation
//! assert!(std::sync::Arc::ptr_eq(&s1, &s2));
//!
//! // Check pool statistics
//! assert_eq!(pool.len(), 1);
//! ```

use std::collections::HashMap;
use std::sync::Arc;

/// A string pool that deduplicates strings using reference counting.
///
/// Strings are stored as `Arc<str>`, allowing multiple references to
/// the same string without duplication. The pool maintains a weak
/// reference to each string, allowing unused strings to be garbage
/// collected.
#[derive(Debug, Default)]
pub struct StringPool {
    /// Interned strings (strong references).
    strings: HashMap<Arc<str>, ()>,
}

impl StringPool {
    /// Creates a new empty string pool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
        }
    }

    /// Creates a string pool with the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            strings: HashMap::with_capacity(capacity),
        }
    }

    /// Interns a string, returning a reference-counted handle.
    ///
    /// If the string already exists in the pool, returns a clone of
    /// the existing `Arc<str>`. Otherwise, creates a new entry.
    pub fn intern(&mut self, s: &str) -> Arc<str> {
        // Check if already interned
        if let Some((existing, _)) = self.strings.get_key_value(s) {
            return Arc::clone(existing);
        }

        // Create new interned string
        let arc: Arc<str> = Arc::from(s);
        self.strings.insert(Arc::clone(&arc), ());
        arc
    }

    /// Interns a string from an owned String.
    ///
    /// This is more efficient than `intern` when you already have a String,
    /// as it avoids an extra allocation if the string is not already pooled.
    pub fn intern_string(&mut self, s: String) -> Arc<str> {
        // Check if already interned
        if let Some((existing, _)) = self.strings.get_key_value(s.as_str()) {
            return Arc::clone(existing);
        }

        // Create new interned string from owned String
        let arc: Arc<str> = Arc::from(s);
        self.strings.insert(Arc::clone(&arc), ());
        arc
    }

    /// Returns the number of unique strings in the pool.
    #[must_use]
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Returns true if the pool is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Clears all strings from the pool.
    pub fn clear(&mut self) {
        self.strings.clear();
    }

    /// Returns the memory usage of the pool in bytes (approximate).
    ///
    /// This includes the HashMap overhead and the string data.
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();

        // HashMap overhead (approximate)
        total += self.strings.capacity() * std::mem::size_of::<(Arc<str>, ())>();

        // String data
        for (s, _) in &self.strings {
            total += std::mem::size_of::<Arc<str>>() + s.len();
        }

        total
    }

    /// Returns statistics about the string pool.
    #[must_use]
    pub fn stats(&self) -> StringPoolStats {
        let mut total_string_bytes = 0;
        let mut min_len = usize::MAX;
        let mut max_len = 0;

        for (s, _) in &self.strings {
            let len = s.len();
            total_string_bytes += len;
            min_len = min_len.min(len);
            max_len = max_len.max(len);
        }

        if self.strings.is_empty() {
            min_len = 0;
        }

        StringPoolStats {
            count: self.strings.len(),
            total_bytes: total_string_bytes,
            avg_length: if self.strings.is_empty() {
                0.0
            } else {
                total_string_bytes as f64 / self.strings.len() as f64
            },
            min_length: min_len,
            max_length: max_len,
            memory_usage: self.memory_usage(),
        }
    }
}

/// Statistics about a string pool.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StringPoolStats {
    /// Number of unique strings.
    pub count: usize,
    /// Total bytes of string data.
    pub total_bytes: usize,
    /// Average string length.
    pub avg_length: f64,
    /// Minimum string length.
    pub min_length: usize,
    /// Maximum string length.
    pub max_length: usize,
    /// Total memory usage (approximate).
    pub memory_usage: usize,
}

/// Thread-safe string pool using a concurrent hashmap.
///
/// This version uses a parking_lot RwLock for thread-safe access.
/// Suitable for multi-threaded dictionary building.
#[derive(Debug, Default)]
pub struct ConcurrentStringPool {
    inner: std::sync::RwLock<StringPool>,
}

impl ConcurrentStringPool {
    /// Creates a new empty concurrent string pool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: std::sync::RwLock::new(StringPool::new()),
        }
    }

    /// Creates a concurrent string pool with the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: std::sync::RwLock::new(StringPool::with_capacity(capacity)),
        }
    }

    /// Interns a string, returning a reference-counted handle.
    pub fn intern(&self, s: &str) -> Arc<str> {
        // First try read lock to check if exists
        {
            let pool = self.inner.read().unwrap();
            if let Some((existing, _)) = pool.strings.get_key_value(s) {
                return Arc::clone(existing);
            }
        }

        // Need write lock to insert
        let mut pool = self.inner.write().unwrap();
        pool.intern(s)
    }

    /// Returns the number of unique strings in the pool.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    /// Returns true if the pool is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.read().unwrap().is_empty()
    }

    /// Returns statistics about the string pool.
    #[must_use]
    pub fn stats(&self) -> StringPoolStats {
        self.inner.read().unwrap().stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool_basic() {
        let mut pool = StringPool::new();

        let s1 = pool.intern("hello");
        let s2 = pool.intern("hello");
        let s3 = pool.intern("world");

        // Same strings should return the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));

        // Different strings should be different
        assert!(!Arc::ptr_eq(&s1, &s3));

        // Pool should have 2 unique strings
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_string_pool_korean() {
        let mut pool = StringPool::new();

        let s1 = pool.intern("안녕하세요");
        let s2 = pool.intern("안녕하세요");
        let s3 = pool.intern("감사합니다");

        assert!(Arc::ptr_eq(&s1, &s2));
        assert!(!Arc::ptr_eq(&s1, &s3));
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_string_pool_stats() {
        let mut pool = StringPool::new();

        pool.intern("a");
        pool.intern("bb");
        pool.intern("ccc");

        let stats = pool.stats();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_bytes, 6); // 1 + 2 + 3
        assert!((stats.avg_length - 2.0).abs() < f64::EPSILON);
        assert_eq!(stats.min_length, 1);
        assert_eq!(stats.max_length, 3);
    }

    #[test]
    fn test_string_pool_intern_string() {
        let mut pool = StringPool::new();

        let owned = String::from("test");
        let s1 = pool.intern_string(owned);
        let s2 = pool.intern("test");

        assert!(Arc::ptr_eq(&s1, &s2));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_concurrent_string_pool() {
        let pool = ConcurrentStringPool::new();

        let s1 = pool.intern("test");
        let s2 = pool.intern("test");

        assert!(Arc::ptr_eq(&s1, &s2));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_string_pool_memory_usage() {
        let mut pool = StringPool::new();

        let initial = pool.memory_usage();

        pool.intern("a short string");
        pool.intern("another string");

        let after = pool.memory_usage();

        // Memory usage should increase
        assert!(after > initial);
    }

    #[test]
    fn test_string_pool_clear() {
        let mut pool = StringPool::new();

        pool.intern("test1");
        pool.intern("test2");
        assert_eq!(pool.len(), 2);

        pool.clear();
        assert!(pool.is_empty());
    }
}
