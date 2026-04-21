//! Compound Noun Decomposition Demo
//!
//! This example demonstrates the compound noun decomposition functionality
//! for Nori (Lucene Korean analyzer) compatibility.
//!
//! Run with: `cargo run --example compound_noun_demo`

#![allow(
    clippy::uninlined_format_args,
    clippy::expect_used,
    clippy::unwrap_used
)]

use mecab_ko_core::nori_compat::{DecompoundMode, NoriTokenizer};

fn main() {
    println!("=== Compound Noun Decomposition Demo ===\n");

    // Test cases demonstrating different compound noun patterns
    let test_cases = vec![
        ("형태소분석", "Basic compound: morpheme + analysis"),
        (
            "자연언어처리",
            "Three-part compound: natural + language + processing",
        ),
        ("대한민국", "Proper noun compound: Korea"),
        (
            "국립국어원",
            "Sino-Korean compound: National Institute of Korean Language",
        ),
        ("학교운동장", "Mixed jongseong: school + playground"),
        ("학생들", "Suffix pattern: student + plural marker"),
        ("신도시", "Prefix pattern: new + city"),
        (
            "형태소분석기",
            "Complex compound: morpheme + analysis + tool",
        ),
    ];

    // Create tokenizer with different decompound modes
    let modes = [
        (DecompoundMode::None, "None (no decomposition)"),
        (DecompoundMode::Discard, "Discard (only components)"),
        (DecompoundMode::Mixed, "Mixed (original + components)"),
    ];

    for (mode, description) in modes {
        println!("Mode: {}", description);
        println!("{}", "=".repeat(70));

        match NoriTokenizer::new(mode, false) {
            Ok(mut tokenizer) => {
                for (text, note) in &test_cases {
                    match tokenizer.tokenize(text) {
                        Ok(tokens) => {
                            println!("\nInput: {} ({})", text, note);
                            println!("Tokens: {}", tokens.len());
                            for (i, token) in tokens.iter().enumerate() {
                                println!(
                                    "  {}. {:8} [{:3}] offset:{:2}..{:2} {}",
                                    i + 1,
                                    token.surface,
                                    token.pos_tag,
                                    token.start_offset,
                                    token.end_offset,
                                    if token.is_decompound {
                                        "[DECOMPOSED]"
                                    } else {
                                        "[ORIGINAL]"
                                    }
                                );
                            }
                        }
                        Err(e) => {
                            println!("\nError tokenizing '{}': {}", text, e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to create tokenizer: {}", e);
                println!(
                    "Note: This requires mecab-ko-dic to be installed or MECAB_DICDIR to be set\n"
                );
                return;
            }
        }

        println!("\n{}\n", "=".repeat(70));
    }

    println!("=== Implementation Details ===");
    println!("\n1. Decomposition Algorithm:");
    println!("   - Minimum 3 syllables required for decomposition");
    println!("   - Jongseong (final consonant) boundary analysis");
    println!("   - Natural break points at jongseong transitions:");
    println!("     * No jongseong → Has jongseong (형태소 + 분석)");
    println!("     * Has jongseong → No jongseong (학교 + 운동장)");
    println!("   - Maximum 3 parts to prevent over-decomposition");
    println!("\n2. Suffix Detection:");
    println!("   - 들 (plural), 님/분 (honorific), 꾼 (person)");
    println!("   - Proper POS tag assignment (XSN for suffixes)");
    println!("\n3. Prefix Detection:");
    println!("   - 신/구 (new/old), 총/부 (rank), 전/후 (before/after)");
    println!("   - Proper POS tag assignment (XPN for prefixes)");
    println!("\n4. Offset Accuracy:");
    println!("   - Character-level offsets (not byte-level)");
    println!("   - Compatible with Lucene Token attributes");
    println!("\n5. Nori Compatibility:");
    println!("   - DecompoundMode: None, Discard, Mixed");
    println!("   - WordType: Known, Unknown, User");
    println!("   - is_decompound flag for decomposed tokens");
}
