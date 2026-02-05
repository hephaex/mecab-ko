//! 분석기 설정 타입

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 복합명사 분해 모드
///
/// Lucene Nori의 decompound 설정과 호환
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecompoundMode {
    /// 분해하지 않음 - 복합명사를 그대로 출력
    ///
    /// # Example
    /// `"형태소분석기" → ["형태소분석기/NNG"]`
    None,

    /// 분해만 출력 - 원본은 버리고 분해된 형태소만 출력
    ///
    /// # Example
    /// `"형태소분석기" → ["형태소/NNG", "분석/NNG", "기/NNG"]`
    Discard,

    /// 혼합 출력 - 원본과 분해된 형태소 모두 출력
    ///
    /// # Example
    /// `"형태소분석기" → ["형태소분석기/NNG", "형태소/NNG", "분석/NNG", "기/NNG"]`
    Mixed,
}

impl DecompoundMode {
    /// 문자열에서 파싱
    ///
    /// # Errors
    ///
    /// 유효하지 않은 모드 문자열인 경우 에러 반환
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "discard" => Ok(Self::Discard),
            "mixed" => Ok(Self::Mixed),
            _ => Err(Error::config(format!("Invalid decompound mode: {s}"))),
        }
    }

    /// 문자열 표현
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Discard => "discard",
            Self::Mixed => "mixed",
        }
    }
}

impl Default for DecompoundMode {
    fn default() -> Self {
        Self::None
    }
}

/// 분석기 설정
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    /// 복합명사 분해 모드
    #[serde(default)]
    pub decompound_mode: DecompoundMode,

    /// 사용자 사전 경로
    pub user_dictionary_path: Option<PathBuf>,

    /// 제거할 품사 태그 목록
    #[serde(default = "default_stoptags")]
    pub stoptags: Vec<String>,

    /// 미등록어를 유니그램으로 출력할지 여부
    #[serde(default)]
    pub output_unknown_unigrams: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            decompound_mode: DecompoundMode::None,
            user_dictionary_path: None,
            stoptags: default_stoptags(),
            output_unknown_unigrams: false,
        }
    }
}

impl AnalyzerConfig {
    /// 새 설정 생성
    #[must_use]
    pub const fn new() -> Self {
        Self {
            decompound_mode: DecompoundMode::None,
            user_dictionary_path: None,
            stoptags: Vec::new(),
            output_unknown_unigrams: false,
        }
    }

    /// 복합명사 분해 모드 설정
    #[must_use]
    pub const fn with_decompound_mode(mut self, mode: DecompoundMode) -> Self {
        self.decompound_mode = mode;
        self
    }

    /// 사용자 사전 경로 설정
    #[must_use]
    pub fn with_user_dictionary(mut self, path: PathBuf) -> Self {
        self.user_dictionary_path = Some(path);
        self
    }

    /// stoptags 설정
    #[must_use]
    pub fn with_stoptags(mut self, tags: Vec<String>) -> Self {
        self.stoptags = tags;
        self
    }

    /// 미등록어 유니그램 출력 설정
    #[must_use]
    pub const fn with_output_unknown_unigrams(mut self, output: bool) -> Self {
        self.output_unknown_unigrams = output;
        self
    }

    /// 유효성 검증
    ///
    /// # Errors
    ///
    /// - 사용자 사전 경로가 존재하지 않는 경우
    pub fn validate(&self) -> Result<()> {
        if let Some(path) = &self.user_dictionary_path {
            if !path.exists() {
                return Err(Error::config(format!(
                    "User dictionary not found: {}",
                    path.display()
                )));
            }
        }
        Ok(())
    }
}

/// 기본 stoptags (조사 J, 어미 E)
fn default_stoptags() -> Vec<String> {
    vec!["J".to_string(), "E".to_string()]
}

/// 토크나이저 설정
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenizerConfig {
    /// 복합명사 분해 모드
    #[serde(default)]
    pub decompound_mode: DecompoundMode,

    /// 사용자 사전 경로
    pub user_dictionary_path: Option<PathBuf>,

    /// 미등록어를 유니그램으로 출력할지 여부
    #[serde(default)]
    pub output_unknown_unigrams: bool,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            decompound_mode: DecompoundMode::None,
            user_dictionary_path: None,
            output_unknown_unigrams: false,
        }
    }
}

impl From<&AnalyzerConfig> for TokenizerConfig {
    fn from(config: &AnalyzerConfig) -> Self {
        Self {
            decompound_mode: config.decompound_mode,
            user_dictionary_path: config.user_dictionary_path.clone(),
            output_unknown_unigrams: config.output_unknown_unigrams,
        }
    }
}

/// 필터 설정
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterConfig {
    /// 필터 타입
    pub filter_type: FilterType,

    /// stoptags (POS 필터용)
    #[serde(default)]
    pub stoptags: Vec<String>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            filter_type: FilterType::PartOfSpeech,
            stoptags: Vec::new(),
        }
    }
}

/// 필터 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterType {
    /// 품사 기반 필터
    PartOfSpeech,
    /// 읽기 형태 필터
    ReadingForm,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompound_mode_from_str() {
        assert_eq!(
            DecompoundMode::from_str("none").ok(),
            Some(DecompoundMode::None)
        );
        assert_eq!(
            DecompoundMode::from_str("discard").ok(),
            Some(DecompoundMode::Discard)
        );
        assert_eq!(
            DecompoundMode::from_str("mixed").ok(),
            Some(DecompoundMode::Mixed)
        );
        assert_eq!(
            DecompoundMode::from_str("NONE").ok(),
            Some(DecompoundMode::None)
        );
        assert!(DecompoundMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_decompound_mode_as_str() {
        assert_eq!(DecompoundMode::None.as_str(), "none");
        assert_eq!(DecompoundMode::Discard.as_str(), "discard");
        assert_eq!(DecompoundMode::Mixed.as_str(), "mixed");
    }

    #[test]
    fn test_analyzer_config_default() {
        let config = AnalyzerConfig::default();
        assert_eq!(config.decompound_mode, DecompoundMode::None);
        assert_eq!(config.stoptags.len(), 2);
        assert!(config.stoptags.contains(&"J".to_string()));
        assert!(config.stoptags.contains(&"E".to_string()));
    }

    #[test]
    fn test_analyzer_config_builder() {
        let config = AnalyzerConfig::new()
            .with_decompound_mode(DecompoundMode::Mixed)
            .with_stoptags(vec!["J".to_string()])
            .with_output_unknown_unigrams(true);

        assert_eq!(config.decompound_mode, DecompoundMode::Mixed);
        assert_eq!(config.stoptags, vec!["J".to_string()]);
        assert!(config.output_unknown_unigrams);
    }

    #[test]
    fn test_config_serialization() {
        let config = AnalyzerConfig::default();
        let json = serde_json::to_string(&config);
        assert!(json.is_ok());

        let deserialized: std::result::Result<AnalyzerConfig, serde_json::Error> =
            serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
}
