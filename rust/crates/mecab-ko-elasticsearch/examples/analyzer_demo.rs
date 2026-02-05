//! 기본 사용 예제

use mecab_ko_elasticsearch::analyzer::NoriAnalyzer;
use mecab_ko_elasticsearch::config::{AnalyzerConfig, DecompoundMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MeCab-Ko Elasticsearch 기본 사용 예제 ===\n");

    // 1. 기본 분석기 생성
    println!("1. 기본 분석기 (조사/어미 제거, 복합명사 분해 없음)");
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None)?;

    let text = "한국어 형태소 분석기를 사용합니다.";
    println!("입력: {text}\n");

    let tokens = analyzer.analyze(text)?;
    println!("결과:");
    for token in &tokens {
        println!(
            "  {} [{}] ({}..{})",
            token.surface, token.pos_tag, token.start_offset, token.end_offset
        );
    }

    // 2. Mixed 모드 (복합명사 분해)
    println!("\n2. Mixed 모드 (복합명사 분해)");
    let analyzer_mixed = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed)?;

    let text2 = "형태소분석기";
    println!("입력: {text2}\n");

    let tokens2 = analyzer_mixed.analyze(text2)?;
    println!("결과:");
    for token in &tokens2 {
        println!(
            "  {} [{}] {}",
            token.surface,
            token.pos_tag,
            if token.is_decompound {
                "(분해됨)"
            } else {
                ""
            }
        );
    }

    // 3. 커스텀 설정
    println!("\n3. 커스텀 설정 (모든 품사 유지)");
    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_stoptags(vec![]); // 필터 없음

    let analyzer_custom = NoriAnalyzer::new(config)?;

    let text3 = "이것은 테스트입니다.";
    println!("입력: {text3}\n");

    let tokens3 = analyzer_custom.analyze(text3)?;
    println!("결과:");
    for token in &tokens3 {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    // 4. 읽기 정보 표시
    println!("\n4. 읽기 정보");
    let tokens4 = analyzer.analyze("자연어처리")?;
    for token in &tokens4 {
        if let Some(reading) = &token.reading {
            println!("  {} → {} [{}]", token.surface, reading, token.pos_tag);
        } else {
            println!("  {} [{}]", token.surface, token.pos_tag);
        }
    }

    // 5. 긴 텍스트 분석
    println!("\n5. 긴 텍스트 분석");
    let long_text = "한국어 형태소 분석기는 자연어 처리의 기본 도구입니다. \
                     이를 통해 텍스트를 의미 있는 단위로 분해할 수 있습니다.";
    println!("입력: {long_text}\n");

    let tokens5 = analyzer.analyze(long_text)?;
    println!("결과: 총 {} 토큰", tokens5.len());
    for (i, token) in tokens5.iter().take(10).enumerate() {
        println!("  {}. {} [{}]", i + 1, token.surface, token.pos_tag);
    }
    if tokens5.len() > 10 {
        println!("  ... (나머지 {} 토큰)", tokens5.len() - 10);
    }

    Ok(())
}
