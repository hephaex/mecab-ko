# Phase 6: 메모리 풀링 최적화

## 개요

Phase 6에서는 프로덕션 환경에서의 성능 향상을 위해 메모리 할당 오버헤드를 최소화하는 풀링 시스템을 구현했습니다.

## 구현된 최적화

### 1. 객체 풀링 (Object Pooling)

#### Token Pool
- **목적**: Token 객체 재사용으로 할당/해제 오버헤드 감소
- **구현**: `TokenPool` - RefCell 기반 스레드 로컬 풀
- **특징**:
  - 자동 용량 관리 (초기 128개, 최대 256개)
  - 반환 시 자동 초기화
  - 메모리 사용량 추적

```rust
use mecab_ko_core::pool::TokenPool;

let pool = TokenPool::new();

// 획득
let mut token = pool.acquire();
token.surface = "안녕".to_string();
token.pos = "NNG".to_string();

// 사용 후 반환
pool.release(token);

// 재사용 (할당 없음)
let token2 = pool.acquire();
```

#### Node Vector Pool
- **목적**: Lattice 구축 시 Node 벡터 재사용
- **구현**: `NodeVecPool` - 벡터 용량 유지하며 재사용
- **효과**: Lattice 재설정 시 재할당 최소화

#### ID Vector Pool
- **목적**: `starts_at`, `ends_at` 벡터 재사용
- **구현**: `IdVecPool` - u32 벡터 풀링
- **효과**: 위치 인덱싱 벡터 재할당 방지

### 2. String Interning

#### SharedStringInterner
- **목적**: 중복 문자열 제거로 메모리 사용량 감소
- **구현**: `string-interner` 크레이트 + `Arc<Mutex<>>`로 스레드 안전성 보장
- **적용 대상**:
  - 품사 태그 (NNG, VV, JKS 등) - 높은 중복도
  - Feature 문자열 - 반복 패턴
  - 표면형 - 일부 중복

```rust
use mecab_ko_core::pool::SharedStringInterner;

let interner = SharedStringInterner::new();

// 같은 문자열은 같은 심볼로 매핑
let s1 = interner.intern("NNG");
let s2 = interner.intern("NNG");
assert_eq!(s1, s2);

// 메모리 절약
let s3 = interner.intern("VV");
assert_ne!(s1, s3);
```

### 3. Lattice 메모리 최적화

- **재사용 전략**: Tokenizer에서 Lattice 인스턴스 재사용
- **용량 유지**: `reset()` 시 벡터 용량 유지
- **효과**: 매 분석마다 재할당 방지

### 4. 통합 풀 관리자 (PoolManager)

모든 풀을 하나의 인터페이스로 관리:

```rust
use mecab_ko_core::pool::PoolManager;

let manager = PoolManager::new();

// 통계 조회
let stats = manager.stats();
println!("{}", stats.format_human_readable());

// 메모리 정리
manager.clear_all();
```

## 성능 측정

### 벤치마크 실행

```bash
# 전체 메모리 풀링 벤치마크
cargo bench --bench memory_pool_bench

# 특정 그룹만 실행
cargo bench --bench memory_pool_bench -- token_allocation
cargo bench --bench memory_pool_bench -- string_interning
cargo bench --bench memory_pool_bench -- real_world
```

### 측정 항목

1. **할당 오버헤드**
   - Token 직접 생성 vs 풀 사용
   - 배치 처리 시 성능 차이

2. **String Interning 효과**
   - 메모리 사용량 감소
   - 조회 성능

3. **실제 워크로드**
   - Tokenizer 연속 분석
   - 풀 재사용률
   - 메모리 압박 상황

4. **동시성**
   - 멀티스레드 환경에서의 풀 성능
   - Lock contention 측정

## 사용 가이드

### Tokenizer에서 풀 활용

```rust
use mecab_ko_core::Tokenizer;

let mut tokenizer = Tokenizer::new()?;

// 여러 문장 연속 분석 (높은 재사용률)
for sentence in sentences {
    let tokens = tokenizer.tokenize(sentence);
    // 처리...
}

// 풀 통계 확인
let stats = tokenizer.pool_stats();
println!("Pool Stats: {}", stats.format_human_readable());

// 장기 실행 프로세스에서 주기적 메모리 정리
tokenizer.clear_pools();
```

