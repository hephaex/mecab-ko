//! mmap trie integration test
//!
//! `LoadOptions`의 `use_mmap_trie=true` 시 동작 검증

#![allow(clippy::expect_used, clippy::unwrap_used)]

use mecab_ko_dict::dictionary::{LoadOptions, SystemDictionary};
use mecab_ko_dict::Dictionary;

fn find_dicdir() -> Option<std::path::PathBuf> {
    let candidates = [
        "/opt/homebrew/lib/mecab/dic/mecab-ko-dic",
        "/usr/local/lib/mecab/dic/mecab-ko-dic",
        "/usr/lib/mecab/dic/mecab-ko-dic",
    ];
    for path in candidates {
        let p = std::path::PathBuf::from(path);
        if p.join("sys.dic").exists() {
            return Some(p);
        }
    }
    // Also check MECAB_DICDIR env
    if let Ok(dir) = std::env::var("MECAB_DICDIR") {
        let p = std::path::PathBuf::from(dir);
        if p.join("sys.dic").exists() {
            return Some(p);
        }
    }
    None
}

#[test]
#[ignore = "requires installed mecab-ko-dic"]
fn mmap_trie_loads_successfully() {
    let dicdir = find_dicdir().expect("dictionary not found — skipping");

    let opts = LoadOptions {
        use_mmap_trie: true,
        use_mmap_matrix: false,
        use_lazy_entries: false,
        lazy_cache_size: None,
    };

    let dict = SystemDictionary::load_with_options(&dicdir, opts);
    assert!(
        dict.is_ok(),
        "Failed to load with mmap trie: {:?}",
        dict.err()
    );
}

#[test]
#[ignore = "requires installed mecab-ko-dic"]
fn mmap_trie_produces_same_results_as_owned() {
    let dicdir = find_dicdir().expect("dictionary not found — skipping");

    // Load with default (owned trie)
    let default_opts = LoadOptions {
        use_mmap_trie: false,
        use_mmap_matrix: false,
        use_lazy_entries: false,
        lazy_cache_size: None,
    };
    let dict_owned =
        SystemDictionary::load_with_options(&dicdir, default_opts).expect("owned load failed");

    // Load with mmap trie
    let mmap_opts = LoadOptions {
        use_mmap_trie: true,
        use_mmap_matrix: false,
        use_lazy_entries: false,
        lazy_cache_size: None,
    };
    let dict_mmap =
        SystemDictionary::load_with_options(&dicdir, mmap_opts).expect("mmap load failed");

    // Test lookups produce same results
    let test_words = ["나", "가", "한국어", "분석", "테스트", "대한민국"];

    for word in &test_words {
        let owned_results = dict_owned.lookup(word);
        let mmap_results = dict_mmap.lookup(word);

        assert_eq!(
            owned_results.len(),
            mmap_results.len(),
            "Different result count for '{word}': owned={}, mmap={}",
            owned_results.len(),
            mmap_results.len()
        );

        for (o, m) in owned_results.iter().zip(mmap_results.iter()) {
            assert_eq!(o.surface, m.surface, "Surface mismatch for '{word}'");
            assert_eq!(o.left_id, m.left_id, "left_id mismatch for '{word}'");
            assert_eq!(o.right_id, m.right_id, "right_id mismatch for '{word}'");
            assert_eq!(o.cost, m.cost, "cost mismatch for '{word}'");
            assert_eq!(o.feature, m.feature, "feature mismatch for '{word}'");
        }
    }
}

#[test]
#[ignore = "requires installed mecab-ko-dic"]
fn memory_optimized_loads_with_all_mmap() {
    let dicdir = find_dicdir().expect("dictionary not found — skipping");

    let opts = LoadOptions::memory_optimized();
    let dict = SystemDictionary::load_with_options(&dicdir, opts);
    assert!(
        dict.is_ok(),
        "Failed to load with memory_optimized: {:?}",
        dict.err()
    );
}
