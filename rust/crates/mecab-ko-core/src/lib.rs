//! # mecab-ko-core
//!
//! 한국어 형태소 분석 핵심 엔진
//!
//! ## 주요 기능
//!
//! - **형태소 분석**: Lattice 기반 Viterbi 알고리즘
//! - **N-best 탐색**: K개의 최적 분석 결과 제공
//! - **미등록어 처리**: 사전에 없는 단어 자동 처리
//! - **스트리밍 처리**: 대용량 파일 청크 단위 처리
//! - **토큰화 캐싱**: LRU 캐시로 반복 입력 최적화
//! - **분석 모드**: 명사/동사/원형 등 맞춤 분석
//! - **Lattice 시각화**: DOT/HTML/JSON 형식 지원
//! - **메모리 최적화**: String interning, 객체 풀링
//!
//! ## 빠른 시작
//!
//! ```rust,no_run
//! use mecab_ko_core::Tokenizer;
//!
//! let mut tokenizer = Tokenizer::new()?;
//! let tokens = tokenizer.tokenize("아버지가방에들어가신다");
//!
//! for token in tokens {
//!     println!("{}: {}", token.surface, token.pos);
//! }
//! # Ok::<(), mecab_ko_core::Error>(())
//! ```
//!
//! ## 고급 기능
//!
//! ### 명사만 추출
//!
//! ```rust,no_run
//! use mecab_ko_core::{Tokenizer, extract_nouns};
//!
//! let mut tokenizer = Tokenizer::new()?;
//! let nouns = extract_nouns(&mut tokenizer, "오늘 서울 날씨가 좋습니다");
//! // ["오늘", "서울", "날씨"]
//! # Ok::<(), mecab_ko_core::Error>(())
//! ```
//!
//! ### 스트리밍 처리
//!
//! ```rust,no_run
//! use mecab_ko_core::{Tokenizer, StreamingTokenizer};
//!
//! let tokenizer = Tokenizer::new()?;
//! let mut stream = StreamingTokenizer::new(tokenizer)
//!     .with_chunk_size(8192);
//!
//! let tokens = stream.process_chunk("첫 번째 청크. ");
//! let more_tokens = stream.process_chunk("두 번째 청크.");
//! let remaining = stream.flush();
//! # Ok::<(), mecab_ko_core::Error>(())
//! ```
//!
//! ### 토큰화 캐싱
//!
//! ```rust,no_run
//! use mecab_ko_core::{Tokenizer, TokenCache, CacheConfig};
//!
//! let mut tokenizer = Tokenizer::new()?;
//! let cache = TokenCache::new(CacheConfig::default());
//!
//! let text = "반복되는 입력";
//! let key = cache.make_key(text);
//!
//! // 캐시 조회 또는 계산
//! let tokens = cache.get_or_insert(key, || {
//!     tokenizer.tokenize(text)
//!         .into_iter()
//!         .map(|t| mecab_ko_core::CachedToken {
//!             surface: t.surface,
//!             pos: t.pos,
//!             start_byte: t.start_byte,
//!             end_byte: t.end_byte,
//!         })
//!         .collect()
//! });
//!
//! println!("캐시 히트율: {:.1}%", cache.stats().hit_rate() * 100.0);
//! # Ok::<(), mecab_ko_core::Error>(())
//! ```
//!
//! ### N-best 경로 탐색
//!
//! ```rust,no_run
//! use mecab_ko_core::{Tokenizer, ImprovedNbestSearcher};
//! use mecab_ko_dict::matrix::Matrix;
//!
//! let mut tokenizer = Tokenizer::new()?;
//! let lattice = tokenizer.tokenize_to_lattice("한국어");
//! // ... N-best 탐색 수행 ...
//! # Ok::<(), mecab_ko_core::Error>(())
//! ```
//!
//! ## 모듈 구조
//!
//! | 모듈 | 설명 |
//! |------|------|
//! | [`tokenizer`] | 형태소 분석 메인 인터페이스 |
//! | [`lattice`] | Lattice 그래프 구조 |
//! | [`viterbi`] | Viterbi 알고리즘 |
//! | [`nbest`] | N-best 경로 탐색 |
//! | [`streaming`] | 스트리밍 토큰화 |
//! | [`cache`] | 토큰화 캐싱 |
//! | [`batch`] | 배치/병렬 처리 |
//! | [`analysis_mode`] | 분석 모드 (명사/동사 추출 등) |
//! | [`memory`] | 메모리 최적화 유틸리티 |
//! | [`lattice_viz`] | Lattice 시각화 |
//! | [`nori_compat`] | Elasticsearch Nori 호환 |
//! | [`kiwi_compat`] | Kiwi 분석기 호환 |
//!
//! ## Feature Flags
//!
//! - `default`: zstd 압축 지원
//! - `async`: 비동기 토크나이저 (`tokio` 필요)
//! - `simd`: SIMD 최적화 (nightly 필요)
//! - `test-utils`: 테스트 유틸리티 공개

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![cfg_attr(feature = "simd", feature(portable_simd))]
#![allow(
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::needless_range_loop,
    clippy::inline_always,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::option_if_let_else,
    clippy::missing_panics_doc,
    clippy::unwrap_used
)]

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

