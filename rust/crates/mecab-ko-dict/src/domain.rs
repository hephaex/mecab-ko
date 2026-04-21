//! Domain overlay dictionary: stack multiple [`UserDictionary`] instances by priority.
//!
//! ## Design
//!
//! Multiple dictionaries can coexist with explicit priority ordering (0 = highest).
//! Searches across all domains are performed in priority order; higher-priority
//! domain entries appear first in results. This enables domain-specific vocabulary
//! to shadow or augment lower-priority entries without merging the underlying data.
//!
//! ## Example
//!
//! ```rust
//! use std::sync::Arc;
//! use mecab_ko_dict::domain::{DomainId, DomainStack};
//! use mecab_ko_dict::user_dict::UserDictionary;
//!
//! let mut stack = DomainStack::new();
//!
//! let mut news = UserDictionary::new();
//! news.add_entry("뉴스피드", "NNG", Some(-1000), None);
//!
//! let mut finance = UserDictionary::new();
//! finance.add_entry("코스피", "NNP", Some(-1000), None);
//!
//! stack.add_domain(DomainId("news".into()), 0, Arc::new(news), None);
//! stack.add_domain(DomainId("finance".into()), 1, Arc::new(finance), None);
//!
//! assert_eq!(stack.len(), 2);
//! ```

use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use crate::user_dict::{UserDictionary, UserEntry};

/// Opaque identifier for a domain.
///
/// Equality and hashing use the inner string, so two `DomainId`s with the
/// same string value are considered the same domain.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DomainId(pub String);

impl std::fmt::Display for DomainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A single domain overlay paired with its priority and metadata.
pub struct DomainDictionary {
    /// Domain identifier.
    pub domain: DomainId,
    /// Search priority — 0 is the highest (searched first).
    pub priority: u8,
    /// The underlying dictionary, shared via reference-counting.
    pub dictionary: Arc<UserDictionary>,
    /// Optional path from which the dictionary was loaded.
    pub source_path: Option<PathBuf>,
    /// Wall-clock time at which this domain was registered.
    pub loaded_at: SystemTime,
}

impl std::fmt::Debug for DomainDictionary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomainDictionary")
            .field("domain", &self.domain)
            .field("priority", &self.priority)
            .field("entry_count", &self.dictionary.len())
            .field("source_path", &self.source_path)
            .field("loaded_at", &self.loaded_at)
            .finish()
    }
}

impl DomainDictionary {
    fn new(
        domain: DomainId,
        priority: u8,
        dictionary: Arc<UserDictionary>,
        source_path: Option<PathBuf>,
    ) -> Self {
        Self {
            domain,
            priority,
            dictionary,
            source_path,
            loaded_at: SystemTime::now(),
        }
    }
}

/// Ordered stack of domain dictionaries searched in priority order.
///
/// The internal vector is kept sorted by `priority` ascending (lowest value =
/// highest priority) at all times. `add_domain` and `remove_domain` preserve
/// this invariant.
#[derive(Debug, Default)]
pub struct DomainStack {
    // Invariant: sorted by `priority` ascending.
    domains: Vec<DomainDictionary>,
}

impl DomainStack {
    /// Create an empty domain stack.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or replace a domain.
    ///
    /// If a domain with the same `DomainId` already exists it is replaced with
    /// the new dictionary and priority. The stack remains sorted after the
    /// operation.
    pub fn add_domain(
        &mut self,
        domain: DomainId,
        priority: u8,
        dict: Arc<UserDictionary>,
        source: Option<PathBuf>,
    ) {
        // Remove any existing entry with the same id so replacement is atomic.
        self.domains.retain(|d| d.domain != domain);

        let entry = DomainDictionary::new(domain, priority, dict, source);
        self.domains.push(entry);
        // Stable sort keeps equal-priority domains in insertion order.
        self.domains.sort_by_key(|d| d.priority);
    }

    /// Remove a domain by id.
    ///
    /// Returns the removed `DomainDictionary`, or `None` if no domain with the
    /// given id existed.
    pub fn remove_domain(&mut self, domain: &DomainId) -> Option<DomainDictionary> {
        if let Some(pos) = self.domains.iter().position(|d| &d.domain == domain) {
            Some(self.domains.remove(pos))
        } else {
            None
        }
    }

    /// Look up a domain by id.
    #[must_use]
    pub fn get_domain(&self, domain: &DomainId) -> Option<&DomainDictionary> {
        self.domains.iter().find(|d| &d.domain == domain)
    }

    /// Return `(DomainId, priority, entry_count)` for every registered domain,
    /// in priority order (highest priority first).
    #[must_use]
    pub fn list_domains(&self) -> Vec<(DomainId, u8, usize)> {
        self.domains
            .iter()
            .map(|d| (d.domain.clone(), d.priority, d.dictionary.len()))
            .collect()
    }

    /// Number of registered domains.
    #[must_use]
    pub fn len(&self) -> usize {
        self.domains.len()
    }

