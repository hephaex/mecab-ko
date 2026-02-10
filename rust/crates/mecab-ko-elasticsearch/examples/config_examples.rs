//! 설정 예제

use mecab_ko_elasticsearch::analyzer::NoriAnalyzer;
use mecab_ko_elasticsearch::config::{AnalyzerConfig, DecompoundMode};

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 설정 예제 ===\n");

    let text = "형태소분석기를 사용합니다.";

    // 1. DecompoundMode::None
    println!("1. DecompoundMode::None");
    println!("   복합명사를 분해하지 않음\n");

    let config1 = AnalyzerConfig::new().with_decompound_mode(DecompoundMode::None);
    let analyzer1 = NoriAnalyzer::new(config1)?;

    println!("입력: {text}");
    let tokens1 = analyzer1.analyze(text)?;
    println!("결과:");
    for token in &tokens1 {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    // 2. DecompoundMode::Discard
    println!("\n2. DecompoundMode::Discard");
    println!("   원본은 버리고 분해된 형태소만 출력\n");

    let config2 = AnalyzerConfig::new().with_decompound_mode(DecompoundMode::Discard);
    let analyzer2 = NoriAnalyzer::new(config2)?;

    println!("입력: {text}");
    let tokens2 = analyzer2.analyze(text)?;
    println!("결과:");
    for token in &tokens2 {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    // 3. DecompoundMode::Mixed
    println!("\n3. DecompoundMode::Mixed");
    println!("   원본과 분해된 형태소 모두 출력\n");

    let config3 = AnalyzerConfig::new().with_decompound_mode(DecompoundMode::Mixed);
    let analyzer3 = NoriAnalyzer::new(config3)?;

    println!("입력: {text}");
    let tokens3 = analyzer3.analyze(text)?;
    println!("결과:");
    for token in &tokens3 {
        println!(
            "  {} [{}] {}",
            token.surface,
            token.pos_tag,
            if token.is_decompound {
                "(분해)"
            } else {
                "(원본)"
            }
        );
    }

    // 4. 사용자 정의 stoptags
    println!("\n4. 사용자 정의 stoptags");
    println!("   조사(J), 어미(E), 구두점(SF) 제거\n");

    let config4 = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_stoptags(vec!["J".to_string(), "E".to_string(), "SF".to_string()]);
    let analyzer4 = NoriAnalyzer::new(config4)?;

    let text2 = "한국어를 분석합니다.";
    println!("입력: {text2}");
    let tokens4 = analyzer4.analyze(text2)?;
    println!("결과:");
    for token in &tokens4 {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    // 5. stoptags 없음 (모든 품사 유지)
    println!("\n5. stoptags 없음 (모든 품사 유지)\n");

    let config5 = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_stoptags(vec![]);
    let analyzer5 = NoriAnalyzer::new(config5)?;

    println!("입력: {text2}");
    let tokens5 = analyzer5.analyze(text2)?;
    println!("결과:");
    for token in &tokens5 {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    // 6. JSON 직렬화/역직렬화
    println!("\n6. JSON 직렬화/역직렬화\n");

    let config6 = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::Mixed)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);

    let json = serde_json::to_string_pretty(&config6)?;
    println!("설정을 JSON으로 직렬화:");
    println!("{json}\n");

    let config6_deserialized: AnalyzerConfig = serde_json::from_str(&json)?;
    println!("JSON에서 설정 역직렬화:");
    println!(
        "  decompound_mode: {}",
        config6_deserialized.decompound_mode.as_str()
    );
    println!("  stoptags: {:?}", config6_deserialized.stoptags);

    // 7. 설정 유효성 검증
    println!("\n7. 설정 유효성 검증\n");

    let valid_config = AnalyzerConfig::new();
    match valid_config.validate() {
        Ok(()) => println!("유효한 설정"),
        Err(e) => println!("유효하지 않은 설정: {e}"),
    }

    // 8. Builder 패턴
    println!("\n8. Builder 패턴을 사용한 설정\n");

    let config8 = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::Mixed)
        .with_stoptags(vec!["J".to_string(), "E".to_string(), "SF".to_string()])
        .with_output_unknown_unigrams(false);

    println!("Builder 패턴으로 생성한 설정:");
    println!("  decompound_mode: {}", config8.decompound_mode.as_str());
    println!("  stoptags: {:?}", config8.stoptags);
    println!(
        "  output_unknown_unigrams: {}",
        config8.output_unknown_unigrams
    );

    let analyzer8 = NoriAnalyzer::new(config8)?;
    let tokens8 = analyzer8.analyze("한국어 분석 예제입니다.")?;
    println!("\n분석 결과: {} 토큰", tokens8.len());
    for token in tokens8 {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    Ok(())
}
