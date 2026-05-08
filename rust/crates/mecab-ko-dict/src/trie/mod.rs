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

/// 사전 엔트리 인덱스
///
/// Trie의 값을 사전 엔트리 배열의 인덱스로 사용합니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryIndex(pub u32);

impl EntryIndex {
    /// 새 인덱스 생성
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// 인덱스 값 반환
    #[must_use]
    pub const fn value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for EntryIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<EntryIndex> for u32 {
    fn from(index: EntryIndex) -> Self {
        index.0
    }
}

/// 공통 접두사 검색 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixMatch {
    /// 엔트리 인덱스
    pub index: EntryIndex,
    /// 일치한 바이트 길이
    pub byte_length: usize,
    /// 시작 바이트 위치
    pub start_byte: usize,
    /// 끝 바이트 위치
    pub end_byte: usize,
}

impl PrefixMatch {
    /// 새 `PrefixMatch` 생성
    #[must_use]
    pub const fn new(index: u32, byte_length: usize, start_byte: usize) -> Self {
        Self {
            index: EntryIndex(index),
            start_byte,
            end_byte: start_byte + byte_length,
            byte_length,
        }
    }
}

/// 사전 검색기
///
/// Trie와 엔트리 배열을 결합하여 사전 검색을 수행합니다.
pub struct DictionarySearcher<'a, E> {
    /// Trie
    trie: &'a Trie<'a>,
    /// 엔트리 배열
    entries: &'a [E],
}

impl<'a, E> DictionarySearcher<'a, E> {
    /// 새 검색기 생성
    pub const fn new(trie: &'a Trie<'a>, entries: &'a [E]) -> Self {
        Self { trie, entries }
    }

    /// 정확히 일치하는 엔트리 검색
    #[must_use]
    pub fn exact_match(&self, key: &str) -> Option<&E> {
        let index = self.trie.exact_match(key)?;
        self.entries.get(index as usize)
    }

    /// 공통 접두사 검색으로 모든 일치 엔트리 반환
    #[must_use]
    pub fn common_prefix_search(&self, text: &str) -> Vec<(&E, PrefixMatch)> {
        self.trie
            .common_prefix_search(text)
            .filter_map(|(index, byte_len)| {
                let entry = self.entries.get(index as usize)?;
                let prefix_match = PrefixMatch::new(index, byte_len, 0);
                Some((entry, prefix_match))
            })
            .collect()
    }

    /// 특정 위치에서 공통 접두사 검색
    #[must_use]
    pub fn common_prefix_search_at(&self, text: &str, start_byte: usize) -> Vec<(&E, PrefixMatch)> {
        self.trie
            .common_prefix_search_at(text, start_byte)
            .into_iter()
            .filter_map(|(index, end_byte)| {
                let entry = self.entries.get(index as usize)?;
                let byte_len = end_byte - start_byte;
                let prefix_match = PrefixMatch::new(index, byte_len, start_byte);
                Some((entry, prefix_match))
            })
            .collect()
    }
}

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
    fn test_entry_index() {
        let idx = EntryIndex::new(42);
        assert_eq!(idx.value(), 42);
        assert_eq!(u32::from(idx), 42);

        let idx2: EntryIndex = 100u32.into();
        assert_eq!(idx2.value(), 100);
    }

    #[test]
    fn test_prefix_match() {
        let pm = PrefixMatch::new(5, 6, 10);
        assert_eq!(pm.index.value(), 5);
        assert_eq!(pm.byte_length, 6);
        assert_eq!(pm.start_byte, 10);
        assert_eq!(pm.end_byte, 16);
    }

    #[test]
    fn test_dictionary_searcher() {
        let bytes = build_test_trie();
        let trie = Trie::new(&bytes);

        let entries = vec![
            "가-entry",
            "가다-entry",
            "가방-entry",
            "가방에-entry",
            "나-entry",
            "나다-entry",
        ];

        let searcher = DictionarySearcher::new(&trie, &entries);

        // exact match
        assert_eq!(searcher.exact_match("가다"), Some(&"가다-entry"));
        assert_eq!(searcher.exact_match("없음"), None);

        // common prefix search
        let results = searcher.common_prefix_search("가방에서");
        assert_eq!(results.len(), 3);

        let found_entries: Vec<_> = results.iter().map(|(e, _)| **e).collect();
        assert!(found_entries.contains(&"가-entry"));
        assert!(found_entries.contains(&"가방-entry"));
        assert!(found_entries.contains(&"가방에-entry"));
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
