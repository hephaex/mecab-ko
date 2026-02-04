# MeCab-Ko Benchmarks

종합적인 성능 벤치마크 스위트입니다. Criterion을 사용하여 정확하고 재현 가능한 성능 측정을 제공합니다.

## 벤치마크 목록

### 1. Cold Start (`cold_start_bench`)
초기화 및 첫 토큰화 성능을 측정합니다.

**측정 항목:**
- Tokenizer 초기화 시간 (사전 로딩 포함)
- 첫 토큰화 vs 이후 토큰화 (캐시 워밍 효과)
- 서버 시작 시나리오
- 재사용 vs 매번 생성
- 병렬 초기화

```bash
cargo bench --bench cold_start_bench
```

### 2. Batch Processing (`batch_bench`)
다양한 배치 크기에서의 처리 성능을 측정합니다.

**측정 항목:**
- 작은/중간/대용량 배치 처리
- 혼합 길이 텍스트 배치
- 처리량 (texts/sec, bytes/sec)
- 메모리 효율성
- 실제 사용 시나리오 (소셜 미디어, 뉴스)

```bash
cargo bench --bench batch_bench
```

### 3. Memory Usage (`memory_bench`)
메모리 사용 패턴을 분석합니다.

**측정 항목:**
- 토큰화당 메모리 할당
- 메모리 재사용 효율성
- 메모리 누적/누수 검사
- 텍스트 크기별 확장성
- 웹 서버 패턴 시뮬레이션

```bash
cargo bench --bench memory_bench
```

### 4. Normalization (`normalization_bench`)
정규화 오버헤드를 측정합니다.

**측정 항목:**
- 정규화 활성화 vs 비활성화
- 유니코드 정규화 (NFC/NFD/NFKC/NFKD)
- 대소문자/공백/숫자/구두점 정규화
- 특수 문자 처리
- 웹 텍스트 정규화

```bash
cargo bench --bench normalization_bench
```

### 5. Comparison (`comparison_bench`)
다양한 처리 모드를 비교합니다.

**측정 항목:**
- wakati vs pos vs full tokenization
- 사용자 사전 유무
- 명사 추출 효율성
- 사용 사례별 최적 모드
- 정확도 vs 속도 트레이드오프

```bash
cargo bench --bench comparison_bench
```

### 6. Tokenizer (`tokenizer_bench`)
전체 토크나이저 성능을 측정합니다.

**측정 항목:**
- 다양한 텍스트 길이 (짧음/중간/긺)
- 배치 처리
- 확장성
- 다양한 텍스트 타입
- 실제 워크로드 시뮬레이션

```bash
cargo bench --bench tokenizer_bench
```

### 7. Trie (`trie_bench`)
사전 검색 성능을 측정합니다.

**측정 항목:**
- exact_match (정확 검색)
- common_prefix_search (접두사 검색)
- 다양한 사전 크기 (소/중/대)
- 빌드 성능
- 메모리 효율성

```bash
cargo bench --bench trie_bench
```

### 8. Matrix (`matrix_bench`)
연접 비용 행렬 성능을 측정합니다.

**측정 항목:**
- 단일/배치 조회
- Viterbi 접근 패턴
- 캐시 지역성
- 다양한 행렬 크기
- 실제 워크로드

```bash
cargo bench --bench matrix_bench
```

### 9. Viterbi (`viterbi_bench`)
Viterbi 알고리즘 성능을 측정합니다.

**측정 항목:**
- Forward/Backward pass
- 띄어쓰기 패널티 오버헤드
- N-best 경로 탐색
- 노드 수별 확장성
- 경로 복잡도별 성능

```bash
cargo bench --bench viterbi_bench
```

## 전체 벤치마크 실행

모든 벤치마크를 실행하려면:

```bash
cd /home/mare/mecab-ko/rust/crates/benchmarks
cargo bench
```

특정 벤치마크만 실행:

```bash
cargo bench --bench <benchmark_name>
```

