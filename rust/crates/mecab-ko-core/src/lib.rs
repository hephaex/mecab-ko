//! # mecab-ko-core
//!
//! 한국어 형태소 분석 핵심 엔진
//!
//! ## 주요 기능
//!
//! - Lattice 구축
//! - Viterbi 알고리즘
//! - N-best 경로 탐색
//! - 미등록어 처리
//!
//! ## 예제
//!
//! ```rust,ignore
//! use mecab_ko_core::Tokenizer;
//!
//! let tokenizer = Tokenizer::new()?;
//! let tokens = tokenizer.tokenize("안녕하세요");
//!
//! for token in tokens {
//!     println!("{}: {}", token.surface, token.pos);
//! }
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod kiwi_compat;
pub mod lattice;
pub mod nori_compat;
pub mod pos_tag;
pub mod unknown;
pub mod viterbi;

pub use error::{Error, Result};
pub use kiwi_compat::{from_kiwi_tag, to_kiwi_tag, KiwiPosTag, KiwiToken};
pub use lattice::{Lattice, Node, NodeBuilder, NodeType};
pub use nori_compat::{
    mecab_to_nori_tag, nori_to_mecab_tag, DecompoundMode, NoriAnalyzer, NoriToken, NoriTokenizer,
    WordType,
};
pub use pos_tag::PosTag;
pub use tokenizer::{Token, Tokenizer};
pub use unknown::{CharCategoryMap, UnknownDictionary, UnknownHandler};
pub use viterbi::{ConnectionCost, NbestSearcher, SpacePenalty, ViterbiSearcher};

/// 에러 모듈
pub mod error {
    use thiserror::Error;

    /// 핵심 엔진 에러 타입
    #[derive(Error, Debug)]
    pub enum Error {
        /// 사전 에러
        #[error("Dictionary error: {0}")]
        Dict(#[from] mecab_ko_dict::error::DictError),

        /// 분석 에러
        #[error("Analysis error: {0}")]
        Analysis(String),

        /// 초기화 에러
        #[error("Initialization error: {0}")]
        Init(String),

        /// Lattice 에러
        #[error("Lattice error: {0}")]
        Lattice(String),

        /// Viterbi 에러
        #[error("Viterbi error: {0}")]
        Viterbi(String),
    }

    /// Result 타입 별칭
    pub type Result<T> = std::result::Result<T, Error>;
}

/// 토크나이저 모듈
pub mod tokenizer {
    //! 토크나이저
    //!
    //! 형태소 분석의 메인 인터페이스

    use super::Result;

    /// 토큰
    #[derive(Debug, Clone, PartialEq)]
    pub struct Token {
        /// 표면형
        pub surface: String,
        /// 품사 태그
        pub pos: String,
        /// 시작 위치 (바이트)
        pub start: usize,
        /// 끝 위치 (바이트)
        pub end: usize,
        /// 읽기
        pub reading: Option<String>,
        /// 원형
        pub lemma: Option<String>,
    }

    /// 토크나이저
    pub struct Tokenizer {
        // TODO: 사전 등 필드 추가
    }

    impl Tokenizer {
        /// 기본 사전으로 토크나이저 생성
        pub fn new() -> Result<Self> {
            Ok(Self {})
        }

        /// 사전 경로 지정하여 생성
        pub fn with_dict(_dict_path: &str) -> Result<Self> {
            todo!("사전 로딩 구현 예정")
        }

        /// 형태소 분석
        pub fn tokenize(&self, text: &str) -> Vec<Token> {
            // TODO: 실제 분석 구현
            // 현재는 더미 구현
            vec![Token {
                surface: text.to_string(),
                pos: "UNK".to_string(),
                start: 0,
                end: text.len(),
                reading: None,
                lemma: None,
            }]
        }

        /// 분리만 수행 (wakati)
        pub fn wakati(&self, text: &str) -> Vec<String> {
            self.tokenize(text).into_iter().map(|t| t.surface).collect()
        }

        /// 명사만 추출
        pub fn nouns(&self, text: &str) -> Vec<String> {
            self.tokenize(text)
                .into_iter()
                .filter(|t| t.pos.starts_with("NN"))
                .map(|t| t.surface)
                .collect()
        }

        /// 형태소만 추출
        pub fn morphs(&self, text: &str) -> Vec<String> {
            self.wakati(text)
        }

        /// 품사 태깅
        pub fn pos(&self, text: &str) -> Vec<(String, String)> {
            self.tokenize(text)
                .into_iter()
                .map(|t| (t.surface, t.pos))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_creation() {
        let tokenizer = Tokenizer::new();
        assert!(tokenizer.is_ok());
    }

    #[test]
    fn test_basic_tokenize() {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("테스트");
        assert!(!tokens.is_empty());
    }
}
