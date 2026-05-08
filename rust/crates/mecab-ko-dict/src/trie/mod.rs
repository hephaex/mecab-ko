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

use std::borrow::Cow;
#[cfg(feature = "zstd")]
use std::io::{Read, Write as IoWrite};
use std::path::Path;

use yada::{builder::DoubleArrayBuilder, DoubleArray};

use crate::error::{DictError, Result};

mod backend;
mod mmap;

pub use backend::{PrefixSearchResult, TrieBackend};
pub use mmap::MmapTrie;

/// Double-Array Trie
///
/// 문자열 키를 효율적으로 검색하는 자료구조입니다.
/// 형태소 분석에서 사전 검색에 사용됩니다.
pub struct Trie<'a> {
    /// 내부 Double-Array
    da: DoubleArray<Cow<'a, [u8]>>,
}

impl<'a> Trie<'a> {
    /// 바이트 슬라이스에서 Trie 생성
    ///
    /// # Arguments
    ///
    /// * `bytes` - 직렬화된 Trie 데이터
    #[must_use]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            da: DoubleArray::new(Cow::Borrowed(bytes)),
        }
    }

    /// 소유 바이트 벡터에서 Trie 생성
    #[must_use]
    pub fn from_vec(bytes: Vec<u8>) -> Trie<'static> {
        Trie {
            da: DoubleArray::new(Cow::Owned(bytes)),
        }
    }

    /// 파일에서 Trie 로드
    ///
    /// # Errors
    ///
    /// 파일을 읽을 수 없는 경우 에러를 반환합니다.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Trie<'static>> {
        let bytes = std::fs::read(path.as_ref()).map_err(DictError::Io)?;
        Ok(Self::from_vec(bytes))
    }

    /// 압축된 파일에서 Trie 로드 (zstd)
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 압축 해제할 수 없는 경우 에러를 반환합니다.
    #[cfg(feature = "zstd")]
    pub fn from_compressed_file<P: AsRef<Path>>(path: P) -> Result<Trie<'static>> {
        let file = std::fs::File::open(path.as_ref()).map_err(DictError::Io)?;
        let mut decoder = zstd::Decoder::new(file).map_err(DictError::Io)?;
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes).map_err(DictError::Io)?;
        Ok(Self::from_vec(bytes))
    }

    /// 압축된 파일에서 Trie 로드 (zstd feature 비활성화 시)
    ///
    /// # Errors
    ///
    /// zstd feature가 비활성화된 경우 항상 에러를 반환합니다.
    #[cfg(not(feature = "zstd"))]
    pub fn from_compressed_file<P: AsRef<Path>>(_path: P) -> Result<Trie<'static>> {
        Err(DictError::Format(
            "zstd feature is not enabled. Use uncompressed files or enable the 'zstd' feature."
                .to_string(),
        ))
    }

    /// 정확히 일치하는 키 검색
    ///
    /// # Arguments
    ///
    /// * `key` - 검색할 키 (UTF-8 문자열)
    ///
    /// # Returns
    ///
    /// 일치하는 값이 있으면 `Some(value)`, 없으면 `None`
    ///
    /// # Example
    ///
    /// ```rust
    /// use mecab_ko_dict::trie::{Trie, TrieBuilder};
    ///
    /// let bytes = TrieBuilder::build(&[("가다", 1u32)]).unwrap();
    /// let trie = Trie::new(&bytes);
    /// let value = trie.exact_match("가다");
    /// assert_eq!(value, Some(1));
    /// ```
    #[must_use]
    pub fn exact_match(&self, key: &str) -> Option<u32> {
        self.da.exact_match_search(key.as_bytes())
    }

    /// 바이트 키로 정확히 일치하는 키 검색
    #[must_use]
    pub fn exact_match_bytes(&self, key: &[u8]) -> Option<u32> {
        self.da.exact_match_search(key)
    }

    /// 공통 접두사 검색
    ///
    /// 주어진 텍스트의 접두사와 일치하는 모든 키를 찾습니다.
    /// 형태소 분석에서 가능한 모든 형태소 후보를 찾는 데 사용됩니다.
    ///
    /// # Arguments
    ///
    /// * `text` - 검색할 텍스트
    ///
    /// # Returns
    ///
    /// (value, `byte_length`) 쌍의 반복자
    /// - value: 일치하는 키의 값
    /// - `byte_length`: 일치하는 키의 바이트 길이
    ///
    /// # Example
    ///
    /// ```rust
    /// use mecab_ko_dict::trie::{Trie, TrieBuilder};
    ///
    /// let mut entries = vec![("가", 0u32), ("가방", 2)];
    /// entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    /// let bytes = TrieBuilder::build(&entries).unwrap();
    /// let trie = Trie::new(&bytes);
    ///
    /// // "가방에" 텍스트에서 접두사 검색
    /// let results: Vec<_> = trie.common_prefix_search("가방에").collect();
    /// assert_eq!(results.len(), 2); // "가", "가방" 매칭
    /// ```
    pub fn common_prefix_search<'b>(
        &'b self,
        text: &'b str,
    ) -> impl Iterator<Item = (u32, usize)> + 'b {
        self.da.common_prefix_search(text.as_bytes())
    }

    /// 바이트 키로 공통 접두사 검색
    pub fn common_prefix_search_bytes<'b>(
        &'b self,
        key: &'b [u8],
    ) -> impl Iterator<Item = (u32, usize)> + 'b {
        self.da.common_prefix_search(key)
    }

    /// 텍스트의 특정 위치에서 공통 접두사 검색
    ///
    /// # Arguments
    ///
    /// * `text` - 전체 텍스트
    /// * `start_byte` - 검색 시작 바이트 위치
    ///
    /// # Returns
    ///
    /// (value, `end_byte`) 쌍의 벡터
    /// - value: 일치하는 키의 값
    /// - `end_byte`: 일치하는 키의 끝 바이트 위치
    #[must_use]
    pub fn common_prefix_search_at(
        &self,
        text: &str,
        start_byte: usize,
    ) -> PrefixSearchResult {
        if start_byte >= text.len() {
            return PrefixSearchResult::new();
        }

        let suffix = &text[start_byte..];
        self.da
            .common_prefix_search(suffix.as_bytes())
            .map(|(value, len)| (value, start_byte + len))
            .collect()
    }
}

