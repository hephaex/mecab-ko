//! 토큰 필터 구현

use crate::error::Result;
use crate::tokenizer::Token;
use std::collections::HashSet;

/// 토큰 필터 인터페이스
///
/// Elasticsearch/Lucene의 `TokenFilter` 추상화
pub trait TokenFilter: Send + Sync {
    /// 토큰 필터링
    ///
    /// # Errors
    ///
    /// 필터링 실패 시 에러 반환
    fn filter(&self, tokens: Vec<Token>) -> Result<Vec<Token>>;
}

/// Nori 품사 기반 필터
///
/// Lucene Nori의 `KoreanPartOfSpeechStopFilter`와 호환
///
/// # Example
///
/// ```rust,ignore
/// use mecab_ko_elasticsearch::filter::{NoriPartOfSpeechStopFilter, TokenFilter};
///
/// let filter = NoriPartOfSpeechStopFilter::new(vec!["J".to_string(), "E".to_string()]);
/// let filtered = filter.filter(tokens)?;
/// ```
pub struct NoriPartOfSpeechStopFilter {
    /// 제거할 품사 태그 집합
    stoptags: HashSet<String>,
}

impl NoriPartOfSpeechStopFilter {
    /// 새 필터 생성
    #[must_use]
    pub fn new(stoptags: Vec<String>) -> Self {
        Self {
            stoptags: stoptags.into_iter().collect(),
        }
    }

    /// 기본 필터 생성 (조사 J, 어미 E 제거)
    #[must_use]
    pub fn default_filter() -> Self {
        Self::new(vec!["J".to_string(), "E".to_string()])
    }

    /// stoptag 추가
    pub fn add_tag(&mut self, tag: String) {
        self.stoptags.insert(tag);
    }

    /// stoptag 제거
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        self.stoptags.remove(tag)
    }

    /// stoptags 목록 반환
    #[must_use]
    pub fn tags(&self) -> Vec<&str> {
        self.stoptags.iter().map(String::as_str).collect()
    }

    /// stoptags 개수
    #[must_use]
    pub fn len(&self) -> usize {
        self.stoptags.len()
    }

    /// stoptags가 비어있는지
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stoptags.is_empty()
    }

    /// 특정 태그 포함 여부
    #[must_use]
    pub fn contains(&self, tag: &str) -> bool {
        self.stoptags.contains(tag)
    }
}

impl TokenFilter for NoriPartOfSpeechStopFilter {
    fn filter(&self, tokens: Vec<Token>) -> Result<Vec<Token>> {
        if self.stoptags.is_empty() {
            return Ok(tokens);
        }

        // In-place filtering with capacity hint
        let mut filtered = Vec::with_capacity(tokens.len());

        for token in tokens {
            if !self.stoptags.contains(&token.pos_tag) {
                filtered.push(token);
            }
        }

        // Shrink to fit if we filtered out a lot
        if filtered.capacity() > filtered.len() * 2 {
            filtered.shrink_to_fit();
        }

        Ok(filtered)
    }
}

/// Nori 읽기 형태 필터
///
/// Lucene Nori의 `KoreanReadingFormFilter`와 호환
///
/// 토큰의 표면형을 읽기(발음)로 대체합니다.
///
/// # Example
///
/// ```rust,ignore
/// use mecab_ko_elasticsearch::filter::{NoriReadingFormFilter, TokenFilter};
///
/// let filter = NoriReadingFormFilter::new();
/// let filtered = filter.filter(tokens)?;
/// ```
pub struct NoriReadingFormFilter {
    /// 읽기가 없을 때 표면형 유지 여부
    keep_original: bool,
}

impl NoriReadingFormFilter {
    /// 새 필터 생성
    #[must_use]
    pub const fn new() -> Self {
        Self {
            keep_original: true,
        }
    }

    /// 읽기가 없을 때 표면형 유지 설정
    #[must_use]
    pub const fn with_keep_original(mut self, keep: bool) -> Self {
        self.keep_original = keep;
        self
    }
}

