//! Validation rules for `MeCab` dictionary entries.
//!
//! This module defines the validation rules and constraints for dictionary entries,
//! including POS tags, cost ranges, CSV format, and encoding validation.

#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::missing_const_for_fn)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::RangeInclusive;

/// Configuration for validation rules.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationConfig {
    /// Rules for CSV format validation
    pub csv_rules: CsvRules,
    /// Rules for POS tag validation
    pub pos_rules: PosRules,
    /// Rules for cost validation
    pub cost_rules: CostRules,
    /// Rules for encoding validation
    pub encoding_rules: EncodingRules,
    /// Rules for duplicate detection
    pub duplicate_rules: DuplicateRules,
    /// Rules for surface form normalization
    pub normalization_rules: NormalizationRules,
}

/// Rules for CSV format validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvRules {
    /// Expected number of fields in a CSV row
    pub expected_field_count: usize,
    /// Whether to allow empty fields
    pub allow_empty_fields: bool,
    /// Whether to trim whitespace from fields
    pub trim_fields: bool,
    /// Maximum field length (0 = unlimited)
    pub max_field_length: usize,
}

impl Default for CsvRules {
    fn default() -> Self {
        Self {
            expected_field_count: 13, // MeCab standard format
            allow_empty_fields: false,
            trim_fields: true,
            max_field_length: 0,
        }
    }
}

/// Rules for POS (Part-of-Speech) tag validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosRules {
    /// Valid POS tags (empty = accept all)
    pub valid_tags: HashSet<String>,
    /// Whether to validate tag hierarchy
    pub validate_hierarchy: bool,
    /// Maximum tag depth
    pub max_tag_depth: usize,
    /// Tag separator
    pub tag_separator: char,
}

impl Default for PosRules {
    fn default() -> Self {
        Self {
            valid_tags: Self::default_korean_pos_tags(),
            validate_hierarchy: true,
            max_tag_depth: 4,
            tag_separator: '+',
        }
    }
}

impl PosRules {
    /// Returns the default set of Korean POS tags.
    #[must_use]
    pub fn default_korean_pos_tags() -> HashSet<String> {
        [
            // 체언 (Nominals)
            "NNG", "NNP", "NNB", "NP", "NR", // 용언 (Predicates)
            "VV", "VA", "VX", "VCP", "VCN", // 관형사 (Determiners)
            "MM",  // 부사 (Adverbs)
            "MAG", "MAJ", // 감탄사 (Interjections)
            "IC",  // 조사 (Particles)
            "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC",
            // 선어말어미 (Pre-final endings)
            "EP", // 어말어미 (Final endings)
            "EF", "EC", "ETN", "ETM", // 접두사 (Prefixes)
            "XPN", // 접미사 (Suffixes)
            "XSN", "XSV", "XSA", // 어근 (Roots)
            "XR",  // 부호 (Symbols)
            "SF", "SE", "SSO", "SSC", "SC", "SY", // 외국어 (Foreign words)
            "SL", // 한자 (Chinese characters)
            "SH", // 숫자 (Numbers)
            "SN", // 기타 (Others)
            "UNA", "NNBC", "NA", "NV", "NF",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect()
    }

    /// Validates a POS tag.
    #[must_use]
    pub fn is_valid_tag(&self, tag: &str) -> bool {
        if self.valid_tags.is_empty() {
            return true;
        }

        // Handle compound tags (e.g., "NNG+JKS")
        if tag.contains(self.tag_separator) {
            let parts: Vec<&str> = tag.split(self.tag_separator).collect();

            if self.validate_hierarchy && parts.len() > self.max_tag_depth {
                return false;
            }

            parts.iter().all(|part| self.valid_tags.contains(*part))
        } else {
            self.valid_tags.contains(tag)
        }
    }
}

/// Rules for cost validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRules {
    /// Valid range for left context ID
    pub left_context_range: RangeInclusive<i32>,
    /// Valid range for right context ID
    pub right_context_range: RangeInclusive<i32>,
    /// Valid range for word cost
    pub word_cost_range: RangeInclusive<i32>,
    /// Whether to warn on unusual costs
    pub warn_unusual_costs: bool,
    /// Threshold for unusual high costs
    pub unusual_high_cost: i32,
    /// Threshold for unusual low costs
    pub unusual_low_cost: i32,
}

