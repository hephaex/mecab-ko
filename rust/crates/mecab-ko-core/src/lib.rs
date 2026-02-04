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
#![cfg_attr(feature = "simd", feature(portable_simd))]

pub mod batch;
pub mod kiwi_compat;
pub mod lattice;
pub mod nori_compat;
pub mod normalizer;
pub mod pool;
pub mod pos_tag;
pub mod streaming;
pub mod tokenizer;
pub mod unknown;
pub mod viterbi;

#[cfg(feature = "async")]
pub mod async_tokenizer;

pub use batch::{BatchTokenizer, ParallelStreamProcessor};
pub use error::{Error, Result};
pub use kiwi_compat::{from_kiwi_tag, to_kiwi_tag, KiwiPosTag, KiwiToken};
pub use lattice::{Lattice, Node, NodeBuilder, NodeType};
pub use nori_compat::{
    mecab_to_nori_tag, nori_to_mecab_tag, DecompoundMode, NoriAnalyzer, NoriToken, NoriTokenizer,
    WordType,
};
pub use normalizer::{NormalizationConfig, NormalizationRule, Normalizer, RuleType};
pub use pool::{
    IdVecPool, NodeVecPool, PoolManager, PoolStats, SharedStringInterner, Symbol, TokenPool,
};
pub use pos_tag::PosTag;
pub use streaming::{StreamingTokenizer, TokenStream};
pub use tokenizer::{Token, Tokenizer};
pub use unknown::{CharCategoryMap, UnknownDictionary, UnknownHandler};
pub use viterbi::{ConnectionCost, NbestSearcher, SpacePenalty, ViterbiSearcher};

#[cfg(feature = "async")]
pub use async_tokenizer::{AsyncStreamingTokenizer, AsyncTokenizer};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_tokenizer_creation() {
        let tokenizer = Tokenizer::new();
        assert!(tokenizer.is_ok());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_basic_tokenize() {
        let mut tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("테스트");
        assert!(!tokens.is_empty());
    }
}
