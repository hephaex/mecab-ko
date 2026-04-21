//! 사용자 정의 분석 모드
//!
//! 다양한 분석 요구사항을 지원하는 분석 모드를 제공합니다.
//!
//! # 개요
//!
//! 기본 토크나이저는 모든 형태소를 반환하지만, 많은 NLP 작업에서는
//! 특정 품사만 필요하거나, 원형 복원이 필요한 경우가 있습니다.
//!
//! 이 모듈은 다음 기능을 제공합니다:
//! - 품사 필터링 (명사, 동사, 형용사 등)
//! - 원형 복원 (동사/형용사 → 기본형)
//! - 커스텀 분석 모드 조합
//!
//! # Example
//!
//! ```rust,no_run
//! use mecab_ko_core::analysis_mode::{AnalysisMode, PosFilter, AnalyzerConfig};
//! use mecab_ko_core::tokenizer::Tokenizer;
//!
//! let mut tokenizer = Tokenizer::new().unwrap();
//!
//! // 명사만 추출
//! let config = AnalyzerConfig::new(AnalysisMode::NounsOnly);
//! let nouns = config.analyze(&mut tokenizer, "한국어 형태소 분석기");
//!
//! // 커스텀 품사 필터
//! let filter = PosFilter::new()
//!     .include_nouns()
//!     .include_verbs();
//! let config = AnalyzerConfig::with_filter(filter);
//! let tokens = config.analyze(&mut tokenizer, "아버지가 방에 들어가신다");
//! ```

use crate::tokenizer::{Token, Tokenizer};

/// 분석 모드
///
/// 토크나이저의 출력을 필터링/변환하는 모드입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnalysisMode {
    /// 모든 형태소 반환 (기본)
    #[default]
    Full,

    /// 명사만 추출 (NNG, NNP, NNB, NR, NP)
    NounsOnly,

    /// 동사만 추출 (VV)
    VerbsOnly,

    /// 형용사만 추출 (VA)
    AdjectivesOnly,

    /// 동사/형용사 추출 (VV, VA)
    PredicatesOnly,

    /// 내용어만 추출 (명사, 동사, 형용사, 부사)
    ContentWordsOnly,

    /// 표면형만 반환 (wakati 모드)
    SurfaceOnly,

    /// 원형 복원 모드 (동사/형용사를 기본형으로)
    Lemmatized,

    /// 품사 태그만 반환
    PosTagsOnly,

    /// 커스텀 필터 사용
    Custom,
}

impl AnalysisMode {
    /// 이 모드가 품사 필터링을 사용하는지 확인
    #[must_use]
    pub const fn uses_pos_filter(&self) -> bool {
        matches!(
            self,
            Self::NounsOnly
                | Self::VerbsOnly
                | Self::AdjectivesOnly
                | Self::PredicatesOnly
                | Self::ContentWordsOnly
                | Self::Custom
        )
    }

    /// 이 모드가 원형 복원을 사용하는지 확인
    #[must_use]
    pub const fn uses_lemmatization(&self) -> bool {
        matches!(self, Self::Lemmatized)
    }
}

/// 품사 필터
///
/// 특정 품사 태그를 포함하거나 제외합니다.
#[derive(Debug, Clone, Default)]
pub struct PosFilter {
    /// 포함할 품사 접두사 (예: "NN", "VV")
    include_prefixes: Vec<String>,
    /// 제외할 품사 접두사
    exclude_prefixes: Vec<String>,
    /// 포함할 정확한 품사 태그
    include_exact: Vec<String>,
    /// 제외할 정확한 품사 태그
    exclude_exact: Vec<String>,
}

impl PosFilter {
    /// 빈 필터 생성
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 명사 포함 (NNG, NNP, NNB, NR, NP)
    #[must_use]
    pub fn include_nouns(mut self) -> Self {
        self.include_prefixes.push("NN".to_string());
        self.include_prefixes.push("NR".to_string());
        self.include_prefixes.push("NP".to_string());
        self
    }

    /// 일반 명사만 포함 (NNG)
    #[must_use]
    pub fn include_common_nouns(mut self) -> Self {
        self.include_exact.push("NNG".to_string());
        self
    }

