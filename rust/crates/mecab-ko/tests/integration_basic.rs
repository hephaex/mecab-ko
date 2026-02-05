//! Basic integration tests for MeCab-Ko
//!
//! This module tests fundamental tokenization functionality including:
//! - Basic sentence tokenization
//! - Morpheme extraction
//! - POS tagging accuracy
//! - Edge cases handling

mod common;

use common::fixtures::SampleTextGenerator;
use common::{load_fixtures, MorphTestCase};

/// Test that basic greeting sentences are tokenized correctly
///
/// This is a placeholder test that demonstrates the expected structure.
/// It will be fully implemented once the tokenizer is available.
#[test]
fn test_basic_greetings() {
    // Load test cases
    let test_cases = load_fixtures("sample_texts.json").expect("Failed to load sample texts");

    // Filter for basic category
    let basic_cases: Vec<&MorphTestCase> = test_cases
        .iter()
        .filter(|tc| tc.category.as_deref() == Some("basic"))
        .collect();

    assert!(
        !basic_cases.is_empty(),
        "Should have basic test cases loaded"
    );

    // TODO: Once tokenizer is implemented, uncomment and implement:
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for test_case in basic_cases {
    //     let result = tokenizer.tokenize(&test_case.input);
    //     let morphs: Vec<String> = result.iter().map(|t| t.surface.clone()).collect();
    //
    //     let comparison = common::compare_morphs(&test_case.expected_morphs, &morphs);
    //     assert_test_result!(comparison, test_case);
    // }

    println!("Loaded {} basic test cases", basic_cases.len());
}

/// Test that empty and whitespace-only inputs are handled correctly
#[test]
fn test_empty_input() {
    let empty_inputs = vec!["", " ", "  ", "\t", "\n", "   \t\n  "];

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for input in empty_inputs {
    //     let result = tokenizer.tokenize(input);
    //     assert!(result.is_empty() || result.iter().all(|t| t.surface.trim().is_empty()),
    //             "Empty input should produce empty or whitespace-only tokens: '{}'", input);
    // }

    println!(
        "Empty input test cases prepared: {} cases",
        empty_inputs.len()
    );
}

/// Test single character inputs (Korean syllables, jamo, numbers, symbols)
#[test]
fn test_single_character_input() {
    let single_chars = vec![
        "가", "ㄱ", "ㅏ", "1", "a", "A", "!", "?", ".", ",", " ",
    ];

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for ch in single_chars {
    //     let result = tokenizer.tokenize(ch);
    //     // Should produce at least one token (unless it's whitespace only)
    //     if !ch.trim().is_empty() {
    //         assert!(!result.is_empty(), "Single character '{}' should produce tokens", ch);
    //     }
    // }

    println!(
        "Single character test cases prepared: {} cases",
        single_chars.len()
    );
}

/// Test basic Korean sentences with common patterns
#[test]
fn test_common_sentences() {
    let sentences = SampleTextGenerator::basic_sentences();

    assert!(!sentences.is_empty(), "Should have sample sentences");

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for sentence in sentences {
    //     let result = tokenizer.tokenize(&sentence);
    //     assert!(!result.is_empty(), "Sentence should produce tokens: '{}'", sentence);
    //
    //     // Verify that all tokens have valid POS tags
    //     for token in result {
    //         assert!(!token.pos.is_empty(), "Token should have POS tag: '{}'", token.surface);
    //     }
    // }

    println!("Common sentence test cases prepared: {} cases", sentences.len());
}

/// Test that morpheme boundaries are correctly identified
#[test]
fn test_morpheme_boundaries() {
    // Test cases with known morpheme boundaries
    let test_cases = vec![
        ("안녕하세요", vec!["안녕", "하", "세요"]),
        ("감사합니다", vec!["감사", "하", "ㅂ니다"]),
        ("좋은", vec!["좋", "은"]),
    ];

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for (input, expected_morphs) in test_cases {
    //     let result = tokenizer.tokenize(input);
    //     let actual_morphs: Vec<String> = result.iter().map(|t| t.surface.clone()).collect();
    //
    //     let comparison = common::compare_morphs(&expected_morphs, &actual_morphs);
    //     assert!(comparison.passed,
    //             "Morpheme boundaries failed for '{}': expected {:?}, got {:?}",
    //             input, expected_morphs, actual_morphs);
    // }

    println!(
        "Morpheme boundary test cases prepared: {} cases",
        test_cases.len()
    );
}

/// Test POS tagging accuracy for common word classes
#[test]
fn test_pos_tagging_accuracy() {
    // Test cases with expected POS tags
    let test_cases = vec![
        ("사람", vec![("사람", "NNG")]),        // General noun
        ("서울", vec![("서울", "NNP")]),        // Proper noun
        ("나", vec![("나", "NP")]),              // Pronoun
        ("가다", vec![("가", "VV"), ("다", "EF")]), // Verb
        ("예쁘다", vec![("예쁘", "VA"), ("다", "EF")]), // Adjective
    ];

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for (input, expected_pos) in test_cases {
    //     let result = tokenizer.tokenize(input);
    //     let actual_pos: Vec<(String, String)> = result
    //         .iter()
    //         .map(|t| (t.surface.clone(), t.pos.clone()))
    //         .collect();
    //
    //     let comparison = common::compare_pos_tags(&expected_pos, &actual_pos);
    //     assert!(comparison.passed,
    //             "POS tagging failed for '{}': expected {:?}, got {:?}",
    //             input, expected_pos, actual_pos);
    // }

    println!("POS tagging test cases prepared: {} cases", test_cases.len());
}

/// Test handling of particles (조사)
#[test]
fn test_particle_handling() {
    let test_cases = vec![
        ("나는", vec![("나", "NP"), ("는", "JX")]),
        ("책을", vec![("책", "NNG"), ("을", "JKO")]),
        ("학교에서", vec![("학교", "NNG"), ("에서", "JKB")]),
        ("친구와", vec![("친구", "NNG"), ("와", "JC")]),
    ];

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for (input, expected_pos) in test_cases {
    //     let result = tokenizer.tokenize(input);
    //     let actual_pos: Vec<(String, String)> = result
    //         .iter()
    //         .map(|t| (t.surface.clone(), t.pos.clone()))
    //         .collect();
    //
    //     assert_eq!(actual_pos, expected_pos,
    //                "Particle handling failed for '{}'", input);
    // }

    println!("Particle handling test cases prepared: {} cases", test_cases.len());
}

/// Test handling of verb conjugations
#[test]
fn test_verb_conjugations() {
    let test_cases = vec![
        ("먹었다", vec![("먹", "VV"), ("었", "EP"), ("다", "EF")]),
        ("갔습니다", vec![("가", "VV"), ("았", "EP"), ("습니다", "EF")]),
        ("하고", vec![("하", "VV"), ("고", "EC")]),
        ("먹어요", vec![("먹", "VV"), ("어요", "EF")]),
    ];

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for (input, expected_morphs) in test_cases {
    //     let result = tokenizer.tokenize(input);
    //     let actual_morphs: Vec<(String, String)> = result
    //         .iter()
    //         .map(|t| (t.surface.clone(), t.pos.clone()))
    //         .collect();
    //
    //     assert_eq!(actual_morphs, expected_morphs,
    //                "Verb conjugation handling failed for '{}'", input);
    // }

    println!(
        "Verb conjugation test cases prepared: {} cases",
        test_cases.len()
    );
}

/// Test that tokens have correct byte positions
#[test]
fn test_token_positions() {
    let input = "안녕하세요 반갑습니다";

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    // let result = tokenizer.tokenize(input);
    //
    // // Verify positions are valid and sequential
    // for i in 0..result.len() {
    //     let token = &result[i];
    //     assert!(token.start < token.end, "Token start must be less than end");
    //     assert!(token.end <= input.len(), "Token end must not exceed input length");
    //
    //     // Verify token surface matches the substring
    //     let surface_from_pos = &input[token.start..token.end];
    //     assert_eq!(token.surface, surface_from_pos,
    //                "Token surface must match substring at position");
    //
    //     // Verify tokens are sequential (no gaps or overlaps)
    //     if i > 0 {
    //         let prev_token = &result[i - 1];
    //         assert!(token.start >= prev_token.end,
    //                 "Tokens must not overlap");
    //     }
    // }

    println!("Token position test prepared for: '{input}'");
}

/// Test consistency: running the same input multiple times should give the same result
#[test]
fn test_tokenization_consistency() {
    let test_inputs = vec!["안녕하세요", "대한민국", "인공지능"];

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for input in test_inputs {
    //     let result1 = tokenizer.tokenize(input);
    //     let result2 = tokenizer.tokenize(input);
    //     let result3 = tokenizer.tokenize(input);
    //
    //     assert_eq!(result1, result2, "Results should be consistent");
    //     assert_eq!(result2, result3, "Results should be consistent");
    // }

    println!(
        "Consistency test cases prepared: {} cases",
        test_inputs.len()
    );
}

#[cfg(test)]
mod hangul_tests {

    /// Test mecab-ko-hangul crate integration
    #[test]
    fn test_hangul_decomposition_integration() {
        use mecab_ko_hangul::{compose, decompose, has_jongseong, is_hangul};

        // Test basic decomposition
        let (cho, jung, jong) = decompose('한').expect("Should decompose");
        assert_eq!(cho, 'ㅎ');
        assert_eq!(jung, 'ㅏ');
        assert_eq!(jong, Some('ㄴ'));

        // Test composition
        let composed = compose('ㅎ', 'ㅏ', Some('ㄴ')).expect("Should compose");
        assert_eq!(composed, '한');

        // Test hangul detection
        assert!(is_hangul('가'));
        assert!(is_hangul('힣'));
        assert!(!is_hangul('a'));
        assert!(!is_hangul('1'));

        // Test jongseong detection
        assert_eq!(has_jongseong('한'), Some(true));
        assert_eq!(has_jongseong('하'), Some(false));
        assert_eq!(has_jongseong('a'), None);
    }

    /// Test all hangul syllables have correct jongseong detection
    #[test]
    fn test_jongseong_detection_comprehensive() {
        use mecab_ko_hangul::has_jongseong;

        // Known syllables with jongseong
        let with_jong = vec!['각', '간', '갈', '감', '갑', '강', '한', '국'];
        for ch in with_jong {
            assert_eq!(
                has_jongseong(ch),
                Some(true),
                "'{ch}' should have jongseong"
            );
        }

        // Known syllables without jongseong
        let without_jong = vec!['가', '나', '다', '라', '마', '바', '사', '아'];
        for ch in without_jong {
            assert_eq!(
                has_jongseong(ch),
                Some(false),
                "'{ch}' should not have jongseong"
            );
        }
    }
}