impl Default for NoriReadingFormFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for NoriReadingFormFilter {
    fn filter(&self, tokens: Vec<Token>) -> Result<Vec<Token>> {
        let mut result = Vec::with_capacity(tokens.len());

        for mut token in tokens {
            if let Some(reading) = token.reading.take() {
                // Move reading instead of cloning
                token.surface = reading;
                result.push(token);
            } else if self.keep_original {
                result.push(token);
            }
            // If !keep_original and no reading, skip token
        }

        if result.capacity() > result.len() * 2 {
            result.shrink_to_fit();
        }

        Ok(result)
    }
}

/// 복합 필터 (여러 필터를 순차적으로 적용)
///
/// # Example
///
/// ```rust,ignore
/// use mecab_ko_elasticsearch::filter::{CompositeFilter, NoriPartOfSpeechStopFilter, NoriReadingFormFilter};
///
/// let mut composite = CompositeFilter::new();
/// composite.add_filter(Box::new(NoriPartOfSpeechStopFilter::default_filter()));
/// composite.add_filter(Box::new(NoriReadingFormFilter::new()));
///
/// let filtered = composite.filter(tokens)?;
/// ```
pub struct CompositeFilter {
    /// 필터 체인
    filters: Vec<Box<dyn TokenFilter>>,
}

impl CompositeFilter {
    /// 새 복합 필터 생성
    #[must_use]
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// 필터 추가
    pub fn add_filter(&mut self, filter: Box<dyn TokenFilter>) {
        self.filters.push(filter);
    }

    /// 필터 개수
    #[must_use]
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// 필터가 비어있는지
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }
}

impl Default for CompositeFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for CompositeFilter {
    fn filter(&self, tokens: Vec<Token>) -> Result<Vec<Token>> {
        let mut result = tokens;
        for filter in &self.filters {
            result = filter.filter(result)?;
        }
        Ok(result)
    }
}

/// 소문자 변환 필터
///
/// 영문 토큰을 소문자로 변환
pub struct LowercaseFilter;

