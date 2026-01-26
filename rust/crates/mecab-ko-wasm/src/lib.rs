//! # mecab-ko-wasm
//!
//! WebAssembly bindings for MeCab-Ko, a Korean morphological analyzer.
//!
//! This crate provides JavaScript/TypeScript bindings for the MeCab-Ko library,
//! enabling Korean morphological analysis in web browsers and Node.js environments.
//!
//! ## Features
//!
//! - Tokenization with detailed morphological information
//! - Simple morpheme extraction
//! - Part-of-speech tagging
//! - Support for both browser and Node.js environments
//!
//! ## Example Usage (JavaScript)
//!
//! ```javascript
//! import { Mecab } from 'mecab-ko-wasm';
//!
//! const mecab = new Mecab();
//! const morphs = mecab.morphs("안녕하세요");
//! console.log(morphs); // ["안녕", "하", "세요"]
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

use mecab_ko_core::{Token, Tokenizer};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

// When the `console_error_panic_hook` feature is enabled, we can call the
// `set_panic_hook` function at least once during initialization to get better
// error messages if code panics.
#[cfg(feature = "console_error_panic_hook")]
fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Initialize the WASM module
///
/// This function should be called once before using the library.
/// It sets up panic hooks for better error messages in development.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();
}

/// A JavaScript-friendly token representation
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmToken {
    surface: String,
    pos: String,
    start: usize,
    end: usize,
    reading: Option<String>,
    lemma: Option<String>,
}

#[wasm_bindgen]
impl WasmToken {
    /// Get the surface form (표면형)
    #[must_use]
    #[wasm_bindgen(getter)]
    pub fn surface(&self) -> String {
        self.surface.clone()
    }

    /// Get the part-of-speech tag (품사)
    #[must_use]
    #[wasm_bindgen(getter)]
    pub fn pos(&self) -> String {
        self.pos.clone()
    }

    /// Get the start position in bytes
    #[must_use]
    #[wasm_bindgen(getter)]
    #[allow(clippy::missing_const_for_fn)] // wasm_bindgen doesn't support const fn
    pub fn start(&self) -> usize {
        self.start
    }

    /// Get the end position in bytes
    #[must_use]
    #[wasm_bindgen(getter)]
    #[allow(clippy::missing_const_for_fn)] // wasm_bindgen doesn't support const fn
    pub fn end(&self) -> usize {
        self.end
    }

    /// Get the reading (if available)
    #[must_use]
    #[wasm_bindgen(getter)]
    pub fn reading(&self) -> Option<String> {
        self.reading.clone()
    }

    /// Get the lemma/base form (if available)
    #[must_use]
    #[wasm_bindgen(getter)]
    pub fn lemma(&self) -> Option<String> {
        self.lemma.clone()
    }

    /// Convert to JSON string for easier JavaScript interop
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&TokenJson {
            surface: &self.surface,
            pos: &self.pos,
            start: self.start,
            end: self.end,
            reading: self.reading.as_deref(),
            lemma: self.lemma.as_deref(),
        })
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {e}")))
    }
}

#[derive(serde::Serialize)]
struct TokenJson<'a> {
    surface: &'a str,
    pos: &'a str,
    start: usize,
    end: usize,
    reading: Option<&'a str>,
    lemma: Option<&'a str>,
}

impl From<Token> for WasmToken {
    fn from(token: Token) -> Self {
        Self {
            surface: token.surface,
            pos: token.pos,
            start: token.start_byte,
            end: token.end_byte,
            reading: token.reading,
            lemma: token.lemma,
        }
    }
}

/// The main MeCab-Ko tokenizer for WebAssembly
///
/// This class provides Korean morphological analysis capabilities
/// in JavaScript/TypeScript environments.
#[wasm_bindgen]
pub struct Mecab {
    tokenizer: RefCell<Tokenizer>,
}

