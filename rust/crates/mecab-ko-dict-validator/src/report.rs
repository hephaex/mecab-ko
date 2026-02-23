//! Validation report generation and formatting.
//!
//! This module provides functionality for generating validation reports in different
//! formats (JSON, text) and displaying error locations with context.

#![allow(clippy::write_with_newline)]
#![allow(clippy::single_char_add_str)]
#![allow(clippy::missing_const_for_fn)]

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// A validation report containing all errors, warnings, and statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// File that was validated
    pub file_path: PathBuf,
    /// Total number of entries processed
    pub total_entries: usize,
    /// Number of valid entries
    pub valid_entries: usize,
    /// Number of entries with errors
    pub error_entries: usize,
    /// Number of entries with warnings only
    pub warning_entries: usize,
    /// All validation issues found
    pub issues: Vec<ValidationIssue>,
    /// Validation statistics
    pub statistics: ValidationStatistics,
    /// Timestamp of validation
    pub timestamp: String,
}

impl ValidationReport {
    /// Creates a new validation report.
    #[must_use]
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            total_entries: 0,
            valid_entries: 0,
            error_entries: 0,
            warning_entries: 0,
            issues: Vec::new(),
            statistics: ValidationStatistics::default(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Adds a validation issue to the report.
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        match issue.severity {
            Severity::Error => self.error_entries += 1,
            Severity::Warning => self.warning_entries += 1,
            Severity::Info => {}
        }
        self.issues.push(issue);
    }

    /// Returns whether the validation passed (no errors).
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.error_entries == 0
    }

    /// Returns whether there are any warnings.
    #[must_use]
    pub const fn has_warnings(&self) -> bool {
        self.warning_entries > 0
    }

    /// Formats the report as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Formats the report as human-readable text.
    #[must_use]
    pub fn to_text(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        output.push_str("═══════════════════════════════════════════════════════════\n");
        output.push_str("  MeCab-Ko Dictionary Validation Report\n");
        output.push_str("═══════════════════════════════════════════════════════════\n\n");

        let _ = write!(output, "File: {}\n", self.file_path.display());
        let _ = write!(output, "Timestamp: {}\n\n", self.timestamp);

        output.push_str("Summary:\n");
        output.push_str("───────────────────────────────────────────────────────────\n");
        let _ = write!(output, "  Total entries:   {}\n", self.total_entries);
        let _ = write!(output, "  Valid entries:   {}\n", self.valid_entries);
        let _ = write!(output, "  Errors:          {}\n", self.error_entries);
        let _ = write!(output, "  Warnings:        {}\n\n", self.warning_entries);

        if !self.is_valid() {
            output.push_str("Status: FAILED\n\n");
        } else if self.has_warnings() {
            output.push_str("Status: PASSED (with warnings)\n\n");
        } else {
            output.push_str("Status: PASSED\n\n");
        }

        // Statistics
        output.push_str("Statistics:\n");
        output.push_str("───────────────────────────────────────────────────────────\n");
        output.push_str(&self.statistics.to_text());
        output.push_str("\n");

        // Issues grouped by severity
        if !self.issues.is_empty() {
            let errors: Vec<_> = self
                .issues
                .iter()
                .filter(|i| i.severity == Severity::Error)
                .collect();
            let warnings: Vec<_> = self
                .issues
                .iter()
                .filter(|i| i.severity == Severity::Warning)
                .collect();
            let info: Vec<_> = self
                .issues
                .iter()
                .filter(|i| i.severity == Severity::Info)
                .collect();

            if !errors.is_empty() {
                output.push_str("Errors:\n");
                output.push_str("───────────────────────────────────────────────────────────\n");
                for (idx, issue) in errors.iter().enumerate() {
                    let _ = writeln!(output, "{}. {issue}", idx + 1);
                }
                output.push('\n');
            }

            if !warnings.is_empty() {
                output.push_str("Warnings:\n");
                output.push_str("───────────────────────────────────────────────────────────\n");
                for (idx, issue) in warnings.iter().enumerate() {
                    let _ = writeln!(output, "{}. {issue}", idx + 1);
                }
                output.push('\n');
            }

            if !info.is_empty() {
                output.push_str("Info:\n");
                output.push_str("───────────────────────────────────────────────────────────\n");
                for (idx, issue) in info.iter().enumerate() {
                    let _ = writeln!(output, "{}. {issue}", idx + 1);
                }
                output.push('\n');
            }
        }

        output.push_str("═══════════════════════════════════════════════════════════\n");

        output
    }
}

/// A single validation issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue severity
    pub severity: Severity,
    /// Issue category
    pub category: IssueCategory,
    /// Issue message
    pub message: String,
    /// Location in file
    pub location: Option<Location>,
    /// Suggested fix (if any)
    pub suggestion: Option<String>,
    /// Additional context
    pub context: Option<String>,
}

impl ValidationIssue {
    /// Creates a new error issue.
    #[must_use]
    pub fn error(category: IssueCategory, message: String) -> Self {
        Self {
            severity: Severity::Error,
            category,
            message,
            location: None,
            suggestion: None,
            context: None,
        }
    }

    /// Creates a new warning issue.
    #[must_use]
    pub fn warning(category: IssueCategory, message: String) -> Self {
        Self {
            severity: Severity::Warning,
            category,
            message,
            location: None,
            suggestion: None,
            context: None,
        }
    }

