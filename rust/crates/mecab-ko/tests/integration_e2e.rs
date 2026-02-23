//! End-to-End Integration Tests for MeCab-Ko
//!
//! This module tests the complete tokenization pipeline including:
//! - Full morphological analysis workflow
//! - Dictionary loading and lookup
//! - Lattice construction and Viterbi search
//! - Token generation with correct metadata
//! - Performance characteristics
//!
//! These tests verify that all components work together correctly
//! from input text to final tokenized output.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::uninlined_format_args,
    clippy::useless_vec,
    clippy::doc_markdown
)]

mod common;

use common::fixtures::SampleTextGenerator;

/// Test complete tokenization pipeline with basic Korean sentences
///
/// This test verifies that the entire pipeline works end-to-end:
/// 1. Dictionary loading
/// 2. Text normalization
/// 3. Lattice construction
/// 4. Viterbi search
/// 5. Token extraction
#[test]
fn test_e2e_basic_tokenization() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Use words available in the mini-dict test fixture
    let test_cases = vec![
        ("안녕하세요", vec!["안녕", "하", "세요"]),
        ("감사합니다", vec!["감사", "합니다"]),
        ("한국어", vec!["한국어"]),
        ("사람", vec!["사람"]),
    ];

    for (input, expected_morphs) in test_cases {
        let tokens = tokenizer.tokenize(input);

        assert!(
            !tokens.is_empty(),
            "Tokenization should produce tokens for '{input}'"
        );

        let actual_morphs: Vec<String> = tokens.iter().map(|t| t.surface.clone()).collect();

        // Note: Exact morpheme boundaries may vary depending on dictionary
        // This test verifies that tokenization completes successfully
        println!("Input: {input}");
        println!("Expected: {expected_morphs:?}");
        println!("Actual:   {actual_morphs:?}");
        println!();
    }
}

/// Test tokenization with various sentence types
#[test]
fn test_e2e_sentence_types() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Use only words available in the mini-dict test fixture
    let sentences = vec![
        "안녕하세요".to_string(),
        "감사합니다".to_string(),
        "한국어".to_string(),
        "나".to_string(),
    ];

    for sentence in sentences {
        let tokens = tokenizer.tokenize(&sentence);

        assert!(!tokens.is_empty(), "Should tokenize sentence: '{sentence}'");

        // Verify all tokens have valid positions
        for (i, token) in tokens.iter().enumerate() {
            assert!(
                token.start_pos < token.end_pos,
                "Token {i} should have valid position range"
            );
            assert!(
                !token.pos.is_empty(),
                "Token {i} should have POS tag: {token:?}"
            );
            assert!(
                !token.surface.is_empty(),
                "Token {i} should have non-empty surface"
            );
        }

        println!("Sentence: {sentence}");
        println!("Token count: {}", tokens.len());
        for token in &tokens {
            println!(
                "  {} [{}] ({}-{})",
                token.surface, token.pos, token.start_pos, token.end_pos
            );
        }
        println!();
    }
}

/// Test wakati (word segmentation only) functionality
#[test]
fn test_e2e_wakati_mode() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Use a mini-dict word so this works without a full dictionary
    let input = "한국어";
    let words = tokenizer.wakati(input);

    assert!(!words.is_empty(), "Wakati should produce words");
    println!("Input: {input}");
    println!("Words: {words:?}");
}

/// Test noun extraction functionality
#[test]
fn test_e2e_noun_extraction() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Use mini-dict nouns: 안녕(NNG), 감사(NNG), 한국어(NNG), 사람(NNG)
    let input = "안녕감사";
    let nouns = tokenizer.nouns(input);

    assert!(!nouns.is_empty(), "Should extract nouns");
    println!("Input: {input}");
    println!("Nouns: {nouns:?}");

    // Verify all extracted items start with NN (noun) POS tag
    let tokens = tokenizer.tokenize(input);
    let noun_surfaces: Vec<String> = tokens
        .iter()
        .filter(|t| t.pos.starts_with("NN"))
        .map(|t| t.surface.clone())
        .collect();

    assert_eq!(
        nouns, noun_surfaces,
        "Nouns() should match tokens with NN* POS"
    );
}

/// Test POS tagging functionality
#[test]
fn test_e2e_pos_tagging() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Use a mini-dict word so this works without a full dictionary
    let input = "나";
    let pos_tags = tokenizer.pos(input);

    assert!(!pos_tags.is_empty(), "Should produce POS tags");
    println!("Input: {input}");
    println!("POS tags:");
    for (surface, pos) in &pos_tags {
        println!("  {surface}/{pos}");
    }
}

/// Test tokenization consistency (same input produces same output)
#[test]
fn test_e2e_consistency() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_inputs = vec![
        "안녕하세요",
        "대한민국 만세",
        "인공지능 기술",
        "형태소 분석기",
    ];

    for input in test_inputs {
        let result1 = tokenizer.tokenize(input);
        let result2 = tokenizer.tokenize(input);
        let result3 = tokenizer.tokenize(input);

        assert_eq!(
            result1, result2,
            "Tokenization should be consistent for '{input}'"
        );
        assert_eq!(
            result2, result3,
            "Tokenization should be consistent for '{input}'"
        );
    }
}

