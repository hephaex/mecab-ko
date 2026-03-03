# Sprint 17 - S17-03: 스트리밍 API 개선 (2026-03-03)

## 세션 개요
스트리밍 토크나이저 API 성능 최적화 및 대용량 파일 처리 개선

## 완료된 작업

### S17-03: 스트리밍 API 개선 ✅

#### 1. TokenStream 성능 최적화
- **문제**: `Vec::remove(0)` 사용으로 O(n) 비효율성
- **해결**: `VecDeque` 사용으로 O(1) `pop_front` 달성
- **추가**: `size_hint()`, `tokens_yielded()` 메서드

```rust
// Before: O(n) dequeue
self.token_buffer.remove(0)

// After: O(1) dequeue
self.token_buffer.pop_front()
```

#### 2. 진행률 콜백 시스템
새로운 `ProgressStreamingTokenizer` 구현:

```rust
pub struct StreamingProgress {
    pub bytes_processed: usize,
    pub total_bytes: Option<usize>,
    pub tokens_generated: usize,
    pub chunks_processed: usize,
}

// 사용 예
let stream = ProgressStreamingTokenizer::new(tokenizer)
    .with_progress_callback(|progress| {
        println!("{}%", progress.percent().unwrap_or(0.0));
    });
```

#### 3. 스마트 문장 경계 청킹
`split_into_chunks_smart()` 구현:
- 단어 중간 분할 방지
- 문장 구분자(`. ! ? 。 ． \n`)에서 분할
- 구분자 미발견 시 공백에서 분할

```rust
// 오버랩 청킹 지원
BatchTokenizer::split_with_overlap(text, chunk_size, overlap)
```

#### 4. 대용량 파일 프로세서
`LargeFileProcessor` 구현:
- 버퍼 기반 스트리밍 (메모리 효율적)
- 진행률 콜백 지원
- Rayon 기반 병렬 파일 처리

```rust
let processor = LargeFileProcessor::new()?
    .with_buffer_size(65536)
    .with_progress_callback(|p| println!("{}%", p.percent()));

let tokens = processor.process_file("large_file.txt")?;
```

#### 5. ChunkedTokenIterator
토큰을 청크 단위로 반환하여 메모리 효율성 향상:

```rust
let iter = ChunkedTokenIterator::new(chunks.into_iter(), tokenizer);
for token_batch in iter {
    process_batch(token_batch);
}
```

## 새로운 API

### streaming 모듈
- `ProgressCallback` - 진행률 콜백 타입
- `StreamingProgress` - 진행 상황 구조체
- `ProgressStreamingTokenizer` - 진행률 추적 토크나이저
- `ChunkedTokenIterator` - 청크별 토큰 이터레이터

### batch 모듈
- `LargeFileProcessor` - 대용량 파일 프로세서
- `LargeFileProgress` - 대용량 파일 진행 상황
- `split_into_chunks_smart()` - 스마트 청킹
- `split_with_overlap()` - 오버랩 청킹

## 변경된 파일
- `rust/crates/mecab-ko-core/src/streaming.rs` - 스트리밍 API 개선
- `rust/crates/mecab-ko-core/src/batch.rs` - 배치 처리 개선
- `rust/crates/mecab-ko-core/src/lib.rs` - 새 타입 export
- `PLAN.md` - S17-03 완료 표시
- `PROGRESS.md` - 진행 상황 업데이트

## 테스트 결과
- **총 테스트**: 213개 통과
- **Clippy**: 경고 없음
- **새 테스트**:
  - `test_token_stream_tokens_yielded`
  - `test_token_stream_size_hint`
  - `test_progress_streaming_tokenizer`
  - `test_progress_callback`
  - `test_progress_percent`
  - `test_chunked_token_iterator`
  - `test_smart_chunking_respects_sentence_boundary`
  - `test_split_with_overlap`
  - `test_large_file_processor_creation`
  - `test_large_file_progress_percent`

## 성능 개선 요약

| 항목 | 개선 전 | 개선 후 |
|------|--------|--------|
| TokenStream dequeue | O(n) | O(1) |
| 문장 경계 탐색 | 순방향 | 역방향 (빠름) |
| 청크 분할 | 임의 위치 | 구분자 기준 |
| 대용량 파일 | 전체 로드 | 스트리밍 |

## 커밋
```
feat(core): optimize streaming API for performance and memory
```

## 학습 포인트
1. `VecDeque`는 양방향 큐 작업에서 `Vec`보다 효율적
2. 역방향 탐색(`rfind` 패턴)이 마지막 위치 찾기에 적합
3. 진행률 콜백은 `Send + Sync` 바운드 필요 (Rayon 병렬 처리)
4. 파일 크기 캐스팅 시 `cast_possible_truncation` 주의

## 다음 작업
- S17-02: PyPI 배포 (BLOCKED - 토큰 필요)
- S17-04: Migration Guide v0.2.0 → v0.3.0
- S17-05: 메모리 최적화 2차
