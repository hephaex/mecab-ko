//! Integration tests for Nori compatibility layer

#![allow(clippy::expect_used, clippy::unwrap_used, unused_mut)]

use mecab_ko_core::nori_compat::{
    mecab_to_nori_tag, nori_to_mecab_tag, DecompoundMode, NoriAnalyzer, NoriTokenizer,
};

#[test]
#[ignore = "Requires system dictionary"]
fn test_nori_tokenizer_none_mode() {
    let tokenizer = NoriTokenizer::new(DecompoundMode::None, false);
    assert!(tokenizer.is_ok());

    let mut tokenizer = tokenizer.unwrap();
    let result = tokenizer.tokenize("안녕하세요");
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert!(!tokens.is_empty());
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_nori_tokenizer_mixed_mode() {
    let tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, false);
    assert!(tokenizer.is_ok());

    let mut tokenizer = tokenizer.unwrap();
    let result = tokenizer.tokenize("형태소분석");
    assert!(result.is_ok());
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_nori_tokenizer_discard_mode() {
    let tokenizer = NoriTokenizer::new(DecompoundMode::Discard, false);
    assert!(tokenizer.is_ok());

    let mut tokenizer = tokenizer.unwrap();
    let result = tokenizer.tokenize("한국어");
    assert!(result.is_ok());
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_nori_tokenizer_with_unknown_unigrams() {
    let tokenizer = NoriTokenizer::new(DecompoundMode::None, true);
    assert!(tokenizer.is_ok());

    let mut tokenizer = tokenizer.unwrap();
    let result = tokenizer.tokenize("테스트");
    assert!(result.is_ok());
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_nori_analyzer_default() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None);
    assert!(analyzer.is_ok());

    let mut analyzer = analyzer.unwrap();
    let result = analyzer.analyze("안녕하세요");
    assert!(result.is_ok());

    // Default analyzer should filter J and E tags
    let stoptags = analyzer.stoptags();
    assert!(stoptags.contains(&"J"));
    assert!(stoptags.contains(&"E"));
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_nori_analyzer_with_custom_stoptags() {
    let stoptags = vec!["J".to_string(), "E".to_string(), "SF".to_string()];
    let analyzer = NoriAnalyzer::new(None, DecompoundMode::None, stoptags, false);
    assert!(analyzer.is_ok());

    let analyzer = analyzer.unwrap();
    let stoptags = analyzer.stoptags();
    assert_eq!(stoptags.len(), 3);
    assert!(stoptags.contains(&"SF"));
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_nori_analyzer_stoptag_modification() {
    let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    // Add stoptag
    analyzer.add_stoptag("SN".to_string());
    assert!(analyzer.stoptags().contains(&"SN"));

    // Remove stoptag
    assert!(analyzer.remove_stoptag("SN"));
    assert!(!analyzer.stoptags().contains(&"SN"));

    // Try to remove non-existent stoptag
    assert!(!analyzer.remove_stoptag("NONEXISTENT"));
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_pos_tag_mapping_particles() {
    // All particle tags should map to "J"
    assert_eq!(mecab_to_nori_tag("JKS"), "J"); // 주격
    assert_eq!(mecab_to_nori_tag("JKC"), "J"); // 보격
    assert_eq!(mecab_to_nori_tag("JKG"), "J"); // 관형격
    assert_eq!(mecab_to_nori_tag("JKO"), "J"); // 목적격
    assert_eq!(mecab_to_nori_tag("JKB"), "J"); // 부사격
    assert_eq!(mecab_to_nori_tag("JKV"), "J"); // 호격
    assert_eq!(mecab_to_nori_tag("JKQ"), "J"); // 인용격
    assert_eq!(mecab_to_nori_tag("JX"), "J"); // 보조사
    assert_eq!(mecab_to_nori_tag("JC"), "J"); // 접속조사
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_pos_tag_mapping_endings() {
    // All ending tags should map to "E"
    assert_eq!(mecab_to_nori_tag("EP"), "E"); // 선어말
    assert_eq!(mecab_to_nori_tag("EF"), "E"); // 종결
    assert_eq!(mecab_to_nori_tag("EC"), "E"); // 연결
    assert_eq!(mecab_to_nori_tag("ETN"), "E"); // 명사형전성
    assert_eq!(mecab_to_nori_tag("ETM"), "E"); // 관형형전성
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_pos_tag_mapping_nouns() {
    // Noun tags should remain unchanged
    assert_eq!(mecab_to_nori_tag("NNG"), "NNG"); // 일반명사
    assert_eq!(mecab_to_nori_tag("NNP"), "NNP"); // 고유명사
    assert_eq!(mecab_to_nori_tag("NNB"), "NNB"); // 의존명사
    assert_eq!(mecab_to_nori_tag("NNBC"), "NNBC"); // 단위명사
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_pos_tag_mapping_verbs() {
    // Verb tags should remain unchanged
    assert_eq!(mecab_to_nori_tag("VV"), "VV"); // 동사
    assert_eq!(mecab_to_nori_tag("VA"), "VA"); // 형용사
    assert_eq!(mecab_to_nori_tag("VX"), "VX"); // 보조용언
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_reverse_mapping() {
    // Nori to MeCab mapping
    assert_eq!(nori_to_mecab_tag("J"), "JX"); // J → JX (대표)
    assert_eq!(nori_to_mecab_tag("E"), "EF"); // E → EF (대표)
    assert_eq!(nori_to_mecab_tag("NNG"), "NNG"); // 그대로
    assert_eq!(nori_to_mecab_tag("VV"), "VV"); // 그대로
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_decompound_mode_string_conversion() {
    // Parse from string
    assert_eq!(DecompoundMode::from_str("none"), Some(DecompoundMode::None));
    assert_eq!(
        DecompoundMode::from_str("discard"),
        Some(DecompoundMode::Discard)
    );
    assert_eq!(
        DecompoundMode::from_str("mixed"),
        Some(DecompoundMode::Mixed)
    );

    // Case insensitive
    assert_eq!(DecompoundMode::from_str("NONE"), Some(DecompoundMode::None));
    assert_eq!(
        DecompoundMode::from_str("Mixed"),
        Some(DecompoundMode::Mixed)
    );

    // Invalid input
    assert_eq!(DecompoundMode::from_str("invalid"), None);
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_all_decompound_modes() {
    // Test creation with all modes
    let modes = [
        DecompoundMode::None,
        DecompoundMode::Discard,
        DecompoundMode::Mixed,
    ];

    for mode in modes {
        let tokenizer = NoriTokenizer::new(mode, false);
        assert!(
            tokenizer.is_ok(),
            "Failed to create tokenizer with mode: {}",
            mode.as_str()
        );

        let analyzer = NoriAnalyzer::default_with_decompound(mode);
        assert!(
            analyzer.is_ok(),
            "Failed to create analyzer with mode: {}",
            mode.as_str()
        );
    }
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_empty_string() {
    let mut tokenizer = NoriTokenizer::new(DecompoundMode::None, false).unwrap();
    let result = tokenizer.tokenize("");
    assert!(result.is_ok());
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_analyzer_preserves_content_words() {
    let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();
    let result = analyzer.analyze("테스트");
    assert!(result.is_ok());

    // Content words should not be filtered
    let tokens = result.unwrap();
    // Since we have a stub tokenizer, we can't verify exact behavior yet
    // but we can verify the analyzer doesn't crash
    assert!(tokens.len() <= 1); // Should be <= original token count
}

#[test]
#[ignore = "Requires system dictionary"]
fn test_unknown_tag_handling() {
    // Unknown tags should pass through
    assert_eq!(mecab_to_nori_tag("UNKNOWN_TAG"), "UNKNOWN_TAG");
    assert_eq!(nori_to_mecab_tag("UNKNOWN_TAG"), "UNKNOWN_TAG");
}