    /// Creates a new info issue.
    #[must_use]
    pub fn info(category: IssueCategory, message: String) -> Self {
        Self {
            severity: Severity::Info,
            category,
            message,
            location: None,
            suggestion: None,
            context: None,
        }
    }

    /// Sets the location for this issue.
    #[must_use]
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    /// Sets a suggestion for fixing this issue.
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Sets additional context for this issue.
    #[must_use]
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:?}] {:?}: {}",
            self.severity, self.category, self.message
        )?;

        if let Some(ref location) = self.location {
            write!(f, " at {location}")?;
        }

        if let Some(ref context) = self.context {
            write!(f, "\n    Context: {context}")?;
        }

        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\n    Suggestion: {suggestion}")?;
        }

        Ok(())
    }
}

/// Issue severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational message
    Info,
    /// Warning (does not fail validation)
    Warning,
    /// Error (fails validation)
    Error,
}

/// Issue categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueCategory {
    /// CSV format issues
    CsvFormat,
    /// POS tag issues
    PosTag,
    /// Cost-related issues
    Cost,
    /// Encoding issues
    Encoding,
    /// Duplicate entry issues
    Duplicate,
    /// Normalization issues
    Normalization,
    /// Connection cost issues
    ConnectionCost,
    /// General format issues
    Format,
}

/// Location in a file.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed, optional)
    pub column: Option<usize>,
}

impl Location {
    /// Creates a new location.
    #[must_use]
    pub const fn new(line: usize) -> Self {
        Self { line, column: None }
    }

    /// Creates a new location with column.
    #[must_use]
    pub const fn with_column(line: usize, column: usize) -> Self {
        Self {
            line,
            column: Some(column),
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(column) = self.column {
            write!(f, "line {}, column {}", self.line, column)
        } else {
            write!(f, "line {}", self.line)
        }
    }
}

/// Validation statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationStatistics {
    /// Count of each POS tag
    pub pos_tag_counts: std::collections::HashMap<String, usize>,
    /// Count of each issue category
    pub category_counts: std::collections::HashMap<String, usize>,
    /// Average word cost
    pub average_cost: Option<f64>,
    /// Minimum word cost
    pub min_cost: Option<i32>,
    /// Maximum word cost
    pub max_cost: Option<i32>,
    /// Number of unique surface forms
    pub unique_surface_forms: usize,
    /// Number of duplicate entries
    pub duplicate_count: usize,
}

impl ValidationStatistics {
    /// Formats statistics as text.
    #[must_use]
    pub fn to_text(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        if let (Some(avg), Some(min), Some(max)) = (self.average_cost, self.min_cost, self.max_cost)
        {
            output.push_str("  Cost statistics:\n");
            let _ = writeln!(output, "    Average: {avg:.2}");
            let _ = writeln!(output, "    Min: {min}");
            let _ = writeln!(output, "    Max: {max}");
        }

        let _ = writeln!(
            output,
            "  Unique surface forms: {}",
            self.unique_surface_forms
        );

        if self.duplicate_count > 0 {
            let _ = writeln!(output, "  Duplicate entries: {}", self.duplicate_count);
        }

        if !self.pos_tag_counts.is_empty() {
            output.push_str("\n  Top POS tags:\n");
            let mut tags: Vec<_> = self.pos_tag_counts.iter().collect();
            tags.sort_by(|a, b| b.1.cmp(a.1));
            for (tag, count) in tags.iter().take(10) {
                let _ = writeln!(output, "    {tag}: {count}");
            }
        }

        output
    }
}

// Add chrono dependency for timestamp
use chrono;

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new(PathBuf::from("test.csv"));
        report.total_entries = 100;
        report.valid_entries = 95;

        let issue =
            ValidationIssue::error(IssueCategory::CsvFormat, "Invalid field count".to_string())
                .with_location(Location::new(42))
                .with_suggestion("Expected 13 fields".to_string());

        report.add_issue(issue);

        assert_eq!(report.error_entries, 1);
        assert!(!report.is_valid());

        let text = report.to_text();
        assert!(text.contains("FAILED"));
        assert!(text.contains("line 42"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn test_location_display() {
        let loc1 = Location::new(10);
        assert_eq!(loc1.to_string(), "line 10");

        let loc2 = Location::with_column(10, 5);
        assert_eq!(loc2.to_string(), "line 10, column 5");
    }

    #[test]
    fn test_issue_builder() {
        let issue = ValidationIssue::warning(IssueCategory::Cost, "Unusual cost value".to_string())
            .with_location(Location::new(100))
            .with_context("Cost: 9999".to_string())
            .with_suggestion("Consider using a lower cost value".to_string());

        assert_eq!(issue.severity, Severity::Warning);
        assert!(issue.location.is_some());
        assert!(issue.suggestion.is_some());
        assert!(issue.context.is_some());
    }

    #[test]
    fn test_json_serialization() {
        let mut report = ValidationReport::new(PathBuf::from("test.csv"));
        report.total_entries = 10;
        report.valid_entries = 9;
        report.add_issue(ValidationIssue::error(
            IssueCategory::Encoding,
            "Invalid UTF-8".to_string(),
        ));

        let json = report.to_json().expect("Failed to serialize to JSON");
        assert!(json.contains("\"total_entries\": 10"));
        assert!(json.contains("\"Encoding\""));
    }
}
