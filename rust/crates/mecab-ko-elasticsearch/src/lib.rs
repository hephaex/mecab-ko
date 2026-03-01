//! Elasticsearch/Lucene Nori 호환 분석기
//!
//! Apache Lucene의 한국어 분석기 Nori와 호환되는 Elasticsearch 플러그인을 제공합니다.
//!
//! # 주요 기능
//!
//! - **Nori 호환 분석기**: `NoriAnalyzer`, `NoriTokenizer`
//! - **토큰 필터**: `NoriPartOfSpeechStopFilter`, `NoriReadingFormFilter`
//! - **JNI 바인딩**: Java/Elasticsearch와의 통합
//! - **설정 옵션**: 복합명사 분해 모드, 사용자 사전, stoptags
//!
//! # 아키텍처
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │         Elasticsearch Plugin                │
//! ├─────────────────────────────────────────────┤
//! │  JNI Bindings (Java ↔ Rust)                │
//! ├─────────────────────────────────────────────┤
//! │  Analysis Pipeline                          │
//! │  ├─ Analyzer                                │
//! │  ├─ Tokenizer                               │
//! │  └─ TokenFilter                             │
//! ├─────────────────────────────────────────────┤
//! │  Nori Compatibility Layer                   │
//! │  ├─ NoriTokenizer                           │
//! │  ├─ NoriPartOfSpeechStopFilter             │
//! │  └─ NoriReadingFormFilter                   │
//! ├─────────────────────────────────────────────┤
//! │  MeCab-Ko Core Engine                       │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # 사용 예제
//!
//! ## Rust API
//!
//! ```rust,no_run
//! use mecab_ko_elasticsearch::analyzer::NoriAnalyzer;
//! use mecab_ko_elasticsearch::config::{AnalyzerConfig, DecompoundMode};
//!
//! // Analyzer 생성
//! let config = AnalyzerConfig {
//!     decompound_mode: DecompoundMode::Mixed,
//!     user_dictionary_path: None,
//!     stoptags: vec!["J".to_string(), "E".to_string()],
//!     output_unknown_unigrams: false,
//! };
//!
//! let analyzer = NoriAnalyzer::new(config).unwrap();
//!
//! // 텍스트 분석
//! let tokens = analyzer.analyze("한국어 형태소 분석기").unwrap();
//!
//! for token in tokens {
//!     println!("{}: {} [{}]", token.surface, token.pos_tag, token.reading.clone().unwrap_or_default());
//! }
//! ```
//!
//! ## Elasticsearch 설정
//!
//! ```json
//! {
//!   "settings": {
//!     "analysis": {
//!       "analyzer": {
//!         "nori_analyzer": {
//!           "type": "custom",
//!           "tokenizer": "nori_tokenizer",
//!           "filter": ["nori_posfilter", "lowercase"]
//!         }
//!       },
//!       "tokenizer": {
//!         "nori_tokenizer": {
//!           "type": "nori_tokenizer",
//!           "decompound_mode": "mixed",
//!           "user_dictionary": "userdict_ko.txt"
//!         }
//!       },
//!       "filter": {
//!         "nori_posfilter": {
//!           "type": "nori_part_of_speech",
//!           "stoptags": ["J", "E", "SF"]
//!         }
//!       }
//!     }
//!   }
//! }
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod analyzer;
pub mod config;
pub mod error;
pub mod filter;
pub mod tokenizer;

#[cfg(feature = "jni-bindings")]
pub mod jni;

// Re-exports for convenience
pub use analyzer::{NoriAnalyzer, NoriTokenizerImpl};
pub use config::{AnalyzerConfig, DecompoundMode};
pub use error::{Error, Result};
pub use filter::{NoriPartOfSpeechStopFilter, NoriReadingFormFilter, TokenFilter};
pub use tokenizer::{Token, TokenStream, Tokenizer};

/// 라이브러리 버전 정보
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
