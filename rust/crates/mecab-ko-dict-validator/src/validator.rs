//! Main dictionary validation logic.
//!
//! This module provides the core validation functionality for `MeCab` dictionary files.

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::redundant_closure_for_method_calls)]

use crate::report::{IssueCategory, Location, ValidationIssue, ValidationReport};
use crate::rules::ValidationConfig;
use csv::StringRecord;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Dictionary entry validator.
pub struct DictValidator {
    config: ValidationConfig,
}

impl DictValidator {
    /// Creates a new validator with the given configuration.
    #[must_use]
    pub const fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Creates a validator with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ValidationConfig::default())
    }

    /// Validates a dictionary file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or processed.
    pub fn validate_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<ValidationReport, ValidationError> {
        let path = path.as_ref();
        let mut report = ValidationReport::new(path.to_path_buf());

        // Read and validate file encoding
        let file = File::open(path).map_err(|e| ValidationError::IoError(e.to_string()))?;

        let reader = BufReader::new(file);

        // Check for BOM
        if !self.config.encoding_rules.allow_bom {
            let mut first_bytes = [0u8; 3];
            let mut peek_reader = BufReader::new(File::open(path)?);
            if std::io::Read::read_exact(&mut peek_reader, &mut first_bytes).is_ok() {
                if first_bytes == [0xEF, 0xBB, 0xBF] {
                    report.add_issue(ValidationIssue::warning(
                        IssueCategory::Encoding,
                        "File contains UTF-8 BOM".to_string(),
                    ));
                }
            }
        }

        // Process entries
        let entries = self.read_entries(reader)?;
        report.total_entries = entries.len();

        // Validate entries in parallel
        let issues: Vec<_> = entries
            .par_iter()
            .enumerate()
            .flat_map(|(line_num, entry)| self.validate_entry(entry, line_num + 1))
            .collect();

        // Detect duplicates
        let duplicate_issues = self.detect_duplicates(&entries);

        // Collect all issues
        for issue in issues.into_iter().chain(duplicate_issues) {
            report.add_issue(issue);
        }

        // Calculate statistics and store entries for analysis
        report.statistics = Self::calculate_statistics(&entries);
        report.entries = Some(entries);
        report.valid_entries = report.total_entries.saturating_sub(report.error_entries);

        Ok(report)
    }

    /// Reads all entries from the reader.
    fn read_entries<R: BufRead>(&self, reader: R) -> Result<Vec<DictEntry>, ValidationError> {
        let mut entries = Vec::new();
        let mut line_num = 0;

        for line in reader.lines() {
            line_num += 1;
            let line = line.map_err(|e| ValidationError::IoError(e.to_string()))?;

            // Validate UTF-8
            if self.config.encoding_rules.validate_utf8 {
                // line is already validated as valid UTF-8 by the lines() iterator
                // But we can check for other encoding issues
                if line.chars().any(|c| c == '\u{FFFD}') {
                    return Err(ValidationError::EncodingError(format!(
                        "Invalid UTF-8 sequence at line {line_num}"
                    )));
                }
            }

            if line.trim().is_empty() || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            match Self::parse_entry(&line, line_num) {
                Ok(entry) => entries.push(entry),
                Err(e) => return Err(e),
            }
        }

        Ok(entries)
    }

    /// Parses a single entry from a CSV line.
    fn parse_entry(line: &str, line_num: usize) -> Result<DictEntry, ValidationError> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(line.as_bytes());

        let mut records = rdr.records();
        let record = records
            .next()
            .ok_or_else(|| ValidationError::ParseError(format!("Empty line at {line_num}")))?
            .map_err(|e| {
                ValidationError::ParseError(format!("CSV parse error at line {line_num}: {e}"))
            })?;

        Self::record_to_entry(&record, line_num)
    }

    /// Converts a CSV record to a dictionary entry.
    fn record_to_entry(
        record: &StringRecord,
        line_num: usize,
    ) -> Result<DictEntry, ValidationError> {
        let field_count = record.len();

        if field_count < 4 {
            return Err(ValidationError::ParseError(format!(
                "Insufficient fields at line {line_num}: expected at least 4, got {field_count}"
            )));
        }

        let surface = record
            .get(0)
            .ok_or_else(|| {
                ValidationError::ParseError(format!("Missing surface form at line {line_num}"))
            })?
            .to_string();

        let left_id = record
            .get(1)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| {
                ValidationError::ParseError(format!("Invalid left context ID at line {line_num}"))
            })?;

        let right_id = record
            .get(2)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| {
                ValidationError::ParseError(format!("Invalid right context ID at line {line_num}"))
            })?;

        let cost = record
            .get(3)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| {
                ValidationError::ParseError(format!("Invalid cost at line {line_num}"))
            })?;

        let pos_tag = record.get(4).unwrap_or("").to_string();

        // Collect additional features
        let features: Vec<String> = (5..field_count)
            .filter_map(|i| record.get(i).map(|s| s.to_string()))
            .collect();

        Ok(DictEntry {
            surface,
            left_id,
            right_id,
            cost,
            pos_tag,
            features,
            line_num,
        })
    }

    /// Validates a single entry.
    fn validate_entry(&self, entry: &DictEntry, line_num: usize) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // CSV format validation
        let total_fields = 5 + entry.features.len();
        if total_fields != self.config.csv_rules.expected_field_count {
            issues.push(
                ValidationIssue::error(
                    IssueCategory::CsvFormat,
                    format!(
                        "Invalid field count: expected {}, got {total_fields}",
                        self.config.csv_rules.expected_field_count
                    ),
                )
                .with_location(Location::new(line_num)),
            );
        }

        // Check for empty fields
        if !self.config.csv_rules.allow_empty_fields {
            if entry.surface.is_empty() {
                issues.push(
                    ValidationIssue::error(
                        IssueCategory::CsvFormat,
                        "Empty surface form".to_string(),
                    )
                    .with_location(Location::new(line_num)),
                );
            }

            if entry.pos_tag.is_empty() {
                issues.push(
                    ValidationIssue::error(IssueCategory::PosTag, "Empty POS tag".to_string())
                        .with_location(Location::new(line_num)),
                );
            }
        }

        // POS tag validation
        if !entry.pos_tag.is_empty() && !self.config.pos_rules.is_valid_tag(&entry.pos_tag) {
            issues.push(
                ValidationIssue::error(
                    IssueCategory::PosTag,
                    format!("Invalid POS tag: '{}'", entry.pos_tag),
                )
                .with_location(Location::new(line_num))
                .with_suggestion("Check against valid Korean POS tags".to_string()),
            );
        }

        // Cost validation
        let cost_result =
            self.config
                .cost_rules
                .validate_costs(entry.left_id, entry.right_id, entry.cost);

        for error in cost_result.errors {
            issues.push(
                ValidationIssue::error(IssueCategory::Cost, error)
                    .with_location(Location::new(line_num)),
            );
        }

        for warning in cost_result.warnings {
            issues.push(
                ValidationIssue::warning(IssueCategory::Cost, warning)
                    .with_location(Location::new(line_num)),
            );
        }

        // Normalization validation
        issues.extend(self.validate_normalization(entry, line_num));

        issues
    }

    /// Validates Unicode normalization for an entry.
    fn validate_normalization(&self, entry: &DictEntry, line_num: usize) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let rules = &self.config.normalization_rules;

        if rules.check_unicode_normalization {
            let normalized = rules.preferred_normalization.normalize(&entry.surface);
            if normalized != entry.surface {
                issues.push(
                    ValidationIssue::warning(
                        IssueCategory::Normalization,
                        format!(
                            "Surface form '{}' is not in {:?} form",
                            entry.surface, rules.preferred_normalization
                        ),
                    )
                    .with_location(Location::new(line_num))
                    .with_suggestion(format!("Use: '{normalized}'")),
                );
            }
        }

        if rules.check_hangul_composition {
            // Check if Hangul characters are properly composed
            let has_decomposed_hangul = entry.surface.chars().any(|c| {
                matches!(c,
                    '\u{1100}'..='\u{11FF}' | // Hangul Jamo
                    '\u{3130}'..='\u{318F}'   // Hangul Compatibility Jamo
                )
            });

            if has_decomposed_hangul {
                issues.push(
                    ValidationIssue::warning(
                        IssueCategory::Normalization,
                        "Surface form contains decomposed Hangul jamo".to_string(),
                    )
                    .with_location(Location::new(line_num))
                    .with_suggestion("Use composed Hangul syllables".to_string()),
                );
            }
        }

        if rules.warn_on_whitespace && entry.surface.contains(char::is_whitespace) {
            issues.push(
                ValidationIssue::warning(
                    IssueCategory::Normalization,
                    "Surface form contains whitespace".to_string(),
                )
                .with_location(Location::new(line_num)),
            );
        }

        issues
    }

    /// Detects duplicate entries.
    fn detect_duplicates(&self, entries: &[DictEntry]) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let rules = &self.config.duplicate_rules;

        if rules.detect_exact_duplicates {
            let mut seen = HashMap::new();

            for entry in entries {
                let key = format!(
                    "{}|{}|{}|{}|{}",
                    entry.surface, entry.left_id, entry.right_id, entry.cost, entry.pos_tag
                );

                if let Some(&first_line) = seen.get(&key) {
                    issues.push(
                        ValidationIssue::error(
                            IssueCategory::Duplicate,
                            format!("Exact duplicate of line {first_line}"),
                        )
                        .with_location(Location::new(entry.line_num))
                        .with_context(format!("Surface: '{}'", entry.surface)),
                    );
                } else {
                    seen.insert(key, entry.line_num);
                }
            }
        }

        if rules.detect_semantic_duplicates && !rules.allow_cost_variants {
            let mut seen = HashMap::new();

            for entry in entries {
                let key = format!("{}|{}", entry.surface, entry.pos_tag);

                if let Some(&first_line) = seen.get(&key) {
                    issues.push(
                        ValidationIssue::warning(
                            IssueCategory::Duplicate,
                            format!("Semantic duplicate of line {first_line} (same surface+POS)"),
                        )
                        .with_location(Location::new(entry.line_num))
                        .with_context(format!(
                            "Surface: '{}', POS: '{}'",
                            entry.surface, entry.pos_tag
                        )),
                    );
                } else {
                    seen.insert(key, entry.line_num);
                }
            }
        }

        issues
    }

    /// Calculates validation statistics.
    fn calculate_statistics(entries: &[DictEntry]) -> crate::report::ValidationStatistics {
        let mut stats = crate::report::ValidationStatistics::default();

        let mut costs = Vec::new();
        let mut surface_forms = HashSet::new();

        for entry in entries {
            // POS tag counts
            *stats
                .pos_tag_counts
                .entry(entry.pos_tag.clone())
                .or_insert(0) += 1;

            // Cost statistics
            costs.push(entry.cost);

            // Unique surface forms
            surface_forms.insert(entry.surface.clone());
        }

        stats.unique_surface_forms = surface_forms.len();

        if !costs.is_empty() {
            stats.min_cost = costs.iter().min().copied();
            stats.max_cost = costs.iter().max().copied();
            stats.average_cost =
                Some(costs.iter().map(|&c| f64::from(c)).sum::<f64>() / costs.len() as f64);
        }

        // Duplicate count
        stats.duplicate_count = entries.len() - surface_forms.len();

        stats
    }
}

