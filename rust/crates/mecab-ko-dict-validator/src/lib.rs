//! MeCab-Ko Dictionary Validator
//!
//! This crate provides comprehensive validation tools for `MeCab` dictionary files,
//! including checks for CSV format, POS tags, costs, encoding, duplicates, and
//! normalization issues.
//!
//! # Features
//!
//! - CSV format validation
//! - POS tag validation with Korean tag support
//! - Cost range checking
//! - Duplicate entry detection
//! - Unicode normalization validation
//! - UTF-8 encoding validation
//! - Customizable validation rules via configuration files
//! - JSON and text report formats
//!
//! # Example
//!
//! ```no_run
//! use mecab_ko_dict_validator::{DictValidator, ValidationConfig};
//!
//! let validator = DictValidator::with_defaults();
//! let report = validator.validate_file("dictionary.csv")
//!     .expect("Failed to validate dictionary");
//!
//! if report.is_valid() {
//!     println!("Dictionary is valid!");
//! } else {
//!     println!("{}", report.to_text());
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod config;
pub mod report;
pub mod rules;
pub mod validator;

// Re-export main types
pub use config::{load_config, save_config, ConfigError};
pub use report::{
    IssueCategory, Location, Severity, ValidationIssue, ValidationReport, ValidationStatistics,
};
pub use rules::{
    CostRules, CsvRules, DuplicateRules, EncodingRules, NormalizationForm, NormalizationRules,
    PosRules, ValidationConfig,
};
pub use validator::{DictEntry, DictValidator, ValidationError};
