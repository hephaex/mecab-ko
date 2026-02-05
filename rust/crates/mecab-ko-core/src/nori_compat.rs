//! Lucene Nori 호환 레이어
//!
//! Apache Lucene의 한국어 분석기 Nori와 호환되는 인터페이스를 제공합니다.
//!
//! # 주요 기능
//!
//! - `NoriTokenizer`: Nori 스타일 토크나이저
//! - `NoriAnalyzer`: 분석기 래퍼 (사용자 사전, stoptags 지원)
//! - POS 태그 매핑: `MeCab` ↔ Nori 변환
//!
//! # 예제
//!
//! ```rust,ignore
//! use mecab_ko_core::nori_compat::{NoriTokenizer, DecompoundMode};
//!
//! let tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, true)?;
//! let tokens = tokenizer.tokenize("형태소분석기")?;
//!
//! for token in tokens {
//!     println!("{}: {}", token.surface, token.pos_tag);
//! }
//! ```

use crate::pos_tag::PosTag;
use crate::tokenizer::{Token, Tokenizer};
use crate::Result;
use std::collections::HashSet;

/// Nori 복합명사 분해 모드
///
/// Lucene Nori의 decompound 설정과 호환
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecompoundMode {
    /// 분해하지 않음 - 복합명사를 그대로 출력
    ///
    /// # Example
    /// "형태소분석기" → \["형태소분석기/NNG"\]
    None,

    /// 분해만 출력 - 원본은 버리고 분해된 형태소만 출력
    ///
    /// # Example
    /// "형태소분석기" → \["형태소/NNG", "분석/NNG", "기/NNG"\]
    Discard,

    /// 혼합 출력 - 원본과 분해된 형태소 모두 출력
    ///
    /// # Example
    /// "형태소분석기" → \["형태소분석기/NNG", "형태소/NNG", "분석/NNG", "기/NNG"\]
    Mixed,
}

impl DecompoundMode {
    /// 문자열에서 파싱
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "discard" => Some(Self::Discard),
            "mixed" => Some(Self::Mixed),
            _ => None,
        }
    }

    /// 문자열에서 파싱 (parse의 별칭)
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Self::parse(s)
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

/// Nori 토큰
///
/// Lucene Nori의 Token 속성과 호환
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoriToken {
    /// 표면형
    pub surface: String,
    /// Nori 스타일 품사 태그 (J, E 통합)
    pub pos_tag: String,
    /// 시작 위치 (문자 오프셋)
    pub start_offset: usize,
    /// 끝 위치 (문자 오프셋)
    pub end_offset: usize,
    /// 원형 (기본형)
    pub lemma: Option<String>,
    /// 읽기 (발음)
    pub reading: Option<String>,
    /// 단어 타입 (KNOWN, UNKNOWN, etc.)
    pub word_type: WordType,
    /// 복합명사 분해 여부
    pub is_decompound: bool,
}

/// 단어 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WordType {
    /// 사전에 등록된 단어
    Known,
    /// 미등록어
    Unknown,
    /// 사용자 사전 단어
    User,
}

impl WordType {
    /// 문자열 표현
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Known => "KNOWN",
            Self::Unknown => "UNKNOWN",
            Self::User => "USER",
        }
    }
}

/// Nori 토크나이저
///
/// Lucene Nori의 `KoreanTokenizer`와 호환되는 인터페이스
pub struct NoriTokenizer {
    /// 내부 `MeCab` 토크나이저
    tokenizer: Tokenizer,
    /// 복합명사 분해 모드
    decompound_mode: DecompoundMode,
    /// 미등록어를 유니그램으로 출력할지 여부
    output_unknown_unigrams: bool,
}

impl NoriTokenizer {
    /// 새 Nori 토크나이저 생성
    ///
    /// # Arguments
    ///
    /// * `decompound_mode` - 복합명사 분해 모드
    /// * `output_unknown_unigrams` - 미등록어 유니그램 출력 여부
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use mecab_ko_core::nori_compat::{NoriTokenizer, DecompoundMode};
    ///
    /// let tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, true)?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the internal tokenizer fails to initialize.
    pub fn new(decompound_mode: DecompoundMode, output_unknown_unigrams: bool) -> Result<Self> {
        Ok(Self {
            tokenizer: Tokenizer::new()?,
            decompound_mode,
            output_unknown_unigrams,
        })
    }

    /// 사전 경로를 지정하여 생성
    ///
    /// # Errors
    ///
    /// Returns an error if the tokenizer fails to load the dictionary.
    pub fn with_dict(
        dict_path: &str,
        decompound_mode: DecompoundMode,
        output_unknown_unigrams: bool,
    ) -> Result<Self> {
        Ok(Self {
            tokenizer: Tokenizer::with_dict(dict_path)?,
            decompound_mode,
            output_unknown_unigrams,
        })
    }