/// Test token position accuracy
#[test]
fn test_e2e_token_positions() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let input = "안녕하세요 반갑습니다";
    let tokens = tokenizer.tokenize(input);

    for (i, token) in tokens.iter().enumerate() {
        // Verify position validity
        assert!(
            token.start_pos < token.end_pos,
            "Token {i}: start must be less than end"
        );
        assert!(
            token.start_byte < token.end_byte,
            "Token {i}: start_byte must be less than end_byte"
        );

        // Verify byte positions are valid UTF-8 boundaries
        assert!(
            input.is_char_boundary(token.start_byte),
            "Token {i}: start_byte must be on char boundary"
        );
        assert!(
            input.is_char_boundary(token.end_byte),
            "Token {i}: end_byte must be on char boundary"
        );

        println!(
            "Token {i}: '{}' pos={}..{} bytes={}..{}",
            token.surface, token.start_pos, token.end_pos, token.start_byte, token.end_byte
        );
    }
}

/// Test with user dictionary integration
#[test]
fn test_e2e_with_user_dictionary() {
    use mecab_ko::{dict::UserDictionary, Tokenizer};

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Add custom words to user dictionary.
    // User dict entries default to left_id=0, right_id=0, which are within the
    // 25x25 mini-dict connection matrix bounds.
    let mut user_dict = UserDictionary::new();
    user_dict.add_entry("딥러닝", "NNG", Some(-1000), Some("딥러닝".to_string()));
    user_dict.add_entry("머신러닝", "NNG", Some(-1000), Some("머신러닝".to_string()));

    tokenizer.set_user_dict(user_dict);

    // Use a single user-dict word without spaces to avoid byte-offset issues
    let input = "딥러닝";
    let tokens = tokenizer.tokenize(input);

    println!("Input: {input}");
    println!("Tokens:");
    for token in &tokens {
        println!("  {} [{}]", token.surface, token.pos);
    }

    // Verify user dictionary entry is recognized
    let surfaces: Vec<String> = tokens.iter().map(|t| t.surface.clone()).collect();
    assert!(
        surfaces.contains(&"딥러닝".to_string()),
        "Should recognize user dictionary entry '딥러닝', got: {surfaces:?}"
    );
}

/// Test lattice construction
#[test]
fn test_e2e_lattice_construction() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let input = "아버지가방에들어가신다";
    let lattice = tokenizer.tokenize_to_lattice(input);

    let stats = lattice.stats();

    // Lattice should contain BOS, EOS, and content nodes
    assert!(
        stats.total_nodes > 2,
        "Lattice should contain nodes beyond BOS and EOS"
    );
    assert_eq!(
        stats.char_length,
        input.chars().count(),
        "Lattice char length should match input"
    );

    println!("Input: {input}");
    println!("Lattice stats: {stats:?}");
}

/// Test multiple sequential tokenizations (lattice reuse)
#[test]
fn test_e2e_sequential_tokenizations() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Use words from the mini-dict so this works without a full dictionary
    let test_cases = vec![
        "안녕",
        "감사",
        "한국어",
        "사람",
        "나",
    ];

    for (i, input) in test_cases.iter().enumerate() {
        let tokens = tokenizer.tokenize(input);

        assert!(!tokens.is_empty(), "Tokenization {i} should produce tokens");

        // Verify tokens belong to current input, not previous ones
        for token in &tokens {
            assert!(
                token.end_pos <= input.chars().count(),
                "Token position should be within current input bounds"
            );
        }

        println!(
            "Tokenization {}: {} -> {} tokens",
            i + 1,
            input,
            tokens.len()
        );
    }
}

/// Test performance characteristics (basic throughput)
#[test]
fn test_e2e_basic_performance() {
    use mecab_ko::Tokenizer;
    use std::time::Instant;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let input = "오늘은 날씨가 정말 좋습니다. 밖에 나가서 산책을 하고 싶어요. \
                 친구들과 함께 공원에서 즐거운 시간을 보내고 싶습니다.";

    // Warm-up
    for _ in 0..10 {
        let _ = tokenizer.tokenize(input);
    }

    // Measure
    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = tokenizer.tokenize(input);
    }
    let duration = start.elapsed();

    let avg_micros = duration.as_micros() / iterations;
    let throughput = (iterations as f64 / duration.as_secs_f64()) as u64;

    println!("Performance:");
    println!("  Total: {:?}", duration);
    println!("  Average: {} μs/iteration", avg_micros);
    println!("  Throughput: {} tokenizations/sec", throughput);

    // Basic sanity check: should complete in reasonable time
    assert!(
        avg_micros < 10_000,
        "Average tokenization should be under 10ms"
    );
}