특정 테스트만 실행:

```bash
cargo bench --bench tokenizer_bench -- short
```

## 결과 확인

벤치마크 결과는 `target/criterion/` 디렉토리에 저장됩니다:

- HTML 보고서: `target/criterion/report/index.html`
- 상세 데이터: `target/criterion/<benchmark_group>/`

브라우저로 HTML 보고서 열기:

```bash
firefox target/criterion/report/index.html
# 또는
google-chrome target/criterion/report/index.html
```

## 벤치마크 옵션

### 샘플 크기 조정

기본적으로 Criterion은 충분한 샘플을 수집합니다. 빠른 테스트를 위해:

```bash
cargo bench -- --sample-size 10
```

### 특정 그룹만 실행

```bash
cargo bench --bench tokenizer_bench -- basic
```

### 베이스라인 저장/비교

첫 실행 (베이스라인 저장):

```bash
cargo bench -- --save-baseline main
```

변경 후 비교:

```bash
cargo bench -- --baseline main
```

## 성능 리포트 생성

벤치마크 스크립트를 사용하여 종합 리포트 생성:

```bash
./run_benchmarks.sh
```

이 스크립트는:
1. 모든 벤치마크 실행
2. 결과 수집 및 정리
3. JSON/CSV 형식으로 내보내기
4. 요약 리포트 생성

## CI/CD 통합

GitHub Actions 워크플로우에서 벤치마크 실행:

```yaml
- name: Run benchmarks
  run: |
    cd rust/crates/benchmarks
    cargo bench --no-fail-fast
```

선택적 실행 (수동 트리거):

```yaml
workflow_dispatch:
  inputs:
    run_benchmarks:
      description: 'Run benchmarks'
      required: false
      default: 'false'
```

## 벤치마크 작성 가이드

새 벤치마크를 추가하려면:

1. `benches/` 디렉토리에 새 파일 생성
2. Criterion 설정:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_feature(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_feature");

    group.bench_function("test_case", |b| {
        b.iter(|| {
            // 측정할 코드
            black_box(my_function());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_my_feature);
criterion_main!(benches);
```

3. `Cargo.toml`에 등록:

```toml
[[bench]]
name = "my_bench"
harness = false
```

## 성능 최적화 팁

### 1. 측정 대상 격리

`black_box()`를 사용하여 컴파일러 최적화 방지:

```rust
b.iter(|| {
    let result = tokenizer.tokenize(black_box(text));
    black_box(result);
});
```

### 2. 설정 비용 제외

`iter_batched`를 사용하여 설정 비용 제외:

```rust
b.iter_batched(
    || create_tokenizer(), // 설정
    |tokenizer| tokenizer.tokenize(text), // 측정
    criterion::BatchSize::SmallInput,
);
```

### 3. 처리량 측정

```rust
group.throughput(Throughput::Bytes(text.len() as u64));
// 또는
group.throughput(Throughput::Elements(batch_size as u64));
```

### 4. 샘플 크기 조정

느린 벤치마크의 경우:

```rust
group.sample_size(10);
```

## 문제 해결

### 벤치마크가 너무 느림

```bash
# 샘플 크기 축소
cargo bench -- --sample-size 10

# 특정 벤치마크만 실행
cargo bench --bench cold_start_bench
```

### 결과가 불안정함

- CPU frequency scaling 비활성화
- 백그라운드 프로세스 종료
- 샘플 크기 증가

### 메모리 부족

- 대용량 벤치마크 샘플 크기 축소
- 배치 크기 감소
- 순차적으로 벤치마크 실행

## 참고 자료

- [Criterion.rs 문서](https://bheisler.github.io/criterion.rs/book/)
- [Rust 벤치마크 가이드](https://rust-lang.github.io/packed_simd/perf-guide/)
- [통계적 벤치마킹](https://github.com/bheisler/criterion.rs/blob/master/book/src/analysis.md)

## 라이선스

MIT OR Apache-2.0
