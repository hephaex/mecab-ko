//! 토크나이저 인터페이스

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 토큰 (`Lucene` `AttributeSource` 호환)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// 표면형 (term)
    pub surface: String,

    /// 품사 태그 (Nori 스타일)
    pub pos_tag: String,

    /// 시작 오프셋 (문자 단위)
    pub start_offset: usize,

    /// 끝 오프셋 (문자 단위)
    pub end_offset: usize,

    /// 위치 증가량 (기본 1, 동의어는 0)
    pub position_increment: u32,

    /// 위치 길이 (그래프 토큰용, 기본 1)
    pub position_length: u32,

    /// 원형 (기본형)
    pub lemma: Option<String>,

    /// 읽기 (발음)
    pub reading: Option<String>,

    /// 단어 타입
    pub word_type: WordType,

    /// 복합명사 분해 여부
    pub is_decompound: bool,

    /// 추가 속성 (확장용)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<TokenAttributes>,
}

impl Token {
    /// 새 토큰 생성
    #[must_use]
    pub const fn new(
        surface: String,
        pos_tag: String,
        start_offset: usize,
        end_offset: usize,
    ) -> Self {
        Self {
            surface,
            pos_tag,
            start_offset,
            end_offset,
            position_increment: 1,
            position_length: 1,
            lemma: None,
            reading: None,
            word_type: WordType::Known,
            is_decompound: false,
            attributes: None,
        }
    }

    /// 토큰 길이 (문자 수)
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end_offset.saturating_sub(self.start_offset)
    }

    /// 빈 토큰 여부
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.surface.is_empty()
    }

    /// Builder 패턴: lemma 설정
    #[must_use]
    pub fn with_lemma(mut self, lemma: Option<String>) -> Self {
        self.lemma = lemma;
        self
    }

    /// Builder 패턴: reading 설정
    #[must_use]
    pub fn with_reading(mut self, reading: Option<String>) -> Self {
        self.reading = reading;
        self
    }

    /// Builder 패턴: `word_type` 설정
    #[must_use]
    pub const fn with_word_type(mut self, word_type: WordType) -> Self {
        self.word_type = word_type;
        self
    }

    /// Builder 패턴: `is_decompound` 설정
    #[must_use]
    pub const fn with_is_decompound(mut self, is_decompound: bool) -> Self {
        self.is_decompound = is_decompound;
        self
    }

    /// Builder 패턴: `position_increment` 설정
    #[must_use]
    pub const fn with_position_increment(mut self, increment: u32) -> Self {
        self.position_increment = increment;
        self
    }

    /// Builder 패턴: `position_length` 설정
    #[must_use]
    pub const fn with_position_length(mut self, length: u32) -> Self {
        self.position_length = length;
        self
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}[{}]({}-{})",
            self.surface, self.pos_tag, self.start_offset, self.end_offset
        )
    }
}

/// 토큰 추가 속성
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenAttributes {
    /// 좌측 문맥 ID
    pub left_id: Option<u32>,
    /// 우측 문맥 ID
    pub right_id: Option<u32>,
    /// 비용
    pub cost: Option<i32>,
}

/// 단어 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
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

impl fmt::Display for WordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 토큰 스트림 (Iterator 패턴)
pub trait TokenStream: Iterator<Item = Token> {
    /// 토큰 스트림 리셋
    fn reset(&mut self);
}

/// 토크나이저 인터페이스
///
/// Elasticsearch/Lucene의 Tokenizer 추상화
pub trait Tokenizer: Send + Sync {
    /// 텍스트를 토큰화
    ///
    /// # Errors
    ///
    /// 토큰화 실패 시 에러 반환
    fn tokenize(&self, text: &str) -> Result<Vec<Token>>;

    /// 토큰 스트림 생성 (스트리밍 API)
    fn token_stream<'a>(&'a self, text: &'a str) -> Box<dyn TokenStream + 'a> {
        Box::new(VecTokenStream::new(self.tokenize(text).unwrap_or_default()))
    }
}

/// Vector 기반 토큰 스트림 구현
pub struct VecTokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl VecTokenStream {
    /// 새 토큰 스트림 생성
    #[must_use]
    pub const fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }
}

impl Iterator for VecTokenStream {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tokens.len() {
            let token = self.tokens[self.index].clone();
            self.index += 1;
            Some(token)
        } else {
            None
        }
    }
}

impl TokenStream for VecTokenStream {
    fn reset(&mut self) {
        self.index = 0;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new("테스트".to_string(), "NNG".to_string(), 0, 3);
        assert_eq!(token.surface, "테스트");
        assert_eq!(token.pos_tag, "NNG");
        assert_eq!(token.start_offset, 0);
        assert_eq!(token.end_offset, 3);
        assert_eq!(token.position_increment, 1);
    }

    #[test]
    fn test_token_builder() {
        let token = Token::new("형태소".to_string(), "NNG".to_string(), 0, 3)
            .with_lemma(Some("형태소".to_string()))
            .with_reading(Some("형태소".to_string()))
            .with_word_type(WordType::Known)
            .with_is_decompound(false);

        assert_eq!(token.lemma, Some("형태소".to_string()));
        assert_eq!(token.word_type, WordType::Known);
    }

    #[test]
    fn test_token_len() {
        let token = Token::new("테스트".to_string(), "NNG".to_string(), 0, 3);
        assert_eq!(token.len(), 3);
        assert!(!token.is_empty());
    }

    #[test]
    fn test_word_type_as_str() {
        assert_eq!(WordType::Known.as_str(), "KNOWN");
        assert_eq!(WordType::Unknown.as_str(), "UNKNOWN");
        assert_eq!(WordType::User.as_str(), "USER");
    }

    #[test]
    fn test_vec_token_stream() {
        let tokens = vec![
            Token::new("형태소".to_string(), "NNG".to_string(), 0, 3),
            Token::new("분석".to_string(), "NNG".to_string(), 3, 5),
        ];

        let mut stream = VecTokenStream::new(tokens);

        assert_eq!(stream.next().unwrap().surface, "형태소");
        assert_eq!(stream.next().unwrap().surface, "분석");
        assert!(stream.next().is_none());

        // Reset 테스트
        stream.reset();
        assert_eq!(stream.next().unwrap().surface, "형태소");
    }

    #[test]
    fn test_token_serialization() {
        let token = Token::new("테스트".to_string(), "NNG".to_string(), 0, 3);
        let json = serde_json::to_string(&token);
        assert!(json.is_ok());

        let deserialized: std::result::Result<Token, serde_json::Error> =
            serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
}