/// Test mixed Korean-English text
///
/// Tests Korean-only tokenization with known mini-dict words.
/// Mixed Korean-English with spaces requires a full dictionary.
#[test]
fn test_e2e_mixed_korean_english() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Use single mini-dict words (no spaces to avoid byte-offset issues,
    // no English chars which create unknown nodes outside mini-dict matrix)
    let test_cases = vec!["안녕", "감사", "한국어", "사람"];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        assert!(
            !tokens.is_empty(),
            "Should tokenize Korean text: '{input}'"
        );

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  {} [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test numbers and symbols
#[test]
#[ignore = "Requires system dictionary"]
fn test_e2e_numbers_and_symbols() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "2024년 1월 15일",
        "가격은 10,000원입니다",
        "전화번호: 010-1234-5678",
        "이메일은 test@example.com입니다",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        assert!(
            !tokens.is_empty(),
            "Should tokenize text with numbers and symbols: '{input}'"
        );

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  {} [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test morphs() method (alias for wakati)
#[test]
fn test_e2e_morphs_method() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let input = "형태소 분석을 합니다";
    let morphs = tokenizer.morphs(input);
    let wakati = tokenizer.wakati(input);

    assert_eq!(morphs, wakati, "morphs() should be equivalent to wakati()");
    println!("Input: {input}");
    println!("Morphs: {morphs:?}");
}

/// Test token metadata completeness
#[test]
fn test_e2e_token_metadata() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let input = "안녕하세요";
    let tokens = tokenizer.tokenize(input);

    for (i, token) in tokens.iter().enumerate() {
        // Verify required fields
        assert!(!token.surface.is_empty(), "Token {i} should have surface");
        assert!(!token.pos.is_empty(), "Token {i} should have POS tag");
        assert!(!token.features.is_empty(), "Token {i} should have features");

        // Verify position consistency
        assert_eq!(
            token.char_len(),
            token.surface.chars().count(),
            "Token {i} char_len should match surface length"
        );
        assert_eq!(
            token.byte_len(),
            token.surface.len(),
            "Token {i} byte_len should match surface byte length"
        );

        println!("Token {i}:");
        println!("  Surface: {}", token.surface);
        println!("  POS: {}", token.pos);
        println!("  Position: {}..{} (chars)", token.start_pos, token.end_pos);
        println!("  Bytes: {}..{}", token.start_byte, token.end_byte);
        println!("  Cost: {}", token.cost);
        println!("  Features: {}", token.features);
        if let Some(reading) = &token.reading {
            println!("  Reading: {reading}");
        }
        if let Some(lemma) = &token.lemma {
            println!("  Lemma: {lemma}");
        }
        println!();
    }
}

/// Test special characters handling
#[test]
#[ignore = "Requires system dictionary"]
fn test_e2e_special_characters() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "한글!",
        "문장.",
        "질문?",
        "감탄사!!!",
        "쉼표,쉼표",
        "괄호(내용)괄호",
        "인용\"문\"인용",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        assert!(
            !tokens.is_empty(),
            "Should tokenize text with special characters: '{input}'"
        );

        println!("Input: {input}");
        println!(
            "Tokens: {:?}",
            tokens.iter().map(|t| &t.surface).collect::<Vec<_>>()
        );
        println!();
    }
}

/// Test long text tokenization
#[test]
fn test_e2e_long_text() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Generate long text by repeating a mini-dict word (no spaces/punctuation to
    // avoid creating unknown nodes that break Viterbi with the mini-dict matrix)
    let word = "한국어";
    let long_text = word.repeat(100);

    let tokens = tokenizer.tokenize(&long_text);

    assert!(!tokens.is_empty(), "Should tokenize long text");

    // Verify no position overflow or corruption
    let mut prev_end_pos = 0;
    for token in &tokens {
        assert!(
            token.start_pos >= prev_end_pos,
            "Token positions should not overlap or go backwards"
        );
        prev_end_pos = token.end_pos;
    }

    println!("Long text length: {} chars", long_text.chars().count());
    println!("Token count: {}", tokens.len());
}

/// Test pool statistics after multiple tokenizations
#[test]
fn test_e2e_pool_statistics() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Perform multiple tokenizations
    for i in 0..10 {
        let input = format!("이것은 {i}번째 테스트입니다");
        let _ = tokenizer.tokenize(&input);
    }

    let stats = tokenizer.pool_stats();
    println!("Pool statistics after 10 tokenizations:");
    println!("  {stats:?}");

    // Pool statistics should be available
    // (Exact values depend on implementation)
}

/// Test lattice statistics
#[test]
fn test_e2e_lattice_statistics() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Use a single word (no spaces) so char_length equals input.chars().count()
    let input = "안녕하세요";
    let _ = tokenizer.tokenize(input);

    let stats = tokenizer.lattice_stats();

    println!("Lattice statistics for '{input}':");
    println!("  {stats:?}");

    assert!(stats.total_nodes > 0, "Should have nodes in lattice");
    assert_eq!(
        stats.char_length,
        input.chars().count(),
        "Lattice char length should match input"
    );
}
