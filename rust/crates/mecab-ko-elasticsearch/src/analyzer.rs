//! Nori 호환 분석기 구현

use crate::config::{AnalyzerConfig, DecompoundMode, TokenizerConfig};
use crate::error::{Error, Result};
use crate::filter::{NoriPartOfSpeechStopFilter, TokenFilter};
use crate::tokenizer::{Token, TokenStream, Tokenizer, WordType};
use mecab_ko_core::nori_compat::{NoriToken, NoriTokenizer as CoreNoriTokenizer};

/// Nori 분석기
///
/// Lucene Nori의 `KoreanAnalyzer`와 호환되는 인터페이스를 제공합니다.
///
/// # 주요 기능
///
/// - 복합명사 분해 (none, discard, mixed)
/// - 사용자 사전 지원
/// - 품사 기반 필터링 (stoptags)
/// - 미등록어 유니그램 출력
///
/// # Example
///
/// ```rust,ignore
/// use mecab_ko_elasticsearch::analyzer::NoriAnalyzer;
/// use mecab_ko_elasticsearch::config::{AnalyzerConfig, DecompoundMode};
///
/// let config = AnalyzerConfig::new()
///     .with_decompound_mode(DecompoundMode::Mixed)
///     .with_stoptags(vec!["J".to_string(), "E".to_string()]);
///
/// let analyzer = NoriAnalyzer::new(config)?;
/// let tokens = analyzer.analyze("한국어 형태소 분석기")?;
///
/// for token in tokens {
///     println!("{}: {}", token.surface, token.pos_tag);
/// }
/// ```
pub struct NoriAnalyzer {
    /// 토크나이저
    tokenizer: NoriTokenizerImpl,
    /// 품사 필터
    pos_filter: Option<NoriPartOfSpeechStopFilter>,
}

impl NoriAnalyzer {
    /// 새 분석기 생성
    ///
    /// # Errors
    ///
    /// - 토크나이저 초기화 실패
    /// - 설정 유효성 검증 실패
    pub fn new(config: AnalyzerConfig) -> Result<Self> {
        config.validate()?;

        let tokenizer = NoriTokenizerImpl::new(TokenizerConfig::from(&config))?;

        let pos_filter = if config.stoptags.is_empty() {
            None
        } else {
            Some(NoriPartOfSpeechStopFilter::new(config.stoptags))
        };

        Ok(Self {
            tokenizer,
            pos_filter,
        })
    }

    /// 기본 설정으로 생성 (조사/어미 제거)
    ///
    /// # Errors
    ///
    /// 토크나이저 초기화 실패 시 에러 반환
    pub fn default_with_decompound(mode: DecompoundMode) -> Result<Self> {
        Self::new(
            AnalyzerConfig::new()
                .with_decompound_mode(mode)
                .with_stoptags(vec!["J".to_string(), "E".to_string()]),
        )
    }

    /// 텍스트 분석
    ///
    /// 토큰화 후 필터 적용
    ///
    /// # Errors
    ///
    /// 토큰화 실패 시 에러 반환
    pub fn analyze(&self, text: &str) -> Result<Vec<Token>> {
        let mut tokens = self.tokenizer.tokenize(text)?;

        if let Some(filter) = &self.pos_filter {
            tokens = filter.filter(tokens)?;
        }

        Ok(tokens)
    }

    /// 토큰 스트림 생성
    pub fn token_stream<'a>(&'a self, text: &'a str) -> Box<dyn TokenStream + 'a> {
        self.tokenizer.token_stream(text)
    }

    /// stoptags 추가
    pub fn add_stoptag(&mut self, tag: String) {
        if let Some(filter) = &mut self.pos_filter {
            filter.add_tag(tag);
        } else {
            self.pos_filter = Some(NoriPartOfSpeechStopFilter::new(vec![tag]));
        }
    }

    /// stoptags 제거
    pub fn remove_stoptag(&mut self, tag: &str) -> bool {
        self.pos_filter
            .as_mut()
            .map_or(false, |filter| filter.remove_tag(tag))
    }

    /// stoptags 목록 반환
    #[must_use]
    pub fn stoptags(&self) -> Vec<&str> {
        self.pos_filter
            .as_ref()
            .map_or_else(Vec::new, NoriPartOfSpeechStopFilter::tags)
    }
}

/// Nori 토크나이저 구현
///
/// `mecab-ko-core`의 `NoriTokenizer`를 래핑하여 Elasticsearch 인터페이스 제공
pub struct NoriTokenizerImpl {
    /// 코어 토크나이저
    inner: CoreNoriTokenizer,
    /// 설정
    config: TokenizerConfig,
}

