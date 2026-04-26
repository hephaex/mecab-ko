//! Wait-free hot-reload using [`arc_swap::ArcSwap`].
//!
//! ## Why `ArcSwap` over `RwLock`
//!
//! `RwLock<Arc<T>>` still requires acquiring the lock on every read, which
//! causes contention under high read concurrency. `ArcSwap` uses lock-free
//! pointer swaps: readers only increment a reference count, writers atomically
//! publish a new `Arc`. This gives truly wait-free reads at the cost of a
//! slightly more expensive write path.
//!
//! ## Usage
//!
//! ```rust
//! use mecab_ko_dict::hot_reload_v2::HotReloadDictV2;
//! use mecab_ko_dict::domain::DomainStack;
//!
//! let hr = HotReloadDictV2::new(DomainStack::new());
//! let snap = hr.load();
//! assert_eq!(snap.version, 1);
//!
//! let new_version = hr.update(|_stack| DomainStack::new());
//! assert_eq!(new_version, 2);
//! ```

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use arc_swap::ArcSwap;

use crate::domain::DomainStack;

/// Maximum history entries retained by default before old snapshots are evicted.
const DEFAULT_MAX_HISTORY: usize = 10;

/// A frozen view of the domain stack at a particular version.
#[derive(Debug)]
pub struct DictionarySnapshot {
    /// Monotonically increasing version counter, starting at 1.
    pub version: u64,
    /// The domain stack captured at this version.
    pub domain_stack: Arc<DomainStack>,
    /// Wall-clock time when this snapshot was created.
    pub timestamp: SystemTime,
}

impl DictionarySnapshot {
    fn new(version: u64, domain_stack: DomainStack) -> Arc<Self> {
        Arc::new(Self {
            version,
            domain_stack: Arc::new(domain_stack),
            timestamp: SystemTime::now(),
        })
    }
}

/// Hot-reload container backed by `ArcSwap`.
///
/// Reads are wait-free: `load()` performs only a single atomic pointer load.
/// Writes acquire a `Mutex` to serialise concurrent updates and maintain the
/// version counter.
pub struct HotReloadDictV2 {
    /// The currently active snapshot, readable without any lock.
    current: ArcSwap<DictionarySnapshot>,
    /// Guards the write path; also owns the history queue and next version.
    write_state: Mutex<WriteState>,
}

struct WriteState {
    /// History queue for rollback, newest-last.
    history: VecDeque<Arc<DictionarySnapshot>>,
    /// The next version number to assign on write.
    next_version: u64,
    /// Maximum number of past snapshots to keep.
    max_history: usize,
}

impl WriteState {
    // VecDeque::new() is not const-stable on MSRV 1.80, so we suppress the lint.
    #[allow(clippy::missing_const_for_fn)]
    fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::new(),
            next_version: 2, // version 1 is the initial snapshot
            max_history,
        }
    }
}

impl HotReloadDictV2 {
    /// Create a new instance with the given initial domain stack.
    ///
    /// The initial snapshot is assigned version 1.
    #[must_use]
    pub fn new(initial: DomainStack) -> Self {
        let snapshot = DictionarySnapshot::new(1, initial);
        Self {
            current: ArcSwap::from(snapshot),
            write_state: Mutex::new(WriteState::new(DEFAULT_MAX_HISTORY)),
        }
    }

    /// Create a new instance with a custom maximum history depth.
    #[must_use]
    pub fn with_max_history(initial: DomainStack, max_history: usize) -> Self {
        let snapshot = DictionarySnapshot::new(1, initial);
        Self {
            current: ArcSwap::from(snapshot),
            write_state: Mutex::new(WriteState::new(max_history)),
        }
    }

    /// Load the current snapshot.
    ///
    /// This is the hot path: no locks are taken, only an atomic pointer load.
    /// The returned guard keeps the snapshot alive until it is dropped.
    pub fn load(&self) -> arc_swap::Guard<Arc<DictionarySnapshot>> {
        self.current.load()
    }

    /// Return the version number of the current snapshot.
    #[must_use]
    pub fn current_version(&self) -> u64 {
        self.current.load().version
    }

    /// Apply `update_fn` to the current domain stack and publish the result.
    ///
    /// The update is serialised through an internal `Mutex` so concurrent
    /// writers do not race. Readers are never blocked.
    ///
    /// # Returns
    ///
    /// The version number assigned to the new snapshot.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (i.e., another thread panicked
    /// while holding it). This is consistent with the rest of the crate's
    /// approach to mutex poisoning.
    // Mutex::lock().expect() is intentional: a poisoned mutex means a thread
    // panicked while holding the write lock, leaving shared state corrupted.
    // Propagating the panic is the only safe response.
    #[allow(clippy::expect_used, clippy::significant_drop_tightening)]
    pub fn update<F>(&self, update_fn: F) -> u64
    where
        F: FnOnce(&DomainStack) -> DomainStack,
    {
        let mut state = self.write_state.lock().expect("write_state mutex poisoned");

        let old_snapshot = self.current.load_full();
        let new_stack = update_fn(&old_snapshot.domain_stack);
        let version = state.next_version;
        state.next_version += 1;

        let new_snapshot = DictionarySnapshot::new(version, new_stack);

        // Push old snapshot to history before replacing it.
        state.history.push_back(Arc::clone(&old_snapshot));
        while state.history.len() > state.max_history {
            state.history.pop_front();
        }
        drop(state);

        self.current.store(Arc::clone(&new_snapshot));
        version
    }

