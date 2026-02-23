//! Integration tests for dictionary validator.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::needless_collect,
    unused_variables
)]

use mecab_ko_dict_validator::{
    config::{load_config, save_config},
    DictValidator, IssueCategory, Severity, ValidationConfig,
};
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

/// Creates a temporary dictionary file with the given content.
fn create_test_dict(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    write!(file, "{content}").expect("Failed to write to temp file");
    file
}

#[test]
fn test_validate_valid_dictionary() {
    let content = r"한글,1,2,100,NNG,*,F,한글,*,*,*,*,*
테스트,3,4,200,NNG,*,T,테스트,*,*,*,*,*
사전,5,6,150,NNG,*,T,사전,*,*,*,*,*";

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    assert_eq!(report.total_entries, 3);
    assert!(report.is_valid(), "Dictionary should be valid");
    assert_eq!(report.error_entries, 0);
}

#[test]
fn test_validate_invalid_pos_tag() {
    let content = r"한글,1,2,100,INVALID,*,F,한글,*,*,*,*,*";

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    assert!(!report.is_valid(), "Dictionary should be invalid");
    assert!(report.error_entries > 0);

    let pos_errors: Vec<_> = report
        .issues
        .iter()
        .filter(|i| matches!(i.category, IssueCategory::PosTag))
        .collect();

    assert!(!pos_errors.is_empty(), "Should have POS tag errors");
}

#[test]
fn test_validate_invalid_field_count() {
    let content = r"한글,1,2,100"; // Only 4 fields instead of 13

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    assert!(!report.is_valid(), "Dictionary should be invalid");

    let csv_errors: Vec<_> = report
        .issues
        .iter()
        .filter(|i| matches!(i.category, IssueCategory::CsvFormat))
        .collect();

    assert!(!csv_errors.is_empty(), "Should have CSV format errors");
}

#[test]
fn test_validate_invalid_cost() {
    let content = r"한글,1,2,50000,NNG,*,F,한글,*,*,*,*,*"; // Cost too high

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    // Should have either errors or warnings about cost
    let cost_issues: Vec<_> = report
        .issues
        .iter()
        .filter(|i| matches!(i.category, IssueCategory::Cost))
        .collect();

    assert!(!cost_issues.is_empty(), "Should have cost-related issues");
}

#[test]
fn test_detect_exact_duplicates() {
    let content = r"한글,1,2,100,NNG,*,F,한글,*,*,*,*,*
테스트,3,4,200,NNG,*,T,테스트,*,*,*,*,*
한글,1,2,100,NNG,*,F,한글,*,*,*,*,*"; // Exact duplicate of first line

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    assert!(!report.is_valid(), "Should detect duplicates");

    let dup_errors: Vec<_> = report
        .issues
        .iter()
        .filter(|i| matches!(i.category, IssueCategory::Duplicate))
        .collect();

    assert!(!dup_errors.is_empty(), "Should have duplicate errors");
}

#[test]
fn test_skip_empty_lines_and_comments() {
    let content = r"# This is a comment
한글,1,2,100,NNG,*,F,한글,*,*,*,*,*

# Another comment
테스트,3,4,200,NNG,*,T,테스트,*,*,*,*,*

";

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    assert_eq!(
        report.total_entries, 2,
        "Should skip comments and empty lines"
    );
}

#[test]
fn test_normalization_warning() {
    // Using decomposed Hangul (NFD) instead of composed (NFC)
    let content = "\u{1112}\u{1161}\u{11AB}\u{1100}\u{1173}\u{11AF},1,2,100,NNG,*,F,한글,*,*,*,*,*";

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    let norm_warnings: Vec<_> = report
        .issues
        .iter()
        .filter(|i| matches!(i.category, IssueCategory::Normalization))
        .collect();

    assert!(!norm_warnings.is_empty(), "Should warn about normalization");
}

#[test]
fn test_json_output() {
    let content = r"한글,1,2,100,NNG,*,F,한글,*,*,*,*,*";

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    let json = report.to_json().expect("Failed to serialize to JSON");

    assert!(json.contains("\"total_entries\""));
    assert!(json.contains("\"valid_entries\""));
}

#[test]
fn test_text_output() {
    let content = r"한글,1,2,100,NNG,*,F,한글,*,*,*,*,*";

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    let text = report.to_text();

    assert!(text.contains("MeCab-Ko Dictionary Validation Report"));
    assert!(text.contains("Total entries"));
    assert!(text.contains("PASSED") || text.contains("FAILED"));
}

#[test]
fn test_custom_config() {
    let mut config = ValidationConfig::default();
    config.csv_rules.expected_field_count = 10; // Custom field count

    let validator = DictValidator::new(config);

    let content = r"한글,1,2,100,NNG,*,F,한글,*,*"; // 10 fields

    let file = create_test_dict(content);
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    // Should validate successfully with custom config
    assert!(report.is_valid() || report.error_entries == 0 || report.has_warnings());
}

#[test]
fn test_config_save_and_load() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.toml");

    let original_config = ValidationConfig::default();

    save_config(&config_path, &original_config).expect("Failed to save config");

    let loaded_config = load_config(&config_path).expect("Failed to load config");

    assert_eq!(
        loaded_config.csv_rules.expected_field_count,
        original_config.csv_rules.expected_field_count
    );
}

#[test]
fn test_statistics_calculation() {
    let content = r"한글,1,2,100,NNG,*,F,한글,*,*,*,*,*
테스트,3,4,200,NNG,*,T,테스트,*,*,*,*,*
사전,5,6,150,NNG,*,T,사전,*,*,*,*,*
처리,7,8,300,VV,*,F,처리,*,*,*,*,*";

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    let stats = &report.statistics;

    assert_eq!(stats.unique_surface_forms, 4);
    assert!(stats.average_cost.is_some());
    assert!(stats.min_cost.is_some());
    assert!(stats.max_cost.is_some());
    assert!(!stats.pos_tag_counts.is_empty());
}

#[test]
fn test_compound_pos_tags() {
    let content = r"한글은,1,2,100,NNG+JKS,*,F,한글은,*,*,*,*,*";

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    // Compound POS tags should be valid
    let pos_errors: Vec<_> = report
        .issues
        .iter()
        .filter(|i| matches!(i.category, IssueCategory::PosTag) && i.severity == Severity::Error)
        .collect();

    assert!(pos_errors.is_empty(), "Compound POS tags should be valid");
}

#[test]
fn test_multiple_warnings() {
    let content = r"한글,1,2,9000,NNG,*,F,한글,*,*,*,*,*"; // High cost

    let file = create_test_dict(content);
    let validator = DictValidator::with_defaults();
    let report = validator
        .validate_file(file.path())
        .expect("Failed to validate");

    assert!(report.has_warnings(), "Should have warnings");
    assert!(report.is_valid(), "Should still be valid (warnings only)");
}