#[wasm_bindgen]
impl Mecab {
    /// Create a new Mecab instance with the default dictionary
    ///
    /// # Example (JavaScript)
    ///
    /// ```javascript
    /// const mecab = new Mecab();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if tokenizer initialization fails
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        Tokenizer::new()
            .map(|tokenizer| Self {
                tokenizer: RefCell::new(tokenizer),
            })
            .map_err(|e| JsValue::from_str(&format!("Failed to initialize tokenizer: {e}")))
    }

    /// Tokenize text and return detailed token information
    ///
    /// Returns an array of tokens with surface form, POS tag, and position information.
    ///
    /// # Example (JavaScript)
    ///
    /// ```javascript
    /// const tokens = mecab.tokenize("안녕하세요");
    /// tokens.forEach(token => {
    ///   console.log(`${token.surface}: ${token.pos}`);
    /// });
    /// ```
    #[wasm_bindgen]
    pub fn tokenize(&self, text: &str) -> Vec<WasmToken> {
        self.tokenizer
            .borrow_mut()
            .tokenize(text)
            .into_iter()
            .map(WasmToken::from)
            .collect()
    }

    /// Extract morphemes (형태소) from text
    ///
    /// Returns an array of morpheme strings without POS information.
    ///
    /// # Example (JavaScript)
    ///
    /// ```javascript
    /// const morphs = mecab.morphs("안녕하세요");
    /// console.log(morphs); // ["안녕", "하", "세요"]
    /// ```
    #[must_use]
    #[wasm_bindgen]
    pub fn morphs(&self, text: &str) -> Vec<String> {
        self.tokenizer.borrow_mut().morphs(text)
    }

    /// Extract part-of-speech tagged pairs
    ///
    /// Returns a JSON string containing an array of [surface, pos] pairs.
    ///
    /// # Example (JavaScript)
    ///
    /// ```javascript
    /// const posJson = mecab.pos("안녕하세요");
    /// const pos = JSON.parse(posJson);
    /// console.log(pos); // [["안녕", "NNG"], ["하", "XSV"], ["세요", "EP+EF"]]
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails
    #[wasm_bindgen]
    pub fn pos(&self, text: &str) -> Result<String, JsValue> {
        let pos_pairs = self.tokenizer.borrow_mut().pos(text);
        serde_json::to_string(&pos_pairs)
            .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {e}")))
    }

    /// Extract nouns (명사) from text
    ///
    /// Returns an array of noun strings.
    ///
    /// # Example (JavaScript)
    ///
    /// ```javascript
    /// const nouns = mecab.nouns("형태소 분석기입니다");
    /// console.log(nouns); // ["형태소", "분석기"]
    /// ```
    #[must_use]
    #[wasm_bindgen]
    pub fn nouns(&self, text: &str) -> Vec<String> {
        self.tokenizer.borrow_mut().nouns(text)
    }

    /// Perform wakati (분리) tokenization
    ///
    /// Returns an array of morpheme strings, similar to `morphs()`.
    ///
    /// # Example (JavaScript)
    ///
    /// ```javascript
    /// const words = mecab.wakati("형태소 분석");
    /// console.log(words); // ["형태소", "분석"]
    /// ```
    #[must_use]
    #[wasm_bindgen]
    pub fn wakati(&self, text: &str) -> Vec<String> {
        self.tokenizer.borrow_mut().wakati(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_mecab_creation() {
        let mecab = Mecab::new();
        assert!(mecab.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_morphs() {
        let mecab = Mecab::new().unwrap();
        let morphs = mecab.morphs("테스트");
        assert!(!morphs.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_tokenize() {
        let mecab = Mecab::new().unwrap();
        let tokens = mecab.tokenize("테스트");
        assert!(!tokens.is_empty());
        assert!(!tokens[0].surface().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_pos() {
        let mecab = Mecab::new().unwrap();
        let pos_json = mecab.pos("테스트");
        assert!(pos_json.is_ok());
        let pos_json = pos_json.unwrap();
        assert!(pos_json.contains('['));
        assert!(pos_json.contains(']'));
    }

    #[wasm_bindgen_test]
    fn test_token_json() {
        let token = WasmToken {
            surface: "테스트".to_string(),
            pos: "NNG".to_string(),
            start: 0,
            end: 9,
            reading: None,
            lemma: None,
        };
        let json = token.to_json();
        assert!(json.is_ok());
    }
}
