//! Dictionary synchronization and conversion utilities for MeCab-Ko.
//!
//! This crate provides tools for:
//! - Converting dictionary entries from various formats to MeCab-Ko format
//! - Mapping POS tags from different Korean NLP systems
//! - Synchronizing with external dictionary sources (e.g., NIKL Open Dictionary)
//!
//! # Examples
//!
//! ```
//! use mecab_ko_dict_sync::{DictConverter, ConverterEntry};
//!
//! let converter = DictConverter::new();
//! let entry = ConverterEntry {
//!     surface: "챗GPT".to_string(),
//!     pos: "고유명사".to_string(),
//!     reading: Some("챗지피티".to_string()),
//!     frequency: Some(1000),
//! };
//!
//! let user_entry = converter.convert_entry(&entry).unwrap();
//! assert_eq!(user_entry.pos, "NNP");
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod client;
pub mod config;
mod converter;
pub mod error;
pub mod models;

pub use converter::{ConverterEntry, DictConverter, UserEntry};

/// Errors that can occur during dictionary synchronization and conversion.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// Unknown POS tag encountered during mapping.
    #[error("Unknown POS tag: {0}")]
    UnknownPosTag(String),

    /// Invalid entry data.
    #[error("Invalid entry: {0}")]
    InvalidEntry(String),

    /// CSV processing error.
    #[error("CSV error: {0}")]
    Csv(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(String),
}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Self::Csv(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}