impl Default for CostRules {
    fn default() -> Self {
        Self {
            left_context_range: 0..=10000,
            right_context_range: 0..=10000,
            word_cost_range: -10000..=10000,
            warn_unusual_costs: true,
            unusual_high_cost: 8000,
            unusual_low_cost: -8000,
        }
    }
}

impl CostRules {
    /// Validates costs for a dictionary entry.
    #[must_use]
    pub fn validate_costs(&self, left_id: i32, right_id: i32, cost: i32) -> CostValidationResult {
        let mut result = CostValidationResult::default();

        if !self.left_context_range.contains(&left_id) {
            result.errors.push(format!(
                "Left context ID {left_id} is outside valid range {:?}",
                self.left_context_range
            ));
        }

        if !self.right_context_range.contains(&right_id) {
            result.errors.push(format!(
                "Right context ID {right_id} is outside valid range {:?}",
                self.right_context_range
            ));
        }

        if !self.word_cost_range.contains(&cost) {
            result.errors.push(format!(
                "Word cost {cost} is outside valid range {:?}",
                self.word_cost_range
            ));
        }

        if self.warn_unusual_costs {
            if cost > self.unusual_high_cost {
                result.warnings.push(format!(
                    "Word cost {cost} is unusually high (threshold: {})",
                    self.unusual_high_cost
                ));
            } else if cost < self.unusual_low_cost {
                result.warnings.push(format!(
                    "Word cost {cost} is unusually low (threshold: {})",
                    self.unusual_low_cost
                ));
            }
        }

        result
    }
}

/// Result of cost validation.
#[derive(Debug, Default, Clone)]
pub struct CostValidationResult {
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl CostValidationResult {
    /// Returns whether the validation passed (no errors).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns whether there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Rules for encoding validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingRules {
    /// Expected encoding (e.g., "UTF-8")
    pub expected_encoding: String,
    /// Whether to validate UTF-8 correctness
    pub validate_utf8: bool,
    /// Whether to detect and report encoding issues
    pub detect_encoding_issues: bool,
    /// Whether to allow BOM (Byte Order Mark)
    pub allow_bom: bool,
}

impl Default for EncodingRules {
    fn default() -> Self {
        Self {
            expected_encoding: "UTF-8".to_string(),
            validate_utf8: true,
            detect_encoding_issues: true,
            allow_bom: false,
        }
    }
}

/// Rules for duplicate entry detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateRules {
    /// Whether to detect exact duplicates
    pub detect_exact_duplicates: bool,
    /// Whether to detect semantic duplicates (same surface + POS)
    pub detect_semantic_duplicates: bool,
    /// Whether to allow duplicates with different costs
    pub allow_cost_variants: bool,
}

impl Default for DuplicateRules {
    fn default() -> Self {
        Self {
            detect_exact_duplicates: true,
            detect_semantic_duplicates: true,
            allow_cost_variants: true,
        }
    }
}

/// Rules for surface form normalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationRules {
    /// Whether to check for Unicode normalization (NFC/NFD)
    pub check_unicode_normalization: bool,
    /// Preferred Unicode normalization form
    pub preferred_normalization: NormalizationForm,
    /// Whether to check for full-width/half-width consistency
    pub check_width_consistency: bool,
    /// Whether to check for Hangul jamo composition
    pub check_hangul_composition: bool,
    /// Whether to warn on whitespace in surface forms
    pub warn_on_whitespace: bool,
}

