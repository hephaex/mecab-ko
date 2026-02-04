//! Batch Processing Example
//!
//! Rayon 기반 병렬 배치 처리 예제

use mecab_ko_core::batch::{BatchTokenizer, ParallelStreamProcessor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Batch Processing Example ===\n");

    // 1. 기본 배치 처리
    println!("1. Basic Batch Processing:");
    basic_batch()?;

    // 2. 대용량 배치 처리
    println!("\n2. Large Batch Processing:");
    large_batch()?;

    // 3. 청크 단위 병렬 처리
    println!("\n3. Chunked Parallel Processing:");
    chunked_processing()?;

    // 4. 병렬 스트리밍 프로세서
    println!("\n4. Parallel Stream Processor:");
    parallel_streaming()?;

    // 5. 성능 비교
    println!("\n5. Performance Comparison:");
    performance_comparison()?;

    Ok(())
}

fn basic_batch() -> Result<(), Box<dyn std::error::Error>> {
    let batch = BatchTokenizer::new()?;

    println!("  Pool size: {}", batch.pool_size());
    println!("  Available tokenizers: {}", batch.available_tokenizers());

    let texts = vec![
        "안녕하세요",
        "감사합니다",
        "좋은 하루 되세요",
        "한국어 형태소 분석",
        "자연어 처리",
    ];

    let results = batch.tokenize_batch(&texts);

    println!("  Processed {} texts:", results.len());
    for (text, tokens) in texts.iter().zip(results.iter()) {
        println!("    - {text}: {} tokens", tokens.len());
    }

    Ok(())
}

fn large_batch() -> Result<(), Box<dyn std::error::Error>> {
    let batch = BatchTokenizer::with_pool_size(8)?;

    // 대용량 텍스트 배치 생성
    let texts: Vec<String> = (0..1000)
        .map(|i| format!("테스트 문장 번호 {}입니다. 형태소 분석을 수행합니다.", i))
        .collect();

    println!("  Processing {} texts with pool_size=8...", texts.len());

    let start = std::time::Instant::now();
    let results = batch.tokenize_batch_owned(&texts);
    let elapsed = start.elapsed();

    let total_tokens: usize = results.iter().map(|r| r.len()).sum();

    println!("  Completed in {elapsed:?}");
    println!("  Total tokens: {total_tokens}");
    println!(
        "  Throughput: {:.2} texts/sec",
        texts.len() as f64 / elapsed.as_secs_f64()
    );

    Ok(())
}

fn chunked_processing() -> Result<(), Box<dyn std::error::Error>> {
    let batch = BatchTokenizer::new()?;

    let long_text = "한국어 형태소 분석은 자연어 처리의 기본입니다. \
                     문장을 형태소 단위로 분석하여 의미를 파악합니다. \
                     이를 통해 다양한 응용 프로그램을 개발할 수 있습니다. \
                     형태소 분석기는 사전과 문법 규칙을 사용합니다. \
                     복잡한 문장도 정확하게 분석할 수 있어야 합니다.";

    println!("  Original text length: {} chars", long_text.len());

    let start = std::time::Instant::now();
    let tokens = batch.tokenize_chunked(long_text, 50);
    let elapsed = start.elapsed();

    println!("  Chunked processing completed in {elapsed:?}");
    println!("  Total tokens: {}", tokens.len());

    // 샘플 토큰 출력
    println!("  Sample tokens:");
    for token in tokens.iter().take(10) {
        println!("    - {}: {}", token.surface, token.pos);
    }

    Ok(())
}

fn parallel_streaming() -> Result<(), Box<dyn std::error::Error>> {
    let processor = ParallelStreamProcessor::new()?.with_chunk_size(8192);

    let simulated_large_file = "한국어 형태소 분석 테스트입니다.\n".repeat(100);

    println!(
        "  Simulated file size: {} bytes",
        simulated_large_file.len()
    );

    let start = std::time::Instant::now();

    // 임시 파일 생성 및 처리 시뮬레이션
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("mecab_test.txt");
    std::fs::write(&temp_file, &simulated_large_file)?;

    let tokens = processor.process_large_file(&temp_file)?;
    let elapsed = start.elapsed();

    // 임시 파일 삭제
    std::fs::remove_file(&temp_file)?;

    println!("  Processing completed in {elapsed:?}");
    println!("  Total tokens: {}", tokens.len());

    Ok(())
}

fn performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    use mecab_ko_core::Tokenizer;

    let texts: Vec<String> = (0..100)
        .map(|i| format!("테스트 문장 번호 {}입니다.", i))
        .collect();

    // 순차 처리
    println!("  Sequential processing:");
    let start = std::time::Instant::now();
    {
        let mut tokenizer = Tokenizer::new()?;
        let _results: Vec<_> = texts.iter().map(|t| tokenizer.tokenize(t)).collect();
    }
    let sequential_time = start.elapsed();
    println!("    Time: {sequential_time:?}");

    // 병렬 처리
    println!("  Parallel processing:");
    let start = std::time::Instant::now();
    {
        let batch = BatchTokenizer::new()?;
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let _results = batch.tokenize_batch(&text_refs);
    }
    let parallel_time = start.elapsed();
    println!("    Time: {parallel_time:?}");

    // 속도 향상 계산
    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("  Speedup: {speedup:.2}x");

    Ok(())
}
