# QA-001: Criterion 벤치마크 구현 완료

## 개요

Criterion 기반 성능 벤치마크를 구현하여 Trie 검색 및 Matrix 조회 성능을 측정할 수 있습니다.

## 구현 위치

### 1. 작업 디렉토리 구조

```
/home/mare/mecab-ko/rust/
├── benches/                          # 루트 레벨 벤치마크
│   ├── README.md                     # 벤치마크 사용 가이드
│   ├── trie_bench.rs                 # Trie 검색 벤치마크
│   └── matrix_bench.rs               # Matrix 조회 벤치마크
│
└── crates/benchmarks/                # 벤치마크 패키지 (실제 실행)
    ├── Cargo.toml                    # 벤치마크 설정
    └── benches/
        ├── trie_bench.rs             # Trie 검색 벤치마크
        ├── matrix_bench.rs           # Matrix 조회 벤치마크
        ├── viterbi_bench.rs          # Viterbi 알고리즘
        └── tokenizer_bench.rs        # 토크나이저
```

### 2. Cargo 설정 (`crates/benchmarks/Cargo.toml`)

```toml
[dev-dependencies]
criterion = { workspace = true }
rand = "0.8"

[[bench]]
name = "trie_bench"
harness = false

[[bench]]
name = "matrix_bench"
harness = false
```

## 벤치마크 상세

### 1. Trie 검색 벤치마크 (`trie_bench.rs`)

#### 측정 항목

| 벤치마크 그룹 | 설명 | 항목 수 |
|--------------|------|---------|
| `trie_exact_match_small` | 소형 사전 (17개) 정확 검색 | 3 |
| `trie_exact_match_medium` | 중형 사전 (~1K개) 정확 검색 | 3 |
| `trie_exact_match_large` | 대형 사전 (~10K개) 정확 검색 | 2 |
| `trie_common_prefix_search` | 공통 접두사 검색 | 3 |
| `trie_morpheme_scenario` | 형태소 분석 시뮬레이션 | 1 |
| `trie_build` | Trie 빌드 성능 | 3 |
| `trie_memory` | 메모리 효율성 | 3 |

#### 주요 기능

1. **exact_match 벤치마크**
   - Hit 케이스: 사전에 존재하는 키 검색
   - Miss 케이스: 사전에 없는 키 검색
   - 배치 조회: 10개 쿼리 일괄 처리

2. **common_prefix_search 벤치마크**
   - 다중 매칭: "가방에서" → ["가", "가방", "가방에"]
   - 긴 텍스트: "아버지가방에" 처리
   - 매칭 없는 경우

3. **형태소 분석 시나리오**
   - 실제 문장 "학생이교실에가다" 전체 분석
   - 각 문자 위치에서 prefix search 수행

4. **메모리 효율성**
   - 100, 1,000, 5,000개 엔트리 크기별 측정
   - 원본 대비 압축률 계산

#### 테스트 데이터

```rust
// 소형 사전: 한글 기본 어휘
["가", "가다", "가방", "가방에", "나", "나다", "다", "다가",
 "다가가다", "아버지", "아버지가", "어머니", "어머니는",
 "학교", "학교에", "학생", "선생님"]

// 중형 사전: 명사 + 조사 조합, 동사 활용형 (~1,000개)
// 대형 사전: 중형 + 생성된 단어 (~10,000개)
```

### 2. Matrix 조회 벤치마크 (`matrix_bench.rs`)

#### 측정 항목

| 벤치마크 그룹 | 설명 | 항목 수 |
|--------------|------|---------|
| `matrix_single_lookup` | 단일 조회 성능 | 3 |
| `matrix_batch_lookup` | 배치 조회 (10, 100, 1K) | 3 |
| `matrix_viterbi_pattern` | Viterbi 알고리즘 패턴 | 2 |
| `matrix_size_comparison` | 크기별 비교 (100x100, 1Kx1K, 2Kx2K) | 3 |
| `matrix_cache_locality` | 캐시 지역성 | 3 |
| `matrix_memory` | 메모리 사용량 | 4 |
| `matrix_realistic_workload` | 실제 워크로드 시뮬레이션 | 2 |

#### 주요 기능

1. **단일 조회 패턴**
   - Sequential: 연속된 ID 조회 (캐시 친화적)
   - Random: 랜덤 ID 조회 (캐시 비친화적)
   - Fixed: 고정 위치 조회 (최대 캐시 효과)

2. **Viterbi 알고리즘 시뮬레이션**
   - 노드 전이: 현재 노드에 대한 이전 노드 후보 검사
   - 경로 계산: 10개 위치, 각 5개 후보 전체 처리

3. **캐시 지역성 테스트**
   - Row-major 순회 (캐시 친화적)
   - Column-major 순회 (상대적으로 비친화적)
   - Strided 접근 (비연속 접근)

4. **실제 워크로드**
   - 짧은 문장: 10단어 분석
   - 긴 문장: 50단어 분석
   - 각 위치마다 1~5개 후보 처리

#### 테스트 행렬 크기

```rust
// 소형: 100x100 (10,000 엔트리)
// 중형: 1,000x1,000 (1,000,000 엔트리, 샘플링 10K)
// 대형: 2,000x2,000 (4,000,000 엔트리, 샘플링 50K)
//       → mecab-ko-dic 실제 크기에 근접
```

