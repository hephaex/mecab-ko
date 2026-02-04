//! Streaming Tokenizer Example
//!
//! 대용량 텍스트 스트리밍 처리 예제

use mecab_ko_core::streaming::StreamingTokenizer;
use mecab_ko_core::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Streaming Tokenizer Example ===\n");

    // 1. 기본 스트리밍 처리
    println!("1. Basic Streaming Processing:");
    basic_streaming()?;

    // 2. 청크 단위 처리
    println!("\n2. Chunk-based Processing:");
    chunk_processing()?;

    // 3. 파일 스트리밍 (가상)
    println!("\n3. Simulated File Streaming:");
    file_streaming_simulation()?;

    // 4. Iterator 기반 스트리밍
    println!("\n4. Iterator-based Streaming:");
    iterator_streaming()?;

    Ok(())
}

fn basic_streaming() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new()?;
    let mut stream = StreamingTokenizer::new(tokenizer);

    // 여러 청크로 나누어 처리
    let chunks = vec!["안녕하세요.\n", "오늘 날씨가 좋습니다.\n", "감사합니다"];

    for chunk in chunks {
        let tokens = stream.process_chunk(chunk);
        if !tokens.is_empty() {
            println!("  Processed chunk: {chunk:?}");
            println!("  Tokens: {}", tokens.len());
            for token in &tokens {
                println!("    - {}: {}", token.surface, token.pos);
            }
        }
    }

    // 남은 버퍼 flush
    let remaining = stream.flush();
    if !remaining.is_empty() {
        println!("  Flushed remaining tokens: {}", remaining.len());
        for token in &remaining {
            println!("    - {}: {}", token.surface, token.pos);
        }
    }

    println!(
        "  Total chars processed: {}",
        stream.total_chars_processed()
    );

    Ok(())
}

fn chunk_processing() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new()?;
    let mut stream = StreamingTokenizer::new(tokenizer).with_chunk_size(1024);

    let long_text = "한국어 형태소 분석은 자연어 처리의 기본입니다. \
                     문장을 형태소 단위로 분석하여 의미를 파악합니다. \
                     이를 통해 다양한 응용 프로그램을 개발할 수 있습니다.";

    // 작은 청크로 분할
    let chunk_size = 50;
    for (i, chunk) in long_text.as_bytes().chunks(chunk_size).enumerate() {
        let chunk_str = std::str::from_utf8(chunk)?;
        let tokens = stream.process_chunk(chunk_str);

        println!("  Chunk {i}: {chunk_str:?}");
        println!("  Tokens: {}", tokens.len());
    }

    let remaining = stream.flush();
    println!("  Final flush: {} tokens", remaining.len());

    Ok(())
}

fn file_streaming_simulation() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new()?;
    let mut stream = StreamingTokenizer::new(tokenizer);

    // 파일 읽기를 시뮬레이션
    let simulated_file_content = vec![
        "첫 번째 문장입니다.\n",
        "두 번째 문장은 조금 더 길게 작성되었습니다.\n",
        "세 번째 문장.\n",
        "마지막 문장입니다.",
    ];

    let mut total_tokens = 0;

    for line in simulated_file_content {
        let tokens = stream.process_chunk(line);
        total_tokens += tokens.len();

        if !tokens.is_empty() {
            println!("  Line processed: {} tokens", tokens.len());
        }
    }

    let remaining = stream.flush();
    total_tokens += remaining.len();

    println!("  Total tokens from file: {total_tokens}");
    println!(
        "  Total chars processed: {}",
        stream.total_chars_processed()
    );

    Ok(())
}

fn iterator_streaming() -> Result<(), Box<dyn std::error::Error>> {
    use mecab_ko_core::streaming::TokenStream;

    let tokenizer = Tokenizer::new()?;

    let chunks = vec![
        "안녕하세요.\n".to_string(),
        "반갑습니다.\n".to_string(),
        "좋은 하루 되세요.\n".to_string(),
    ];

    let stream = TokenStream::new(chunks.into_iter(), tokenizer);

    println!("  Processing with iterator:");
    for (i, token) in stream.enumerate() {
        println!("  Token {i}: {} ({})", token.surface, token.pos);
    }

    Ok(())
}
