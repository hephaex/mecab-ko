//! 필터 사용 예제

use mecab_ko_elasticsearch::analyzer::NoriTokenizerImpl;
use mecab_ko_elasticsearch::config::TokenizerConfig;
use mecab_ko_elasticsearch::filter::{
    CompositeFilter, LengthFilter, LowercaseFilter, NoriPartOfSpeechStopFilter,
    NoriReadingFormFilter, TokenFilter,
};
use mecab_ko_elasticsearch::tokenizer::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 필터 사용 예제 ===\n");

    let tokenizer = NoriTokenizerImpl::new(TokenizerConfig::default())?;
    let text = "한국어 형태소 분석기를 사용하여 텍스트를 처리합니다.";

    println!("입력: {text}\n");

    // 1. 기본 토큰화 (필터 없음)
    println!("1. 기본 토큰화 (필터 없음)");
    let tokens = tokenizer.tokenize(text)?;
    println!("결과: {} 토큰", tokens.len());
    for token in &tokens {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    // 2. 품사 필터 (조사 제거)
    println!("\n2. 품사 필터 (조사 J 제거)");
    let pos_filter = NoriPartOfSpeechStopFilter::new(vec!["J".to_string()]);
    let filtered = pos_filter.filter(tokens.clone())?;
    println!("결과: {} 토큰", filtered.len());
    for token in &filtered {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    // 3. 품사 필터 (조사 + 어미 제거)
    println!("\n3. 품사 필터 (조사 J + 어미 E 제거)");
    let pos_filter2 = NoriPartOfSpeechStopFilter::new(vec!["J".to_string(), "E".to_string()]);
    let filtered2 = pos_filter2.filter(tokens.clone())?;
    println!("결과: {} 토큰", filtered2.len());
    for token in &filtered2 {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    // 4. 길이 필터
    println!("\n4. 길이 필터 (2자 이상)");
    let length_filter = LengthFilter::new(2, 100);
    let filtered3 = length_filter.filter(tokens.clone())?;
    println!("결과: {} 토큰", filtered3.len());
    for token in &filtered3 {
        println!("  {} [{}] ({}자)", token.surface, token.pos_tag, token.len());
    }

    // 5. 소문자 필터 (영문 포함 텍스트)
    println!("\n5. 소문자 필터");
    let text_with_eng = "Korean Analyzer TEST";
    let tokens_eng = tokenizer.tokenize(text_with_eng)?;

    println!("원본:");
    for token in &tokens_eng {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    let lowercase_filter = LowercaseFilter::new();
    let filtered4 = lowercase_filter.filter(tokens_eng)?;
    println!("\n소문자 변환 후:");
    for token in &filtered4 {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    // 6. 복합 필터 (여러 필터 조합)
    println!("\n6. 복합 필터 (품사 + 길이 + 소문자)");
    let mut composite = CompositeFilter::new();
    composite.add_filter(Box::new(NoriPartOfSpeechStopFilter::new(vec![
        "J".to_string(),
        "E".to_string(),
    ])));
    composite.add_filter(Box::new(LengthFilter::new(2, 10)));
    composite.add_filter(Box::new(LowercaseFilter::new()));

    let test_text = "Test 한국어 형태소를 분석하는 ANALYZER입니다.";
    println!("입력: {test_text}");

    let tokens_test = tokenizer.tokenize(test_text)?;
    let filtered5 = composite.filter(tokens_test)?;
    println!("결과: {} 토큰", filtered5.len());
    for token in &filtered5 {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    // 7. 읽기 변환 필터
    println!("\n7. 읽기 변환 필터");
    let reading_filter = NoriReadingFormFilter::new();

    // 읽기 정보가 있는 토큰 생성 (예제용)
    use mecab_ko_elasticsearch::tokenizer::{Token, WordType};
    let tokens_with_reading = vec![
        Token::new("形態素".to_string(), "NNG".to_string(), 0, 3)
            .with_reading(Some("형태소".to_string())),
        Token::new("分析".to_string(), "NNG".to_string(), 3, 5)
            .with_reading(Some("분석".to_string())),
        Token::new("한자".to_string(), "NNG".to_string(), 5, 7), // 읽기 없음
    ];

    println!("원본:");
    for token in &tokens_with_reading {
        println!("  {} [{}] {:?}", token.surface, token.pos_tag, token.reading);
    }

    let filtered6 = reading_filter.filter(tokens_with_reading)?;
    println!("\n읽기 변환 후:");
    for token in &filtered6 {
        println!("  {} [{}]", token.surface, token.pos_tag);
    }

    Ok(())
}
