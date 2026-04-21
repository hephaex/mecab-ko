//! Edge case tests for mecab-ko-core
//!
//! Tests for various edge cases and boundary conditions.
//! Note: These tests use mini-dict, so many words may not be recognized.
//! Tests verify that the tokenizer handles various inputs without panicking.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use mecab_ko_core::Tokenizer;

/// Helper to create tokenizer
fn create_tokenizer() -> Tokenizer {
    Tokenizer::new().expect("Failed to create tokenizer")
}

// ========================================
// Empty and Whitespace Tests
// ========================================

#[test]
fn test_empty_string() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("");
    assert!(tokens.is_empty());
}

#[test]
fn test_single_space() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize(" ");
    // Should handle gracefully (may or may not return token)
    println!("Single space tokens: {}", tokens.len());
}

#[test]
fn test_multiple_spaces() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("     ");
    // Multiple spaces should be handled gracefully
    println!("Multiple spaces tokens: {}", tokens.len());
}

#[test]
fn test_newline_only() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("\n");
    println!("Newline tokens: {}", tokens.len());
}

#[test]
fn test_mixed_whitespace() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("\t\n\r ");
    // Should handle mixed whitespace
    println!("Mixed whitespace tokens: {}", tokens.len());
}

// ========================================
// Single Character Tests
// ========================================

#[test]
fn test_single_hangul_char() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("가");
    // Mini-dict may not recognize single char, just verify no panic
    println!("Single hangul char tokens: {}", tokens.len());
}

#[test]
fn test_single_number() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("1");
    println!("Single number tokens: {}", tokens.len());
}

#[test]
fn test_single_english_char() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("A");
    println!("Single English char tokens: {}", tokens.len());
}

#[test]
fn test_single_special_char() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("!");
    println!("Single special char tokens: {}", tokens.len());
}

// ========================================
// Unicode Edge Cases
// ========================================

#[test]
fn test_emoji() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("😀");
    // Should not panic
    println!("Emoji tokens: {}", tokens.len());
}

#[test]
fn test_emoji_with_text() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("안녕😀하세요");
    // Should tokenize around emoji without panic
    println!("Emoji with text tokens: {}", tokens.len());
}

#[test]
fn test_hangul_jamo_isolated() {
    let mut tokenizer = create_tokenizer();
    // Isolated jamo (consonants/vowels)
    let tokens = tokenizer.tokenize("ㄱㄴㄷ");
    println!("Jamo tokens: {}", tokens.len());
}

#[test]
fn test_hangul_compatibility_jamo() {
    let mut tokenizer = create_tokenizer();
    // Hangul compatibility jamo
    let tokens = tokenizer.tokenize("ㅏㅓㅗ");
    println!("Compatibility jamo tokens: {}", tokens.len());
}

#[test]
fn test_cjk_characters() {
    let mut tokenizer = create_tokenizer();
    // Chinese characters
    let tokens = tokenizer.tokenize("漢字");
    println!("CJK tokens: {}", tokens.len());
}

#[test]
fn test_japanese_hiragana() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("あいうえお");
    println!("Hiragana tokens: {}", tokens.len());
}

#[test]
fn test_zero_width_chars() {
    let mut tokenizer = create_tokenizer();
    // Zero-width non-joiner and joiner
    let tokens = tokenizer.tokenize("\u{200B}\u{200C}\u{200D}");
    // Should handle gracefully
    println!("Zero-width tokens: {}", tokens.len());
}

// ========================================
// Long Text Tests
// ========================================

#[test]
fn test_very_long_word() {
    let mut tokenizer = create_tokenizer();
    // Create a very long repeated word
    let long_word = "가".repeat(1000);
    let tokens = tokenizer.tokenize(&long_word);
    // Should not panic
    println!("Long word tokens: {}", tokens.len());
}

#[test]
fn test_very_long_text() {
    let mut tokenizer = create_tokenizer();
    // Create a long text with repeated mini-dict words
    let long_text = "안녕 감사 ".repeat(100);
    let tokens = tokenizer.tokenize(&long_text);
    // Should handle without panic
    println!("Long text tokens: {}", tokens.len());
}

// ========================================
// Repeated Tokenization Tests
// ========================================

#[test]
fn test_repeated_tokenization_same_text() {
    let mut tokenizer = create_tokenizer();
    let text = "안녕하세요";

    let tokens1 = tokenizer.tokenize(text);
    let tokens2 = tokenizer.tokenize(text);
    let tokens3 = tokenizer.tokenize(text);

    // Results should be consistent
    assert_eq!(tokens1.len(), tokens2.len());
    assert_eq!(tokens2.len(), tokens3.len());

    for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
        assert_eq!(t1.surface, t2.surface);
        assert_eq!(t1.pos, t2.pos);
    }
}

#[test]
fn test_repeated_tokenization_different_texts() {
    let mut tokenizer = create_tokenizer();

    let tokens1 = tokenizer.tokenize("안녕하세요");
    let _tokens2 = tokenizer.tokenize("감사합니다");
    let tokens3 = tokenizer.tokenize("안녕하세요");

    // First and third should be same
    assert_eq!(tokens1.len(), tokens3.len());
    for (t1, t3) in tokens1.iter().zip(tokens3.iter()) {
        assert_eq!(t1.surface, t3.surface);
    }
}

// ========================================
// Boundary Condition Tests
// ========================================

#[test]
fn test_text_ending_with_punctuation() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("안녕하세요.");
    println!("Text with trailing punct tokens: {}", tokens.len());
}

