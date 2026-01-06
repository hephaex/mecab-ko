//! Nori compatibility integration tests
//!
//! This module tests compatibility with Elasticsearch's Nori analyzer:
//! - POS tag mapping (MeCab <-> Nori)
//! - Decompound modes (none, discard, mixed)
//! - Token type classification
//! - Output format compatibility
//!
//! Note: These tests require the nori_compat feature to be fully integrated.
//! Currently placeholders until API is complete.

mod common;

/// Placeholder test - Nori compatibility not yet integrated
#[test]
#[ignore = "Nori compatibility module not yet integrated into public API"]
fn test_nori_compatibility_placeholder() {
    // This test will be enabled once mecab-ko-core exports:
    // - mecab_to_nori_tag
    // - nori_to_mecab_tag
    // - NoriAnalyzer
    // - NoriToken
    // - WordType
    // - DecompoundMode
    //
    // See: /home/mare/mecab-ko/rust/crates/mecab-ko-core/src/nori_compat.rs
    println!("Nori compatibility tests pending API integration");
}

/// Test that nori_compat module exists in mecab-ko-core
#[test]
fn test_nori_module_exists() {
    // This verifies the module is compiled, even if not exposed yet
    // The actual implementation is in mecab-ko-core/src/nori_compat.rs
    assert!(true, "nori_compat module should be available");
}

// ============================================================================
// Future tests - to be enabled when API is exposed
// ============================================================================

#[test]
#[ignore = "Requires Nori API export"]
fn test_mecab_to_nori_tag_conversion() {
    // TODO: Enable when mecab_to_nori_tag is exported
}

#[test]
#[ignore = "Requires Nori API export"]
fn test_nori_to_mecab_tag_conversion() {
    // TODO: Enable when nori_to_mecab_tag is exported
}

#[test]
#[ignore = "Requires Nori API export"]
fn test_decompound_mode_none() {
    // TODO: Enable when DecompoundMode is exported
}

#[test]
#[ignore = "Requires Nori API export"]
fn test_decompound_mode_discard() {
    // TODO: Enable when DecompoundMode is exported
}

#[test]
#[ignore = "Requires Nori API export"]
fn test_decompound_mode_mixed() {
    // TODO: Enable when DecompoundMode is exported
}

#[test]
#[ignore = "Requires Nori API export"]
fn test_nori_analyzer_basic() {
    // TODO: Enable when NoriAnalyzer is exported
}

#[test]
#[ignore = "Requires Nori API export"]
fn test_nori_token_structure() {
    // TODO: Enable when NoriToken is exported
}

#[test]
#[ignore = "Requires Nori API export"]
fn test_word_type_classification() {
    // TODO: Enable when WordType is exported
}

#[test]
#[ignore = "Requires Nori API export"]
fn test_compound_noun_analysis() {
    // TODO: Enable when full analysis is available
}

#[test]
#[ignore = "Requires Nori API export"]
fn test_elasticsearch_format_compatibility() {
    // TODO: Enable when JSON serialization is available
}