pub mod analysis_mode;
pub mod batch;
pub mod cache;
pub mod evaluate;
pub mod kiwi_compat;
pub mod lattice;
pub mod lattice_viz;
pub mod memory;
pub mod nbest;
pub mod nori_compat;
pub mod normalizer;
pub mod pool;
pub mod pos_tag;
pub mod sejong;
pub mod streaming;
pub mod tokenizer;
pub mod unknown;
pub mod viterbi;

#[cfg(feature = "async")]
pub mod async_tokenizer;

pub use analysis_mode::{
    extract_adjectives, extract_content_words, extract_lemmas, extract_nouns, extract_verbs,
    AnalysisMode, AnalyzedToken, AnalyzerConfig, LemmatizationMode, PosFilter,
};
pub use batch::{BatchTokenizer, LargeFileProcessor, LargeFileProgress, ParallelStreamProcessor};
pub use cache::{CacheConfig, CacheStats, CachedToken, CachingTokenizer, TokenCache};
pub use error::{Error, Result};
pub use evaluate::{
    evaluate_dataset, evaluate_dataset_sejong, evaluate_tokens, EvaluateError, EvaluationResult,
    GoldSentence, GoldToken, PosStats, TestDataset,
};
pub use kiwi_compat::{from_kiwi_tag, to_kiwi_tag, KiwiPosTag, KiwiToken};
pub use lattice::{Lattice, Node, NodeBuilder, NodeType};
pub use lattice_viz::{
    lattice_to_dot, lattice_to_html, lattice_to_json, lattice_to_text, LatticeViz, VizFormat,
    VizOptions,
};
pub use memory::{
    estimate_tokens_memory, FeatureCache, InternerStats, MemoryStats, PosTagInterner,
};
pub use nbest::{ImprovedNbestSearcher, NbestPath, NbestResult};
pub use nori_compat::{
    mecab_to_nori_tag, nori_to_mecab_tag, DecompoundMode, NoriAnalyzer, NoriToken, NoriTokenizer,
    WordType,
};
pub use normalizer::{NormalizationConfig, NormalizationRule, Normalizer, RuleType};
pub use pool::{
    IdVecPool, NodeVecPool, PoolManager, PoolStats, SharedStringInterner, Symbol, TokenPool,
};
pub use pos_tag::PosTag;
pub use sejong::{EndingRule, SejongConverter, SejongToken};
pub use streaming::{
    ChunkedTokenIterator, ProgressCallback, ProgressStreamingTokenizer, StreamingProgress,
    StreamingTokenizer, TokenStream,
};
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_creation() {
        let tokenizer = Tokenizer::new();
        assert!(tokenizer.is_ok());
    }

    #[test]
    fn test_basic_tokenize() {
        let mut tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("안녕");
        assert!(!tokens.is_empty());
    }
}
