//! Integration tests for streaming API
//!
//! These tests verify the streaming tokenizer functionality

#![allow(clippy::expect_used, clippy::unwrap_used)]

use mecab_ko_core::streaming::{StreamingTokenizer, TokenStream};
use mecab_ko_core::Tokenizer;

/// Helper to create a test string using mini-dict words with newline delimiters
fn create_test_text() -> String {
    // Use words from the mini-dict so this works without a full dictionary.
    // Newlines serve as sentence delimiters (included in StreamingTokenizer defaults).
    "안녕\n감사\n한국어\n".to_string()
}

#[test]
fn test_streaming_basic() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut stream = StreamingTokenizer::new(tokenizer);

    let text = create_test_text();
    // Split on '\n' and feed each line back with the newline delimiter
    let lines: Vec<&str> = text.split('\n').filter(|s| !s.is_empty()).collect();

    let mut all_tokens = Vec::new();

    for line in lines {
        let chunk_with_delimiter = format!("{line}\n");
        let tokens = stream.process_chunk(&chunk_with_delimiter);
        all_tokens.extend(tokens);
    }

    let remaining = stream.flush();
    all_tokens.extend(remaining);

    assert!(!all_tokens.is_empty());
}

#[test]
fn test_streaming_large_text() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut stream = StreamingTokenizer::new(tokenizer).with_chunk_size(100);

    // Create large text using mini-dict words with newline delimiters
    // (avoid spaces and periods which create unknown nodes outside the 25x25 mini-dict matrix)
    let large_text = create_test_text().repeat(100);

    // Process in whole-line chunks to preserve UTF-8 boundaries
    let mut total_tokens = 0;

    for line in large_text.lines() {
        let chunk_with_newline = format!("{line}\n");
        let tokens = stream.process_chunk(&chunk_with_newline);
        total_tokens += tokens.len();
    }

    let remaining = stream.flush();
    total_tokens += remaining.len();

    assert!(total_tokens > 0);
}

#[test]
fn test_token_stream_iterator() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // Use mini-dict words with newline delimiters so this works without a full dictionary
    let chunks = vec![
        "안녕\n".to_string(),
        "감사\n".to_string(),
        "한국어\n".to_string(),
    ];

    let stream = TokenStream::new(chunks.into_iter(), tokenizer);

    let tokens: Vec<_> = stream.collect();

    assert!(!tokens.is_empty());
}

#[test]
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
fn test_streaming_reset() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut stream = StreamingTokenizer::new(tokenizer);

    // First batch: use mini-dict word + newline delimiter (shorter, 3 chars total)
    stream.process_chunk("안녕\n");
    stream.flush();

    let first_total = stream.total_chars_processed();

    // Reset
    stream.reset();

    assert_eq!(stream.buffer_len(), 0);
    assert_eq!(stream.total_chars_processed(), 0);

    // Second batch: use a longer mini-dict word + newline (4 chars total)
    stream.process_chunk("한국어\n");
    stream.flush();

    let second_total = stream.total_chars_processed();

    assert!(first_total > 0);
    assert!(second_total > 0);
    // first_total (3 chars) != second_total (4 chars)
    assert_ne!(first_total, second_total);
}

#[test]
fn test_streaming_custom_delimiters() {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    // Use newline as the only custom delimiter; it is treated as whitespace by
    // the tokenizer (stripped before Viterbi) so it doesn't create unknown nodes.
    let mut stream =
        StreamingTokenizer::new(tokenizer).with_sentence_delimiters(vec!['\n']);

    // Use mini-dict words separated by newlines so this works without a full dictionary
    let text = "안녕\n감사\n한국어";

    for ch in text.chars() {
        let tokens = stream.process_chunk(&ch.to_string());
        if !tokens.is_empty() {
            assert!(tokens.iter().any(|t| !t.surface.is_empty()));
        }
    }

    // The last segment "한국어" has no trailing '\n', so it remains in the buffer
    let remaining = stream.flush();
    assert!(!remaining.is_empty());
}