/// A dictionary entry.
#[derive(Debug, Clone)]
pub struct DictEntry {
    /// Surface form
    pub surface: String,
    /// Left context ID
    pub left_id: i32,
    /// Right context ID
    pub right_id: i32,
    /// Word cost
    pub cost: i32,
    /// POS tag
    pub pos_tag: String,
    /// Additional features
    pub features: Vec<String>,
    /// Line number in source file
    pub line_num: usize,
}

/// Validation error.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Encoding error
    #[error("Encoding error: {0}")]
    EncodingError(String),
}

impl From<std::io::Error> for ValidationError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::needless_collect)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_entry() {
        let line = "한글,1,2,100,NNG,*,F,한글,*,*,*,*,*";
        let entry = DictValidator::parse_entry(line, 1).expect("Failed to parse entry");

        assert_eq!(entry.surface, "한글");
        assert_eq!(entry.left_id, 1);
        assert_eq!(entry.right_id, 2);
        assert_eq!(entry.cost, 100);
        assert_eq!(entry.pos_tag, "NNG");
    }

    #[test]
    fn test_validate_entry_valid() {
        let entry = DictEntry {
            surface: "테스트".to_string(),
            left_id: 100,
            right_id: 200,
            cost: 500,
            pos_tag: "NNG".to_string(),
            features: vec!["*".to_string(); 8],
            line_num: 1,
        };

        let validator = DictValidator::with_defaults();
        let issues = validator.validate_entry(&entry, 1);

        // Should have no errors, might have warnings
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == crate::report::Severity::Error)
            .collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_entry_invalid_pos() {
        let entry = DictEntry {
            surface: "테스트".to_string(),
            left_id: 100,
            right_id: 200,
            cost: 500,
            pos_tag: "INVALID".to_string(),
            features: vec!["*".to_string(); 8],
            line_num: 1,
        };

        let validator = DictValidator::with_defaults();
        let issues = validator.validate_entry(&entry, 1);

        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == crate::report::Severity::Error)
            .collect();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_detect_exact_duplicates() {
        let entries = vec![
            DictEntry {
                surface: "중복".to_string(),
                left_id: 1,
                right_id: 2,
                cost: 100,
                pos_tag: "NNG".to_string(),
                features: vec![],
                line_num: 1,
            },
            DictEntry {
                surface: "중복".to_string(),
                left_id: 1,
                right_id: 2,
                cost: 100,
                pos_tag: "NNG".to_string(),
                features: vec![],
                line_num: 2,
            },
        ];

        let validator = DictValidator::with_defaults();
        let issues = validator.detect_duplicates(&entries);

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| matches!(i.category, IssueCategory::Duplicate)));
    }
}