/// Trie 빌더
///
/// 키-값 쌍에서 Double-Array Trie를 빌드합니다.
pub struct TrieBuilder;

impl TrieBuilder {
    /// 정렬된 키-값 쌍에서 Trie 빌드
    ///
    /// # Arguments
    ///
    /// * `entries` - 키로 정렬된 (키, 값) 쌍의 슬라이스
    ///
    /// # Returns
    ///
    /// 성공 시 직렬화된 Trie 바이트, 실패 시 에러
    ///
    /// # Errors
    ///
    /// 엔트리가 비어있거나 Trie 빌드에 실패한 경우 에러를 반환합니다.
    ///
    /// # Note
    ///
    /// 입력 엔트리는 **반드시 키 순으로 정렬**되어야 합니다.
    /// 빈 엔트리는 에러를 반환합니다.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mecab_ko_dict::trie::TrieBuilder;
    ///
    /// let mut entries = vec![
    ///     ("가방", 2u32),
    ///     ("가", 0),
    ///     ("가다", 1),
    /// ];
    /// entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    ///
    /// let bytes = TrieBuilder::build(&entries).unwrap();
    /// assert!(!bytes.is_empty());
    /// ```
    pub fn build(entries: &[(&str, u32)]) -> Result<Vec<u8>> {
        if entries.is_empty() {
            return Err(DictError::Format(
                "Cannot build Trie from empty entries".to_string(),
            ));
        }

        let keyset: Vec<_> = entries.iter().map(|(k, v)| (k.as_bytes(), *v)).collect();

        DoubleArrayBuilder::build(&keyset)
            .ok_or_else(|| DictError::Format("Failed to build Double-Array Trie".to_string()))
    }

    /// 바이트 키-값 쌍에서 Trie 빌드
    ///
    /// # Errors
    ///
    /// 엔트리가 비어있거나 Trie 빌드에 실패한 경우 에러를 반환합니다.
    pub fn build_bytes(entries: &[(&[u8], u32)]) -> Result<Vec<u8>> {
        if entries.is_empty() {
            return Err(DictError::Format(
                "Cannot build Trie from empty entries".to_string(),
            ));
        }

        DoubleArrayBuilder::build(entries)
            .ok_or_else(|| DictError::Format("Failed to build Double-Array Trie".to_string()))
    }

    /// 정렬되지 않은 엔트리에서 Trie 빌드
    ///
    /// 내부적으로 정렬을 수행합니다.
    ///
    /// # Errors
    ///
    /// 엔트리가 비어있거나 Trie 빌드에 실패한 경우 에러를 반환합니다.
    pub fn build_unsorted(entries: &mut [(&str, u32)]) -> Result<Vec<u8>> {
        entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        Self::build(entries)
    }

    /// Trie를 파일로 저장
    ///
    /// # Errors
    ///
    /// 파일을 쓸 수 없는 경우 에러를 반환합니다.
    pub fn save_to_file<P: AsRef<Path>>(bytes: &[u8], path: P) -> Result<()> {
        std::fs::write(path.as_ref(), bytes).map_err(DictError::Io)
    }

    /// Trie를 압축하여 파일로 저장 (zstd)
    ///
    /// # Errors
    ///
    /// 파일을 쓰거나 압축할 수 없는 경우 에러를 반환합니다.
    #[cfg(feature = "zstd")]
    pub fn save_to_compressed_file<P: AsRef<Path>>(
        bytes: &[u8],
        path: P,
        level: i32,
    ) -> Result<()> {
        let file = std::fs::File::create(path.as_ref()).map_err(DictError::Io)?;
        let mut encoder = zstd::Encoder::new(file, level).map_err(DictError::Io)?;
        encoder.write_all(bytes).map_err(DictError::Io)?;
        encoder.finish().map_err(DictError::Io)?;
        Ok(())
    }

    /// Trie를 압축하여 파일로 저장 (zstd feature 비활성화 시)
    ///
    /// # Errors
    ///
    /// zstd feature가 비활성화된 경우 항상 에러를 반환합니다.
    #[cfg(not(feature = "zstd"))]
    pub fn save_to_compressed_file<P: AsRef<Path>>(
        _bytes: &[u8],
        _path: P,
        _level: i32,
    ) -> Result<()> {
        Err(DictError::Format(
            "zstd feature is not enabled. Use uncompressed files or enable the 'zstd' feature."
                .to_string(),
        ))
    }
}

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
