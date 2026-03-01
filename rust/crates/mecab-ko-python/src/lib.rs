//! # mecab-ko-python
//!
//! Python bindings for MeCab-Ko (Korean morphological analyzer)
//!
//! This crate provides Python bindings compatible with the `KoNLPy` Mecab API.
//!
//! ## Features
//!
//! - `morphs(text)` - Extract morphemes
//! - `nouns(text)` - Extract nouns
//! - `pos(text)` - Part-of-speech tagging
//! - `parse(text)` - `MeCab` format output
//!
//! ## Example (Python)
//!
//! ```python
//! from mecab_ko import Mecab
//!
//! mecab = Mecab()
//! print(mecab.morphs("안녕하세요"))
//! print(mecab.nouns("아버지가방에들어가신다"))
//! print(mecab.pos("나는 학생입니다"))
//! ```

use mecab_ko_core::Tokenizer;
use parking_lot::Mutex;
use pyo3::prelude::*;
use pyo3::types::PyModule;

/// MeCab-Ko tokenizer class for Python.
///
/// This class provides a KoNLPy-compatible interface for Korean morphological analysis.
///
/// # Examples
///
/// ```python
/// from mecab_ko import Mecab
///
/// mecab = Mecab()
/// morphemes = mecab.morphs("안녕하세요")
/// nouns = mecab.nouns("아버지가방에들어가신다")
/// pos_tags = mecab.pos("나는 학생입니다")
/// ```
#[pyclass(name = "Mecab")]
struct PyMecab {
    tokenizer: Mutex<Tokenizer>,
}

#[pymethods]
impl PyMecab {
    /// Create a new Mecab instance.
    ///
    /// # Arguments
    ///
    /// * `dicpath` - Optional path to dictionary directory
    ///
    /// # Returns
    ///
    /// A new Mecab instance
    ///
    /// # Example
    ///
    /// ```python
    /// mecab = Mecab()
    /// mecab_custom = Mecab(dicpath="/path/to/dict")
    /// ```
    #[new]
    #[pyo3(signature = (dicpath=None))]
    fn new(dicpath: Option<&str>) -> PyResult<Self> {
        let tokenizer = if let Some(path) = dicpath {
            Tokenizer::with_dict(path).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to initialize tokenizer with dictionary: {e}"
                ))
            })?
        } else {
            Tokenizer::new().map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to initialize tokenizer: {e}"
                ))
            })?
        };

        Ok(Self {
            tokenizer: Mutex::new(tokenizer),
        })
    }

    /// Extract morphemes from text.
    ///
    /// # Arguments
    ///
    /// * `text` - Input text to analyze
    ///
    /// # Returns
    ///
    /// List of morphemes (surface forms)
    ///
    /// # Example
    ///
    /// ```python
    /// mecab = Mecab()
    /// morphemes = mecab.morphs("안녕하세요")
    /// # ['안녕', '하', '세요']
    /// ```
    #[pyo3(text_signature = "($self, text)")]
    fn morphs(&self, text: &str) -> Vec<String> {
        self.tokenizer.lock().morphs(text)
    }

    /// Extract nouns from text.
    ///
    /// # Arguments
    ///
    /// * `text` - Input text to analyze
    ///
    /// # Returns
    ///
    /// List of nouns
    ///
    /// # Example
    ///
    /// ```python
    /// mecab = Mecab()
    /// nouns = mecab.nouns("아버지가방에들어가신다")
    /// # ['아버지', '가방']
    /// ```
    #[pyo3(text_signature = "($self, text)")]
    fn nouns(&self, text: &str) -> Vec<String> {
        self.tokenizer.lock().nouns(text)
    }

    /// Perform part-of-speech tagging.
    ///
    /// # Arguments
    ///
    /// * `text` - Input text to analyze
    ///
    /// # Returns
    ///
    /// List of tuples (surface, `pos_tag`)
    ///
    /// # Example
    ///
    /// ```python
    /// mecab = Mecab()
    /// tagged = mecab.pos("나는 학생입니다")
    /// # [('나', 'NP'), ('는', 'JX'), ('학생', 'NNG'), ('이', 'VCP'), ('ㅂ니다', 'EF')]
    /// ```
    #[pyo3(text_signature = "($self, text)")]
    fn pos(&self, text: &str) -> Vec<(String, String)> {
        self.tokenizer.lock().pos(text)
    }

    /// Parse text and return `MeCab` format output.
    ///
    /// # Arguments
    ///
    /// * `text` - Input text to analyze
    ///
    /// # Returns
    ///
    /// `MeCab` format string with tab-separated values
    ///
    /// # Example
    ///
    /// ```python
    /// mecab = Mecab()
    /// result = mecab.parse("안녕하세요")
    /// # "안녕\tNNG,*,F,안녕,*,*,*,*\n하\tXSV,*,F,하,*,*,*,*\n세요\tEF,*,F,세요,*,*,*,*\nEOS\n"
    /// ```
    #[pyo3(text_signature = "($self, text)")]
    fn parse(&self, text: &str) -> String {
        use std::fmt::Write;

        let tokens = self.tokenizer.lock().tokenize(text);
        let mut result = String::new();

        for token in tokens {
            // MeCab format: surface\tPOS,feature1,feature2,...
            // 기본 형식: surface\tPOS,*,*,lemma,*,*,*,*
            let features = format!(
                "{},*,*,{},*,*,*,*",
                token.pos,
                token.lemma.as_deref().unwrap_or(&token.surface)
            );
            let _ = writeln!(result, "{}\t{}", token.surface, features);
        }

        result.push_str("EOS\n");
        result
    }

    /// Alias for `morphs()` - extract morphemes.
    ///
    /// This method is provided for compatibility with some interfaces.
    #[pyo3(text_signature = "($self, text)")]
    fn wakati(&self, text: &str) -> Vec<String> {
        self.morphs(text)
    }

    /// Python __repr__ method.
    #[allow(clippy::unused_self)]
    fn __repr__(&self) -> String {
        "Mecab()".to_string()
    }

    /// Python __str__ method.
    #[allow(clippy::unused_self)]
    fn __str__(&self) -> String {
        "MeCab-Ko tokenizer".to_string()
    }
}

/// Python module initialization.
///
/// This function is called when the module is imported in Python.
/// It registers the Mecab class and module metadata.
#[pymodule]
fn mecab_ko(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMecab>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__doc__", "MeCab-Ko: Korean morphological analyzer")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mecab_creation() {
        let mecab = PyMecab::new(None);
        assert!(mecab.is_ok());
    }

    #[test]
    fn test_morphs() {
        let mecab = PyMecab::new(None).unwrap();
        let morphemes = mecab.morphs("테스트");
        assert!(!morphemes.is_empty());
    }

    #[test]
    fn test_nouns() {
        let mecab = PyMecab::new(None).unwrap();
        let _result = mecab.nouns("테스트");
        // No assertions needed, just verify it doesn't panic
    }

    #[test]
    fn test_pos() {
        let mecab = PyMecab::new(None).unwrap();
        let tagged = mecab.pos("테스트");
        assert!(!tagged.is_empty());
    }

    #[test]
    fn test_parse() {
        let mecab = PyMecab::new(None).unwrap();
        let output = mecab.parse("테스트");
        assert!(output.contains("EOS"));
    }
}
