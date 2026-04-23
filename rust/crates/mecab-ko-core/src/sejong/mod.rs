//! 세종 코퍼스 호환 모듈
//!
//! mecab-ko-dic 출력을 세종 코퍼스 형식으로 변환합니다.
//!
//! # 배경
//!
//! mecab-ko-dic과 세종 코퍼스는 토큰화 기준이 다릅니다:
//! - mecab-ko-dic: 어미 결합 (갔다/VV+EF)
//! - 세종 코퍼스: 어미 분리 (갔/VV 다/EF)
//!
//! # 분析결과 활용
//!
//! mecab-ko-dic의 12번째 컬럼에는 형태소 분해 정보가 저장되어 있습니다:
//! - 형식: `stem/POS/*+ending/POS/*`
//! - 예시: `가깝/VA/*+아/EC/*` (가까와 → 가깝 + 아)
//!
//! 이 정보를 활용하면 불규칙 활용도 정확하게 분리할 수 있습니다.
//!
//! # 예제
//!
//! ```rust,no_run
//! use mecab_ko_core::sejong::{SejongConverter, SejongToken};
//! use mecab_ko_core::tokenizer::Tokenizer;
//!
//! let mut tokenizer = Tokenizer::new().unwrap();
//! let converter = SejongConverter::new();
//!
//! let tokens = tokenizer.tokenize("갔다");
//! let sejong_tokens = converter.convert_tokens(&tokens);
//!
//! // "갔다/VV+EF" -> ["갔/VV", "다/EF"]
//! ```

pub mod converter;
pub mod corrections;
pub mod ending_rules;
pub mod hangul;
pub mod lexicon;
pub mod postprocess;
pub mod splitter;
pub mod tag_map;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export public types for backward compatibility
pub use converter::SejongConverter;
pub use types::{DecomposedMorpheme, EndingRule, SejongToken};
