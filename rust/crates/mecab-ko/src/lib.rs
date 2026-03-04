//! # mecab-ko
//!
//! 한국어 형태소 분석기 - MeCab-Ko의 순수 Rust 구현
//!
//! ## 소개
//!
//! `mecab-ko`는 한국어 형태소 분석을 위한 라이브러리입니다.
//! 은전한닢(mecab-ko)의 순수 Rust 재구현으로, 빠르고 안전한 형태소 분석을 제공합니다.
//!
//! ## 주요 특징
//!
//! - **순수 Rust**: unsafe 코드 없이 메모리 안전성 보장
//! - **한국어 최적화**: 띄어쓰기 패널티, 한글 자모 처리
//! - **고성능**: Zero-copy 사전, 효율적인 Viterbi 구현
//! - **유연성**: 사용자 사전, 필터 시스템 지원
//!
//! ## 빠른 시작
//!
//! ```rust,no_run
//! use mecab_ko::Tokenizer;
//!
//! let mut tokenizer = Tokenizer::new().unwrap();
//!
//! // 기본 형태소 분석
//! let tokens = tokenizer.tokenize("아버지가방에들어가신다");
//! for token in tokens {
//!     println!("{}\t{}", token.surface, token.pos);
//! }
//!
//! // 분리만 (wakati)
//! let words = tokenizer.wakati("안녕하세요");
//! println!("{:?}", words);  // ["안녕", "하", "세요"]
//!
//! // 명사만 추출
//! let nouns = tokenizer.nouns("오늘 날씨가 좋습니다");
//! println!("{:?}", nouns);  // ["오늘", "날씨"]
//! ```
//!
//! ## 사용자 사전 추가
//!
//! ```rust,no_run
//! use mecab_ko::Tokenizer;
//! use mecab_ko::dict::UserDictionary;
//!
//! let mut user_dict = UserDictionary::new();
//! user_dict.add_entry("챗GPT", "NNP", Some(-2000), None);
//! user_dict.add_entry("딥러닝", "NNG", Some(-1500), None);
//!
//! let tokenizer = Tokenizer::new().unwrap()
//!     .with_user_dict(user_dict);
//! ```
//!
//! ## 성능 팁
//!
//! 1. **토크나이저 재사용**: `Tokenizer`는 내부에 Lattice를 재사용하므로 매번 새로 생성하지 마세요.
//! 2. **배치 처리**: 많은 텍스트를 처리할 때는 `mecab_ko_core::BatchTokenizer`를 사용하세요.
//! 3. **캐싱**: 반복되는 입력이 있으면 `mecab_ko_core::CachingTokenizer`를 활용하세요.
//!
//! ## 고급 기능 (v0.3.0+)
//!
//! ### N-best 경로 탐색
//!
//! ```rust,no_run
//! use mecab_ko::{Tokenizer, ImprovedNbestSearcher};
//!
//! let tokenizer = Tokenizer::new().unwrap();
//! let nbest = ImprovedNbestSearcher::new(5); // 상위 5개 경로
//! ```
//!
//! ### 분석 모드
//!
//! ```rust,no_run
//! use mecab_ko::{AnalysisMode, PosFilter};
//!
//! // 명사만 추출
//! let filter = PosFilter::new()
//!     .include_prefix("NNG")
//!     .include_prefix("NNP");
//! ```
//!
//! ### 토큰화 캐싱
//!
//! ```rust,no_run
//! use mecab_ko::{Tokenizer, CachingTokenizer, CacheConfig};
//!
//! let tokenizer = Tokenizer::new().unwrap();
//! let cached = CachingTokenizer::new(tokenizer, CacheConfig::default());
//! ```
//!
//! ### 배치/스트리밍 처리
//!
//! ```rust,no_run
//! use mecab_ko::BatchTokenizer;
//!
//! // 병렬 배치 처리 (기본 스레드 수)
//! let batch = BatchTokenizer::new().unwrap();
//! ```
//!
//! ## 모듈 구조
//!
//! - [`tokenizer`]: 형태소 분석 메인 인터페이스
//! - [`pos_tag`]: 품사 태그 정의 (세종 품사 체계)
//! - [`hangul`]: 한글 자모 처리 유틸리티
//! - [`dict`]: 사전 로딩 및 검색
//!
//! ## Feature Flags
//!
//! - `builder`: 사전 빌더 기능 포함

#![warn(missing_docs)]
#![deny(unsafe_code)]

// Re-export core types
pub use mecab_ko_core::{
    pos_tag::PosTag,
    tokenizer::{Token, Tokenizer},
    Error, Result,
};

// Re-export v0.3.0 features
pub use mecab_ko_core::{
    analysis_mode::{AnalysisMode, AnalyzerConfig, PosFilter},
    batch::{BatchTokenizer, LargeFileProcessor},
    cache::{CacheConfig, CacheStats, CachingTokenizer, TokenCache},
    nbest::{ImprovedNbestSearcher, NbestPath, NbestResult},
    streaming::{StreamingTokenizer, TokenStream},
};

// Re-export hangul utilities
pub use mecab_ko_hangul::{
    classify_char, compose, compose_str, decompose, decompose_str, has_jongseong, is_hangul,
    is_hangul_syllable, is_jamo, CharType,
};

// Re-export dictionary types
pub use mecab_ko_dict::{Dictionary, Entry};

/// 한글 자모 처리
///
/// 한글 음절의 자모 분리/결합, 종성 판별 등을 제공합니다.
pub mod hangul {
    pub use mecab_ko_hangul::*;
}

/// 품사 태그 정의
///
/// 세종 품사 태그 체계 + mecab-ko-dic 확장 태그
pub mod pos_tag {
    pub use mecab_ko_core::pos_tag::*;
}

/// 형태소 분석기
///
/// 메인 형태소 분석 인터페이스
pub mod tokenizer {
    pub use mecab_ko_core::tokenizer::*;
}

/// 형태소 사전
///
/// 사전 로딩, 검색, 연접 비용 조회
pub mod dict {
    pub use mecab_ko_dict::*;
}

/// 사전 빌더 (builder feature 활성화 시)
#[cfg(feature = "builder")]
pub mod builder {
    //! 사전 빌더
    //!
    //! CSV에서 바이너리 사전 생성

    pub use mecab_ko_dict_builder::*;
}

/// 버전 정보
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
#[allow(clippy::similar_names, clippy::unwrap_used, clippy::const_is_empty)]
mod tests {
    use super::*;

    #[test]
    fn test_hangul_decompose() {
        let (cho, jung, jong) = decompose('한').unwrap();
        assert_eq!(cho, 'ㅎ');
        assert_eq!(jung, 'ㅏ');
        assert_eq!(jong, Some('ㄴ'));
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