    /// True when no domains are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.domains.is_empty()
    }

    /// Common-prefix search across all domains.
    ///
    /// Returns all matching `UserEntry` references in priority order (higher
    /// priority = lower numeric value appears first). Within the same domain,
    /// entry order follows the domain's own iteration order.
    ///
    /// The returned references are valid for the lifetime of `&self`.
    #[must_use]
    pub fn common_prefix_search<'a>(&'a self, text: &str) -> Vec<&'a UserEntry> {
        self.domains
            .iter()
            .flat_map(|d| d.dictionary.common_prefix_search(text))
            .collect()
    }

    /// Exact surface lookup across all domains.
    ///
    /// Returns all `UserEntry` references whose surface equals `surface`,
    /// in priority order.
    ///
    /// The returned references are valid for the lifetime of `&self`.
    #[must_use]
    pub fn lookup<'a>(&'a self, surface: &str) -> Vec<&'a UserEntry> {
        self.domains
            .iter()
            .flat_map(|d| d.dictionary.lookup(surface))
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_dict(entries: &[(&str, &str, i16)]) -> Arc<UserDictionary> {
        let mut d = UserDictionary::new();
        for &(surface, pos, cost) in entries {
            d.add_entry(surface, pos, Some(cost), None);
        }
        Arc::new(d)
    }

    #[test]
    fn test_empty_stack() {
        let stack = DomainStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert!(stack.list_domains().is_empty());
        assert!(stack.lookup("anything").is_empty());
        assert!(stack.common_prefix_search("anything").is_empty());
    }

    #[test]
    fn test_add_two_domains_priority_ordering() {
        let mut stack = DomainStack::new();
        let low = make_dict(&[("하위", "NNG", -100)]);
        let high = make_dict(&[("상위", "NNP", -1000)]);

        // Add lower priority first to verify that the sort is correct regardless
        // of insertion order.
        stack.add_domain(DomainId("low".into()), 10, low, None);
        stack.add_domain(DomainId("high".into()), 0, high, None);

        let listing = stack.list_domains();
        assert_eq!(listing.len(), 2);
        // priority 0 ("high") must come before priority 10 ("low")
        assert_eq!(listing[0].0, DomainId("high".into()));
        assert_eq!(listing[0].1, 0);
        assert_eq!(listing[1].0, DomainId("low".into()));
        assert_eq!(listing[1].1, 10);
    }

    #[test]
    fn test_common_prefix_search_returns_entries_from_all_domains() {
        let mut stack = DomainStack::new();
        let d1 = make_dict(&[("형태", "NNG", -100), ("형태소", "NNG", -200)]);
        let d2 = make_dict(&[("형태소분석", "NNG", -300)]);

        stack.add_domain(DomainId("d1".into()), 0, d1, None);
        stack.add_domain(DomainId("d2".into()), 1, d2, None);

        let results = stack.common_prefix_search("형태소분석기");
        // "형태" and "형태소" from d1, "형태소분석" from d2
        assert_eq!(results.len(), 3);

        // Higher priority domain (d1, priority=0) entries come first.
        assert_eq!(results[0].surface, "형태");
        assert_eq!(results[1].surface, "형태소");
        assert_eq!(results[2].surface, "형태소분석");
    }

    #[test]
    fn test_remove_domain_returns_correct_domain() {
        let mut stack = DomainStack::new();
        let d1 = make_dict(&[("단어1", "NNG", 0)]);
        let d2 = make_dict(&[("단어2", "NNG", 0)]);

        stack.add_domain(DomainId("alpha".into()), 0, d1, None);
        stack.add_domain(DomainId("beta".into()), 1, d2, None);
        assert_eq!(stack.len(), 2);

        let removed = stack.remove_domain(&DomainId("alpha".into()));
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().domain, DomainId("alpha".into()));
        assert_eq!(stack.len(), 1);

        // Removing a non-existent domain returns None.
        let none = stack.remove_domain(&DomainId("alpha".into()));
        assert!(none.is_none());
    }

    #[test]
    fn test_list_domains_returns_all_ids_with_entry_counts() {
        let mut stack = DomainStack::new();
        stack.add_domain(
            DomainId("a".into()),
            2,
            make_dict(&[("x", "NNG", 0), ("y", "NNG", 0)]),
            None,
        );
        stack.add_domain(DomainId("b".into()), 1, make_dict(&[("z", "NNG", 0)]), None);

        let listing = stack.list_domains();
        // Sorted by priority: b(1) then a(2)
        assert_eq!(listing[0].0, DomainId("b".into()));
        assert_eq!(listing[0].2, 1); // entry count for "b"
        assert_eq!(listing[1].0, DomainId("a".into()));
        assert_eq!(listing[1].2, 2); // entry count for "a"
    }

    #[test]
    fn test_duplicate_domain_add_replaces_existing() {
        let mut stack = DomainStack::new();
        let v1 = make_dict(&[("old_entry", "NNG", 0)]);
        let v2 = make_dict(&[("new_entry", "NNP", -500)]);

        stack.add_domain(DomainId("same".into()), 0, v1, None);
        assert_eq!(stack.len(), 1);
        assert!(!stack.lookup("old_entry").is_empty());

        stack.add_domain(DomainId("same".into()), 0, v2, None);
        // Still exactly one domain after replacement.
        assert_eq!(stack.len(), 1);
        // Old entry is gone.
        assert!(stack.lookup("old_entry").is_empty());
        // New entry is present.
        assert!(!stack.lookup("new_entry").is_empty());
    }

    #[test]
    fn test_lookup_returns_entries_in_priority_order() {
        let mut stack = DomainStack::new();
        let high = make_dict(&[("공통", "NNP", -2000)]);
        let low = make_dict(&[("공통", "NNG", -100)]);

        stack.add_domain(DomainId("high".into()), 0, high, None);
        stack.add_domain(DomainId("low".into()), 5, low, None);

        let results = stack.lookup("공통");
        assert_eq!(results.len(), 2);
        // High-priority domain result appears first.
        assert_eq!(results[0].pos, "NNP");
        assert_eq!(results[1].pos, "NNG");
    }
}
