//! Integration tests for hot-reload-v2 + `SystemDictionary` integration.
//!
//! These tests verify that:
//! 1. `HotReloadDictV2` can be attached to `SystemDictionary`
//! 2. Domain overlay entries appear in `common_prefix_search` results
//! 3. Hot-reload updates are visible through the dictionary's read path
//!
//! Feature-gated: requires `hot-reload-v2`.

#![cfg(feature = "hot-reload-v2")]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::items_after_statements,
    clippy::needless_collect
)]

use std::path::PathBuf;
use std::sync::Arc;

use mecab_ko_dict::dictionary::{DictEntry, SystemDictionary};
use mecab_ko_dict::domain::{DomainId, DomainStack};
use mecab_ko_dict::hot_reload_v2::HotReloadDictV2;
use mecab_ko_dict::matrix::{ConnectionMatrix, DenseMatrix};
use mecab_ko_dict::trie::TrieBuilder;
use mecab_ko_dict::user_dict::UserDictionary;
use mecab_ko_dict::{Trie, TrieBackend};

/// Helper: build a minimal `SystemDictionary` with a small trie for testing.
fn build_test_dict() -> SystemDictionary {
    let trie_entries = vec![("가", 0u32), ("가방", 1)];
    let trie_bytes = TrieBuilder::build(&trie_entries).expect("should build trie");
    let trie = TrieBackend::Owned(Trie::from_vec(trie_bytes));

    let matrix = ConnectionMatrix::Dense(DenseMatrix::new(10, 10, 100));

    let dict_entries = vec![
        DictEntry::new("가", 1, 1, 100, "NNG,*,T,가,*,*,*,*"),
        DictEntry::new("가방", 2, 2, 200, "NNG,*,T,가방,*,*,*,*"),
    ];

    SystemDictionary::new_test(PathBuf::from("./test_dic"), trie, matrix, dict_entries)
}

/// Helper: build a `DomainStack` containing a single domain with the given entries.
fn make_stack(entries: &[(&str, &str, i16)]) -> DomainStack {
    let mut dict = UserDictionary::new();
    for &(surface, pos, cost) in entries {
        dict.add_entry(surface, pos, Some(cost), None);
    }
    let mut stack = DomainStack::new();
    stack.add_domain(DomainId("test".into()), 0, Arc::new(dict), None);
    stack
}

// -----------------------------------------------------------------------
// S69-04: SystemDictionary + HotReloadDictV2 field integration
// -----------------------------------------------------------------------

#[test]
fn test_with_hot_reload_builder_pattern() {
    let dict = build_test_dict();
    let hr = Arc::new(HotReloadDictV2::new(DomainStack::new()));

    let dict = dict.with_hot_reload(Arc::clone(&hr));

    assert!(dict.hot_reload().is_some());
    assert_eq!(dict.hot_reload().unwrap().current_version(), 1);
}

#[test]
fn test_set_hot_reload_in_place() {
    let mut dict = build_test_dict();
    assert!(dict.hot_reload().is_none());

    let hr = Arc::new(HotReloadDictV2::new(DomainStack::new()));
    dict.set_hot_reload(Arc::clone(&hr));

    assert!(dict.hot_reload().is_some());
}

// -----------------------------------------------------------------------
// S69-05: Read-path integration (common_prefix_search + lookup_combined)
// -----------------------------------------------------------------------

#[test]
fn test_common_prefix_search_includes_hot_reload_entries() {
    let stack = make_stack(&[("뉴스피드", "NNG", -1000)]);
    let hr = Arc::new(HotReloadDictV2::new(stack));
    let dict = build_test_dict().with_hot_reload(hr);

    // "뉴스피드를" should match "뉴스피드" from the hot-reload overlay.
    let results = dict
        .common_prefix_search("뉴스피드를")
        .expect("search should succeed");

    let surfaces: Vec<&str> = results.iter().map(|(e, _)| e.surface.as_str()).collect();
    assert!(
        surfaces.contains(&"뉴스피드"),
        "Expected '뉴스피드' in results, got: {surfaces:?}"
    );
}

#[test]
fn test_common_prefix_search_merges_system_and_hot_reload() {
    let stack = make_stack(&[("가격", "NNG", -500)]);
    let hr = Arc::new(HotReloadDictV2::new(stack));
    let dict = build_test_dict().with_hot_reload(hr);

    // "가격표" should match system "가" + hot-reload "가격".
    let results = dict
        .common_prefix_search("가격표")
        .expect("search should succeed");

    let surfaces: Vec<&str> = results.iter().map(|(e, _)| e.surface.as_str()).collect();
    assert!(
        surfaces.contains(&"가"),
        "System entry '가' should be present"
    );
    assert!(
        surfaces.contains(&"가격"),
        "Hot-reload entry '가격' should be present"
    );
}

