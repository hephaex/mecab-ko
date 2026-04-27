//! Nori compatibility integration tests for the `mecab-ko` facade crate.
//!
//! Full nori compatibility tests (tag conversion, decompound modes, analyzer,
//! tokenizer, WordType) live in:
//!   `mecab-ko-core/tests/nori_compat_integration.rs`
//!
//! This file verifies that the public re-exports from the `mecab-ko` crate
//! are accessible, so users of the top-level crate can import nori types
//! without reaching into `mecab_ko_core` directly.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::no_effect_underscore_binding
)]

#[test]
fn test_nori_types_reexported_via_core() {
    // Verify that the types used in nori_compat are importable through mecab_ko_core
    // (the facade crate does not re-export nori_compat yet; this test ensures the
    // core crate exposes the necessary symbols for downstream integration).
    use mecab_ko_core::nori_compat::{
        mecab_to_nori_tag, nori_to_mecab_tag, DecompoundMode, NoriAnalyzer, NoriTokenizer, WordType,
    };

    // Confirm tag conversion functions are callable
    assert_eq!(mecab_to_nori_tag("NNG"), "NNG");
    assert_eq!(nori_to_mecab_tag("J"), "JX");

    // Confirm DecompoundMode variants exist
    let _mode = DecompoundMode::None;
    let _mode = DecompoundMode::Discard;
    let _mode = DecompoundMode::Mixed;

    // Confirm WordType variants exist
    let _wt = WordType::Known;

    // Confirm NoriTokenizer and NoriAnalyzer are constructable
    let tokenizer = NoriTokenizer::new(DecompoundMode::None, false);
    assert!(tokenizer.is_ok(), "NoriTokenizer::new should succeed");

    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None);
    assert!(
        analyzer.is_ok(),
        "NoriAnalyzer::default_with_decompound should succeed"
    );
}

#[test]
fn test_nori_tag_conversions() {
    use mecab_ko_core::nori_compat::{mecab_to_nori_tag, nori_to_mecab_tag};

    // Particle family collapses to "J"
    assert_eq!(mecab_to_nori_tag("JKS"), "J");
    assert_eq!(mecab_to_nori_tag("JX"), "J");

    // Ending family collapses to "E"
    assert_eq!(mecab_to_nori_tag("EF"), "E");
    assert_eq!(mecab_to_nori_tag("EC"), "E");

    // Nouns pass through unchanged
    assert_eq!(mecab_to_nori_tag("NNG"), "NNG");
    assert_eq!(mecab_to_nori_tag("NNP"), "NNP");

    // Reverse mapping
    assert_eq!(nori_to_mecab_tag("NNG"), "NNG");
    assert_eq!(nori_to_mecab_tag("J"), "JX");
}

#[test]
fn test_decompound_modes() {
    use mecab_ko_core::nori_compat::{DecompoundMode, NoriTokenizer};

    for mode in [
        DecompoundMode::None,
        DecompoundMode::Discard,
        DecompoundMode::Mixed,
    ] {
        let result = NoriTokenizer::new(mode, false);
        assert!(
            result.is_ok(),
            "NoriTokenizer should be created for mode {:?}",
            mode.as_str()
        );
    }
}

#[test]
fn test_nori_analyzer_stoptags() {
    use mecab_ko_core::nori_compat::{DecompoundMode, NoriAnalyzer};

    let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None)
        .expect("Failed to create NoriAnalyzer");

    // Default stoptags include particle and ending families
    assert!(analyzer.stoptags().contains(&"J"));
    assert!(analyzer.stoptags().contains(&"E"));

    // Stoptag mutation
    analyzer.add_stoptag("SN".to_string());
    assert!(analyzer.stoptags().contains(&"SN"));

    assert!(analyzer.remove_stoptag("SN"));
    assert!(!analyzer.stoptags().contains(&"SN"));
}