    /// 텍스트를 Nori 스타일로 토큰화
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let tokens = tokenizer.tokenize("형태소분석")?;
    /// for token in tokens {
    ///     println!("{}: {}", token.surface, token.pos_tag);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if tokenization fails.
    pub fn tokenize(&mut self, text: &str) -> Result<Vec<NoriToken>> {
        let mecab_tokens = self.tokenizer.tokenize(text);
        let mut nori_tokens = Vec::new();

        for token in &mecab_tokens {
            let nori_token = self.convert_token(token, text);
            nori_tokens.extend(nori_token);
        }

        Ok(nori_tokens)
    }

    /// `MeCab` 토큰을 Nori 토큰으로 변환
    fn convert_token(&self, token: &Token, text: &str) -> Vec<NoriToken> {
        let pos_tag = token.pos.parse::<PosTag>().unwrap_or(PosTag::Unknown);
        let nori_tag = pos_tag.to_nori_compat();

        // 기본 토큰 생성
        let mut tokens = vec![NoriToken {
            surface: token.surface.clone(),
            pos_tag: nori_tag.as_str().to_string(),
            start_offset: char_offset(text, token.start_byte),
            end_offset: char_offset(text, token.end_byte),
            lemma: token.lemma.clone(),
            reading: token.reading.clone(),
            word_type: if pos_tag == PosTag::Unknown {
                WordType::Unknown
            } else {
                WordType::Known
            },
            is_decompound: false,
        }];

        // 복합명사 분해 처리
        if self.should_decompound(pos_tag) {
            let decompounded = Self::decompound_token(token, text);
            tokens = self.apply_decompound_mode(tokens, decompounded);
        }

        // 미등록어 유니그램 처리
        if self.output_unknown_unigrams && pos_tag == PosTag::Unknown {
            tokens = Self::split_unknown_to_unigrams(token, text);
        }

        tokens
    }

    /// 복합명사 분해 대상인지 확인
    fn should_decompound(&self, pos_tag: PosTag) -> bool {
        self.decompound_mode != DecompoundMode::None && matches!(pos_tag, PosTag::NNG | PosTag::NNP)
    }

    /// 복합명사 분해 (실제 구현은 추후)
    const fn decompound_token(_token: &Token, _text: &str) -> Vec<NoriToken> {
        // TODO: 복합명사 분해 로직 구현
        // 현재는 빈 벡터 반환
        Vec::new()
    }

    /// 분해 모드에 따라 토큰 결합
    fn apply_decompound_mode(
        &self,
        original: Vec<NoriToken>,
        decompounded: Vec<NoriToken>,
    ) -> Vec<NoriToken> {
        match self.decompound_mode {
            DecompoundMode::None => original,
            DecompoundMode::Discard => {
                if decompounded.is_empty() {
                    original
                } else {
                    decompounded
                }
            }
            DecompoundMode::Mixed => {
                let mut result = original;
                result.extend(decompounded);
                result
            }
        }
    }

    /// 미등록어를 유니그램으로 분리
    fn split_unknown_to_unigrams(token: &Token, text: &str) -> Vec<NoriToken> {
        let chars: Vec<char> = token.surface.chars().collect();
        let mut tokens = Vec::new();
        let mut char_pos = token.start_byte;

        for ch in chars {
            let surface = ch.to_string();
            let char_len = ch.len_utf8();

            tokens.push(NoriToken {
                surface,
                pos_tag: "UNKNOWN".to_string(),
                start_offset: char_offset(text, char_pos),
                end_offset: char_offset(text, char_pos + char_len),
                lemma: None,
                reading: None,
                word_type: WordType::Unknown,
                is_decompound: false,
            });

            char_pos += char_len;
        }

        tokens
    }
}

/// Nori 분석기
///
/// Lucene Nori의 `KoreanAnalyzer`와 호환되는 인터페이스
pub struct NoriAnalyzer {
    /// Nori 토크나이저
    tokenizer: NoriTokenizer,
    /// 제거할 품사 태그 (stoptags)
    stoptags: HashSet<String>,
    /// 사용자 사전 (향후 구현)
    _user_dictionary: Option<String>,
}

