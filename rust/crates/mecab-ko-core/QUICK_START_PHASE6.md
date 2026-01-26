# Phase 6 Quick Start Guide

## 빠른 시작

### 1. 스트리밍 처리 (대용량 파일)

```rust
use mecab_ko_core::streaming::StreamingTokenizer;
use mecab_ko_core::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new()?;
    let mut stream = StreamingTokenizer::new(tokenizer);

    // 파일 스트리밍
    let tokens = stream.process_file("large_file.txt")?;

    println!("Total tokens: {}", tokens.len());
    Ok(())
}
```

### 2. 비동기 처리 (웹 서비스)

```rust
use mecab_ko_core::async_tokenizer::AsyncTokenizer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = AsyncTokenizer::new().await?;

    // 단일 텍스트
    let tokens = tokenizer.tokenize_async("안녕하세요").await;

    // 배치 처리
    let texts = vec!["첫 번째".to_string(), "두 번째".to_string()];
    let results = tokenizer.tokenize_batch(texts).await;

    Ok(())
}
```

### 3. 병렬 배치 처리 (대량 문서)

```rust
use mecab_ko_core::batch::BatchTokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let batch = BatchTokenizer::new()?;

    let texts = vec!["문장 1", "문장 2", "문장 3"];
    let results = batch.tokenize_batch(&texts);

    for (text, tokens) in texts.iter().zip(results.iter()) {
        println!("{}: {} tokens", text, tokens.len());
    }

    Ok(())
}
```

## Cargo.toml 설정

```toml
[dependencies]
mecab-ko-core = "0.1"

# Async 기능 사용 시
mecab-ko-core = { version = "0.1", features = ["async"] }
```

## 기능 비교

| 기능 | Streaming | Async | Batch |
|-----|-----------|-------|-------|
| 메모리 | ⭐⭐⭐ 매우 효율적 | ⭐⭐ 효율적 | ⭐ 보통 |
| 속도 | ⭐ 순차 | ⭐⭐ I/O 최적화 | ⭐⭐⭐ 매우 빠름 |
| 사용 난이도 | 쉬움 | 중간 | 쉬움 |
| 용도 | 대용량 파일 | 웹 서비스 | 배치 작업 |

## 선택 가이드

### Streaming을 사용하세요:
- 매우 큰 파일 (GB 단위)
- 메모리 제약이 있는 환경
- 실시간 스트림 처리

### Async를 사용하세요:
- 웹 서버/API
- 네트워크 I/O가 많은 경우
- 여러 파일 동시 처리

### Batch를 사용하세요:
- CPU 집약적 배치 작업
- 많은 짧은 텍스트 처리
- 최대 성능이 필요한 경우

## 예제 실행

```bash
# Streaming
cargo run --example streaming_example

# Batch
cargo run --example batch_example

# Async (async feature 필요)
cargo run --example async_example --features async
```

## API 치트시트

### StreamingTokenizer
```rust
let mut stream = StreamingTokenizer::new(tokenizer)
    .with_chunk_size(8192)
    .with_sentence_delimiters(vec!['.', '!', '?']);

let tokens = stream.process_chunk(chunk);
let remaining = stream.flush();
```

### AsyncTokenizer
```rust
let tokenizer = AsyncTokenizer::new().await?
    .with_max_concurrent(4);

let tokens = tokenizer.tokenize_async(text).await;
let results = tokenizer.tokenize_batch(texts).await;
```

### BatchTokenizer
```rust
let batch = BatchTokenizer::with_pool_size(8)?;

let results = batch.tokenize_batch(&texts);
let tokens = batch.tokenize_chunked(text, 100);
```

## 일반적인 패턴

### 1. 파일 처리
```rust
// Streaming
let tokens = stream.process_file("file.txt")?;

// Async
let tokens = tokenizer.tokenize_file("file.txt").await?;
```

### 2. Iterator 사용
```rust
use mecab_ko_core::streaming::TokenStream;

let chunks = vec!["chunk1".to_string(), "chunk2".to_string()];
let stream = TokenStream::new(chunks.into_iter(), tokenizer);

for token in stream {
    println!("{}", token.surface);
}
```

### 3. 에러 처리
```rust
match tokenizer.tokenize_async(text).await {
    Ok(tokens) => process(tokens),
    Err(e) => eprintln!("Error: {}", e),
}
```

## 성능 팁

1. **Streaming**: 청크 크기를 적절히 설정 (기본 8KB)
2. **Async**: `max_concurrent`를 CPU 코어 수에 맞춤
3. **Batch**: `pool_size`를 CPU 코어 수로 설정

## 추가 정보

- 📖 전체 문서: `README_PHASE6.md`
- 🔧 구현 세부사항: `PHASE6_IMPLEMENTATION.md`
- ✅ 체크리스트: `PHASE6_CHECKLIST.md`

## 지원

- GitHub Issues: 버그 리포트 및 기능 제안
- Documentation: `cargo doc --open`
