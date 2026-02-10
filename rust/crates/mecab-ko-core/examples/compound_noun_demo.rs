//! Compound Noun Decomposition Demo
//!
//! This example demonstrates the basic compound noun decomposition
//! functionality for Nori compatibility.
//!
//! Run with: `cargo run --example compound_noun_demo`

use mecab_ko_core::nori_compat::{DecompoundMode, NoriTokenizer};

fn main() {
    println!("=== Compound Noun Decomposition Demo ===\n");

    // Create tokenizer with different decompound modes
    let modes = [
        (DecompoundMode::None, "None (no decomposition)"),
        (DecompoundMode::Discard, "Discard (only components)"),
        (DecompoundMode::Mixed, "Mixed (original + components)"),
    ];

    let test_texts = vec![
        "형태소분석",
        "자연언어처리",
        "한국어형태소분석기",
    ];

    for (mode, description) in modes {
        println!("Mode: {}", description);
        println!("{}", "=".repeat(50));

        match NoriTokenizer::new(mode, false) {
            Ok(mut tokenizer) => {
                for text in &test_texts {
                    match tokenizer.tokenize(text) {
                        Ok(tokens) => {
                            println!("\nInput: {}", text);
                            println!("Tokens:");
                            for (i, token) in tokens.iter().enumerate() {
                                println!(
                                    "  {}. {} [{}] ({}..{}) {}",
                                    i + 1,
                                    token.surface,
                                    token.pos_tag,
                                    token.start_offset,
                                    token.end_offset,
                                    if token.is_decompound {
                                        "[DECOMPOSED]"
                                    } else {
                                        ""
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
            }
        }

        println!("\n{}\n", "=".repeat(50));
    }

    println!("\n=== Implementation Notes ===");
    println!("- Current implementation uses syllable-based heuristics");
    println!("- Looks for natural break points at jongseong boundaries");
    println!("- Minimum 2 syllables required for decomposition");
    println!("- Future versions will use dictionary-based decomposition");
}
