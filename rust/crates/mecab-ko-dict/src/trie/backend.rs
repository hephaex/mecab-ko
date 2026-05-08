//! Trie 백엔드 통합 타입

use std::path::Path;

use smallvec::SmallVec;

use crate::error::Result;

use super::{mmap::MmapTrie, Trie};

/// Trie 백엔드 통합 타입
///
/// 소유 벡터 또는 mmap 중 하나를 런타임에 선택합니다.
pub enum TrieBackend {
    /// 소유 바이트 벡터 백엔드 (압축 해제 포함)
    Owned(Trie<'static>),
    /// 메모리 맵 백엔드
    Mmap(MmapTrie),
}

/// 공통 접두사 검색 결과 타입.
/// 형태소 분석에서 한 위치의 매칭은 보통 1~5건이므로
/// 스택 버퍼 16으로 대부분 힙 할당 없이 처리됩니다.
pub type PrefixSearchResult = SmallVec<[(u32, usize); 16]>;

impl TrieBackend {
    /// 파일을 읽어 소유 벡터로 로드
    ///
    /// # Errors
    ///
    /// 파일을 읽을 수 없는 경우 에러를 반환합니다.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self::Owned(Trie::from_file(path)?))
    }

    /// 파일을 메모리 맵으로 로드
    ///
    /// # Errors
    ///
    /// 파일을 열거나 매핑할 수 없는 경우 에러를 반환합니다.
    pub fn from_mmap_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self::Mmap(MmapTrie::from_file(path)?))
    }

    /// 압축 파일에서 소유 벡터로 로드 (zstd)
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 압축 해제할 수 없는 경우 에러를 반환합니다.
    pub fn from_compressed_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self::Owned(Trie::from_compressed_file(path)?))
    }

    /// 정확히 일치하는 키 검색
    #[must_use]
    pub fn exact_match(&self, key: &str) -> Option<u32> {
        match self {
            Self::Owned(t) => t.exact_match(key),
            Self::Mmap(t) => t.exact_match(key),
        }
    }

    /// 바이트 키로 정확히 일치하는 키 검색
    #[must_use]
    pub fn exact_match_bytes(&self, key: &[u8]) -> Option<u32> {
        match self {
            Self::Owned(t) => t.exact_match_bytes(key),
            Self::Mmap(t) => t.exact_match_bytes(key),
        }
    }

    /// 공통 접두사 검색
    #[must_use]
    pub fn common_prefix_search(&self, text: &str) -> PrefixSearchResult {
        match self {
            Self::Owned(t) => t.common_prefix_search(text).collect(),
            Self::Mmap(t) => t.common_prefix_search(text).collect(),
        }
    }

    /// 바이트 키로 공통 접두사 검색
    #[must_use]
    pub fn common_prefix_search_bytes(&self, key: &[u8]) -> PrefixSearchResult {
        match self {
            Self::Owned(t) => t.common_prefix_search_bytes(key).collect(),
            Self::Mmap(t) => t.common_prefix_search_bytes(key).collect(),
        }
    }

    /// 특정 위치에서 공통 접두사 검색
    #[must_use]
    pub fn common_prefix_search_at(&self, text: &str, start_byte: usize) -> PrefixSearchResult {
        match self {
            Self::Owned(t) => t.common_prefix_search_at(text, start_byte),
            Self::Mmap(t) => t.common_prefix_search_at(text, start_byte),
        }
    }
}
