//! Integration tests for streaming API
//!
//! These tests verify the streaming tokenizer functionality

#![allow(clippy::expect_used, clippy::unwrap_used)]

use mecab_ko_core::streaming::{StreamingTokenizer, TokenStream};
use mecab_ko_core::Tokenizer;

/// Helper to create a test string
fn create_test_text() -> String {
    "안녕하세요. 오늘 날씨가 좋습니다. 감사합니다.".to_string()
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_streaming_basic() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut stream = StreamingTokenizer::new(tokenizer);

    let text = create_test_text();
    let chunks: Vec<&str> = text.split('.').collect();

    let mut all_tokens = Vec::new();

    for chunk in chunks {
        if !chunk.is_empty() {
            let chunk_with_delimiter = format!("{chunk}.");
            let tokens = stream.process_chunk(&chunk_with_delimiter);
            all_tokens.extend(tokens);
        }
    }

    let remaining = stream.flush();
    all_tokens.extend(remaining);

    assert!(!all_tokens.is_empty());
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_streaming_large_text() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut stream = StreamingTokenizer::new(tokenizer).with_chunk_size(100);

    // Create large text
    let large_text = create_test_text().repeat(100);

    // Process in chunks
    let chunk_size = 100;
    let mut total_tokens = 0;

    for chunk in large_text.as_bytes().chunks(chunk_size) {
        if let Ok(chunk_str) = std::str::from_utf8(chunk) {
            let tokens = stream.process_chunk(chunk_str);
            total_tokens += tokens.len();
        }
    }

    let remaining = stream.flush();
    total_tokens += remaining.len();

    assert!(total_tokens > 0);
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_token_stream_iterator() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let chunks = vec![
        "첫 번째 문장입니다.\n".to_string(),
        "두 번째 문장입니다.\n".to_string(),
        "세 번째 문장입니다.\n".to_string(),
    ];

    let stream = TokenStream::new(chunks.into_iter(), tokenizer);

    let tokens: Vec<_> = stream.collect();

    assert!(!tokens.is_empty());
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_streaming_position_tracking() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut stream = StreamingTokenizer::new(tokenizer);

    let text = "안녕하세요.\n감사합니다.\n";
    let lines: Vec<&str> = text.lines().collect();

    for line in lines {
        let line_with_newline = format!("{line}\n");
        stream.process_chunk(&line_with_newline);
    }

    let total_processed = stream.total_chars_processed();
    assert!(total_processed > 0);
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_streaming_reset() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut stream = StreamingTokenizer::new(tokenizer);

    // First batch
    stream.process_chunk("안녕하세요.\n");
    stream.flush();

    let first_total = stream.total_chars_processed();

    // Reset
    stream.reset();

    assert_eq!(stream.buffer_len(), 0);
    assert_eq!(stream.total_chars_processed(), 0);

    // Second batch
    stream.process_chunk("감사합니다.\n");
    stream.flush();

    let second_total = stream.total_chars_processed();

    assert!(first_total > 0);
    assert!(second_total > 0);
    assert_ne!(first_total, second_total);
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_streaming_custom_delimiters() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut stream =
        StreamingTokenizer::new(tokenizer).with_sentence_delimiters(vec!['.', '!', '?']);

    let text = "안녕하세요! 감사합니다? 좋은 하루 되세요.";

    for ch in text.chars() {
        let tokens = stream.process_chunk(&ch.to_string());
        if !tokens.is_empty() {
            assert!(tokens.iter().any(|t| !t.surface.is_empty()));
        }
    }

    let remaining = stream.flush();
    assert!(!remaining.is_empty());
}
