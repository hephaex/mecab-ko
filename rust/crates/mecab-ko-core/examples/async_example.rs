//! Async Tokenizer Example
//!
//! 비동기 형태소 분석 예제 (tokio 기반)

use mecab_ko_core::async_tokenizer::{AsyncStreamingTokenizer, AsyncTokenizer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Async Tokenizer Example ===\n");

    // 1. 기본 비동기 토큰화
    println!("1. Basic Async Tokenization:");
    basic_async().await?;

    // 2. 배치 비동기 처리
    println!("\n2. Batch Async Processing:");
    batch_async().await?;

    // 3. 동시 실행 제어
    println!("\n3. Concurrent Execution Control:");
    concurrent_control().await?;

    // 4. 비동기 스트리밍
    println!("\n4. Async Streaming:");
    async_streaming().await?;

    // 5. 대용량 배치 처리
    println!("\n5. Large Batch Processing:");
    large_batch().await?;

    Ok(())
}

async fn basic_async() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = AsyncTokenizer::new().await?;

    let texts = vec!["안녕하세요", "감사합니다", "좋은 하루 되세요"];

    for text in texts {
        let tokens = tokenizer.tokenize_async(text).await;
        println!("  Text: {text}");
        println!("  Tokens: {}", tokens.len());
        for token in tokens {
            println!("    - {}: {}", token.surface, token.pos);
        }
    }

    Ok(())
}

async fn batch_async() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = AsyncTokenizer::new().await?;

    let texts = vec![
        "첫 번째 문장입니다".to_string(),
        "두 번째 문장입니다".to_string(),
        "세 번째 문장입니다".to_string(),
    ];

    println!("  Processing {} texts in batch...", texts.len());

    let start = std::time::Instant::now();
    let results = tokenizer.tokenize_batch(texts.clone()).await;
    let elapsed = start.elapsed();

    println!("  Batch processing completed in {elapsed:?}");
    println!("  Results:");
    for (text, tokens) in texts.iter().zip(results.iter()) {
        println!("    - {text}: {} tokens", tokens.len());
    }

    Ok(())
}

async fn concurrent_control() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = AsyncTokenizer::new().await?.with_max_concurrent(2);

    println!("  Max concurrent: {}", tokenizer.max_concurrent());

    let texts: Vec<String> = (0..10).map(|i| format!("테스트 문장 번호 {i}")).collect();

    let start = std::time::Instant::now();
    let results = tokenizer.tokenize_batch(texts).await;
    let elapsed = start.elapsed();

    println!("  Processed {} texts in {elapsed:?}", results.len());
    println!("  Average per text: {:?}", elapsed / results.len() as u32);

    Ok(())
}

async fn async_streaming() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = AsyncTokenizer::new().await?;
    let mut stream = AsyncStreamingTokenizer::new(tokenizer);

    let chunks = vec!["안녕하세요.\n", "오늘 날씨가 좋습니다.\n", "감사합니다"];

    for chunk in chunks {
        let tokens = stream.process_chunk(chunk).await;
        if !tokens.is_empty() {
            println!("  Chunk processed: {} tokens", tokens.len());
        }
    }

    let remaining = stream.flush().await;
    println!("  Flushed: {} tokens", remaining.len());

    Ok(())
}

async fn large_batch() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = AsyncTokenizer::new().await?.with_max_concurrent(4);

    // 대용량 배치 생성
    let texts: Vec<String> = (0..100)
        .map(|i| {
            format!(
                "이것은 테스트 문장 번호 {}입니다. 형태소 분석을 수행합니다.",
                i
            )
        })
        .collect();

    println!(
        "  Processing {} texts with max_concurrent=4...",
        texts.len()
    );

    let start = std::time::Instant::now();
    let results = tokenizer.tokenize_batch(texts).await;
    let elapsed = start.elapsed();

    let total_tokens: usize = results.iter().map(|r| r.len()).sum();

    println!("  Completed in {elapsed:?}");
    println!("  Total tokens: {total_tokens}");
    println!(
        "  Throughput: {:.2} texts/sec",
        results.len() as f64 / elapsed.as_secs_f64()
    );

    Ok(())
}
