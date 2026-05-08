//! mmap 백엔드 Trie

use std::path::Path;

use yada::DoubleArray;

use crate::error::{DictError, Result};

/// mmap 백엔드 Trie
///
/// 파일을 메모리 맵으로 직접 접근하여 불필요한 복사 없이 Trie를 사용합니다.
pub struct MmapTrie {
    pub(super) da: DoubleArray<memmap2::Mmap>,
}

impl MmapTrie {
    /// 파일을 메모리 맵으로 열어 Trie 로드
    ///
    /// # Errors
    ///
    /// 파일을 열거나 매핑할 수 없는 경우 에러를 반환합니다.
    #[allow(unsafe_code)]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref()).map_err(DictError::Io)?;
        // SAFETY: The mapped file is a read-only dictionary installed immutably.
        // Caller must ensure no concurrent writes or truncation to this path.
        let mmap = unsafe { memmap2::Mmap::map(&file).map_err(DictError::Io)? };
        Ok(Self {
            da: DoubleArray::new(mmap),
        })
    }

    /// 정확히 일치하는 키 검색
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
    pub fn common_prefix_search<'a>(
        &'a self,
        text: &'a str,
    ) -> impl Iterator<Item = (u32, usize)> + 'a {
        self.da.common_prefix_search(text.as_bytes())
    }

    /// 바이트 키로 공통 접두사 검색
    pub fn common_prefix_search_bytes<'a>(
        &'a self,
        key: &'a [u8],
    ) -> impl Iterator<Item = (u32, usize)> + 'a {
        self.da.common_prefix_search(key)
    }

    /// 특정 위치에서 공통 접두사 검색
    #[must_use]
    pub fn common_prefix_search_at(
        &self,
        text: &str,
        start_byte: usize,
    ) -> super::backend::PrefixSearchResult {
        if start_byte >= text.len() {
            return super::backend::PrefixSearchResult::new();
        }
        let suffix = &text[start_byte..];
        self.da
            .common_prefix_search(suffix.as_bytes())
            .map(|(value, len)| (value, start_byte + len))
            .collect()
    }
}