### 메모리 사용량 모니터링

```rust
// 초기 상태
let stats = tokenizer.pool_stats();
println!("Initial memory: {} KB", stats.total_memory / 1024);

// 분석 수행
for _ in 0..1000 {
    tokenizer.tokenize("테스트 문장");
}

// 풀 증가 확인
let stats = tokenizer.pool_stats();
println!("After 1000 calls:");
println!("  Token pool: {}", stats.token_pool_size);
println!("  Interned strings: {}", stats.interned_strings);
println!("  Total memory: {} KB", stats.total_memory / 1024);
```

## 최적화 효과

### 예상 성능 향상

1. **할당 횟수 감소**
   - 기존: 매 분석마다 N개 Token 할당
   - 최적화 후: 첫 분석 후 재사용 → ~90% 할당 감소

2. **메모리 사용량 감소**
   - String interning으로 중복 문자열 제거
   - 품사 태그 (수백 종류) → 심볼 참조
   - 예상 감소율: 20-40% (텍스트 특성에 따라)

3. **캐시 효율성**
   - 객체 재사용으로 캐시 locality 향상
   - 메모리 단편화 감소

### 트레이드오프

1. **메모리 점유**
   - 풀이 메모리를 보유하므로 최대 사용량 증가 가능
   - `clear_pools()`로 주기적 정리 필요

2. **복잡도**
   - 풀 관리 오버헤드
   - RefCell 동적 borrowing 체크

3. **스레드 안전성**
   - TokenPool은 스레드 로컬 (멀티스레드 시 각 스레드마다 풀)
   - SharedStringInterner는 Arc<Mutex<>> (lock contention 가능)

## 설계 결정

### RefCell vs Arc<Mutex<>>

- **TokenPool, NodeVecPool**: `RefCell`
  - 이유: 단일 스레드 내 순차 사용 (Tokenizer가 &mut self)
  - 장점: 낮은 오버헤드, 간단한 구현

- **SharedStringInterner**: `Arc<Mutex<>>`
  - 이유: 여러 Tokenizer 인스턴스 간 공유 가능성
  - 장점: 스레드 안전, Clone 가능

### 풀 크기 제한

- **자동 제한**: `max_size = capacity * 2`
- **이유**: 무한 증가 방지
- **조정**: 필요시 `with_capacity()`로 초기 크기 설정

### String Interning 전략

- **타이밍**: 현재는 수동 interning (필요시 호출)
- **미래**: Tokenizer 내부에서 자동 interning 고려
- **Trade-off**: 자동화 시 오버헤드 vs 메모리 절약

## 다음 단계

### Phase 7 예정 사항

1. **자동 String Interning**
   - Token 생성 시 자동으로 품사 태그 intern
   - Feature 문자열 자동 intern

2. **Arena Allocator**
   - `typed-arena` 도입 검토
   - Lattice 전체를 단일 arena에 할당

3. **Zero-Copy 최적화**
   - Cow<'static, str> 활용 확대
   - 사전 데이터 직접 참조

4. **프로파일링 기반 최적화**
   - 실제 워크로드 분석
   - Hot path 식별 및 최적화

## 테스트

```bash
# 풀 모듈 단위 테스트
cargo test --lib pool

# 통합 테스트
cargo test --test integration_test_pooling

# 벤치마크
cargo bench --bench memory_pool_bench
```

## 참고 자료

- [Object Pool Pattern](https://en.wikipedia.org/wiki/Object_pool_pattern)
- [String Interning](https://en.wikipedia.org/wiki/String_interning)
- [string-interner crate](https://docs.rs/string-interner/)
- [Rust Performance Book - Memory Management](https://nnethercote.github.io/perf-book/heap-allocations.html)

## 기여

풀링 최적화 개선 아이디어나 버그 발견 시 이슈를 열어주세요.

## 라이선스

MIT OR Apache-2.0