    /// Restore the most recent previous snapshot.
    ///
    /// If there is no history, returns `None`. Otherwise returns the version
    /// number that became current after the rollback.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    // Same reasoning as `update` — poisoned mutex indicates corrupted state.
    #[allow(clippy::expect_used, clippy::significant_drop_tightening)]
    pub fn rollback(&self) -> Option<u64> {
        let mut state = self.write_state.lock().expect("write_state mutex poisoned");

        let previous = state.history.pop_back()?;
        let restored_version = previous.version;
        drop(state);

        self.current.store(Arc::clone(&previous));
        Some(restored_version)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;
    use std::thread;

    use super::*;
    use crate::domain::DomainId;
    use crate::user_dict::UserDictionary;

    fn empty_stack() -> DomainStack {
        DomainStack::new()
    }

    fn stack_with_entry(surface: &str) -> DomainStack {
        let mut dict = UserDictionary::new();
        dict.add_entry(surface, "NNG", Some(-1000), None);

        let mut stack = DomainStack::new();
        stack.add_domain(DomainId("test".into()), 0, Arc::new(dict), None);
        stack
    }

    #[test]
    fn test_initial_snapshot_version_is_one() {
        let hr = HotReloadDictV2::new(empty_stack());
        assert_eq!(hr.current_version(), 1);
    }

    #[test]
    fn test_load_returns_current_snapshot() {
        let hr = HotReloadDictV2::new(stack_with_entry("테스트"));
        let snap = hr.load();
        assert_eq!(snap.version, 1);
        let results = snap.domain_stack.lookup("테스트");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_update_increments_version_and_swaps_stack() {
        let hr = HotReloadDictV2::new(empty_stack());

        let v2 = hr.update(|_| stack_with_entry("뉴스"));
        assert_eq!(v2, 2);
        assert_eq!(hr.current_version(), 2);

        let snap = hr.load();
        assert!(!snap.domain_stack.lookup("뉴스").is_empty());
    }

    #[test]
    fn test_multiple_updates_produce_monotonic_versions() {
        let hr = HotReloadDictV2::new(empty_stack());

        let v2 = hr.update(|_| empty_stack());
        let v3 = hr.update(|_| empty_stack());
        let v4 = hr.update(|_| empty_stack());

        assert_eq!(v2, 2);
        assert_eq!(v3, 3);
        assert_eq!(v4, 4);
    }

    #[test]
    fn test_rollback_restores_previous_version() {
        let hr = HotReloadDictV2::new(stack_with_entry("원본"));

        hr.update(|_| stack_with_entry("수정됨"));
        assert_eq!(hr.current_version(), 2);

        let rolled_back = hr.rollback();
        assert_eq!(rolled_back, Some(1));
        assert_eq!(hr.current_version(), 1);

        let snap = hr.load();
        // After rollback, the original entry must be present again.
        assert!(!snap.domain_stack.lookup("원본").is_empty());
        // The modified entry must be gone.
        assert!(snap.domain_stack.lookup("수정됨").is_empty());
    }

    #[test]
    fn test_rollback_on_empty_history_returns_none() {
        let hr = HotReloadDictV2::new(empty_stack());
        assert_eq!(hr.rollback(), None);
        // Version must be unchanged.
        assert_eq!(hr.current_version(), 1);
    }

    #[test]
    fn test_concurrent_reads_during_update_do_not_panic() {
        let hr = Arc::new(HotReloadDictV2::new(empty_stack()));

        // Spawn multiple reader threads.
        let readers: Vec<_> = (0..8)
            .map(|_| {
                let hr = Arc::clone(&hr);
                thread::spawn(move || {
                    for _ in 0..1000 {
                        let snap = hr.load();
                        // Just touch the version to force the load.
                        let _ = snap.version;
                    }
                })
            })
            .collect();

        // Writer thread in parallel.
        let writer = {
            let hr = Arc::clone(&hr);
            thread::spawn(move || {
                for i in 0..20u64 {
                    hr.update(|_| {
                        let mut d = UserDictionary::new();
                        d.add_entry(format!("단어{i}"), "NNG", Some(-1000), None);
                        let mut s = DomainStack::new();
                        s.add_domain(DomainId("t".into()), 0, Arc::new(d), None);
                        s
                    });
                }
            })
        };

        for r in readers {
            r.join().expect("reader thread panicked");
        }
        writer.join().expect("writer thread panicked");
    }
}