impl NoriTokenizerImpl {
    /// 새 토크나이저 생성
    ///
    /// # Errors
    ///
    /// MeCab 코어 초기화 실패 시 에러 반환
    pub fn new(config: TokenizerConfig) -> Result<Self> {
        let inner = if let Some(dict_path) = &config.user_dictionary_path {
            CoreNoriTokenizer::with_dict(
                dict_path
                    .to_str()
                    .ok_or_else(|| Error::config("Invalid user dictionary path"))?,
                convert_decompound_mode(config.decompound_mode),
                config.output_unknown_unigrams,
            )?
        } else {
            CoreNoriTokenizer::new(
                convert_decompound_mode(config.decompound_mode),
                config.output_unknown_unigrams,
            )?
        };

        Ok(Self { inner, config })
    }

    /// 설정 반환
    #[must_use]
    pub const fn config(&self) -> &TokenizerConfig {
        &self.config
    }
}

impl Tokenizer for NoriTokenizerImpl {
    fn tokenize(&self, text: &str) -> Result<Vec<Token>> {
        let nori_tokens = self.inner.tokenize(text)?;
        Ok(nori_tokens
            .into_iter()
            .map(convert_nori_token)
            .collect())
    }
}

/// Core의 `DecompoundMode`를 ES의 `DecompoundMode`로 변환
fn convert_decompound_mode(
    mode: DecompoundMode,
) -> mecab_ko_core::nori_compat::DecompoundMode {
    match mode {
        DecompoundMode::None => mecab_ko_core::nori_compat::DecompoundMode::None,
        DecompoundMode::Discard => mecab_ko_core::nori_compat::DecompoundMode::Discard,
        DecompoundMode::Mixed => mecab_ko_core::nori_compat::DecompoundMode::Mixed,
    }
}

/// Core의 `NoriToken`을 ES의 `Token`으로 변환
fn convert_nori_token(nori: NoriToken) -> Token {
    Token {
        surface: nori.surface,
        pos_tag: nori.pos_tag,
        start_offset: nori.start_offset,
        end_offset: nori.end_offset,
        position_increment: 1,
        position_length: 1,
        lemma: nori.lemma,
        reading: nori.reading,
        word_type: convert_word_type(nori.word_type),
        is_decompound: nori.is_decompound,
        attributes: None,
    }
}

/// Core의 `WordType`을 ES의 `WordType`으로 변환
fn convert_word_type(
    wt: mecab_ko_core::nori_compat::WordType,
) -> WordType {
    match wt {
        mecab_ko_core::nori_compat::WordType::Known => WordType::Known,
        mecab_ko_core::nori_compat::WordType::Unknown => WordType::Unknown,
        mecab_ko_core::nori_compat::WordType::User => WordType::User,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nori_tokenizer_creation() {
        let config = TokenizerConfig::default();
        let tokenizer = NoriTokenizerImpl::new(config);
        assert!(tokenizer.is_ok());
    }

    #[test]
    fn test_nori_analyzer_creation() {
        let config = AnalyzerConfig::default();
        let analyzer = NoriAnalyzer::new(config);
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_nori_analyzer_default() {
        let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None);
        assert!(analyzer.is_ok());

        let analyzer = analyzer.unwrap();
        assert_eq!(analyzer.stoptags().len(), 2);
    }

    #[test]
    fn test_nori_analyzer_stoptag_management() {
        let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

        // 초기 상태
        assert_eq!(analyzer.stoptags().len(), 2);

        // 추가
        analyzer.add_stoptag("SF".to_string());
        assert_eq!(analyzer.stoptags().len(), 3);

        // 제거
        assert!(analyzer.remove_stoptag("SF"));
        assert_eq!(analyzer.stoptags().len(), 2);

        // 없는 태그 제거
        assert!(!analyzer.remove_stoptag("NONEXISTENT"));
    }

    #[test]
    fn test_tokenizer_basic() {
        let tokenizer = NoriTokenizerImpl::new(TokenizerConfig::default()).unwrap();
        let result = tokenizer.tokenize("테스트");
        assert!(result.is_ok());

        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_analyzer_basic() {
        let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();
        let result = analyzer.analyze("형태소 분석");
        assert!(result.is_ok());

        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_decompound_mode_conversion() {
        use mecab_ko_core::nori_compat::DecompoundMode as CoreMode;

        assert_eq!(
            convert_decompound_mode(DecompoundMode::None),
            CoreMode::None
        );
        assert_eq!(
            convert_decompound_mode(DecompoundMode::Discard),
            CoreMode::Discard
        );
        assert_eq!(
            convert_decompound_mode(DecompoundMode::Mixed),
            CoreMode::Mixed
        );
    }

    #[test]
    fn test_word_type_conversion() {
        use mecab_ko_core::nori_compat::WordType as CoreWordType;

        assert_eq!(convert_word_type(CoreWordType::Known), WordType::Known);
        assert_eq!(
            convert_word_type(CoreWordType::Unknown),
            WordType::Unknown
        );
        assert_eq!(convert_word_type(CoreWordType::User), WordType::User);
    }
}