impl NoriAnalyzer {
    /// 새 Nori 분석기 생성
    ///
    /// # Arguments
    ///
    /// * `user_dictionary` - 사용자 사전 경로 (옵션)
    /// * `decompound_mode` - 복합명사 분해 모드
    /// * `stoptags` - 필터링할 품사 태그 (예: \["J", "E"\])
    /// * `output_unknown_unigrams` - 미등록어 유니그램 출력 여부
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use mecab_ko_core::nori_compat::{NoriAnalyzer, DecompoundMode};
    ///
    /// let stoptags = vec!["J".to_string(), "E".to_string()];
    /// let analyzer = NoriAnalyzer::new(
    ///     None,
    ///     DecompoundMode::Mixed,
    ///     stoptags,
    ///     false
    /// )?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the tokenizer initialization fails.
    pub fn new(
        user_dictionary: Option<String>,
        decompound_mode: DecompoundMode,
        stoptags: Vec<String>,
        output_unknown_unigrams: bool,
    ) -> Result<Self> {
        Ok(Self {
            tokenizer: NoriTokenizer::new(decompound_mode, output_unknown_unigrams)?,
            stoptags: stoptags.into_iter().collect(),
            _user_dictionary: user_dictionary,
        })
    }

    /// 기본 설정으로 생성 (조사/어미 제거)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed)?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    pub fn default_with_decompound(decompound_mode: DecompoundMode) -> Result<Self> {
        Self::new(
            None,
            decompound_mode,
            vec!["J".to_string(), "E".to_string()],
            false,
        )
    }

    /// 텍스트 분석 (stoptags 필터링 적용)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let tokens = analyzer.analyze("형태소 분석기")?;
    /// // 조사/어미가 제거된 결과만 반환
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails.
    pub fn analyze(&mut self, text: &str) -> Result<Vec<NoriToken>> {
        let tokens = self.tokenizer.tokenize(text)?;
        Ok(self.filter_stoptags(tokens))
    }

    /// stoptags 필터링 적용
    fn filter_stoptags(&self, tokens: Vec<NoriToken>) -> Vec<NoriToken> {
        if self.stoptags.is_empty() {
            return tokens;
        }

        tokens
            .into_iter()
            .filter(|token| !self.stoptags.contains(&token.pos_tag))
            .collect()
    }

    /// stoptags 추가
    pub fn add_stoptag(&mut self, tag: String) {
        self.stoptags.insert(tag);
    }

    /// stoptags 제거
    pub fn remove_stoptag(&mut self, tag: &str) -> bool {
        self.stoptags.remove(tag)
    }

    /// stoptags 목록 반환
    #[must_use]
    pub fn stoptags(&self) -> Vec<&str> {
        self.stoptags.iter().map(String::as_str).collect()
    }
}

/// `MeCab` 태그를 Nori 태그로 변환
///
/// # Example
///
/// ```
/// use mecab_ko_core::nori_compat::mecab_to_nori_tag;
///
/// assert_eq!(mecab_to_nori_tag("JKS"), "J");  // 주격 조사 → J
/// assert_eq!(mecab_to_nori_tag("EF"), "E");   // 종결 어미 → E
/// assert_eq!(mecab_to_nori_tag("NNG"), "NNG"); // 일반 명사 → NNG
/// ```
#[must_use]
pub fn mecab_to_nori_tag(mecab_tag: &str) -> String {
    mecab_tag.parse::<PosTag>().map_or_else(
        |_| mecab_tag.to_string(),
        |tag| tag.to_nori_compat().as_str().to_string(),
    )
}

/// Nori 태그를 `MeCab` 태그로 변환 (부분 변환)
///
/// Nori의 통합 태그(J, E)는 대표 태그로 변환합니다.
///
/// # Example
///
/// ```
/// use mecab_ko_core::nori_compat::nori_to_mecab_tag;
///
/// assert_eq!(nori_to_mecab_tag("J"), "JX");   // 조사 → 보조사(대표)
/// assert_eq!(nori_to_mecab_tag("E"), "EF");   // 어미 → 종결어미(대표)
/// assert_eq!(nori_to_mecab_tag("NNG"), "NNG"); // 일반명사 → NNG
/// ```
#[must_use]
pub fn nori_to_mecab_tag(nori_tag: &str) -> String {
    match nori_tag {
        // 조사 통합 → 보조사를 대표로 사용
        "J" => "JX".to_string(),
        // 어미 통합 → 종결 어미를 대표로 사용
        "E" => "EF".to_string(),
        // 기타는 그대로
        _ => nori_tag.to_string(),
    }
}