    /// 고유 명사만 포함 (NNP)
    #[must_use]
    pub fn include_proper_nouns(mut self) -> Self {
        self.include_exact.push("NNP".to_string());
        self
    }

    /// 동사 포함 (VV)
    #[must_use]
    pub fn include_verbs(mut self) -> Self {
        self.include_exact.push("VV".to_string());
        self
    }

    /// 형용사 포함 (VA)
    #[must_use]
    pub fn include_adjectives(mut self) -> Self {
        self.include_exact.push("VA".to_string());
        self
    }

    /// 용언 포함 (VV, VA, VX, VCP, VCN)
    #[must_use]
    pub fn include_predicates(mut self) -> Self {
        self.include_prefixes.push("V".to_string());
        self
    }

    /// 부사 포함 (MAG, MAJ)
    #[must_use]
    pub fn include_adverbs(mut self) -> Self {
        self.include_prefixes.push("MA".to_string());
        self
    }

    /// 조사 제외 (JK*, JX, JC)
    #[must_use]
    pub fn exclude_particles(mut self) -> Self {
        self.exclude_prefixes.push("J".to_string());
        self
    }

    /// 어미 제외 (E*)
    #[must_use]
    pub fn exclude_endings(mut self) -> Self {
        self.exclude_prefixes.push("E".to_string());
        self
    }

    /// 접사 제외 (XP*, XS*)
    #[must_use]
    pub fn exclude_affixes(mut self) -> Self {
        self.exclude_prefixes.push("X".to_string());
        self
    }

    /// 특수 기호 제외 (S*)
    #[must_use]
    pub fn exclude_symbols(mut self) -> Self {
        self.exclude_prefixes.push("S".to_string());
        self
    }

    /// 품사 접두사 포함 추가
    #[must_use]
    pub fn include_prefix(mut self, prefix: &str) -> Self {
        self.include_prefixes.push(prefix.to_string());
        self
    }

    /// 품사 접두사 제외 추가
    #[must_use]
    pub fn exclude_prefix(mut self, prefix: &str) -> Self {
        self.exclude_prefixes.push(prefix.to_string());
        self
    }

    /// 정확한 품사 태그 포함 추가
    #[must_use]
    pub fn include_tag(mut self, tag: &str) -> Self {
        self.include_exact.push(tag.to_string());
        self
    }

    /// 정확한 품사 태그 제외 추가
    #[must_use]
    pub fn exclude_tag(mut self, tag: &str) -> Self {
        self.exclude_exact.push(tag.to_string());
        self
    }

    /// 내용어 필터 생성 (명사, 동사, 형용사, 부사)
    #[must_use]
    pub fn content_words() -> Self {
        Self::new()
            .include_nouns()
            .include_verbs()
            .include_adjectives()
            .include_adverbs()
    }

    /// 품사가 필터를 통과하는지 확인
    #[must_use]
    pub fn matches(&self, pos: &str) -> bool {
        // 제외 목록 먼저 확인
        for excluded in &self.exclude_exact {
            if pos == excluded {
                return false;
            }
        }
        for excluded in &self.exclude_prefixes {
            if pos.starts_with(excluded) {
                return false;
            }
        }

        // 포함 목록이 비어있으면 모두 통과
        if self.include_exact.is_empty() && self.include_prefixes.is_empty() {
            return true;
        }

        // 포함 목록 확인
        for included in &self.include_exact {
            if pos == included {
                return true;
            }
        }
        for included in &self.include_prefixes {
            if pos.starts_with(included) {
                return true;
            }
        }

        false
    }
}

/// 원형 복원 설정
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LemmatizationMode {
    /// 원형 복원 안함
    #[default]
    None,

    /// 동사/형용사만 원형 복원
    PredicatesOnly,

    /// 모든 굴절 형태 원형 복원
    All,
}

