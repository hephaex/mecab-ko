//! Nori compatibility integration tests
//!
//! This module tests compatibility with Elasticsearch's Nori analyzer:
//! - POS tag mapping (MeCab <-> Nori)
//! - Decompound modes (none, discard, mixed)
//! - Token type classification
//! - Output format compatibility
//!
//! Note: These tests require the `nori_compat` feature to be fully integrated.
//! Currently placeholders until API is complete.

#![allow(
    clippy::expect_used,
    clippy::assertions_on_constants,
    clippy::doc_markdown
)]

mod common;

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_nori_compatibility_placeholder() {
    println!("Nori compatibility tests pending API integration");
}

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_nori_module_exists() {
    assert!(true, "nori_compat module should be available");
}

// ============================================================================
// Future tests - to be enabled when API is exposed
// ============================================================================

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_mecab_to_nori_tag_conversion() {
    // TODO: Enable when mecab_to_nori_tag is exported
}

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_nori_to_mecab_tag_conversion() {
    // TODO: Enable when nori_to_mecab_tag is exported
}

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_decompound_mode_none() {
    // TODO: Enable when DecompoundMode is exported
}

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_decompound_mode_discard() {
    // TODO: Enable when DecompoundMode is exported
}

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_decompound_mode_mixed() {
    // TODO: Enable when DecompoundMode is exported
}

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_nori_analyzer_basic() {
    // TODO: Enable when NoriAnalyzer is exported
}

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_nori_token_structure() {
    // TODO: Enable when NoriToken is exported
}

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_word_type_classification() {
    // TODO: Enable when WordType is exported
}

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_compound_noun_analysis() {
    // TODO: Enable when full analysis is available
}

#[test]
#[ignore = "placeholder: not yet implemented"]
fn test_elasticsearch_format_compatibility() {
    // TODO: Enable when JSON serialization is available
}
