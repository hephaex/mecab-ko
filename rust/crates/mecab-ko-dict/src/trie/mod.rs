//! # Double-Array Trie 모듈
//!
//! 사전 검색을 위한 Double-Array Trie 래퍼입니다.
//! [yada](https://crates.io/crates/yada) 라이브러리를 기반으로 합니다.
//!
//! ## 주요 기능
//!
//! - 정확히 일치하는 키 검색 (exact match)
//! - 공통 접두사 검색 (common prefix search)
//! - Trie 빌드 및 직렬화
//!
//! ## 예제
//!
//! ```rust
//! use mecab_ko_dict::trie::{Trie, TrieBuilder};
//!
//! // Trie 빌드
//! let entries = vec![
//!     ("가", 0u32),
//!     ("가다", 1),
//!     ("가방", 2),
//! ];
//! let trie_bytes = TrieBuilder::build(&entries).unwrap();
//!
//! // Trie 검색
//! let trie = Trie::new(&trie_bytes);
//! assert_eq!(trie.exact_match("가다"), Some(1));
//! ```

mod backend;
mod mmap;
mod owned;

pub use backend::{PrefixSearchResult, TrieBackend};
pub use mmap::MmapTrie;
pub use owned::{Trie, TrieBuilder};

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn build_test_trie() -> Vec<u8> {
        let entries = vec![
            ("가", 0u32),
            ("가다", 1),
            ("가방", 2),
            ("가방에", 3),
            ("나", 4),
            ("나다", 5),
        ];
        TrieBuilder::build(&entries).unwrap()
    }

    #[test]
    fn test_exact_match() {
        let bytes = build_test_trie();
        let trie = Trie::new(&bytes);

        assert_eq!(trie.exact_match("가"), Some(0));
        assert_eq!(trie.exact_match("가다"), Some(1));
        assert_eq!(trie.exact_match("가방"), Some(2));
        assert_eq!(trie.exact_match("가방에"), Some(3));
        assert_eq!(trie.exact_match("나"), Some(4));
        assert_eq!(trie.exact_match("없음"), None);
    }

    #[test]
    fn test_common_prefix_search() {
        let bytes = build_test_trie();
        let trie = Trie::new(&bytes);

        // "가방에서" 검색 -> "가", "가방", "가방에" 매칭
        let results: Vec<_> = trie.common_prefix_search("가방에서").collect();
        assert_eq!(results.len(), 3);

        // 값 확인
        let values: Vec<_> = results.iter().map(|(v, _)| *v).collect();
        assert!(values.contains(&0)); // "가"
        assert!(values.contains(&2)); // "가방"
        assert!(values.contains(&3)); // "가방에"
    }

    #[test]
    fn test_common_prefix_search_at() {
        let bytes = build_test_trie();
        let trie = Trie::new(&bytes);

        // "나가다" 에서 위치 3("가다" 시작)부터 검색
        let text = "나가다";
        let start = "나".len(); // 3 bytes

        let results = trie.common_prefix_search_at(text, start);

        // "가", "가다" 매칭
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_build_unsorted() {
        let mut entries = vec![("가방", 2u32), ("가", 0), ("가다", 1)];

        let bytes = TrieBuilder::build_unsorted(&mut entries).unwrap();
        let trie = Trie::new(&bytes);

        assert_eq!(trie.exact_match("가"), Some(0));
        assert_eq!(trie.exact_match("가다"), Some(1));
        assert_eq!(trie.exact_match("가방"), Some(2));
    }

    #[test]
    fn test_from_vec() {
        let bytes = build_test_trie();
        let trie = Trie::from_vec(bytes);

        assert_eq!(trie.exact_match("가"), Some(0));
    }

    #[test]
    fn test_korean_morphemes() {
        // 실제 형태소 분석 시나리오
        // yada는 바이트 순으로 정렬된 입력 필요
        let mut entries = vec![
            ("아버지", 0u32),
            ("아버지가", 1),
            ("가", 2),
            ("가방", 3),
            ("가방에", 4),
            ("방", 5),
            ("방에", 6),
            ("에", 7),
        ];

        let bytes = TrieBuilder::build_unsorted(&mut entries).expect("should build trie");
        let trie = Trie::new(&bytes);

        // "아버지가방에" 분석
        let text = "아버지가방에";

        // 위치 0에서: "아버지", "아버지가"
        let at_0: Vec<_> = trie.common_prefix_search(text).collect();
        assert!(at_0.iter().any(|(v, _)| *v == 0)); // 아버지
        assert!(at_0.iter().any(|(v, _)| *v == 1)); // 아버지가

        // 위치 "아버지" 이후(9바이트)에서: "가", "가방", "가방에"
        let at_9 = trie.common_prefix_search_at(text, 9);
        assert!(at_9.iter().any(|(v, _)| *v == 2)); // 가
        assert!(at_9.iter().any(|(v, _)| *v == 3)); // 가방
        assert!(at_9.iter().any(|(v, _)| *v == 4)); // 가방에
    }

    #[test]
    fn test_mmap_trie_exact_match() {
        let bytes = build_test_trie();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        TrieBuilder::save_to_file(&bytes, tmp.path()).unwrap();

        let trie = MmapTrie::from_file(tmp.path()).unwrap();
        assert_eq!(trie.exact_match("가"), Some(0));
        assert_eq!(trie.exact_match("가다"), Some(1));
        assert_eq!(trie.exact_match("가방"), Some(2));
        assert_eq!(trie.exact_match("없음"), None);
    }

    #[test]
    fn test_mmap_trie_common_prefix_search() {
        let bytes = build_test_trie();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        TrieBuilder::save_to_file(&bytes, tmp.path()).unwrap();

        let trie = MmapTrie::from_file(tmp.path()).unwrap();
        let results: Vec<_> = trie.common_prefix_search("가방에서").collect();
        assert_eq!(results.len(), 3);

        let values: Vec<_> = results.iter().map(|(v, _)| *v).collect();
        assert!(values.contains(&0));
        assert!(values.contains(&2));
        assert!(values.contains(&3));
    }

    #[test]
    fn test_trie_backend_owned_vs_mmap() {
        let bytes = build_test_trie();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        TrieBuilder::save_to_file(&bytes, tmp.path()).unwrap();

        let owned = TrieBackend::from_file(tmp.path()).unwrap();
        let mmap = TrieBackend::from_mmap_file(tmp.path()).unwrap();

        for key in &["가", "가다", "가방", "가방에", "나", "나다", "없음"] {
            assert_eq!(
                owned.exact_match(key),
                mmap.exact_match(key),
                "mismatch for key {key}"
            );
        }

        let owned_results = owned.common_prefix_search("가방에서");
        let mmap_results = mmap.common_prefix_search("가방에서");
        assert_eq!(owned_results.as_slice(), mmap_results.as_slice());
    }

    #[test]
    fn test_empty_trie() {
        let entries: Vec<(&str, u32)> = vec![];
        let result = TrieBuilder::build(&entries);
        // 빈 엔트리로 빌드 시도 - 에러 반환
        assert!(result.is_err());
    }

    #[test]
    fn test_single_entry() {
        let entries = vec![("테스트", 42u32)];
        let bytes = TrieBuilder::build(&entries).unwrap();
        let trie = Trie::new(&bytes);

        assert_eq!(trie.exact_match("테스트"), Some(42));
        assert_eq!(trie.exact_match("테스"), None);
        assert_eq!(trie.exact_match("테스트입니다"), None);
    }
}
