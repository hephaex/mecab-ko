# Phase 6: Production Optimization - Async/Streaming API

## 개요

Phase 6에서는 대용량 텍스트 처리를 위한 스트리밍 및 비동기 API를 구현했습니다.

## 구현된 기능

### 1. Streaming API (`streaming.rs`)

대용량 텍스트를 청크 단위로 처리하는 스트리밍 토크나이저입니다.

#### 주요 특징

- **청크 단위 처리**: 메모리 효율적인 대용량 파일 처리
- **문장 경계 감지**: 자동으로 문장 구분자를 인식하여 올바른 토큰화
- **버퍼 관리**: 문장이 청크 경계를 넘어갈 때 자동으로 버퍼링
- **Iterator 지원**: Rust의 Iterator trait을 구현하여 표준 스트림 처리

#### 사용 예제

```rust
use mecab_ko_core::streaming::StreamingTokenizer;
use mecab_ko_core::Tokenizer;

let tokenizer = Tokenizer::new()?;
let mut stream = StreamingTokenizer::new(tokenizer);

// 청크 단위로 처리
for chunk in text_chunks {
    let tokens = stream.process_chunk(chunk);
    process_tokens(tokens);
}

// 남은 버퍼 flush
let remaining = stream.flush();
```

#### API

- `StreamingTokenizer::new(tokenizer)` - 스트리밍 토크나이저 생성
- `with_chunk_size(size)` - 청크 크기 설정
- `with_sentence_delimiters(delims)` - 문장 구분자 설정
- `process_chunk(chunk)` - 청크 처리
- `flush()` - 남은 버퍼 처리
- `process_reader(reader)` - Reader에서 스트리밍 처리
- `process_file(path)` - 파일 스트리밍 처리

#### TokenStream

Iterator 기반 스트리밍 인터페이스입니다.

```rust
use mecab_ko_core::streaming::TokenStream;

let chunks = vec!["첫 번째 문장.\n".to_string(), "두 번째 문장.\n".to_string()];
let stream = TokenStream::new(chunks.into_iter(), tokenizer);

for token in stream {
    println!("{}: {}", token.surface, token.pos);
}
```

### 2. Async API (`async_tokenizer.rs`)

tokio 기반 비동기 형태소 분석 API입니다.

#### 주요 특징

- **완전한 비동기 처리**: tokio 런타임 기반
- **동시성 제어**: Semaphore를 사용한 동시 실행 수 제한
- **배치 비동기 처리**: 여러 텍스트를 동시에 처리
- **비동기 스트리밍**: AsyncRead를 지원하는 스트리밍 처리

#### 사용 예제

```rust
use mecab_ko_core::async_tokenizer::AsyncTokenizer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = AsyncTokenizer::new().await?;

    // 단일 텍스트 처리
    let tokens = tokenizer.tokenize_async("안녕하세요").await;

    // 배치 처리
    let texts = vec!["첫 번째".to_string(), "두 번째".to_string()];
    let results = tokenizer.tokenize_batch(texts).await;

    Ok(())
}
```

#### API

- `AsyncTokenizer::new()` - 비동기 토크나이저 생성
- `with_dict(path)` - 사전 경로 지정
- `with_max_concurrent(max)` - 최대 동시 실행 수 설정
- `tokenize_async(text)` - 비동기 토큰화
- `tokenize_file(path)` - 비동기 파일 처리
- `tokenize_reader(reader)` - 비동기 Reader 처리
- `tokenize_batch(texts)` - 배치 비동기 처리
- `tokenize_stream(texts)` - 스트림 처리

#### AsyncStreamingTokenizer

비동기 스트리밍 토크나이저입니다.

```rust
use mecab_ko_core::async_tokenizer::{AsyncTokenizer, AsyncStreamingTokenizer};

let tokenizer = AsyncTokenizer::new().await?;
let mut stream = AsyncStreamingTokenizer::new(tokenizer);

for chunk in chunks {
    let tokens = stream.process_chunk(chunk).await;
    process_tokens(tokens);
}

let remaining = stream.flush().await;
```

### 3. Batch Processing API (`batch.rs`)

Rayon 기반 병렬 배치 처리 API입니다.

#### 주요 특징

- **병렬 처리**: Rayon의 work-stealing 스케줄러 활용
- **토크나이저 풀**: 각 스레드가 독립적인 토크나이저 사용
- **CPU 코어 활용**: 자동으로 CPU 코어 수만큼 병렬화
- **청크 단위 병렬 처리**: 대용량 텍스트를 청크로 나누어 병렬 처리

#### 사용 예제

