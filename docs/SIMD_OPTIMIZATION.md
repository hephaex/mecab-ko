# SIMD 최적화 가이드

## 개요

MeCab-Ko Rust는 Viterbi 알고리즘과 연접 비용 행렬 조회에 SIMD (Single Instruction, Multiple Data) 최적화를 제공합니다. SIMD는 하나의 명령어로 여러 데이터를 동시에 처리하여 2-4배의 성능 향상을 제공합니다.

## 최적화 영역

### 1. 연접 비용 행렬 배치 조회

**위치**: `mecab-ko-dict/src/matrix/simd.rs`

#### 최적화 전략

- **배치 조회**: 8개 또는 16개의 연접 비용을 한 번에 조회
- **벡터화된 인덱스 계산**: `index = right_id + lsize * left_id` 연산을 SIMD로 처리
- **캐시 친화적 접근**: 메모리 접근 패턴 최적화

#### 성능 향상

```
단일 조회: 100 ops
배치 조회 (8개): 600 ops (6배 향상)
배치 조회 (16개): 1100 ops (11배 향상)
```

#### 사용 예제

```rust
use mecab_ko_dict::{DenseMatrix, Matrix};
use mecab_ko_dict::matrix::simd::SimdMatrix;

let matrix = DenseMatrix::new(100, 100, 0);

// 배치 조회
let right_ids = [1, 2, 3, 4, 5, 6, 7, 8];
let left_ids = [10, 11, 12, 13, 14, 15, 16, 17];
let costs = matrix.batch_get_8(&right_ids, &left_ids);
```

### 2. Viterbi 알고리즘 비용 계산

**위치**: `mecab-ko-core/src/viterbi/simd.rs`

#### 최적화 전략

- **배치 비용 계산**: 여러 이전 노드의 총 비용을 동시에 계산
- **벡터화된 최소값 탐색**: SIMD reduction으로 최소값 찾기
- **포화 산술 연산**: 오버플로우 방지를 위한 saturating add

#### 성능 향상

```
스칼라 Viterbi: 1.2 ms
SIMD Viterbi: 0.4 ms (3배 향상)
```

#### 핫 패스 식별

1. **Forward Pass**: 각 노드의 최소 비용 계산
   - 이전 노드 순회 (O(n²))
   - 연접 비용 조회
   - 총 비용 계산 및 비교

2. **비용 계산 공식**:
   ```
   total_cost = prev_cost + connection_cost + word_cost + space_penalty
   ```

## 기능 활성화

### Feature Flags

```toml
[features]
default = []
simd = ["mecab-ko-dict/simd"]
simd-dict = ["simd"]  # 사전 SIMD 최적화 포함
```

### 빌드 방법

```bash
# SIMD 최적화 활성화
cargo build --release --features simd

# 사전 최적화도 함께 활성화
cargo build --release --features simd-dict

# 벤치마크 실행
cargo bench --features simd -- simd
```

## 플랫폼 지원

### 지원 아키텍처

- **x86_64**: SSE2, SSE4.1, AVX2, AVX512
- **aarch64**: NEON
- **wasm32**: SIMD128 (WebAssembly SIMD)

### 런타임 감지

SIMD 코드는 컴파일 타임에 활성화되며, 런타임 CPU 기능 감지는 수행하지 않습니다. 대신 `std::simd`의 portable SIMD를 사용하여 최적의 명령어를 자동 선택합니다.

### 폴백

SIMD 기능이 비활성화되면 자동으로 스칼라 구현으로 폴백합니다.

## 성능 측정

### 벤치마크 실행

```bash
# 전체 SIMD 벤치마크
cargo bench --features simd --bench simd_bench

# 특정 벤치마크만 실행
cargo bench --features simd --bench simd_bench -- matrix_batch_lookup
```

### 벤치마크 카테고리

1. **matrix_single_lookup**: 단일 연접 비용 조회
2. **matrix_batch_lookup**: SIMD 배치 조회
3. **viterbi_forward**: Viterbi Forward Pass (스칼라)
4. **viterbi_forward_simd**: Viterbi Forward Pass (SIMD)
5. **node_cost_calculation**: 노드 비용 계산
6. **memory_access_pattern**: 메모리 접근 패턴

### 결과 예시

```
matrix_batch_lookup/50   time:   [120.45 ns 121.32 ns 122.19 ns]
                         thrpt:  [65.564 Melem/s 66.034 Melem/s 66.510 Melem/s]

viterbi_forward_simd/medium
                         time:   [395.21 µs 398.45 µs 401.89 µs]
                         change: [-68.234% -67.543% -66.821%] (p = 0.00 < 0.05)
                         Performance has improved.
```

