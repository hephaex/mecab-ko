//! Test utilities for full-dictionary integration tests.
//!
//! This module provides helpers for tests that require a fully-built MeCab-Ko
//! dictionary. In CI environments without the dictionary, these tests are
//! skipped gracefully rather than failing.
//!
//! # Environment Variable
//!
//! Set `MECAB_KO_FULL_DICT` to the path of a built dictionary directory
//! (one that contains `sys.dic`, `matrix.bin`, `unk.bin`, etc.) to enable
//! full-dictionary tests.
//!
//! # Examples
//!
//! ```rust,ignore
//! #[test]
//! fn test_with_full_dict() {
//!     mecab_ko_core::full_dict_test!();
//!
//!     let dict_path = mecab_ko_core::test_utils::full_dict_path().unwrap();
//!     // ... use dict_path to load the dictionary ...
//! }
//! ```

/// Skips the current test when the `MECAB_KO_FULL_DICT` environment variable
/// is not set.
///
/// When the variable is set it must point to a directory that contains a
/// pre-built MeCab-Ko dictionary (`sys.dic`, `matrix.bin`, `unk.bin`, …).
///
/// # Usage
///
/// Place `full_dict_test!();` at the very top of a `#[test]` function body.
/// If the environment variable is absent the test prints a notice to `stderr`
/// and returns immediately without failing.
///
/// # Examples
///
/// ```rust,ignore
/// #[test]
/// fn test_tokenize_sentence() {
///     mecab_ko_core::full_dict_test!();
///
///     // Only reached when MECAB_KO_FULL_DICT is set.
///     let path = mecab_ko_core::test_utils::full_dict_path().unwrap();
///     assert!(path.exists());
/// }
/// ```
#[macro_export]
macro_rules! full_dict_test {
    () => {
        if std::env::var("MECAB_KO_FULL_DICT").is_err() {
            eprintln!("[skip] Set MECAB_KO_FULL_DICT=/path/to/dict to run full-dictionary tests");
            return;
        }
    };
}

/// Returns the full-dictionary path from the `MECAB_KO_FULL_DICT` environment
/// variable, or `None` when the variable is not set.
///
/// # Examples
///
/// ```rust,ignore
/// if let Some(path) = mecab_ko_core::test_utils::full_dict_path() {
///     println!("Using dictionary at {}", path.display());
/// }
/// ```
#[must_use]
pub fn full_dict_path() -> Option<std::path::PathBuf> {
    std::env::var("MECAB_KO_FULL_DICT")
        .ok()
        .map(std::path::PathBuf::from)
}