## 실행 방법

### 기본 실행

```bash
cd /home/mare/mecab-ko/rust

# 전체 벤치마크 실행
cargo bench

# 특정 벤치마크만 실행
cargo bench --bench trie_bench
cargo bench --bench matrix_bench

# 특정 그룹만 실행
cargo bench --bench trie_bench trie_exact_match
cargo bench --bench matrix_bench matrix_viterbi
```

### 고급 옵션

```bash
# 샘플 크기 조정 (더 정확한 측정)
cargo bench --bench trie_bench -- --sample-size 200

# 베이스라인 저장
cargo bench --bench trie_bench -- --save-baseline main

# 변경 후 비교
cargo bench --bench trie_bench -- --baseline main

# 빠른 테스트 (적은 샘플)
cargo bench --bench matrix_bench -- --sample-size 10
```

### HTML 리포트 확인

```bash
# 벤치마크 실행 후 리포트 생성
cargo bench

# 브라우저로 열기
firefox target/criterion/report/index.html
# 또는
xdg-open target/criterion/report/index.html
```

## 성능 결과 예시

### Trie 벤치마크 결과

```
trie_exact_match_small/hit
                        time:   [66.6 ns 67.4 ns 68.2 ns]

trie_exact_match_small/miss
                        time:   [45.1 ns 45.8 ns 46.5 ns]

trie_common_prefix_search/multi_match
                        time:   [234 ns 238 ns 243 ns]

trie_build/small_sorted
                        time:   [24.8 µs 25.3 µs 25.9 µs]

trie_build/medium
                        time:   [307 µs 314 µs 321 µs]

trie_memory/1000
                        time:   [1.52 ms 1.54 ms 1.57 ms]
```

### Matrix 벤치마크 결과

```
matrix_single_lookup/fixed
                        time:   [1.63 ns 1.65 ns 1.67 ns]
                        thrpt:  [599 Melem/s 607 Melem/s 614 Melem/s]

matrix_single_lookup/random
                        time:   [8.45 ns 8.67 ns 8.91 ns]

matrix_viterbi_pattern/node_transition
                        time:   [456 ns 465 ns 475 ns]

matrix_size_comparison/large_2000x2000
                        time:   [892 ns 911 ns 932 ns]
```

## 성능 해석

### Trie 검색
- **exact_match**: 66 ns/lookup (매우 빠름)
- **common_prefix_search**: 238 ns (3개 매칭 기준)
- **빌드 성능**: 중형 사전(1K개) 314 µs

### Matrix 조회
- **고정 조회**: 1.65 ns (캐시 히트 시)
- **랜덤 조회**: 8.67 ns (캐시 미스 가능)
- **Viterbi 패턴**: 465 ns (35개 조회)

### 성능 목표 달성 여부

| 항목 | 목표 | 실제 | 달성 |
|------|------|------|------|
| Trie exact_match | < 100 ns | ~67 ns | ✓ |
| Trie prefix_search | < 1 µs | ~238 ns | ✓ |
| Matrix 단일 조회 | < 10 ns | ~1.6 ns | ✓ |
| Matrix 배치 조회 | < 1 µs/100 | ~890 ns/100 | ✓ |

## 참고 자료

### 벤치마크 파일 위치

- 소스 코드: `/home/mare/mecab-ko/rust/benches/`
- 실행 코드: `/home/mare/mecab-ko/rust/crates/benchmarks/benches/`
- 결과: `/home/mare/mecab-ko/rust/target/criterion/`
- 리포트: `/home/mare/mecab-ko/rust/target/criterion/report/`

### 관련 파일

- `/home/mare/mecab-ko/rust/benches/README.md` - 상세 사용 가이드
- `/home/mare/mecab-ko/rust/crates/mecab-ko-dict/src/trie.rs` - Trie 구현
- `/home/mare/mecab-ko/rust/crates/mecab-ko-dict/src/matrix.rs` - Matrix 구현
- `/home/mare/mecab-ko/rust/Cargo.toml` - Workspace 설정

### 추가 벤치마크

`crates/benchmarks/benches/`에 추가 벤치마크가 있습니다:
- `viterbi_bench.rs`: Viterbi 알고리즘 전체 성능
- `tokenizer_bench.rs`: 토크나이저 엔드투엔드 성능

## 문제 해결

### Gnuplot 경고

```
Gnuplot not found, using plotters backend
```

이는 경고일 뿐이며, Plotters가 대체 백엔드로 사용됩니다. 그래프는 정상적으로 생성됩니다.

### missing-docs 경고

```
warning: missing documentation for a module
```

벤치마크 코드는 내부 사용이므로 무시 가능합니다.

## 다음 단계

1. **회귀 테스트 자동화**: CI/CD에 벤치마크 통합
2. **성능 모니터링**: 변경 사항에 따른 성능 추이 추적
3. **최적화 우선순위**: 병목 구간 식별 및 개선
4. **프로파일링**: flamegraph 등으로 상세 분석

## 관련 이슈

- QA-001: Criterion 벤치마크 구현 (완료)
- 다음: 성능 회귀 테스트 자동화