/// 분석기 설정
///
/// 분석 모드, 필터, 원형 복원 설정을 조합합니다.
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// 분석 모드
    pub mode: AnalysisMode,
    /// 품사 필터 (Custom 모드에서 사용)
    pub pos_filter: Option<PosFilter>,
    /// 원형 복원 모드
    pub lemmatization: LemmatizationMode,
    /// 최소 토큰 길이 (문자 단위)
    pub min_length: usize,
    /// 최대 토큰 길이 (문자 단위, 0이면 제한 없음)
    pub max_length: usize,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            mode: AnalysisMode::Full,
            pos_filter: None,
            lemmatization: LemmatizationMode::None,
            min_length: 0,
            max_length: 0,
        }
    }
}

impl AnalyzerConfig {
    /// 새 분석기 설정 생성
    #[must_use]
    pub fn new(mode: AnalysisMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    /// 커스텀 필터로 설정 생성
    #[must_use]
    pub fn with_filter(filter: PosFilter) -> Self {
        Self {
            mode: AnalysisMode::Custom,
            pos_filter: Some(filter),
            ..Self::default()
        }
    }

    /// 원형 복원 모드 설정
    #[must_use]
    pub const fn with_lemmatization(mut self, mode: LemmatizationMode) -> Self {
        self.lemmatization = mode;
        self
    }

    /// 최소 토큰 길이 설정
    #[must_use]
    pub const fn with_min_length(mut self, len: usize) -> Self {
        self.min_length = len;
        self
    }

    /// 최대 토큰 길이 설정
    #[must_use]
    pub const fn with_max_length(mut self, len: usize) -> Self {
        self.max_length = len;
        self
    }

    /// 분석 수행
    ///
    /// 토크나이저를 사용하여 텍스트를 분석하고,
    /// 설정에 따라 결과를 필터링/변환합니다.
    pub fn analyze(&self, tokenizer: &mut Tokenizer, text: &str) -> Vec<AnalyzedToken> {
        let tokens = tokenizer.tokenize(text);
        self.process_tokens(tokens)
    }

    /// 토큰 목록 처리
    ///
    /// 이미 토크나이징된 결과를 필터링/변환합니다.
    #[must_use]
    pub fn process_tokens(&self, tokens: Vec<Token>) -> Vec<AnalyzedToken> {
        tokens
            .into_iter()
            .filter(|t| self.filter_token(t))
            .map(|t| self.transform_token(t))
            .collect()
    }

    /// 토큰 필터링
    fn filter_token(&self, token: &Token) -> bool {
        // 길이 필터
        let char_len = token.char_len();
        if self.min_length > 0 && char_len < self.min_length {
            return false;
        }
        if self.max_length > 0 && char_len > self.max_length {
            return false;
        }

        // 품사 필터
        match self.mode {
            AnalysisMode::Full
            | AnalysisMode::SurfaceOnly
            | AnalysisMode::Lemmatized
            | AnalysisMode::PosTagsOnly => true,
            AnalysisMode::NounsOnly => {
                token.pos.starts_with("NN")
                    || token.pos.starts_with("NR")
                    || token.pos.starts_with("NP")
            }
            AnalysisMode::VerbsOnly => token.pos == "VV",
            AnalysisMode::AdjectivesOnly => token.pos == "VA",
            AnalysisMode::PredicatesOnly => token.pos == "VV" || token.pos == "VA",
            AnalysisMode::ContentWordsOnly => {
                token.pos.starts_with("NN")
                    || token.pos.starts_with("NR")
                    || token.pos.starts_with("NP")
                    || token.pos == "VV"
                    || token.pos == "VA"
                    || token.pos.starts_with("MA")
            }
            AnalysisMode::Custom => self
                .pos_filter
                .as_ref()
                .map_or(true, |f| f.matches(&token.pos)),
        }
    }

    /// 토큰 변환
    fn transform_token(&self, token: Token) -> AnalyzedToken {
        let surface = match self.lemmatization {
            LemmatizationMode::None => token.surface.clone(),
            LemmatizationMode::PredicatesOnly => {
                if token.pos == "VV" || token.pos == "VA" {
                    token.lemma.clone().unwrap_or_else(|| token.surface.clone())
                } else {
                    token.surface.clone()
                }
            }
            LemmatizationMode::All => token.lemma.clone().unwrap_or_else(|| token.surface.clone()),
        };

        AnalyzedToken {
            surface,
            original_surface: token.surface,
            pos: token.pos,
            start_pos: token.start_pos,
            end_pos: token.end_pos,
            lemma: token.lemma,
            is_lemmatized: self.lemmatization != LemmatizationMode::None,
        }
    }
}

/// 분석된 토큰
///
/// 분석 모드에 따라 변환된 토큰입니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzedToken {
    /// 표면형 (원형 복원 시 기본형)
    pub surface: String,
    /// 원본 표면형
    pub original_surface: String,
    /// 품사 태그
    pub pos: String,
    /// 시작 위치
    pub start_pos: usize,
    /// 끝 위치
    pub end_pos: usize,
    /// 원형 (사전에 있는 경우)
    pub lemma: Option<String>,
    /// 원형 복원 적용 여부
    pub is_lemmatized: bool,
}

