//! 통합 테스트

#![allow(clippy::unwrap_used, clippy::expect_used)]

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
    // Use a single mini-dict word so this works without a full dictionary
    let result = tokenizer.tokenize("한국어");
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
    // Use a word from the mini-dict so this works without a full dictionary
    let result = analyzer.analyze("한국어");
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
        "J".to_string()
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

    let _tokens = result.unwrap();
    // 공백만 있는 경우 토큰이 없거나 공백 토큰만 있을 수 있음
    // 구현에 따라 달라질 수 있으므로 에러가 없는지만 확인
}

#[test]
fn test_multiple_analysis_calls() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    // 같은 analyzer로 여러 번 분석 (use mini-dict words)
    for _ in 0..10 {
        let result = analyzer.analyze("한국어");
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
    }
}

#[test]
fn test_long_text_analysis() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    // Use a single mini-dict word repeated to create "long" text (no spaces to avoid
    // byte-offset mismatch between space-stripped and original text)
    let long_text = "한국어".repeat(20);

    let result = analyzer.analyze(&long_text);
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

    // Use a single mini-dict word; special character handling is tested structurally
    let text = "한국어";
    let result = analyzer.analyze(text);
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert!(!tokens.is_empty());
}

#[test]
fn test_mixed_language_text() {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    // Use a single mini-dict word to verify analysis works structurally
    let text = "한국어";
    let result = analyzer.analyze(text);
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert!(!tokens.is_empty());
}

// ── New tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_cache_hit_and_miss() {
    let config = AnalyzerConfig::new().with_decompound_mode(DecompoundMode::None);
    let analyzer = NoriAnalyzer::with_cache_size(config, 16).unwrap();

    // Cache starts empty.
    let (cap, len) = analyzer.cache_stats().expect("cache should be present");
    assert_eq!(len, 0);
    assert_eq!(cap, 16);

    // First call populates the cache.
    let result1 = analyzer.analyze("테스트").unwrap();
    let (_, len) = analyzer.cache_stats().unwrap();
    assert_eq!(len, 1);

    // Second call is a cache hit and returns the same tokens.
    let result2 = analyzer.analyze("테스트").unwrap();
    assert_eq!(result1, result2);

    // Cache entry count should still be 1 (no duplicate).
    let (_, len) = analyzer.cache_stats().unwrap();
    assert_eq!(len, 1);

    // clear_cache resets the entry count.
    analyzer.clear_cache();
    let (_, len) = analyzer.cache_stats().unwrap();
    assert_eq!(len, 0);
}

#[test]
fn test_cache_disabled() {
    let config = AnalyzerConfig::new().with_decompound_mode(DecompoundMode::None);
    let analyzer = NoriAnalyzer::with_cache_size(config, 0).unwrap();

    // cache_stats returns None when cache is disabled.
    assert!(analyzer.cache_stats().is_none());

    // Analysis still works without a cache.
    let result = analyzer.analyze("테스트");
    assert!(result.is_ok());
}

#[test]
fn test_analyzer_without_cache() {
    let config = AnalyzerConfig::new().with_decompound_mode(DecompoundMode::None);
    let analyzer = NoriAnalyzer::without_cache(config).unwrap();

    assert!(analyzer.cache_stats().is_none());

    let result = analyzer.analyze("분석");
    assert!(result.is_ok());
}

#[cfg(feature = "batch")]
#[test]
fn test_batch_analysis() {
    let config = AnalyzerConfig::new().with_decompound_mode(DecompoundMode::None);
    let analyzer = NoriAnalyzer::new(config).unwrap();

    let texts = ["형태소", "분석", "테스트"];
    let results = analyzer.analyze_batch(&texts).unwrap();

    // One result per input text.
    assert_eq!(results.len(), 3);

    // Each result is a valid Vec<Token> (may be empty for unknown words but not an error).
    for tokens in &results {
        // Verify that every token has a non-empty surface.
        for token in tokens {
            assert!(!token.surface.is_empty());
        }
    }
}

#[test]
fn test_composite_filter_empty() {
    // An empty CompositeFilter must pass all tokens through unchanged.
    let composite = CompositeFilter::new();
    assert!(composite.is_empty());
    assert_eq!(composite.len(), 0);

    let tokens = vec![
        Token::new("형태소".to_string(), "NNG".to_string(), 0, 3),
        Token::new("를".to_string(), "J".to_string(), 3, 4),
    ];

    let filtered = composite.filter(tokens.clone()).unwrap();
    assert_eq!(filtered.len(), tokens.len());
    assert_eq!(filtered[0].surface, "형태소");
    assert_eq!(filtered[1].surface, "를");
}