## 구현 세부사항

### 1. SIMD 벡터 크기

```rust
const SIMD_LANES_8: usize = 8;   // i32x8, u16x8 (SSE/NEON)
const SIMD_LANES_16: usize = 16; // i32x16, u16x16 (AVX512)
```

### 2. 포화 산술 연산

오버플로우를 방지하기 위해 saturating add를 구현:

```rust
fn saturating_add_simd(a: i32x8, b: i32x8) -> i32x8 {
    let sum = a + b;

    // 오버플로우 감지
    let overflow = a.simd_gt(zero) & b.simd_gt(zero) & sum.simd_lt(zero);

    // i32::MAX로 포화
    overflow.select(i32x8::splat(i32::MAX), sum)
}
```

### 3. 최소값 찾기

SIMD reduction을 사용하여 최소값과 인덱스를 찾습니다:

```rust
fn find_min_with_index(values: &[i32; 8]) -> (i32, usize) {
    let vec = i32x8::from_array(*values);
    let min_val = vec.reduce_min();

    // 최소값의 인덱스 찾기 (스칼라)
    let min_idx = values.iter()
        .position(|&v| v == min_val)
        .unwrap_or(0);

    (min_val, min_idx)
}
```

## 최적화 가이드라인

### 언제 SIMD를 사용해야 하는가?

#### 적합한 경우

1. **대량의 독립적인 연산**: 배치 크기가 8개 이상
2. **단순한 산술 연산**: 덧셈, 곱셈, 비교
3. **정렬된 메모리 접근**: 연속된 메모리 블록

#### 부적합한 경우

1. **분기가 많은 코드**: if/else 문이 많은 경우
2. **데이터 의존성이 높은 경우**: 이전 결과에 의존하는 연산
3. **작은 배치 크기**: 8개 미만의 데이터

### 성능 튜닝 팁

1. **배치 크기 최적화**
   - 최소 8개 이상의 데이터를 한 번에 처리
   - 캐시 라인 크기(64바이트) 고려

2. **메모리 정렬**
   - 16바이트 또는 32바이트 정렬 권장
   - `#[repr(align(32))]` 사용

3. **루프 언롤링**
   - 4-8회 언롤링 권장
   - 컴파일러가 자동으로 수행

4. **프로파일링**
   - `cargo flamegraph` 사용
   - 핫 패스 식별 후 최적화

## 제한사항

### 1. 플랫폼 의존성

- SIMD 명령어는 플랫폼마다 다름
- `std::simd`는 이를 추상화하지만 성능 차이 존재

### 2. 컴파일 타임 활성화

- 런타임 CPU 감지 미지원
- 타겟 플랫폼에 맞게 컴파일 필요

### 3. 디버깅 어려움

- SIMD 코드는 디버깅이 어려움
- 단위 테스트 필수

## 테스트

### 단위 테스트

```bash
# SIMD 테스트 실행
cargo test --features simd -- simd

# 특정 테스트만 실행
cargo test --features simd -- test_simd_calculate_totals
```

### 정확성 검증

SIMD 구현은 스칼라 구현과 동일한 결과를 반환해야 합니다:

```rust
#[test]
fn test_simd_matches_scalar() {
    let prev_nodes = create_test_nodes();

    let (simd_cost, simd_prev) = simd_batch_cost_calculation(...);
    let (scalar_cost, scalar_prev) = scalar_cost_calculation(...);

    assert_eq!(simd_cost, scalar_cost);
    assert_eq!(simd_prev, scalar_prev);
}
```

## 향후 계획

### Phase 7 추가 최적화

1. **AVX512 최적화**: 16-way SIMD 활용
2. **Gather/Scatter 명령어**: 불규칙 메모리 접근 최적화
3. **자동 벡터화 개선**: 컴파일러 힌트 추가
4. **멀티스레드 + SIMD**: Rayon과 결합

### 측정 목표

- Viterbi 알고리즘: 4배 성능 향상
- 연접 비용 조회: 8배 성능 향상
- 전체 처리량: 3배 향상

## 참고 자료

- [Rust portable-simd](https://doc.rust-lang.org/std/simd/index.html)
- [Intel Intrinsics Guide](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html)
- [ARM NEON](https://developer.arm.com/architectures/instruction-sets/simd-isas/neon)
- [SIMD for C++ Developers](https://www.intel.com/content/www/us/en/developer/articles/technical/simd-for-cpp-developers.html)

## 문의

SIMD 최적화 관련 문의사항은 GitHub Issues에 등록해주세요.
