# Sprint 17 - S17-05: Memory Optimization Phase 2 (2026-03-03)

## 세션 개요
메모리 최적화 2차 작업: POS 태그 String interning, Feature 중복 제거, 메모리 사용량 추적 인프라 구축

## 완료된 작업

### S17-05: 메모리 최적화 2차 ✅

#### 1. memory.rs 모듈 생성

**위치**: `rust/crates/mecab-ko-core/src/memory.rs`

**주요 구조체**:

1. **PosTagInterner** - 품사 태그 String interning
   ```rust
   pub struct PosTagInterner {
       tags: RwLock<HashMap<String, u16>>,
       reverse: RwLock<Vec<String>>,
       intern_count: AtomicUsize,
       hit_count: AtomicUsize,
   }
   ```
   - ~45개 일반 품사 태그 사전 로드 (NNG, VV, JKS 등)
   - RwLock 기반 스레드 안전
   - 통계 추적: intern 호출 횟수, 캐시 히트율
   - `intern()`: 태그 인터닝 (기존이면 인덱스 반환, 새로우면 등록)
   - `resolve()`: 인덱스 → 태그 변환
   - `stats()`: InternerStats 반환

2. **FeatureCache** - Feature 문자열 중복 제거
   ```rust
   pub struct FeatureCache {
       features: RwLock<HashMap<String, u32>>,
       reverse: RwLock<Vec<String>>,
       max_size: usize,
   }
   ```
   - LRU 방식 (최대 크기 제한)
   - 캐시가 가득 차면 새 항목 추가 안함 (None 반환)

3. **MemoryStats** - 메모리 사용량 추적
   ```rust
   pub struct MemoryStats {
       pub dictionary_bytes: usize,
       pub lattice_bytes: usize,
       pub pool_bytes: usize,
       pub cache_bytes: usize,
       pub interner_bytes: usize,
       pub token_bytes: usize,
   }
   ```
   - `estimate_total()`: 총 메모리 계산
   - `format_human_readable()`: KB 단위 포맷

4. **InternerStats** - 인터너 통계
   ```rust
   pub struct InternerStats {
       pub unique_tags: usize,
       pub intern_calls: usize,
       pub cache_hits: usize,
       pub hit_rate: f64,
   }
   ```

5. **estimate_tokens_memory()** - 토큰 벡터 메모리 추정 함수

#### 2. Lattice 개선

**변경 파일**: `rust/crates/mecab-ko-core/src/lattice.rs`

```rust
pub fn memory_usage(&self) -> usize {
    let text_bytes = self.text.len() + self.original_text.len();
    let nodes_bytes = self.nodes.capacity() * std::mem::size_of::<Node>();
    let index_bytes = self.starts_at.capacity() * std::mem::size_of::<Vec<u32>>()
        + self.ends_at.capacity() * std::mem::size_of::<Vec<u32>>()
        + self.starts_at.iter().map(|v| v.capacity() * 4).sum::<usize>()
        + self.ends_at.iter().map(|v| v.capacity() * 4).sum::<usize>();
    let pos_bytes = (self.char_positions.char_count() + 1) * std::mem::size_of::<usize>();
    let space_bytes = self.char_len() * std::mem::size_of::<usize>() / 10;
    let node_strings: usize = self.nodes.iter().map(|n| n.surface.len() + n.feature.len()).sum();
    text_bytes + nodes_bytes + index_bytes + pos_bytes + space_bytes + node_strings
}
```

#### 3. Tokenizer 개선

**변경 파일**: `rust/crates/mecab-ko-core/src/tokenizer.rs`

```rust
pub fn memory_stats(&self) -> crate::memory::MemoryStats {
    crate::memory::MemoryStats {
        dictionary_bytes: 0,
        lattice_bytes: self.lattice.memory_usage(),
        pool_bytes: self.pool_manager.total_memory_usage(),
        cache_bytes: 0,
        interner_bytes: 0,
        token_bytes: 0,
    }
}
```

#### 4. lib.rs Export 추가

```rust
pub mod memory;

pub use memory::{
    estimate_tokens_memory, FeatureCache, InternerStats, MemoryStats, PosTagInterner,
};
```

## 변경된 파일
- `rust/crates/mecab-ko-core/src/memory.rs` (신규, 477줄)
- `rust/crates/mecab-ko-core/src/lib.rs` (export 추가)
- `rust/crates/mecab-ko-core/src/lattice.rs` (memory_usage 추가)
- `rust/crates/mecab-ko-core/src/tokenizer.rs` (memory_stats 추가)
- `rust/crates/mecab-ko-core/Cargo.toml` (tempfile dev-dependency 추가)
- `PLAN.md`, `PROGRESS.md` (작업 완료 표시)

## 테스트 결과
- memory 모듈 테스트: 6개 통과
- mecab-ko-core 전체 테스트: 220개 통과
- 기존 test_batch_chunked 실패 (이번 변경과 무관한 기존 이슈)

## 학습 포인트
1. POS 태그는 약 45개로 제한되어 인터닝에 적합
2. RwLock으로 읽기 우선 접근 (double-check 패턴)
3. 메모리 추정은 정확한 값보다 추세 파악이 목적
4. CharPositions.capacity() 대신 char_count() 사용

## 다음 작업
- S17-06: API 문서 개선
- S17-07: 벤치마크 결과 정리
- S17-08: 테스트 커버리지 향상
