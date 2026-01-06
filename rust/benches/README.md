# MeCab-Ko Rust Benchmarks

Criterion 기반 성능 벤치마크 모음입니다.

## 벤치마크 목록

### 1. Trie 검색 성능 (`trie_bench.rs`)

Double-Array Trie 자료구조의 검색 성능을 측정합니다.

#### 측정 항목

- **exact_match**: 정확한 키 검색
  - 소형 사전 (17개 엔트리)
  - 중형 사전 (~1,000개 엔트리)
  - 대형 사전 (~10,000개 엔트리)
  - hit/miss 케이스 분리
  - 배치 조회

- **common_prefix_search**: 공통 접두사 검색
  - 다중 매칭 시나리오
  - 긴 텍스트 처리
  - 매칭 없는 경우

- **형태소 분석 시나리오**: 실제 사용 패턴 시뮬레이션
  - 문장 전체 처리 (각 위치에서 prefix search)

- **Trie 빌드 성능**
  - 정렬된 엔트리
  - 비정렬 엔트리
  - 다양한 크기

- **메모리 효율성**
  - 100, 1,000, 5,000개 엔트리
  - 압축률 측정

#### 실행 방법

```bash
# 전체 실행
cargo bench --bench trie_bench

# 특정 그룹만 실행
cargo bench --bench trie_bench trie_exact_match_small
cargo bench --bench trie_bench trie_common_prefix_search

# HTML 리포트 생성
cargo bench --bench trie_bench
# target/criterion/report/index.html 확인
```

### 2. Matrix 연접 비용 조회 성능 (`matrix_bench.rs`)

연접 비용 행렬의 조회 성능을 측정합니다.

#### 측정 항목

- **단일 조회 성능**
  - 순차 조회 (캐시 친화적)
  - 랜덤 조회 (캐시 비친화적)
  - 고정 위치 조회

- **배치 조회 성능**
  - 10, 100, 1,000개 배치 크기

- **Viterbi 알고리즘 패턴**
  - 노드 전이 비용 계산
  - 전체 경로 계산 시뮬레이션

- **다양한 크기 비교**
  - 100x100 (소형)
  - 1,000x1,000 (중형)
  - 2,000x2,000 (대형, mecab-ko-dic 실제 크기)

- **캐시 지역성 테스트**
  - 행 우선 순회
  - 열 우선 순회
  - 스트라이드 접근

- **메모리 사용량**
  - 다양한 크기별 메모리 풋프린트

- **실제 워크로드 시뮬레이션**
  - 10단어 문장
  - 50단어 긴 문장

#### 실행 방법

```bash
# 전체 실행
cargo bench --bench matrix_bench

# 특정 그룹만 실행
cargo bench --bench matrix_bench matrix_single_lookup
cargo bench --bench matrix_bench matrix_viterbi_pattern

# HTML 리포트 생성
cargo bench --bench matrix_bench
# target/criterion/report/index.html 확인
```

## 벤치마크 결과 해석

### Criterion 출력 이해하기

```
trie_exact_match_small/hit
                        time:   [45.123 ns 46.789 ns 48.456 ns]
```

- **첫 번째 값**: 하한 추정치
- **두 번째 값**: 중앙값 (가장 신뢰할 수 있는 값)
- **세 번째 값**: 상한 추정치

### 성능 목표

#### Trie 검색
- exact_match: < 100 ns (소형 사전)
- common_prefix_search: < 1 µs (평균 3개 매칭)

#### Matrix 조회
- 단일 조회: < 10 ns (캐시 히트)
- 배치 조회: < 1 µs per 100 queries

## 벤치마크 모범 사례

### 1. 안정적인 환경에서 실행
```bash
# CPU 주파수 고정 (Linux)
sudo cpupower frequency-set --governor performance

# 백그라운드 프로세스 최소화
# 전용 벤치마크 세션 사용
```

### 2. 충분한 샘플 수집
```bash
# 더 많은 샘플로 정확도 향상
cargo bench --bench trie_bench -- --sample-size 200
```

### 3. 베이스라인 비교
```bash
# 베이스라인 저장
cargo bench --bench trie_bench -- --save-baseline main

# 변경 후 비교
cargo bench --bench trie_bench -- --baseline main
```

### 4. 특정 벤치마크만 실행
```bash
# 패턴 매칭으로 필터링
cargo bench --bench trie_bench -- exact_match
```

## 추가 벤치마크 (crates/benchmarks/)

더 많은 벤치마크가 `crates/benchmarks/benches/`에 있습니다:

- `viterbi_bench.rs`: Viterbi 알고리즘 전체 성능
- `tokenizer_bench.rs`: 토크나이저 엔드투엔드 성능

## 참고 자료

- [Criterion.rs 공식 문서](https://bheisler.github.io/criterion.rs/book/)
- [Rust 성능 측정 가이드](https://nnethercote.github.io/perf-book/)
