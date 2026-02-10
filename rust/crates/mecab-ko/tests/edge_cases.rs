//! Edge Case Tests for MeCab-Ko
//!
//! This module tests boundary conditions, error handling, and unusual inputs:
//! - Empty and whitespace-only inputs
//! - Very long texts
//! - Special characters and emojis
//! - Malformed or unusual inputs
//! - Error conditions and recovery
//!
//! These tests ensure robustness and graceful degradation.

mod common;

/// Test empty string input
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_empty_string() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let tokens = tokenizer.tokenize("");

    assert!(tokens.is_empty(), "Empty string should produce no tokens");

    // Should be safe to call multiple times
    let tokens2 = tokenizer.tokenize("");
    assert!(tokens2.is_empty());
}

/// Test whitespace-only inputs
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_whitespace_only() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let whitespace_inputs = vec![
        " ",
        "  ",
        "   ",
        "\t",
        "\n",
        "\r",
        "\r\n",
        "   \t\n  ",
        "\t\t\t",
        "\n\n\n",
    ];

    for input in whitespace_inputs {
        let tokens = tokenizer.tokenize(input);

        // Whitespace-only input should produce no meaningful tokens
        // (implementation may strip whitespace)
        println!("Input: {input:?}");
        println!(
            "Tokens: {:?}",
            tokens.iter().map(|t| &t.surface).collect::<Vec<_>>()
        );
        println!("Token count: {}", tokens.len());
        println!();
    }
}

/// Test single character inputs of various types
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_single_characters() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        // Korean syllables
        ("가", "Korean syllable"),
        ("힣", "Korean syllable (last)"),
        // Jamo
        ("ㄱ", "Korean jamo (initial)"),
        ("ㅏ", "Korean jamo (medial)"),
        ("ㄴ", "Korean jamo (final)"),
        // Latin
        ("a", "Latin lowercase"),
        ("A", "Latin uppercase"),
        ("Z", "Latin uppercase"),
        // Numbers
        ("0", "Digit zero"),
        ("9", "Digit nine"),
        // Symbols
        ("!", "Exclamation"),
        ("?", "Question"),
        (".", "Period"),
        (",", "Comma"),
        ("-", "Hyphen"),
        ("_", "Underscore"),
        ("@", "At sign"),
        ("#", "Hash"),
        ("$", "Dollar"),
        ("%", "Percent"),
        ("&", "Ampersand"),
        ("*", "Asterisk"),
        ("+", "Plus"),
        ("=", "Equal"),
    ];

    for (input, description) in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: '{input}' ({description})");
        if tokens.is_empty() {
            println!("  No tokens produced");
        } else {
            for token in &tokens {
                println!("  Token: '{}' [{}]", token.surface, token.pos);
            }
        }
        println!();
    }
}

/// Test very long single word
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_very_long_word() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Generate a very long word
    let long_word = "가".repeat(1000);
    let tokens = tokenizer.tokenize(&long_word);

    assert!(!tokens.is_empty(), "Very long word should be tokenized");

    println!("Long word length: {} chars", long_word.chars().count());
    println!("Token count: {}", tokens.len());
}

/// Test very long text (stress test)
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_very_long_text() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Generate very long text (10000+ characters)
    let sentence = "한국어 형태소 분석은 자연어 처리의 중요한 부분입니다. ";
    let very_long_text = sentence.repeat(200);

    println!("Text length: {} chars", very_long_text.chars().count());

    let tokens = tokenizer.tokenize(&very_long_text);

    assert!(!tokens.is_empty(), "Very long text should be tokenized");

    // Verify position integrity
    for token in &tokens {
        assert!(
            token.start_pos < token.end_pos,
            "Token positions should be valid"
        );
        assert!(
            token.end_pos <= very_long_text.chars().count(),
            "Token end position should not exceed text length"
        );
    }

    println!("Token count: {}", tokens.len());
    println!("Average tokens per sentence: {}", tokens.len() / 200);
}

/// Test emoji characters
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_emoji() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "안녕하세요😀",
        "😀😁😂",
        "오늘은🌞날씨가☁️좋아요",
        "❤️💙💚",
        "👍👏🎉",
        "🇰🇷한국🇰🇷",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Token count: {}", tokens.len());
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test mixed scripts (Korean + English + Chinese + Japanese)
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_mixed_scripts() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "한글English混合",
        "Hello世界안녕",
        "日本語と韓国語",
        "中文한글English",
        "αβγ가나다abc",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test URLs and email addresses
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_urls_emails() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "웹사이트는 https://example.com입니다",
        "이메일: user@example.com",
        "GitHub: https://github.com/user/repo",
        "http://www.naver.com에서 검색하세요",
        "문의: contact@company.co.kr",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test file paths and code-like strings
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_paths_code() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "파일경로: /usr/local/bin/mecab",
        "윈도우경로: C:\\Program Files\\App",
        "코드: fn main() { }",
        "변수명: user_name",
        "함수: get_user_data()",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test repeated characters
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_repeated_characters() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "ㅋㅋㅋㅋㅋ",
        "ㅎㅎㅎ",
        "아아아아아",
        "!!!!!!",
        "??????",
        "......",
        "-----",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Token count: {}", tokens.len());
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test mathematical expressions
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_math_expressions() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "1 + 2 = 3",
        "10 × 5 = 50",
        "100 ÷ 4 = 25",
        "√16 = 4",
        "x² + y² = z²",
        "α + β = γ",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test currency and units
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_currency_units() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "가격: ₩10,000",
        "달러: $100",
        "유로: €50",
        "무게: 5kg",
        "거리: 100m",
        "온도: 25℃",
        "퍼센트: 50%",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test HTML/XML-like text
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_html_xml() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "<div>안녕하세요</div>",
        "<p class=\"text\">한글</p>",
        "<!-- 주석 -->",
        "<?xml version=\"1.0\"?>",
        "<tag attr=\"value\">내용</tag>",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test markdown-like text
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_markdown() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "# 제목",
        "## 부제목",
        "**굵게**",
        "*기울임*",
        "[링크](https://example.com)",
        "`코드`",
        "- 목록 항목",
        "1. 번호 목록",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test control characters and invisible characters
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_control_characters() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "안녕\u{0000}하세요",   // NULL character
        "테스트\u{200B}입니다", // Zero-width space
        "문장\u{FEFF}시작",     // BOM
        "줄\n바꿈",
        "탭\t문자",
        "캐리지\r리턴",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input:?}");
        println!("Token count: {}", tokens.len());
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test consecutive tokenizations with different inputs
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_rapid_context_switching() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Rapidly switch between very different input types
    let long_text = "매우 긴 문장을 여러 번 반복합니다. ".repeat(10);
    let test_sequence = [
        "",
        "안녕하세요",
        "",
        "123",
        "Hello",
        "가",
        "",
        long_text.as_str(),
        "😀",
        "",
    ];

    for (i, input) in test_sequence.iter().enumerate() {
        let tokens = tokenizer.tokenize(input);
        println!(
            "Iteration {}: input_len={}, token_count={}",
            i,
            input.chars().count(),
            tokens.len()
        );
    }
}