impl LowercaseFilter {
    /// 새 필터 생성
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for LowercaseFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for LowercaseFilter {
    fn filter(&self, tokens: Vec<Token>) -> Result<Vec<Token>> {
        let mut result = Vec::with_capacity(tokens.len());

        for mut token in tokens {
            token.surface.make_ascii_lowercase();
            if let Some(ref mut lemma) = token.lemma {
                lemma.make_ascii_lowercase();
            }
            result.push(token);
        }

        Ok(result)
    }
}

/// 길이 기반 필터
///
/// 지정된 길이 범위의 토큰만 유지
pub struct LengthFilter {
    /// 최소 길이 (문자 수)
    min_length: usize,
    /// 최대 길이 (문자 수)
    max_length: usize,
}

impl LengthFilter {
    /// 새 필터 생성
    #[must_use]
    pub const fn new(min_length: usize, max_length: usize) -> Self {
        Self {
            min_length,
            max_length,
        }
    }
}

impl TokenFilter for LengthFilter {
    fn filter(&self, tokens: Vec<Token>) -> Result<Vec<Token>> {
        Ok(tokens
            .into_iter()
            .filter(|token| {
                let len = token.surface.chars().count();
                len >= self.min_length && len <= self.max_length
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_token(surface: &str, pos_tag: &str) -> Token {
        Token::new(surface.to_string(), pos_tag.to_string(), 0, surface.len())
    }

    #[test]
    fn test_pos_filter_creation() {
        let filter = NoriPartOfSpeechStopFilter::new(vec!["J".to_string(), "E".to_string()]);
        assert_eq!(filter.len(), 2);
        assert!(filter.contains("J"));
        assert!(filter.contains("E"));
    }

    #[test]
    fn test_pos_filter_default() {
        let filter = NoriPartOfSpeechStopFilter::default_filter();
        assert_eq!(filter.len(), 2);
    }

    #[test]
    fn test_pos_filter_add_remove() {
        let mut filter = NoriPartOfSpeechStopFilter::new(vec!["J".to_string()]);
        assert_eq!(filter.len(), 1);

        filter.add_tag("E".to_string());
        assert_eq!(filter.len(), 2);

        assert!(filter.remove_tag("E"));
        assert_eq!(filter.len(), 1);

        assert!(!filter.remove_tag("NONEXISTENT"));
    }

    #[test]
    fn test_pos_filter_filtering() {
        let filter = NoriPartOfSpeechStopFilter::new(vec!["J".to_string(), "E".to_string()]);

        let tokens = vec![
            create_test_token("형태소", "NNG"),
            create_test_token("분석", "NNG"),
            create_test_token("을", "J"),
            create_test_token("하", "VV"),
            create_test_token("다", "E"),
        ];

        let filtered = filter.filter(tokens).unwrap();
        assert_eq!(filtered.len(), 3); // NNG, NNG, VV만 남음
        assert_eq!(filtered[0].surface, "형태소");
        assert_eq!(filtered[1].surface, "분석");
        assert_eq!(filtered[2].surface, "하");
    }

    #[test]
    fn test_reading_form_filter() {
        let filter = NoriReadingFormFilter::new();

        let tokens = vec![
            Token::new("형태소".to_string(), "NNG".to_string(), 0, 3)
                .with_reading(Some("형태소".to_string())),
            Token::new("분석".to_string(), "NNG".to_string(), 3, 5)
                .with_reading(Some("분석".to_string())),
            Token::new("기".to_string(), "NNG".to_string(), 5, 6), // 읽기 없음
        ];

        let filtered = filter.filter(tokens).unwrap();
        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0].surface, "형태소");
        assert_eq!(filtered[1].surface, "분석");
        assert_eq!(filtered[2].surface, "기"); // 원본 유지
    }

    #[test]
    fn test_reading_form_filter_no_keep() {
        let filter = NoriReadingFormFilter::new().with_keep_original(false);

        let tokens = vec![
            Token::new("형태소".to_string(), "NNG".to_string(), 0, 3)
                .with_reading(Some("형태소".to_string())),
            Token::new("기".to_string(), "NNG".to_string(), 5, 6), // 읽기 없음
        ];

        let filtered = filter.filter(tokens).unwrap();
        assert_eq!(filtered.len(), 1); // 읽기가 있는 것만
        assert_eq!(filtered[0].surface, "형태소");
    }

    #[test]
    fn test_composite_filter() {
        let mut composite = CompositeFilter::new();
        composite.add_filter(Box::new(NoriPartOfSpeechStopFilter::new(vec![
            "J".to_string()
        ])));
        composite.add_filter(Box::new(LowercaseFilter::new()));

        let tokens = vec![
            create_test_token("Test", "NNG"),
            create_test_token("을", "J"),
            create_test_token("HELLO", "NNG"),
        ];

        let filtered = composite.filter(tokens).unwrap();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].surface, "test");
        assert_eq!(filtered[1].surface, "hello");
    }

    #[test]
    fn test_lowercase_filter() {
        let filter = LowercaseFilter::new();

        let tokens = vec![
            create_test_token("Test", "NNG"),
            create_test_token("HELLO", "NNG"),
            create_test_token("WoRlD", "NNG"),
        ];

        let filtered = filter.filter(tokens).unwrap();
        assert_eq!(filtered[0].surface, "test");
        assert_eq!(filtered[1].surface, "hello");
        assert_eq!(filtered[2].surface, "world");
    }

    #[test]
    fn test_length_filter() {
        let filter = LengthFilter::new(2, 4);

        let tokens = vec![
            create_test_token("가", "NNG"),           // 1자 - 제거
            create_test_token("형태소", "NNG"),       // 3자 - 유지
            create_test_token("분석기", "NNG"),       // 3자 - 유지
            create_test_token("형태소분석기", "NNG"), // 6자 - 제거
        ];

        let filtered = filter.filter(tokens).unwrap();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].surface, "형태소");
        assert_eq!(filtered[1].surface, "분석기");
    }
}