impl AnalyzedToken {
    /// 토큰 길이 (문자 단위)
    #[must_use]
    pub const fn char_len(&self) -> usize {
        self.end_pos - self.start_pos
    }
}

/// 편의 함수: 명사만 추출
pub fn extract_nouns(tokenizer: &mut Tokenizer, text: &str) -> Vec<String> {
    AnalyzerConfig::new(AnalysisMode::NounsOnly)
        .analyze(tokenizer, text)
        .into_iter()
        .map(|t| t.surface)
        .collect()
}

/// 편의 함수: 동사만 추출
pub fn extract_verbs(tokenizer: &mut Tokenizer, text: &str) -> Vec<String> {
    AnalyzerConfig::new(AnalysisMode::VerbsOnly)
        .analyze(tokenizer, text)
        .into_iter()
        .map(|t| t.surface)
        .collect()
}

/// 편의 함수: 형용사만 추출
pub fn extract_adjectives(tokenizer: &mut Tokenizer, text: &str) -> Vec<String> {
    AnalyzerConfig::new(AnalysisMode::AdjectivesOnly)
        .analyze(tokenizer, text)
        .into_iter()
        .map(|t| t.surface)
        .collect()
}

/// 편의 함수: 내용어만 추출
pub fn extract_content_words(tokenizer: &mut Tokenizer, text: &str) -> Vec<String> {
    AnalyzerConfig::new(AnalysisMode::ContentWordsOnly)
        .analyze(tokenizer, text)
        .into_iter()
        .map(|t| t.surface)
        .collect()
}