/// Test tokenization state isolation
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_state_isolation() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Tokenize first input
    let input1 = "첫번째 문장";
    let tokens1 = tokenizer.tokenize(input1);

    // Tokenize second input
    let input2 = "두번째 문장";
    let tokens2 = tokenizer.tokenize(input2);

    // Re-tokenize first input - should get identical results
    let tokens1_again = tokenizer.tokenize(input1);

    assert_eq!(
        tokens1, tokens1_again,
        "Tokenization should not be affected by previous calls"
    );

    // Verify tokens belong to correct inputs
    for token in &tokens1 {
        assert!(
            token.end_pos <= input1.chars().count(),
            "Token from input1 should have valid positions for input1"
        );
    }

    for token in &tokens2 {
        assert!(
            token.end_pos <= input2.chars().count(),
            "Token from input2 should have valid positions for input2"
        );
    }
}

/// Test special Unicode categories
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_unicode_categories() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        ("한글", "Hangul syllables"),
        ("ㄱㄴㄷ", "Hangul jamo"),
        ("𐍈𐍊𐍐", "Gothic"),
        ("𓀀𓀁𓀂", "Egyptian hieroglyphs"),
        ("①②③", "Circled numbers"),
        ("Ⅰ Ⅱ Ⅲ", "Roman numerals"),
        ("♠♣♥♦", "Card suits"),
        ("←→↑↓", "Arrows"),
        ("⚠️⛔️", "Warning signs"),
    ];

    for (input, description) in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input} ({description})");
        println!("Token count: {}", tokens.len());
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test zero-width characters
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_zero_width_characters() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        ("안녕\u{200B}하세요", "Zero-width space"),
        ("테스트\u{200C}입니다", "Zero-width non-joiner"),
        ("문장\u{200D}연결", "Zero-width joiner"),
        ("숨김\u{FEFF}문자", "Zero-width no-break space (BOM)"),
    ];

    for (input, description) in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input:?} ({description})");
        println!("Token count: {}", tokens.len());
        for token in &tokens {
            println!(
                "  '{}' [{}] pos={}..{}",
                token.surface, token.pos, token.start_pos, token.end_pos
            );
        }
        println!();
    }
}

/// Test normalization forms (NFD, NFC, NFKC, NFKD)
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_unicode_normalization_forms() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Same text in different Unicode normalization forms
    let nfc = "한글"; // NFC (precomposed)
    let nfd = "ㅎㅏㄴㄱㅡㄹ"; // NFD-like (decomposed jamo)

    let tokens_nfc = tokenizer.tokenize(nfc);
    let tokens_nfd = tokenizer.tokenize(nfd);

    println!("NFC form: {nfc:?}");
    println!(
        "Tokens: {:?}",
        tokens_nfc.iter().map(|t| &t.surface).collect::<Vec<_>>()
    );
    println!();

    println!("NFD-like form: {nfd:?}");
    println!(
        "Tokens: {:?}",
        tokens_nfd.iter().map(|t| &t.surface).collect::<Vec<_>>()
    );
    println!();
}

/// Test various quote types
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_quotes() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "\"인용문\"",
        "'작은따옴표'",
        "\u{201C}왼쪽 오른쪽\u{201D}",
        "\u{2018}왼쪽 오른쪽\u{2019}",
        "「전각 괄호」",
        "『이중 전각』",
        "‹단일 각괄호›",
        "«이중 각괄호»",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test various bracket types
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_brackets() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "(소괄호)",
        "[대괄호]",
        "{중괄호}",
        "⟨부등호⟩",
        "【전각 대괄호】",
        "『이중 전각』",
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input}");
        println!("Tokens:");
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}

/// Test ligatures and combined characters
#[test]
#[ignore = "Requires system dictionary"]
fn test_edge_ligatures() {
    use mecab_ko::Tokenizer;

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let test_cases = vec![
        "ﬁ", // fi ligature
        "ﬂ", // fl ligature
        "ﬀ", // ff ligature
        "ﬃ", // ffi ligature
        "ﬄ", // ffl ligature
    ];

    for input in test_cases {
        let tokens = tokenizer.tokenize(input);

        println!("Input: {input:?}");
        println!("Token count: {}", tokens.len());
        for token in &tokens {
            println!("  '{}' [{}]", token.surface, token.pos);
        }
        println!();
    }
}
