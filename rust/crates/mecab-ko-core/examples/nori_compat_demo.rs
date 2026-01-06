//! Nori Compatibility Layer Demo
//!
//! This example demonstrates how to use the Lucene Nori compatibility layer
//! for Korean text analysis.
//!
//! Run with:
//! ```bash
//! cargo run --example nori_compat_demo
//! ```

use mecab_ko_core::nori_compat::{
    mecab_to_nori_tag, nori_to_mecab_tag, DecompoundMode, NoriAnalyzer, NoriTokenizer,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lucene Nori Compatibility Layer Demo ===\n");

    // 1. Basic Nori Tokenizer
    demo_basic_tokenizer()?;

    // 2. Decompound Modes
    demo_decompound_modes()?;

    // 3. Nori Analyzer with Stoptags
    demo_analyzer()?;

    // 4. POS Tag Mapping
    demo_tag_mapping();

    Ok(())
}

fn demo_basic_tokenizer() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Basic Nori Tokenizer");
    println!("{}", "─".repeat(50));

    let tokenizer = NoriTokenizer::new(DecompoundMode::None, false)?;
    let text = "한국어 형태소 분석기";

    println!("Input: {text}");
    let tokens = tokenizer.tokenize(text)?;

    println!("Tokens:");
    for token in &tokens {
        println!(
            "  - {}: {} [{}..{}]",
            token.surface, token.pos_tag, token.start_offset, token.end_offset
        );
    }

    println!();
    Ok(())
}

fn demo_decompound_modes() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Decompound Modes");
    println!("{}", "─".repeat(50));

    let text = "형태소분석";

    for mode in [
        DecompoundMode::None,
        DecompoundMode::Discard,
        DecompoundMode::Mixed,
    ] {
        let tokenizer = NoriTokenizer::new(mode, false)?;
        let tokens = tokenizer.tokenize(text)?;

        println!("Mode: {} - {} tokens", mode.as_str(), tokens.len());
        for token in &tokens {
            println!(
                "  - {}: {} {}",
                token.surface,
                token.pos_tag,
                if token.is_decompound {
                    "(decompounded)"
                } else {
                    ""
                }
            );
        }
    }

    println!();
    Ok(())
}

fn demo_analyzer() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Nori Analyzer (with stoptags filtering)");
    println!("{}", "─".repeat(50));

    // Default analyzer removes particles (J) and endings (E)
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed)?;

    let text = "안녕하세요";
    println!("Input: {text}");
    println!("Stoptags: {:?}", analyzer.stoptags());

    let tokens = analyzer.analyze(text)?;
    println!("Filtered tokens ({}):", tokens.len());
    for token in &tokens {
        println!("  - {}: {}", token.surface, token.pos_tag);
    }

    // Custom stoptags
    println!("\nCustom analyzer (removing J, E, SF):");
    let custom_analyzer = NoriAnalyzer::new(
        None,
        DecompoundMode::None,
        vec!["J".to_string(), "E".to_string(), "SF".to_string()],
        false,
    )?;

    let text2 = "안녕하세요.";
    let tokens2 = custom_analyzer.analyze(text2)?;
    println!("Filtered tokens ({}):", tokens2.len());
    for token in &tokens2 {
        println!("  - {}: {}", token.surface, token.pos_tag);
    }

    println!();
    Ok(())
}

fn demo_tag_mapping() {
    println!("4. POS Tag Mapping (MeCab ↔ Nori)");
    println!("{}", "─".repeat(50));

    // MeCab → Nori mapping
    println!("MeCab → Nori:");
    let mecab_tags = vec![
        ("JKS", "주격조사"),
        ("JKO", "목적격조사"),
        ("EF", "종결어미"),
        ("EC", "연결어미"),
        ("NNG", "일반명사"),
        ("VV", "동사"),
    ];

    for (mecab_tag, description) in mecab_tags {
        let nori_tag = mecab_to_nori_tag(mecab_tag);
        println!("  {mecab_tag:5} ({description:12}) → {nori_tag}");
    }

    // Nori → MeCab mapping
    println!("\nNori → MeCab (representative tags):");
    let nori_tags = vec![("J", "조사 통합"), ("E", "어미 통합"), ("NNG", "일반명사")];

    for (nori_tag, description) in nori_tags {
        let mecab_tag = nori_to_mecab_tag(nori_tag);
        println!("  {nori_tag:5} ({description:12}) → {mecab_tag}");
    }

    println!();
}