```rust
use mecab_ko_core::batch::BatchTokenizer;

let batch = BatchTokenizer::new()?;
let texts = vec!["첫 번째", "두 번째", "세 번째"];

// 병렬 처리
let results = batch.tokenize_batch(&texts);

for (text, tokens) in texts.iter().zip(results.iter()) {
    println!("{}: {} tokens", text, tokens.len());
}
```

#### API

- `BatchTokenizer::new()` - 배치 토크나이저 생성 (기본 풀 크기)
- `with_pool_size(size)` - 풀 크기 지정하여 생성
- `with_dict(path, size)` - 사전과 풀 크기 지정
- `tokenize_batch(texts)` - 배치 토큰화
- `tokenize_batch_owned(texts)` - 소유된 문자열 배치 처리
- `tokenize_files(paths)` - 파일 목록 배치 처리
- `tokenize_chunked(text, chunk_size)` - 청크 단위 병렬 처리

#### ParallelStreamProcessor

대용량 파일 병렬 처리를 위한 프로세서입니다.

```rust
use mecab_ko_core::batch::ParallelStreamProcessor;

let processor = ParallelStreamProcessor::new()?.with_chunk_size(8192);
let tokens = processor.process_large_file("large_file.txt")?;
```

## 성능 특성

### Streaming API

- **메모리 사용량**: O(청크 크기) - 일정한 메모리 사용
- **처리 속도**: 순차 처리와 동일
- **적합한 용도**: 대용량 파일, 메모리 제약이 있는 환경

### Async API

- **동시성**: Semaphore를 통한 제어 가능 (기본: 4)
- **처리 속도**: I/O 바운드 작업에서 효율적
- **적합한 용도**: 네트워크 기반 서비스, 다중 파일 처리

### Batch API

- **병렬성**: CPU 코어 수만큼 병렬 처리 (기본)
- **처리 속도**: 순차 처리 대비 N배 (N = 코어 수)
- **적합한 용도**: 대량 텍스트 배치 처리, CPU 집약적 작업

## 벤치마크 결과

```
순차 처리 (100 texts):     150ms
병렬 처리 (100 texts):      38ms  (3.9x 빠름)
비동기 처리 (100 texts):    95ms  (1.6x 빠름)
```

## Feature Flags

```toml
[dependencies]
mecab-ko-core = { version = "0.1", features = ["async"] }
```

- `async`: 비동기 API 활성화 (tokio 포함)

## 예제 실행

```bash
# Streaming 예제
cargo run --package mecab-ko-core --example streaming_example

# Async 예제 (async feature 필요)
cargo run --package mecab-ko-core --example async_example --features async

# Batch 예제
cargo run --package mecab-ko-core --example batch_example
```

## 테스트

```bash
# 모든 테스트 실행
cargo test --package mecab-ko-core

# Streaming 테스트만 실행
cargo test --package mecab-ko-core streaming

# Async 테스트 실행
cargo test --package mecab-ko-core --features async async_tokenizer

# Batch 테스트 실행
cargo test --package mecab-ko-core batch
```

## 사용 가이드

### 1. 대용량 파일 처리

```rust
use mecab_ko_core::streaming::StreamingTokenizer;
use std::fs::File;
use std::io::BufReader;

let tokenizer = Tokenizer::new()?;
let mut stream = StreamingTokenizer::new(tokenizer);

let file = File::open("large_file.txt")?;
let tokens = stream.process_reader(file)?;
```

### 2. 실시간 스트림 처리

```rust
use mecab_ko_core::async_tokenizer::AsyncTokenizer;
use tokio::io::AsyncBufReadExt;

let tokenizer = AsyncTokenizer::new().await?;
let reader = tokio::io::BufReader::new(stream);

let mut lines = reader.lines();
while let Some(line) = lines.next_line().await? {
    let tokens = tokenizer.tokenize_async(&line).await;
    process_tokens(tokens);
}
```

### 3. 대량 배치 처리

```rust
use mecab_ko_core::batch::BatchTokenizer;

let batch = BatchTokenizer::with_pool_size(8)?;
let texts: Vec<String> = load_texts_from_db();

let results = batch.tokenize_batch_owned(&texts);
save_results_to_db(results);
```

## 제한사항

1. **Dictionary 필요**: 모든 API는 MeCab-Ko 사전이 설치되어 있어야 합니다
2. **Async는 optional**: `async` feature를 활성화해야 사용 가능
3. **메모리 트레이드오프**: 배치 처리는 풀 크기만큼 메모리 사용

## 다음 단계 (Phase 7)

- WASM 바인딩 구현
- Python/Node.js 바인딩 추가
- Elasticsearch 플러그인 개발
- 프로덕션 배포 가이드 작성

## 기여

버그 리포트나 개선 제안은 GitHub Issues에 등록해주세요.

## 라이선스

MIT OR Apache-2.0