/// 바이트 오프셋을 문자 오프셋으로 변환
fn char_offset(text: &str, byte_offset: usize) -> usize {
    text[..byte_offset.min(text.len())].chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompound_mode_from_str() {
        assert_eq!(DecompoundMode::parse("none"), Some(DecompoundMode::None));
        assert_eq!(
            DecompoundMode::parse("discard"),
            Some(DecompoundMode::Discard)
        );
        assert_eq!(
            DecompoundMode::parse("mixed"),
            Some(DecompoundMode::Mixed)
        );
        assert_eq!(DecompoundMode::parse("NONE"), Some(DecompoundMode::None));
        assert_eq!(DecompoundMode::parse("invalid"), None);
    }

    #[test]
    fn test_decompound_mode_as_str() {
        assert_eq!(DecompoundMode::None.as_str(), "none");
        assert_eq!(DecompoundMode::Discard.as_str(), "discard");
        assert_eq!(DecompoundMode::Mixed.as_str(), "mixed");
    }

    #[test]
    fn test_word_type_as_str() {
        assert_eq!(WordType::Known.as_str(), "KNOWN");
        assert_eq!(WordType::Unknown.as_str(), "UNKNOWN");
        assert_eq!(WordType::User.as_str(), "USER");
    }

    #[test]
    fn test_mecab_to_nori_tag() {
        // 조사 → J
        assert_eq!(mecab_to_nori_tag("JKS"), "J");
        assert_eq!(mecab_to_nori_tag("JKO"), "J");
        assert_eq!(mecab_to_nori_tag("JX"), "J");

        // 어미 → E
        assert_eq!(mecab_to_nori_tag("EF"), "E");
        assert_eq!(mecab_to_nori_tag("EC"), "E");
        assert_eq!(mecab_to_nori_tag("ETM"), "E");

        // 기타 → 그대로
        assert_eq!(mecab_to_nori_tag("NNG"), "NNG");
        assert_eq!(mecab_to_nori_tag("VV"), "VV");
        assert_eq!(mecab_to_nori_tag("MAG"), "MAG");
    }

    #[test]
    fn test_nori_to_mecab_tag() {
        assert_eq!(nori_to_mecab_tag("J"), "JX");
        assert_eq!(nori_to_mecab_tag("E"), "EF");
        assert_eq!(nori_to_mecab_tag("NNG"), "NNG");
        assert_eq!(nori_to_mecab_tag("VV"), "VV");
    }

    #[test]
    fn test_char_offset() {
        let text = "안녕하세요";
        assert_eq!(char_offset(text, 0), 0);
        assert_eq!(char_offset(text, 3), 1); // '안' = 3 bytes
        assert_eq!(char_offset(text, 6), 2); // '안녕' = 6 bytes
        assert_eq!(char_offset(text, 100), 5); // overflow → 전체 길이
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_nori_tokenizer_creation() {
        let tokenizer = NoriTokenizer::new(DecompoundMode::None, false);
        assert!(tokenizer.is_ok());

        let tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, true);
        assert!(tokenizer.is_ok());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_nori_analyzer_creation() {
        let analyzer = NoriAnalyzer::new(
            None,
            DecompoundMode::None,
            vec!["J".to_string(), "E".to_string()],
            false,
        );
        assert!(analyzer.is_ok());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_nori_analyzer_default() {
        let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed);
        assert!(analyzer.is_ok());

        let analyzer = analyzer.unwrap();
        let stoptags = analyzer.stoptags();
        assert_eq!(stoptags.len(), 2);
        assert!(stoptags.contains(&"J"));
        assert!(stoptags.contains(&"E"));
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_nori_analyzer_stoptag_management() {
        let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

        // 초기 상태
        assert_eq!(analyzer.stoptags().len(), 2);

        // 추가
        analyzer.add_stoptag("SF".to_string());
        assert_eq!(analyzer.stoptags().len(), 3);
        assert!(analyzer.stoptags().contains(&"SF"));

        // 제거
        assert!(analyzer.remove_stoptag("SF"));
        assert_eq!(analyzer.stoptags().len(), 2);
        assert!(!analyzer.stoptags().contains(&"SF"));

        // 없는 태그 제거
        assert!(!analyzer.remove_stoptag("NONEXISTENT"));
    }

    #[test]
    fn test_pos_tag_nori_mapping() {
        // 조사 통합
        assert_eq!(PosTag::JKS.to_nori_compat().as_str(), "J");
        assert_eq!(PosTag::JKO.to_nori_compat().as_str(), "J");
        assert_eq!(PosTag::JX.to_nori_compat().as_str(), "J");

        // 어미 통합
        assert_eq!(PosTag::EF.to_nori_compat().as_str(), "E");
        assert_eq!(PosTag::EC.to_nori_compat().as_str(), "E");
        assert_eq!(PosTag::ETM.to_nori_compat().as_str(), "E");

        // 기타
        assert_eq!(PosTag::NNG.to_nori_compat().as_str(), "NNG");
        assert_eq!(PosTag::VV.to_nori_compat().as_str(), "VV");
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_tokenizer_basic_functionality() {
        let mut tokenizer = NoriTokenizer::new(DecompoundMode::None, false).unwrap();
        let result = tokenizer.tokenize("테스트");
        assert!(result.is_ok());

        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_analyzer_basic_functionality() {
        let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();
        let result = analyzer.analyze("테스트");
        assert!(result.is_ok());
    }
}
