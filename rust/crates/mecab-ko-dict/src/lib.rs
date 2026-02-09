//! # mecab-ko-dict
//!
//! 한국어 형태소 사전 관리 라이브러리
//!
//! ## 주요 기능
//!
//! - 바이너리 사전 포맷 (v3.0)
//! - FST 기반 형태소 검색
//! - 연접 비용 매트릭스
//! - 사전 빌더/컴파일러
//!
//! ## 예제
//!
//! ```rust,ignore
//! use mecab_ko_dict::Dictionary;
//!
//! let dict = Dictionary::load("path/to/dict")?;
//! let entries = dict.lookup("안녕");
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![cfg_attr(feature = "simd", feature(portable_simd))]

pub mod dictionary;
pub mod file_watcher;
pub mod hot_reload;
pub mod loader;
pub mod matrix;
pub mod trie;
pub mod user_dict;

pub use dictionary::{DictEntry, DictionaryLoader, SystemDictionary};
pub use error::{DictError, Result};
pub use file_watcher::{FileEvent, FileWatcher, WatchConfig};
pub use hot_reload::{
    DeltaUpdate, DeltaUpdateBuilder, EntryChange, HotReloadDictionary, Version, VersionInfo,
};
pub use loader::{LazyDictionary, LoaderConfig, MmapDictionary};
pub use matrix::{ConnectionMatrix, DenseMatrix, Matrix, MatrixLoader, MmapMatrix, SparseMatrix};
pub use trie::{DictionarySearcher, EntryIndex, PrefixMatch, Trie, TrieBuilder};
pub use user_dict::{UserDictionary, UserDictionaryBuilder, UserEntry};

/// 사전 엔트리
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// 표면형
    pub surface: String,
    /// 좌문맥 ID
    pub left_id: u16,
    /// 우문맥 ID
    pub right_id: u16,
    /// 비용
    pub cost: i16,
    /// 품사 정보
    pub feature: String,
}

/// 사전 인터페이스
pub trait Dictionary {
    /// 형태소 검색
    fn lookup(&self, surface: &str) -> Vec<Entry>;

    /// 연접 비용 조회
    fn get_connection_cost(&self, left_id: u16, right_id: u16) -> i16;
}

/// 에러 모듈
pub mod error {
    use thiserror::Error;

    /// 사전 에러 타입
    #[derive(Error, Debug)]
    pub enum DictError {
        /// IO 에러
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),

        /// 포맷 에러
        #[error("Invalid dictionary format: {0}")]
        Format(String),

        /// 버전 불일치
        #[error("Version mismatch: expected {expected}, found {found}")]
        Version {
            /// 예상 버전
            expected: u32,
            /// 실제 버전
            found: u32,
        },
    }

    /// Result 타입 별칭
    pub type Result<T> = std::result::Result<T, DictError>;
}

/// 사전 포맷 모듈 (스텁)
///
/// 바이너리 사전 포맷 정의
pub mod format {

    /// 사전 헤더
    pub struct Header {
        /// 매직 넘버
        pub magic: [u8; 4],
        /// 버전
        pub version: u32,
        /// 엔트리 수
        pub entry_count: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_creation() {
        let entry = Entry {
            surface: "안녕".to_string(),
            left_id: 1,
            right_id: 1,
            cost: 100,
            feature: "NNG,*,T,안녕,*,*,*,*".to_string(),
        };

        assert_eq!(entry.surface, "안녕");
    }
}
