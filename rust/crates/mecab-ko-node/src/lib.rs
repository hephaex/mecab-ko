//! # mecab-ko-node
//!
//! Node.js bindings for MeCab-Ko Korean morphological analyzer.
//!
//! This crate provides N-API bindings using napi-rs for high-performance
//! Korean text analysis in Node.js applications.
//!
//! ## Features
//!
//! - Zero-copy operations where possible
//! - Thread-safe tokenization
//! - TypeScript type definitions
//! - Cross-platform support
//!
//! ## Architecture
//!
//! ```text
//! Node.js
//!   ↓
//! N-API (napi-rs)
//!   ↓
//! mecab-ko-core (Rust)
//! ```

#![deny(clippy::all)]

use mecab_ko_core::{Token as CoreToken, Tokenizer as CoreTokenizer};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use parking_lot::Mutex;

/// Token represents a morpheme in the analyzed text.
///
/// Each token contains the surface form, part-of-speech tag,
/// and position information.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct Token {
    /// The surface form (actual text)
    pub surface: String,
    /// Part-of-speech tag
    pub pos: String,
    /// Start position in bytes
    pub start: u32,
    /// End position in bytes
    pub end: u32,
    /// Reading (optional)
    pub reading: Option<String>,
    /// Lemma/base form (optional)
    pub lemma: Option<String>,
}

impl From<CoreToken> for Token {
    fn from(token: CoreToken) -> Self {
        Self {
            surface: token.surface,
            pos: token.pos,
            start: token.start_byte as u32,
            end: token.end_byte as u32,
            reading: token.reading,
            lemma: token.lemma,
        }
    }
}

/// Mecab is the main interface for Korean morphological analysis.
///
/// # Examples
///
/// ```javascript
/// const { Mecab } = require('@mecab-ko/node');
///
/// const mecab = new Mecab();
/// const tokens = mecab.tokenize('안녕하세요');
/// console.log(tokens);
/// ```
#[napi]
pub struct Mecab {
    tokenizer: Mutex<CoreTokenizer>,
}

#[napi]
impl Mecab {
    /// Creates a new Mecab instance with the default dictionary.
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary cannot be loaded or initialized.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const mecab = new Mecab();
    /// ```
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let tokenizer = CoreTokenizer::new()
            .map_err(|e| Error::from_reason(format!("Failed to initialize tokenizer: {e}")))?;

        Ok(Self {
            tokenizer: Mutex::new(tokenizer),
        })
    }

    /// Creates a new Mecab instance with a custom dictionary path.
    ///
    /// # Arguments
    ///
    /// * `dict_path` - Path to the dictionary directory
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary cannot be loaded.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const mecab = Mecab.withDict('/path/to/dict');
    /// ```
    #[napi(factory)]
    pub fn with_dict(dict_path: String) -> Result<Self> {
        let tokenizer = CoreTokenizer::with_dict(&dict_path)
            .map_err(|e| Error::from_reason(format!("Failed to load dictionary: {e}")))?;

        Ok(Self {
            tokenizer: Mutex::new(tokenizer),
        })
    }

    /// Tokenizes the input text and returns an array of tokens.
    ///
    /// Each token contains the surface form, part-of-speech tag, and position.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze
    ///
    /// # Returns
    ///
    /// An array of Token objects.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const tokens = mecab.tokenize('형태소 분석기');
    /// // Returns: [
    /// //   { surface: '형태소', pos: 'NNG', ... },
    /// //   { surface: '분석기', pos: 'NNG', ... }
    /// // ]
    /// ```
    #[napi]
    pub fn tokenize(&self, text: String) -> Vec<Token> {
        self.tokenizer
            .lock()
            .tokenize(&text)
            .into_iter()
            .map(Token::from)
            .collect()
    }

    /// Extracts morphemes (surface forms) from the input text.
    ///
    /// This is equivalent to calling `tokenize` and extracting surface forms.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze
    ///
    /// # Returns
    ///
    /// An array of morpheme strings.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const morphs = mecab.morphs('형태소 분석');
    /// // Returns: ['형태소', '분석']
    /// ```
    #[napi]
    pub fn morphs(&self, text: String) -> Vec<String> {
        self.tokenizer.lock().morphs(&text)
    }

    /// Extracts nouns from the input text.
    ///
    /// Returns only tokens whose POS tag starts with 'NN'.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze
    ///
    /// # Returns
    ///
    /// An array of noun strings.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const nouns = mecab.nouns('대한민국의 수도는 서울입니다');
    /// // Returns: ['대한민국', '수도', '서울']
    /// ```
    #[napi]
    pub fn nouns(&self, text: String) -> Vec<String> {
        self.tokenizer.lock().nouns(&text)
    }

    /// Returns part-of-speech tagged pairs.
    ///
    /// Each pair consists of [surface, pos].
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze
    ///
    /// # Returns
    ///
    /// An array of [surface, pos] tuples.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const pairs = mecab.pos('안녕하세요');
    /// // Returns: [['안녕하세요', 'NNG']]
    /// ```
    #[napi]
    pub fn pos(&self, text: String) -> Vec<Vec<String>> {
        self.tokenizer
            .lock()
            .pos(&text)
            .into_iter()
            .map(|(surface, pos)| vec![surface, pos])
            .collect()
    }
}

/// Returns the version of the mecab-ko-node library.
///
/// # Examples
///
/// ```javascript
/// const version = getVersion();
/// console.log(version); // "0.1.0"
/// ```
#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_conversion() {
        let core_token = CoreToken {
            surface: "테스트".to_string(),
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
        };

        let token = Token::from(core_token);
        assert_eq!(token.surface, "테스트");
        assert_eq!(token.pos, "NNG");
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 9);
    }

    #[test]
    #[ignore = "Requires system dictionary"]
    fn test_mecab_creation() {
        let result = Mecab::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_version() {
        let version = get_version();
        assert!(!version.is_empty());
        assert!(version.starts_with('0'));
    }
}
