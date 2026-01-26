//! Integration tests for batch processing API
//!
//! These tests verify the batch tokenizer functionality

use mecab_ko_core::batch::{BatchTokenizer, ParallelStreamProcessor};

#[test]
#[ignore = "Requires dictionary installation"]
fn test_batch_basic() {
    let batch = BatchTokenizer::new().expect("Failed to create batch tokenizer");

    let texts = vec!["안녕하세요", "감사합니다", "좋은 하루 되세요"];

    let results = batch.tokenize_batch(&texts);

    assert_eq!(results.len(), texts.len());

    for tokens in &results {
        assert!(!tokens.is_empty());
    }
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_batch_large_scale() {
    let batch = BatchTokenizer::with_pool_size(4).expect("Failed to create batch tokenizer");

    // Create large batch
    let texts: Vec<String> = (0..100)
        .map(|i| format!("테스트 문장 번호 {i}입니다."))
        .collect();

    let start = std::time::Instant::now();
    let results = batch.tokenize_batch_owned(&texts);
    let elapsed = start.elapsed();

    assert_eq!(results.len(), 100);

    let total_tokens: usize = results.iter().map(|r| r.len()).sum();
    assert!(total_tokens > 0);

    println!("Batch processing of 100 texts took: {elapsed:?}");
    println!("Total tokens: {total_tokens}");
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_batch_vs_sequential() {
    use mecab_ko_core::Tokenizer;

    let texts: Vec<String> = (0..50)
        .map(|i| format!("이것은 테스트 문장 번호 {i}입니다."))
        .collect();

    // Sequential
    let start = std::time::Instant::now();
    {
        let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
        let _results: Vec<_> = texts.iter().map(|t| tokenizer.tokenize(t)).collect();
    }
    let sequential_time = start.elapsed();

    // Batch
    let start = std::time::Instant::now();
    {
        let batch = BatchTokenizer::new().expect("Failed to create batch tokenizer");
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let _results = batch.tokenize_batch(&text_refs);
    }
    let batch_time = start.elapsed();

    println!("Sequential: {sequential_time:?}");
    println!("Batch:      {batch_time:?}");
    println!(
        "Speedup:    {:.2}x",
        sequential_time.as_secs_f64() / batch_time.as_secs_f64()
    );

    // Batch should be faster for large batches
    assert!(batch_time < sequential_time * 2);
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_batch_empty() {
    let batch = BatchTokenizer::new().expect("Failed to create batch tokenizer");

    let texts: Vec<&str> = vec![];
    let results = batch.tokenize_batch(&texts);

    assert!(results.is_empty());
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_batch_chunked() {
    let batch = BatchTokenizer::new().expect("Failed to create batch tokenizer");

    let long_text = "한국어 형태소 분석은 자연어 처리의 기본입니다. ".repeat(10);

    let tokens = batch.tokenize_chunked(&long_text, 50);

    assert!(!tokens.is_empty());
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_parallel_stream_processor() {
    let processor = ParallelStreamProcessor::new().expect("Failed to create processor");

    // Create temporary file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("mecab_batch_test.txt");

    let content = "안녕하세요.\n감사합니다.\n좋은 하루 되세요.\n".repeat(10);
    std::fs::write(&temp_file, &content).expect("Failed to write temp file");

    let tokens = processor
        .process_large_file(&temp_file)
        .expect("Failed to process file");

    // Cleanup
    std::fs::remove_file(&temp_file).ok();

    assert!(!tokens.is_empty());
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_batch_pool_management() {
    let batch = BatchTokenizer::with_pool_size(8).expect("Failed to create batch tokenizer");

    assert_eq!(batch.pool_size(), 8);
    assert_eq!(batch.available_tokenizers(), 8);

    // Process some texts
    let texts = vec!["테스트", "문장"];
    let _results = batch.tokenize_batch(&texts);

    // Pool should be restored
    assert_eq!(batch.available_tokenizers(), 8);
}

#[test]
#[ignore = "Requires dictionary installation"]
fn test_batch_multiple_calls() {
    let batch = BatchTokenizer::new().expect("Failed to create batch tokenizer");

    // First batch
    let texts1 = vec!["안녕하세요", "감사합니다"];
    let results1 = batch.tokenize_batch(&texts1);

    assert_eq!(results1.len(), 2);

    // Second batch
    let texts2 = vec!["좋은", "하루", "되세요"];
    let results2 = batch.tokenize_batch(&texts2);

    assert_eq!(results2.len(), 3);

    // Results should be independent
    assert_ne!(results1[0].len(), results2[0].len());
}
