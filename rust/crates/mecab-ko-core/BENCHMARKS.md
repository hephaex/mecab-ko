# MeCab-Ko Core 벤치마크 가이드

이 문서는 mecab-ko-core의 성능 벤치마크 스위트에 대한 종합 가이드입니다.

## 목차

- [벤치마크 개요](#벤치마크-개요)
- [벤치마크 실행](#벤치마크-실행)
- [벤치마크 종류](#벤치마크-종류)
- [프로파일링](#프로파일링)
- [결과 해석](#결과-해석)
- [성능 최적화 팁](#성능-최적화-팁)

## 벤치마크 개요

mecab-ko-core는 Criterion.rs를 사용한 포괄적인 벤치마크 스위트를 제공합니다:

- **tokenizer_bench**: 토크나이저 전반적인 성능 측정
- **lattice_bench**: Lattice 구축 및 관리 성능
- **viterbi_bench**: Viterbi 알고리즘 최적 경로 탐색 성능
- **memory_bench**: 메모리 할당 및 재사용 패턴
- **comparison_bench**: 다양한 분석 모드 및 실제 사용 시나리오

## 벤치마크 실행

### 전체 벤치마크 실행

```bash
cd rust/crates/mecab-ko-core

# 모든 벤치마크 실행
./scripts/run_all_benchmarks.sh

# 또는 cargo 직접 사용
cargo bench
```

### 특정 벤치마크 실행

```bash
# 토크나이저 벤치마크만 실행
./scripts/run_specific_benchmark.sh tokenizer_bench

# 특정 테스트만 실행
./scripts/run_specific_benchmark.sh tokenizer_bench tokenize_basic

# cargo 직접 사용
cargo bench --bench tokenizer_bench
cargo bench --bench tokenizer_bench -- tokenize_basic
```

### 벤치마크 비교

코드 변경 전후의 성능을 비교할 수 있습니다:

```bash
# 1. 현재 상태를 baseline으로 저장
./scripts/compare_benchmarks.sh save tokenizer_bench

# 2. 코드 수정

# 3. baseline과 비교
cargo bench --bench tokenizer_bench -- --baseline tokenizer_bench-baseline
```

## 벤치마크 종류

### 1. tokenizer_bench

토크나이저의 전반적인 성능을 측정합니다.

#### 포함된 벤치마크

- **tokenize_basic**: 짧은/중간/긴 텍스트 분석
- **throughput_by_size**: 1KB/10KB/100KB 크기별 처리량
- **by_text_type**: 뉴스/SNS/기술문서/법률 문서별 성능
- **wakati**: 표면형만 추출
- **nouns**: 명사만 추출
- **pos**: 품사 태깅
- **tokenizer_creation**: 토크나이저 생성 오버헤드
- **consecutive_analysis**: 연속 분석 (Lattice 재사용)

#### 실행 예시

```bash
cargo bench --bench tokenizer_bench
```

#### 예상 결과

```
tokenize_basic/short       time:   [15.234 µs 15.567 µs 15.912 µs]
tokenize_basic/medium      time:   [45.123 µs 46.234 µs 47.456 µs]
tokenize_basic/long        time:   [180.23 µs 185.67 µs 191.23 µs]

throughput_by_size/1KB     time:   [234.56 µs 245.67 µs 256.78 µs]
                           thrpt:  [3.8942 MiB/s 4.0712 MiB/s 4.2634 MiB/s]
```

### 2. lattice_bench

Lattice 구축 및 노드 관리 성능을 측정합니다.

#### 포함된 벤치마크

- **lattice_creation**: Lattice 생성 오버헤드
- **node_addition**: 노드 추가 성능
- **lattice_reset**: Lattice 재사용 (reset) 성능
- **node_lookup**: 노드 검색 성능
- **substring**: 부분 문자열 추출
- **stats**: 통계 정보 계산
- **large_lattice**: 대규모 Lattice 구축

#### 실행 예시

```bash
cargo bench --bench lattice_bench
```

### 3. viterbi_bench

Viterbi 알고리즘과 연접 비용 계산 성능을 측정합니다.

#### 포함된 벤치마크

- **viterbi_search**: 다양한 길이의 텍스트에 대한 최적 경로 탐색
- **space_penalty**: 띄어쓰기 패널티 적용 효과
- **large_lattice_search**: 대규모 Lattice 탐색
- **connection_cost**: 연접 비용 행렬 조회
- **repeated_search**: 반복 탐색 (캐시 효과)

#### 실행 예시

```bash
cargo bench --bench viterbi_bench
```

### 4. memory_bench

메모리 할당 패턴과 재사용 효율성을 측정합니다.

#### 포함된 벤치마크

- **token_clone**: 토큰 복사 비용
- **string_allocation**: 문자열 할당 패턴
- **vec_allocation**: Vec 할당 전략 (new vs with_capacity)
- **tokenizer_reuse**: 토크나이저 재사용 vs 재생성
- **bulk_token_creation**: 대량 토큰 생성
- **string_interning**: 문자열 intern 효과
- **token_vec_sizes**: 다양한 크기의 토큰 벡터

#### 실행 예시

```bash
cargo bench --bench memory_bench
```

### 5. comparison_bench

실제 사용 시나리오와 다양한 분석 모드를 비교합니다.

#### 포함된 벤치마크

- **analysis_modes**: tokenize/wakati/nouns/pos 비교
- **linguistic_features**: 교착어/복합명사/외래어/혼합 텍스트
- **real_world_scenarios**: 검색/문서색인/채팅/감성분석/키워드추출
- **edge_cases**: 빈 문자열/특수문자/극단적 입력
- **batch_vs_single**: 배치 처리 vs 단일 처리

#### 실행 예시

```bash
cargo bench --bench comparison_bench
```

## 프로파일링

### Flamegraph 생성

CPU 프로파일링을 위한 Flamegraph를 생성할 수 있습니다:

```bash
# cargo-flamegraph 설치 (최초 1회)
cargo install flamegraph

# Flamegraph 생성
./scripts/flamegraph.sh tokenizer_bench

# 생성된 파일 확인
open flamegraph-tokenizer_bench.svg
```

### perf를 이용한 프로파일링

Linux에서 perf를 사용한 상세 프로파일링:

```bash
# 벤치마크 바이너리 빌드
cargo build --release --bench tokenizer_bench

# perf record 실행
perf record --call-graph=dwarf \
    target/release/deps/tokenizer_bench-* \
    --bench --profile-time 10

# perf report로 분석
perf report
```

### Valgrind를 이용한 메모리 프로파일링

```bash
# Valgrind 설치 필요: sudo apt-get install valgrind

# 메모리 프로파일링
valgrind --tool=massif \
    target/release/deps/tokenizer_bench-* \
    --bench --profile-time 5

# 결과 분석
ms_print massif.out.*
```

## 결과 해석

### Criterion 출력 이해하기

```
tokenize_basic/short    time:   [15.234 µs 15.567 µs 15.912 µs]
                        change: [-2.3421% -0.8765% +0.5432%] (p = 0.23 > 0.05)
                        No change in performance detected.
```

- **time**: [하한, 추정값, 상한] - 95% 신뢰구간
- **change**: 이전 실행 대비 변화율
- **p-value**: 통계적 유의성 (< 0.05면 유의미한 변화)

### 처리량 (Throughput) 이해하기

```
throughput_by_size/1KB  time:   [234.56 µs 245.67 µs 256.78 µs]
                        thrpt:  [3.8942 MiB/s 4.0712 MiB/s 4.2634 MiB/s]
```

- **thrpt**: 초당 처리 데이터량
- MiB/s (Mebibytes per second) 단위

### HTML 리포트 확인

```bash
# 브라우저로 상세 리포트 열기
open target/criterion/report/index.html
```

HTML 리포트에서 확인 가능한 정보:
- 시간 경과에 따른 성능 변화 그래프
- 다양한 입력에 대한 성능 비교
- 이상치(outlier) 분석
- 회귀 분석 결과

## 성능 최적화 팁

### 1. 메모리 할당 최소화

```rust
// Bad: 매번 새로 할당
let mut vec = Vec::new();
for i in 0..n {
    vec.push(item);
}

// Good: 용량 사전 할당
let mut vec = Vec::with_capacity(n);
for i in 0..n {
    vec.push(item);
}
```

### 2. 토크나이저 재사용

```rust
// Bad: 매번 새로 생성
for text in texts {
    let tokenizer = Tokenizer::new()?;
    tokenizer.tokenize(text);
}

// Good: 토크나이저 재사용
let mut tokenizer = Tokenizer::new()?;
for text in texts {
    tokenizer.tokenize(text);  // Lattice 내부 재사용
}
```

### 3. 불필요한 복사 피하기

```rust
// Bad: 매번 복사
let tokens = tokenizer.tokenize(text);
let surfaces: Vec<String> = tokens.iter()
    .map(|t| t.surface.clone())
    .collect();

// Good: wakati 사용 (이미 최적화됨)
let surfaces = tokenizer.wakati(text);
```

### 4. 적절한 분석 모드 선택

- **전체 정보 필요**: `tokenize()`
- **표면형만 필요**: `wakati()` 또는 `morphs()`
- **명사만 필요**: `nouns()`
- **품사 태깅만 필요**: `pos()`

### 5. 배치 처리 고려

```rust
// 여러 짧은 텍스트를 처리할 때
// 가능하다면 합쳐서 한 번에 처리하는 것이 효율적
let combined = texts.join(" ");
let tokens = tokenizer.tokenize(&combined);
```

## 벤치마크 결과 예시

### 기준 환경
- CPU: Intel Core i7-12700K @ 3.6GHz
- RAM: 32GB DDR4-3200
- OS: Ubuntu 22.04 LTS
- Rust: 1.75.0

### 예상 성능 지표

| 벤치마크 | 입력 크기 | 예상 처리 시간 | 처리량 |
|---------|----------|---------------|--------|
| tokenize_basic/short | 5자 | ~15 µs | - |
| tokenize_basic/medium | 50자 | ~45 µs | - |
| tokenize_basic/long | 200자 | ~180 µs | - |
| throughput/1KB | 1KB | ~250 µs | ~4 MiB/s |
| throughput/10KB | 10KB | ~2.5 ms | ~4 MiB/s |
| throughput/100KB | 100KB | ~25 ms | ~4 MiB/s |

### 성능 목표

- **짧은 텍스트 (< 20자)**: < 20 µs
- **중간 텍스트 (< 100자)**: < 100 µs
- **긴 텍스트 (< 1000자)**: < 1 ms
- **처리량**: > 3 MiB/s
- **메모리 오버헤드**: < 2x 입력 크기

## 지속적인 성능 모니터링

### CI/CD 통합

GitHub Actions 예시:

```yaml
name: Benchmark

on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: |
          cd rust/crates/mecab-ko-core
          cargo bench --no-fail-fast
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion
```

### 성능 회귀 감지

```bash
# baseline 저장 (main 브랜치)
git checkout main
cargo bench --bench tokenizer_bench -- --save-baseline main

# feature 브랜치와 비교
git checkout feature-branch
cargo bench --bench tokenizer_bench -- --baseline main

# 성능 저하 확인
# p < 0.05 이고 change > 5% 이면 성능 회귀로 판단
```

## 문제 해결

### 벤치마크가 너무 오래 걸림

```bash
# 샘플 크기 줄이기
cargo bench -- --sample-size 10

# 특정 벤치마크만 실행
cargo bench --bench tokenizer_bench -- short
```

### 일관성 없는 결과

- CPU governor를 performance로 설정
- 다른 프로세스 종료
- 여러 번 실행하여 평균 확인

```bash
# Linux: CPU governor 설정
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

### 메모리 부족

```bash
# 작은 입력으로 테스트
cargo bench --bench memory_bench -- short

# 스왑 확인
free -h
```

## 추가 리소스

- [Criterion.rs 문서](https://bheisler.github.io/criterion.rs/book/)
- [Rust 성능 최적화 가이드](https://nnethercote.github.io/perf-book/)
- [Flamegraph 사용법](https://github.com/flamegraph-rs/flamegraph)
- [perf 튜토리얼](https://perf.wiki.kernel.org/index.php/Tutorial)

## 기여하기

벤치마크 개선 제안이나 새로운 벤치마크 추가는 언제나 환영합니다!

1. 새로운 벤치마크 파일 생성: `benches/my_bench.rs`
2. `Cargo.toml`에 등록
3. PR 제출

벤치마크 작성 시 주의사항:
- `black_box()`로 컴파일러 최적화 방지
- 현실적인 입력 데이터 사용
- 다양한 시나리오 커버
- 명확한 벤치마크 이름과 설명
