//! 통합 테스트

use mecab_ko_elasticsearch::analyzer::{NoriAnalyzer, NoriTokenizerImpl};
use mecab_ko_elasticsearch::config::{AnalyzerConfig, DecompoundMode, TokenizerConfig};
use mecab_ko_elasticsearch::filter::{
    CompositeFilter, LengthFilter, LowercaseFilter, NoriPartOfSpeechStopFilter,
    NoriReadingFormFilter, TokenFilter,
};
use mecab_ko_elasticsearch::tokenizer::{Token, Tokenizer};

#[test]
fn test_nori_tokenizer_basic() {
    let config = TokenizerConfig {
        decompound_mode: DecompoundMode::None,
        user_dictionary_path: None,
        output_unknown_unigrams: false,
    };

    let tokenizer = NoriTokenizerImpl::new(config);
    assert!(tokenizer.is_ok());

    let tokenizer = tokenizer.unwrap();
    let result = tokenizer.tokenize("한국어 형태소 분석");
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert!(!tokens.is_empty());

    // 모든 토큰이 유효한 오프셋을 가지는지 확인
    for token in &tokens {
        assert!(token.end_offset >= token.start_offset);
        assert!(!token.surface.is_empty());
        assert!(!token.pos_tag.is_empty());
    }
}

#[test]
fn test_nori_analyzer_with_stoptags() {
    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);

    let analyzer = NoriAnalyzer::new(config);
    assert!(analyzer.is_ok());

    let analyzer = analyzer.unwrap();
    let result = analyzer.analyze("형태소를 분석합니다");
    assert!(result.is_ok());

    let tokens = result.unwrap();

    // 조사(J)와 어미(E)가 제거되었는지 확인
    for token in &tokens {
        assert_ne!(token.pos_tag, "J");
        assert_ne!(token.pos_tag, "E");
    }
}

#[test]
fn test_nori_analyzer_default() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None);
    assert!(analyzer.is_ok());

    let analyzer = analyzer.unwrap();
    let stoptags = analyzer.stoptags();
    assert_eq!(stoptags.len(), 2);
    assert!(stoptags.contains(&"J"));
    assert!(stoptags.contains(&"E"));
}

#[test]
fn test_decompound_mode_none() {
    let config = AnalyzerConfig::new().with_decompound_mode(DecompoundMode::None);

    let analyzer = NoriAnalyzer::new(config);
    assert!(analyzer.is_ok());

    let analyzer = analyzer.unwrap();
    let result = analyzer.analyze("형태소분석기");
    assert!(result.is_ok());

    // None 모드: 복합명사를 분해하지 않음
    let tokens = result.unwrap();
    assert!(!tokens.is_empty());
}

#[test]
fn test_pos_filter_standalone() {
    let filter = NoriPartOfSpeechStopFilter::new(vec!["J".to_string(), "SF".to_string()]);

    let tokens = vec![
        Token::new("형태소".to_string(), "NNG".to_string(), 0, 3),
        Token::new("를".to_string(), "J".to_string(), 3, 4),
        Token::new("분석".to_string(), "NNG".to_string(), 4, 6),
        Token::new(".".to_string(), "SF".to_string(), 6, 7),
    ];

    let filtered = filter.filter(tokens);
    assert!(filtered.is_ok());

    let filtered = filtered.unwrap();
    assert_eq!(filtered.len(), 2); // NNG만 남음
    assert_eq!(filtered[0].surface, "형태소");
    assert_eq!(filtered[1].surface, "분석");
}

#[test]
fn test_reading_form_filter_standalone() {
    let filter = NoriReadingFormFilter::new();

    let tokens = vec![
        Token::new("形態素".to_string(), "NNG".to_string(), 0, 3)
            .with_reading(Some("형태소".to_string())),
        Token::new("分析".to_string(), "NNG".to_string(), 3, 5)
            .with_reading(Some("분석".to_string())),
    ];

    let filtered = filter.filter(tokens);
    assert!(filtered.is_ok());

    let filtered = filtered.unwrap();
    assert_eq!(filtered.len(), 2);
    // 읽기로 변환되었는지 확인
    assert_eq!(filtered[0].surface, "형태소");
    assert_eq!(filtered[1].surface, "분석");
}

