//! Owned-byte Trie + `TrieBuilder`

use std::borrow::Cow;
#[cfg(feature = "zstd")]
use std::io::{Read, Write as IoWrite};
use std::path::Path;

use yada::{builder::DoubleArrayBuilder, DoubleArray};

use crate::error::{DictError, Result};

use super::backend::PrefixSearchResult;

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
    /// # Errors
    ///
    /// 엔트리가 비어있거나 Trie 빌드에 실패한 경우 에러를 반환합니다.
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