#[test]
fn test_text_starting_with_punctuation() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize(".안녕하세요");
    println!("Text with leading punct tokens: {}", tokens.len());
}

#[test]
fn test_multiple_consecutive_punctuation() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("안녕!!!???...");
    println!("Multiple punct tokens: {}", tokens.len());
}

#[test]
fn test_mixed_scripts() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("Hello안녕世界こんにちは");
    println!("Mixed scripts tokens: {}", tokens.len());
}

// ========================================
// Number Edge Cases
// ========================================

#[test]
fn test_large_number() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("12345678901234567890");
    println!("Large number tokens: {}", tokens.len());
}

#[test]
fn test_decimal_number() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("3.14159");
    println!("Decimal number tokens: {}", tokens.len());
}

#[test]
fn test_negative_number() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("-123");
    println!("Negative number tokens: {}", tokens.len());
}

#[test]
fn test_korean_number_unit() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("100원");
    println!("Korean number unit tokens: {}", tokens.len());
}

#[test]
fn test_date_format() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("2024년 1월 1일");
    println!("Date format tokens: {}", tokens.len());
}

// ========================================
// API Method Tests
// ========================================

#[test]
fn test_wakati_empty() {
    let mut tokenizer = create_tokenizer();
    let result = tokenizer.wakati("");
    assert!(result.is_empty());
}

#[test]
fn test_morphs_empty() {
    let mut tokenizer = create_tokenizer();
    let result = tokenizer.morphs("");
    assert!(result.is_empty());
}

#[test]
fn test_pos_empty() {
    let mut tokenizer = create_tokenizer();
    let result = tokenizer.pos("");
    assert!(result.is_empty());
}

#[test]
fn test_nouns_empty() {
    let mut tokenizer = create_tokenizer();
    let result = tokenizer.nouns("");
    assert!(result.is_empty());
}

#[test]
fn test_wakati_basic() {
    let mut tokenizer = create_tokenizer();
    let result = tokenizer.wakati("안녕하세요");
    // May or may not return tokens depending on dictionary
    println!("Wakati result: {result:?}");
}

#[test]
fn test_morphs_basic() {
    let mut tokenizer = create_tokenizer();
    let result = tokenizer.morphs("안녕하세요");
    println!("Morphs result: {result:?}");
}

#[test]
fn test_pos_basic() {
    let mut tokenizer = create_tokenizer();
    let result = tokenizer.pos("안녕하세요");
    println!("POS result: {result:?}");
    // Each result should be (surface, pos) tuple
    for (surface, pos) in &result {
        assert!(!surface.is_empty());
        assert!(!pos.is_empty());
    }
}

// ========================================
// Memory and State Tests
// ========================================

#[test]
fn test_tokenizer_reuse_after_edge_case() {
    let mut tokenizer = create_tokenizer();

    // Normal tokenization
    let tokens1 = tokenizer.tokenize("안녕하세요");
    println!("First tokenization: {} tokens", tokens1.len());

    // Empty tokenization (edge case)
    let tokens2 = tokenizer.tokenize("");
    assert!(tokens2.is_empty());

    // Should still work after edge case
    let tokens3 = tokenizer.tokenize("감사합니다");
    println!("Third tokenization: {} tokens", tokens3.len());
}

#[test]
fn test_tokenizer_reuse_after_special_input() {
    let mut tokenizer = create_tokenizer();

    // Emoji input
    let _ = tokenizer.tokenize("😀");

    // Should still work normally
    let tokens = tokenizer.tokenize("안녕");
    println!("After emoji: {} tokens", tokens.len());
}

#[test]
fn test_lattice_stats_after_tokenization() {
    let mut tokenizer = create_tokenizer();

    tokenizer.tokenize("안녕하세요");
    let stats = tokenizer.lattice_stats();

    println!(
        "Lattice stats - total_nodes: {}, char_length: {}",
        stats.total_nodes, stats.char_length
    );
}

// ========================================
// Special Text Patterns
// ========================================

#[test]
fn test_url_pattern() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("https://example.com에서 확인하세요");
    println!("URL pattern tokens: {}", tokens.len());
}

#[test]
fn test_email_pattern() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("연락처는 test@example.com입니다");
    println!("Email pattern tokens: {}", tokens.len());
}

#[test]
fn test_hashtag_pattern() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("#해시태그 #테스트");
    println!("Hashtag tokens: {}", tokens.len());
}

#[test]
fn test_mention_pattern() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("@사용자님 안녕하세요");
    println!("Mention tokens: {}", tokens.len());
}

#[test]
fn test_bracket_patterns() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("(괄호) [대괄호] {중괄호}");
    println!("Bracket tokens: {}", tokens.len());
}

#[test]
fn test_quote_patterns() {
    let mut tokenizer = create_tokenizer();
    let tokens = tokenizer.tokenize("\"인용문\" '작은따옴표'");
    println!("Quote tokens: {}", tokens.len());
}

// ========================================
// Stress Tests
// ========================================

#[test]
fn test_many_consecutive_calls() {
    let mut tokenizer = create_tokenizer();

    for i in 0..100 {
        let text = format!("테스트 {i}");
        let _ = tokenizer.tokenize(&text);
    }

    // Should complete without panic or memory issues
    println!("Completed 100 consecutive calls");
}

#[test]
fn test_alternating_empty_and_text() {
    let mut tokenizer = create_tokenizer();

    for _ in 0..50 {
        let _ = tokenizer.tokenize("");
        let _ = tokenizer.tokenize("안녕");
    }

    println!("Completed alternating empty/text calls");
}