impl Default for NormalizationRules {
    fn default() -> Self {
        Self {
            check_unicode_normalization: true,
            preferred_normalization: NormalizationForm::Nfc,
            check_width_consistency: true,
            check_hangul_composition: true,
            warn_on_whitespace: true,
        }
    }
}

/// Unicode normalization forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormalizationForm {
    /// Normalization Form C (Canonical Composition)
    Nfc,
    /// Normalization Form D (Canonical Decomposition)
    Nfd,
    /// Normalization Form KC (Compatibility Composition)
    Nfkc,
    /// Normalization Form KD (Compatibility Decomposition)
    Nfkd,
}

impl NormalizationForm {
    /// Normalizes a string according to this form.
    #[must_use]
    pub fn normalize(&self, text: &str) -> String {
        use unicode_normalization::UnicodeNormalization;

        match self {
            Self::Nfc => text.nfc().collect(),
            Self::Nfd => text.nfd().collect(),
            Self::Nfkc => text.nfkc().collect(),
            Self::Nfkd => text.nfkd().collect(),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::field_reassign_with_default
)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ValidationConfig::default();
        assert_eq!(config.csv_rules.expected_field_count, 13);
        assert!(config.pos_rules.validate_hierarchy);
        assert!(config.encoding_rules.validate_utf8);
    }

    #[test]
    fn test_pos_tag_validation() {
        let rules = PosRules::default();

        // Valid single tags
        assert!(rules.is_valid_tag("NNG"));
        assert!(rules.is_valid_tag("VV"));
        assert!(rules.is_valid_tag("JKS"));

        // Valid compound tags
        assert!(rules.is_valid_tag("NNG+JKS"));
        assert!(rules.is_valid_tag("VV+EC"));

        // Invalid tags
        assert!(!rules.is_valid_tag("XXX"));
        assert!(!rules.is_valid_tag("NNG+XXX"));
    }

    #[test]
    fn test_cost_validation() {
        let rules = CostRules::default();

        // Valid costs
        let result = rules.validate_costs(100, 200, 500);
        assert!(result.is_valid());
        assert!(!result.has_warnings());

        // Invalid left context
        let result = rules.validate_costs(-1, 200, 500);
        assert!(!result.is_valid());

        // Unusual high cost (warning only)
        let result = rules.validate_costs(100, 200, 9000);
        assert!(result.is_valid());
        assert!(result.has_warnings());
    }

    #[test]
    fn test_normalization_form() {
        let nfc = NormalizationForm::Nfc;

        // Test Hangul normalization
        let composed = "한글";
        let normalized = nfc.normalize(composed);
        assert_eq!(composed, normalized);

        // Test that decomposed Hangul gets normalized to NFC
        let decomposed = "\u{1112}\u{1161}\u{11AB}\u{1100}\u{1173}\u{11AF}"; // 한글 (NFD)
        let normalized = nfc.normalize(decomposed);
        assert_eq!("한글", normalized);
    }

    #[test]
    fn test_max_tag_depth() {
        let mut rules = PosRules::default();
        rules.max_tag_depth = 2;

        assert!(rules.is_valid_tag("NNG+JKS"));
        assert!(!rules.is_valid_tag("NNG+JKS+EC"));
    }

    #[test]
    fn test_cost_ranges() {
        let rules = CostRules {
            left_context_range: 0..=100,
            right_context_range: 0..=100,
            word_cost_range: -1000..=1000,
            warn_unusual_costs: false,
            unusual_high_cost: 800,
            unusual_low_cost: -800,
        };

        let result = rules.validate_costs(50, 75, 500);
        assert!(result.is_valid());

        let result = rules.validate_costs(150, 75, 500);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_empty_valid_tags() {
        let mut rules = PosRules::default();
        rules.valid_tags.clear();

        // With empty valid_tags, any tag should be valid
        assert!(rules.is_valid_tag("ANYTHING"));
        assert!(rules.is_valid_tag("XXX+YYY"));
    }
}
