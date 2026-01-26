# MeCab-Ko Core 벤치마크

이 디렉토리에는 mecab-ko-core의 성능 벤치마크가 포함되어 있습니다.

## 벤치마크 파일

### tokenizer_bench.rs
토크나이저 전반적인 성능을 측정합니다.
- 다양한 입력 크기 (short, medium, long)
- 처리량 측정 (1KB, 10KB, 100KB)
- 텍스트 유형별 성능 (뉴스, SNS, 기술문서, 법률)
- 분석 모드별 성능 (tokenize, wakati, nouns, pos)

### lattice_bench.rs
Lattice 구축 및 관리 성능을 측정합니다.
- Lattice 생성/리셋
- 노드 추가/검색
- 부분 문자열 추출
- 대규모 Lattice 구축

### viterbi_bench.rs
Viterbi 알고리즘 성능을 측정합니다.
- 최적 경로 탐색
- 띄어쓰기 패널티 효과
- 연접 비용 조회
- 대규모 Lattice 탐색

### memory_bench.rs
메모리 할당 패턴과 효율성을 측정합니다.
- 토큰 복사 비용
- 문자열 할당 패턴
- Vec 할당 전략
- 토크나이저 재사용 효과
- 문자열 interning 효과

### comparison_bench.rs
실제 사용 시나리오를 시뮬레이션합니다.
- 다양한 분석 모드 비교
- 언어적 특성별 성능 (교착어, 복합명사, 외래어)
- 실제 사용 사례 (검색, 문서색인, 채팅, 감성분석)
- 엣지 케이스 처리

## 실행 방법

### 전체 벤치마크 실행
```bash
cd rust/crates/mecab-ko-core
cargo bench
```

### 특정 벤치마크 실행
```bash
# 토크나이저만
cargo bench --bench tokenizer_bench

# Lattice만
cargo bench --bench lattice_bench

# 특정 테스트만
cargo bench --bench tokenizer_bench -- tokenize_basic
```

### 빠른 테스트
```bash
# --test 플래그로 빠르게 실행
cargo bench --bench tokenizer_bench -- --test
```

## 도구 스크립트

프로젝트 루트의 `scripts/` 디렉토리에 유용한 스크립트가 있습니다:

```bash
# 모든 벤치마크 실행
./scripts/run_all_benchmarks.sh

# 특정 벤치마크 실행
./scripts/run_specific_benchmark.sh tokenizer_bench

# Flamegraph 생성 (cargo-flamegraph 필요)
./scripts/flamegraph.sh tokenizer_bench

# 벤치마크 비교
./scripts/compare_benchmarks.sh save tokenizer_bench  # baseline 저장
# 코드 수정 후
cargo bench --bench tokenizer_bench -- --baseline tokenizer_bench-baseline
```

## 결과 확인

벤치마크 실행 후 결과는 다음 위치에 저장됩니다:

```
target/criterion/
├── report/
│   └── index.html      # HTML 리포트
├── tokenize_basic/
│   └── short/
│       ├── base/
│       └── new/
└── ...
```

HTML 리포트 열기:
```bash
open target/criterion/report/index.html
```

## 주의사항

### 사전 파일 필요
대부분의 벤치마크는 실제 MeCab-Ko 사전이 필요합니다:
- 환경변수 `MECAB_DICDIR` 설정
- 또는 기본 경로에 사전 설치

사전 없이 실행하면 "Tokenizer creation failed" 경고가 표시됩니다.

### 성능 측정 시
- CPU governor를 performance로 설정
- 다른 무거운 프로세스 종료
- 여러 번 실행하여 안정적인 결과 확인

## 성능 목표

| 메트릭 | 목표 |
|--------|------|
| 짧은 텍스트 (< 20자) | < 20 µs |
| 중간 텍스트 (< 100자) | < 100 µs |
| 긴 텍스트 (< 1000자) | < 1 ms |
| 처리량 | > 3 MiB/s |
| 메모리 오버헤드 | < 2x 입력 크기 |

## 문제 해결

### 벤치마크가 너무 느림
```bash
# 샘플 크기 줄이기
cargo bench -- --sample-size 10

# 특정 케이스만 실행
cargo bench --bench tokenizer_bench -- short
```

### 메모리 부족
```bash
# 작은 입력으로만 테스트
cargo bench --bench memory_bench -- short
```

## 기여

새로운 벤치마크 시나리오 제안이나 성능 개선 PR은 언제나 환영합니다!

벤치마크 작성 가이드:
1. `black_box()`로 컴파일러 최적화 방지
2. 현실적인 입력 데이터 사용
3. 다양한 시나리오 커버
4. 명확한 이름과 문서화

자세한 내용은 [BENCHMARKS.md](../BENCHMARKS.md)를 참고하세요.