#[test]
fn test_lookup_combined_includes_hot_reload_entries() {
    let stack = make_stack(&[("가", "JKS", -2000)]);
    let hr = Arc::new(HotReloadDictV2::new(stack));
    let dict = build_test_dict().with_hot_reload(hr);

    let entries = dict.lookup_combined("가");
    // System has "가" (NNG) + hot-reload has "가" (JKS) = 2 entries.
    assert!(
        entries.len() >= 2,
        "Expected at least 2 entries for '가', got: {}",
        entries.len()
    );

    let features: Vec<&str> = entries.iter().map(|e| e.feature.as_str()).collect();
    assert!(
        features.iter().any(|f| f.starts_with("NNG")),
        "System NNG entry should be present"
    );
}

#[test]
fn test_no_hot_reload_does_not_break_search() {
    // Without hot-reload configured, common_prefix_search should work as before.
    let dict = build_test_dict();
    assert!(dict.hot_reload().is_none());

    let results = dict
        .common_prefix_search("가방에")
        .expect("search should succeed");

    let surfaces: Vec<&str> = results.iter().map(|(e, _)| e.surface.as_str()).collect();
    assert!(surfaces.contains(&"가"));
    assert!(surfaces.contains(&"가방"));
}

// -----------------------------------------------------------------------
// S69-06: Hot-reload update visibility
// -----------------------------------------------------------------------

#[test]
fn test_hot_reload_update_is_visible_through_dictionary() {
    // Step 1: start with an empty domain stack.
    let hr = Arc::new(HotReloadDictV2::new(DomainStack::new()));
    let dict = build_test_dict().with_hot_reload(Arc::clone(&hr));

    // Initially, "뉴스피드" should NOT be found.
    let results = dict
        .common_prefix_search("뉴스피드")
        .expect("search should succeed");
    let surfaces: Vec<&str> = results.iter().map(|(e, _)| e.surface.as_str()).collect();
    assert!(
        !surfaces.contains(&"뉴스피드"),
        "Should not find '뉴스피드' before update"
    );

    // Step 2: update the hot-reload dictionary (simulating a hot-reload event).
    let v2 = hr.update(|_old_stack| make_stack(&[("뉴스피드", "NNG", -1000)]));
    assert_eq!(v2, 2);

    // Step 3: verify the new entry is visible through the same dictionary ref.
    let results = dict
        .common_prefix_search("뉴스피드를읽다")
        .expect("search should succeed");
    let surfaces: Vec<&str> = results.iter().map(|(e, _)| e.surface.as_str()).collect();
    assert!(
        surfaces.contains(&"뉴스피드"),
        "Expected '뉴스피드' after hot-reload update, got: {surfaces:?}"
    );
}

#[test]
fn test_hot_reload_rollback_removes_entry() {
    let hr = Arc::new(HotReloadDictV2::new(DomainStack::new()));
    let dict = build_test_dict().with_hot_reload(Arc::clone(&hr));

    // Add an entry.
    hr.update(|_| make_stack(&[("코스피", "NNP", -2000)]));

    // Confirm it is found.
    let results = dict.common_prefix_search("코스피").expect("search ok");
    assert!(results.iter().any(|(e, _)| e.surface == "코스피"));

    // Rollback.
    let rolled = hr.rollback();
    assert!(rolled.is_some());

    // After rollback, the entry should be gone.
    let results = dict.common_prefix_search("코스피").expect("search ok");
    assert!(
        !results.iter().any(|(e, _)| e.surface == "코스피"),
        "Entry should be gone after rollback"
    );
}

#[test]
fn test_multiple_domains_in_hot_reload() {
    let mut stack = DomainStack::new();

    let mut news = UserDictionary::new();
    news.add_entry("뉴스피드", "NNG", Some(-1000), None);

    let mut finance = UserDictionary::new();
    finance.add_entry("코스피", "NNP", Some(-2000), None);

    stack.add_domain(DomainId("news".into()), 0, Arc::new(news), None);
    stack.add_domain(DomainId("finance".into()), 1, Arc::new(finance), None);

    let hr = Arc::new(HotReloadDictV2::new(stack));
    let dict = build_test_dict().with_hot_reload(hr);

    // Both domain entries should appear.
    let r1 = dict.common_prefix_search("뉴스피드").expect("ok");
    assert!(r1.iter().any(|(e, _)| e.surface == "뉴스피드"));

    let r2 = dict.common_prefix_search("코스피").expect("ok");
    assert!(r2.iter().any(|(e, _)| e.surface == "코스피"));
}