/// 편의 함수: 원형 복원된 형태소 추출
pub fn extract_lemmas(tokenizer: &mut Tokenizer, text: &str) -> Vec<String> {
    AnalyzerConfig::new(AnalysisMode::Lemmatized)
        .with_lemmatization(LemmatizationMode::All)
        .analyze(tokenizer, text)
        .into_iter()
        .map(|t| t.surface)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_mode_uses_filter() {
        assert!(!AnalysisMode::Full.uses_pos_filter());
        assert!(AnalysisMode::NounsOnly.uses_pos_filter());
        assert!(AnalysisMode::Custom.uses_pos_filter());
    }

    #[test]
    fn test_pos_filter_matches_nouns() {
        let filter = PosFilter::new().include_nouns();

        assert!(filter.matches("NNG"));
        assert!(filter.matches("NNP"));
        assert!(filter.matches("NNB"));
        assert!(filter.matches("NR"));
        assert!(filter.matches("NP"));
        assert!(!filter.matches("VV"));
        assert!(!filter.matches("JKS"));
    }

    #[test]
    fn test_pos_filter_matches_verbs() {
        let filter = PosFilter::new().include_verbs();

        assert!(filter.matches("VV"));
        assert!(!filter.matches("VA"));
        assert!(!filter.matches("NNG"));
    }

    #[test]
    fn test_pos_filter_matches_predicates() {
        let filter = PosFilter::new().include_predicates();

        assert!(filter.matches("VV"));
        assert!(filter.matches("VA"));
        assert!(filter.matches("VX"));
        assert!(filter.matches("VCP"));
        assert!(!filter.matches("NNG"));
    }

    #[test]
    fn test_pos_filter_content_words() {
        let filter = PosFilter::content_words();

        assert!(filter.matches("NNG"));
        assert!(filter.matches("VV"));
        assert!(filter.matches("VA"));
        assert!(filter.matches("MAG"));
        assert!(!filter.matches("JKS"));
        assert!(!filter.matches("EC"));
    }

    #[test]
    fn test_pos_filter_exclude() {
        let filter = PosFilter::new().include_prefix("N").exclude_tag("NNB");

        assert!(filter.matches("NNG"));
        assert!(filter.matches("NNP"));
        assert!(!filter.matches("NNB")); // 제외됨
        assert!(!filter.matches("VV"));
    }

    #[test]
    fn test_pos_filter_empty_includes_all() {
        let filter = PosFilter::new();

        assert!(filter.matches("NNG"));
        assert!(filter.matches("VV"));
        assert!(filter.matches("JKS"));
    }

    #[test]
    fn test_analyzer_config_default() {
        let config = AnalyzerConfig::default();

        assert_eq!(config.mode, AnalysisMode::Full);
        assert!(config.pos_filter.is_none());
        assert_eq!(config.lemmatization, LemmatizationMode::None);
    }

    #[test]
    fn test_analyzer_config_with_filter() {
        let filter = PosFilter::new().include_nouns();
        let config = AnalyzerConfig::with_filter(filter);

        assert_eq!(config.mode, AnalysisMode::Custom);
        assert!(config.pos_filter.is_some());
    }

    #[test]
    fn test_analyzer_config_process_tokens() {
        let tokens = vec![
            Token {
                surface: "한국어".to_string(),
                pos: "NNG".to_string(),
                start_pos: 0,
                end_pos: 3,
                start_byte: 0,
                end_byte: 9,
                reading: None,
                lemma: None,
                cost: 0,
                features: String::new(),
                normalized: None,
            },
            Token {
                surface: "가".to_string(),
                pos: "JKS".to_string(),
                start_pos: 3,
                end_pos: 4,
                start_byte: 9,
                end_byte: 12,
                reading: None,
                lemma: None,
                cost: 0,
                features: String::new(),
                normalized: None,
            },
        ];

        // NounsOnly 모드
        let config = AnalyzerConfig::new(AnalysisMode::NounsOnly);
        let result = config.process_tokens(tokens);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].surface, "한국어");
    }

    #[test]
    fn test_analyzer_config_min_length() {
        let tokens = vec![
            Token {
                surface: "가".to_string(),
                pos: "NNG".to_string(),
                start_pos: 0,
                end_pos: 1,
                start_byte: 0,
                end_byte: 3,
                reading: None,
                lemma: None,
                cost: 0,
                features: String::new(),
                normalized: None,
            },
            Token {
                surface: "한국어".to_string(),
                pos: "NNG".to_string(),
                start_pos: 1,
                end_pos: 4,
                start_byte: 3,
                end_byte: 12,
                reading: None,
                lemma: None,
                cost: 0,
                features: String::new(),
                normalized: None,
            },
        ];

        let config = AnalyzerConfig::new(AnalysisMode::NounsOnly).with_min_length(2);
        let result = config.process_tokens(tokens);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].surface, "한국어");
    }

    #[test]
    fn test_lemmatization_mode() {
        let tokens = vec![Token {
            surface: "먹었".to_string(),
            pos: "VV".to_string(),
            start_pos: 0,
            end_pos: 2,
            start_byte: 0,
            end_byte: 6,
            reading: Some("먹".to_string()),
            lemma: Some("먹다".to_string()),
            cost: 0,
            features: String::new(),
            normalized: None,
        }];

        // 원형 복원 없음
        let config = AnalyzerConfig::new(AnalysisMode::Full);
        let result = config.process_tokens(tokens.clone());
        assert_eq!(result[0].surface, "먹었");

        // 용언 원형 복원
        let config = AnalyzerConfig::new(AnalysisMode::Lemmatized)
            .with_lemmatization(LemmatizationMode::PredicatesOnly);
        let result = config.process_tokens(tokens);
        assert_eq!(result[0].surface, "먹다");
    }
}