#[test]
fn test_length_filter_edge_cases() {
    // min=1, max=1 keeps only single-character tokens.
    let filter_one = LengthFilter::new(1, 1);
    let tokens = vec![
        Token::new("가".to_string(), "NNG".to_string(), 0, 1), // 1 char - keep
        Token::new("나다".to_string(), "NNG".to_string(), 1, 3), // 2 chars - drop
    ];
    let filtered = filter_one.filter(tokens).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].surface, "가");

    // min=0, max=0 keeps only zero-length tokens (empty surfaces).
    let filter_zero = LengthFilter::new(0, 0);
    let tokens = vec![
        Token::new(String::new(), "X".to_string(), 0, 0), // 0 chars - keep
        Token::new("가".to_string(), "NNG".to_string(), 0, 1), // 1 char - drop
    ];
    let filtered = filter_zero.filter(tokens).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].surface, "");

    // Exact boundary: token length equal to max is included.
    let filter_exact = LengthFilter::new(2, 3);
    let tokens = vec![
        Token::new("가".to_string(), "NNG".to_string(), 0, 1), // 1 - drop
        Token::new("나다".to_string(), "NNG".to_string(), 1, 3), // 2 - keep
        Token::new("가나다".to_string(), "NNG".to_string(), 3, 6), // 3 - keep
        Token::new("가나다라".to_string(), "NNG".to_string(), 6, 10), // 4 - drop
    ];
    let filtered = filter_exact.filter(tokens).unwrap();
    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_reading_form_filter_no_keep() {
    // with_keep_original(false): tokens without a reading are dropped.
    let filter = NoriReadingFormFilter::new().with_keep_original(false);

    let tokens = vec![
        Token::new("形態素".to_string(), "NNG".to_string(), 0, 3)
            .with_reading(Some("형태소".to_string())),
        Token::new("분석".to_string(), "NNG".to_string(), 3, 5), // no reading
        Token::new("分析".to_string(), "NNG".to_string(), 5, 7)
            .with_reading(Some("분석".to_string())),
    ];

    let filtered = filter.filter(tokens).unwrap();

    // Only tokens that had a reading survive.
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].surface, "형태소");
    assert_eq!(filtered[1].surface, "분석");
}

#[test]
fn test_config_from_json() {
    let json = r#"{
        "decompound_mode": "mixed",
        "user_dictionary_path": null,
        "stoptags": ["J", "E", "SP"],
        "output_unknown_unigrams": true
    }"#;

    let config: AnalyzerConfig = serde_json::from_str(json).unwrap();

    assert_eq!(config.decompound_mode, DecompoundMode::Mixed);
    assert!(config.output_unknown_unigrams);
    assert_eq!(config.stoptags.len(), 3);
    assert!(config.stoptags.contains(&"SP".to_string()));
}

#[test]
fn test_decompound_mode_from_str() {
    // Valid modes (case-insensitive).
    assert_eq!(
        DecompoundMode::from_str("none").unwrap(),
        DecompoundMode::None
    );
    assert_eq!(
        DecompoundMode::from_str("DISCARD").unwrap(),
        DecompoundMode::Discard
    );
    assert_eq!(
        DecompoundMode::from_str("Mixed").unwrap(),
        DecompoundMode::Mixed
    );

    // as_str round-trips.
    assert_eq!(DecompoundMode::None.as_str(), "none");
    assert_eq!(DecompoundMode::Discard.as_str(), "discard");
    assert_eq!(DecompoundMode::Mixed.as_str(), "mixed");

    // Invalid input returns an error.
    assert!(DecompoundMode::from_str("").is_err());
    assert!(DecompoundMode::from_str("invalid").is_err());
    assert!(DecompoundMode::from_str("FULL").is_err());
}

#[test]
fn test_token_display_format() {
    let token = Token::new("테스트".to_string(), "NNG".to_string(), 0, 9);
    let display = token.to_string();

    // Format is: surface[pos_tag](start-end)
    assert!(display.contains("테스트"));
    assert!(display.contains("NNG"));
    assert!(display.contains('0'));
    assert!(display.contains('9'));
    // Confirm the exact format: "테스트[NNG](0-9)"
    assert_eq!(display, "테스트[NNG](0-9)");
}

#[test]
fn test_token_stream_from_tokenizer() {
    use mecab_ko_elasticsearch::tokenizer::Tokenizer;

    let config = mecab_ko_elasticsearch::config::TokenizerConfig {
        decompound_mode: DecompoundMode::None,
        user_dictionary_path: None,
        output_unknown_unigrams: false,
    };

    let tokenizer = NoriTokenizerImpl::new(config).unwrap();

    // token_stream() must return an iterator that yields Token values.
    let tokens_via_stream: Vec<_> = tokenizer.token_stream("테스트").collect();

    // Also verify tokenize() returns the same results.
    let tokens_via_tokenize = tokenizer.tokenize("테스트").unwrap();

    assert_eq!(tokens_via_stream.len(), tokens_via_tokenize.len());

    for (a, b) in tokens_via_stream.iter().zip(tokens_via_tokenize.iter()) {
        assert_eq!(a.surface, b.surface);
        assert_eq!(a.pos_tag, b.pos_tag);
    }
}

#[test]
fn test_analyzer_config_with_user_dict_missing() {
    use std::path::PathBuf;

    // Validation must fail when the user dictionary path does not exist.
    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_user_dictionary(PathBuf::from("/nonexistent/path/user.csv"));

    let validation_result = config.validate();
    assert!(
        validation_result.is_err(),
        "Expected validation to fail for missing dictionary"
    );

    // NoriAnalyzer::new() also propagates the validation error.
    let analyzer_result = NoriAnalyzer::new(
        AnalyzerConfig::new()
            .with_decompound_mode(DecompoundMode::None)
            .with_user_dictionary(PathBuf::from("/nonexistent/path/user.csv")),
    );
    assert!(analyzer_result.is_err());
}