#[test]
fn test_composite_filter_chain() {
    let mut composite = CompositeFilter::new();

    // 1. 품사 필터 (조사 제거)
    composite.add_filter(Box::new(NoriPartOfSpeechStopFilter::new(vec![
        "J".to_string(),
    ])));

    // 2. 길이 필터 (2자 이상)
    composite.add_filter(Box::new(LengthFilter::new(2, 10)));

    // 3. 소문자 변환
    composite.add_filter(Box::new(LowercaseFilter::new()));

    let tokens = vec![
        Token::new("Test".to_string(), "NNG".to_string(), 0, 4),
        Token::new("를".to_string(), "J".to_string(), 4, 5),
        Token::new("A".to_string(), "NNG".to_string(), 5, 6),
        Token::new("HELLO".to_string(), "NNG".to_string(), 6, 11),
    ];

    let filtered = composite.filter(tokens);
    assert!(filtered.is_ok());

    let filtered = filtered.unwrap();
    assert_eq!(filtered.len(), 2); // "Test", "HELLO"만 남음 (조사, 1자 제거)
    assert_eq!(filtered[0].surface, "test");
    assert_eq!(filtered[1].surface, "hello");
}

#[test]
fn test_analyzer_config_validation() {
    let config = AnalyzerConfig::new().with_decompound_mode(DecompoundMode::Mixed);

    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_analyzer_stoptag_management() {
    let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    // 초기 상태
    let tags = analyzer.stoptags();
    assert_eq!(tags.len(), 2);

    // 추가
    analyzer.add_stoptag("SF".to_string());
    let tags = analyzer.stoptags();
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"SF"));

    // 제거
    let removed = analyzer.remove_stoptag("SF");
    assert!(removed);
    let tags = analyzer.stoptags();
    assert_eq!(tags.len(), 2);

    // 없는 태그 제거
    let removed = analyzer.remove_stoptag("NONEXISTENT");
    assert!(!removed);
}

#[test]
fn test_token_builder_pattern() {
    let token = Token::new("테스트".to_string(), "NNG".to_string(), 0, 3)
        .with_lemma(Some("테스트".to_string()))
        .with_reading(Some("테스트".to_string()))
        .with_position_increment(1)
        .with_position_length(1);

    assert_eq!(token.surface, "테스트");
    assert_eq!(token.lemma, Some("테스트".to_string()));
    assert_eq!(token.reading, Some("테스트".to_string()));
    assert_eq!(token.position_increment, 1);
}

#[test]
fn test_config_serialization() {
    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::Mixed)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);

    // JSON 직렬화
    let json = serde_json::to_string(&config);
    assert!(json.is_ok());

    // JSON 역직렬화
    let deserialized: std::result::Result<AnalyzerConfig, serde_json::Error> =
        serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());

    let deserialized = deserialized.unwrap();
    assert_eq!(deserialized.decompound_mode, DecompoundMode::Mixed);
    assert_eq!(deserialized.stoptags.len(), 2);
}

#[test]
fn test_token_serialization() {
    let token = Token::new("테스트".to_string(), "NNG".to_string(), 0, 3)
        .with_lemma(Some("테스트".to_string()));

    // JSON 직렬화
    let json = serde_json::to_string(&token);
    assert!(json.is_ok());

    // JSON 역직렬화
    let deserialized: std::result::Result<Token, serde_json::Error> =
        serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());

    let deserialized = deserialized.unwrap();
    assert_eq!(deserialized.surface, "테스트");
    assert_eq!(deserialized.pos_tag, "NNG");
}

#[test]
fn test_empty_text_handling() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    let result = analyzer.analyze("");
    assert!(result.is_ok());

    // 빈 텍스트는 토큰이 없거나 EOS 토큰만 있을 수 있음
    // 에러 없이 처리되면 성공
}

#[test]
fn test_whitespace_only_text() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    let result = analyzer.analyze("   \t\n   ");
    assert!(result.is_ok());

    let tokens = result.unwrap();
    // 공백만 있는 경우 토큰이 없거나 공백 토큰만 있을 수 있음
    // 구현에 따라 달라질 수 있으므로 에러가 없는지만 확인
}

#[test]
fn test_multiple_analysis_calls() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    // 같은 analyzer로 여러 번 분석
    for _ in 0..10 {
        let result = analyzer.analyze("한국어 형태소 분석");
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
    }
}

#[test]
fn test_long_text_analysis() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    let long_text = "한국어 형태소 분석기는 자연어 처리의 기본 도구입니다. \
                     이를 통해 텍스트를 의미 있는 단위로 분해할 수 있습니다. \
                     Elasticsearch와 통합하여 강력한 검색 기능을 제공합니다.";

    let result = analyzer.analyze(long_text);
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert!(!tokens.is_empty());

    // 모든 토큰의 오프셋이 순차적인지 확인
    for i in 1..tokens.len() {
        assert!(tokens[i].start_offset >= tokens[i - 1].start_offset);
    }
}

#[test]
fn test_special_characters() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    let text = "한국어! 형태소? 분석.";
    let result = analyzer.analyze(text);
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert!(!tokens.is_empty());
}

#[test]
fn test_mixed_language_text() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    let text = "한국어 Korean 형태소 analyzer 분석";
    let result = analyzer.analyze(text);
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert!(!tokens.is_empty());
}
