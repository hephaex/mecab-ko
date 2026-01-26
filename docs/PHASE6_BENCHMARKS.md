# Phase 6: 프로덕션 최적화 - 성능 벤치마크 확장

## 작업 완료 요약

Phase 6의 성능 벤치마크 확장 작업이 완료되었습니다.

## 구현 내용

### 1. 포괄적 벤치마크 스위트

5개의 벤치마크 파일을 생성하여 다양한 측면의 성능을 측정할 수 있습니다:

#### tokenizer_bench.rs
- **기본 토크나이저 성능**: 짧은/중간/긴 텍스트 분석
- **처리량 측정**: 1KB, 10KB, 100KB 크기별 throughput
- **텍스트 유형별**: 뉴스, SNS, 기술문서, 법률 문서
- **분석 모드별**: tokenize(), wakati(), nouns(), pos()
- **토크나이저 생성**: 초기화 오버헤드
- **연속 분석**: Lattice 재사용 효과

#### lattice_bench.rs
- **Lattice 생성**: 다양한 길이의 텍스트
- **노드 추가**: 단일/다중 노드 추가 성능
- **Lattice 리셋**: 재사용 성능
- **노드 검색**: 단일/다중 검색
- **부분 문자열 추출**: substring 성능
- **통계 정보**: stats() 계산 오버헤드
- **대규모 Lattice**: 실제 사용 패턴 시뮬레이션

#### viterbi_bench.rs
- **Viterbi 탐색**: 다양한 길이의 텍스트
- **띄어쓰기 패널티**: 패널티 유무/종류별 비교
- **대규모 Lattice**: 복잡한 그래프 탐색
- **연접 비용**: 단일/배치 조회 성능
- **반복 탐색**: 캐시 효과 측정

#### memory_bench.rs
- **토큰 복사**: clone() 비용
- **문자열 할당**: 다양한 크기의 문자열
- **Vec 할당**: new() vs with_capacity()
- **토크나이저 재사용**: 재사용 vs 재생성
- **대량 토큰 생성**: 메모리 패턴
- **문자열 interning**: intern 효과
- **토큰 벡터 크기**: 10/50/100/500/1000 토큰

#### comparison_bench.rs
- **분석 모드 비교**: 5가지 모드 성능 비교
- **언어적 특성**: 교착어/복합명사/외래어/혼합
- **실제 시나리오**:
  - 검색 쿼리 (짧고 빠른 응답)
  - 문서 색인 (대량 처리)
  - 실시간 채팅 (짧은 메시지 반복)
  - 감성 분석 (형용사/동사 추출)
  - 키워드 추출 (명사 중심)
- **엣지 케이스**: 빈 문자열, 특수문자, 긴 토큰 등
- **배치 처리**: 순차 vs 결합 처리

### 2. 헬퍼 스크립트

#### run_all_benchmarks.sh
모든 벤치마크를 순차적으로 실행하는 스크립트

```bash
./scripts/run_all_benchmarks.sh
```

#### run_specific_benchmark.sh
특정 벤치마크만 실행

```bash
./scripts/run_specific_benchmark.sh tokenizer_bench
./scripts/run_specific_benchmark.sh tokenizer_bench tokenize_basic
```

#### flamegraph.sh
Flamegraph 생성으로 CPU 프로파일링

```bash
./scripts/flamegraph.sh tokenizer_bench
```

필수: `cargo install flamegraph`

#### compare_benchmarks.sh
벤치마크 결과 비교 (회귀 테스트)

```bash
# baseline 저장
./scripts/compare_benchmarks.sh save tokenizer_bench

# 코드 수정 후 비교
cargo bench --bench tokenizer_bench -- --baseline tokenizer_bench-baseline
```

### 3. 종합 문서

#### BENCHMARKS.md
- 벤치마크 개요 및 실행 방법
- 각 벤치마크 상세 설명
- 프로파일링 도구 사용법 (Flamegraph, perf, Valgrind)
- 결과 해석 가이드
- 성능 최적화 팁
- CI/CD 통합 예시
- 문제 해결 가이드

#### benches/README.md
- 벤치마크 파일별 간단한 설명
- 빠른 시작 가이드
- 스크립트 사용법
- 성능 목표 지표

## 파일 구조

```
rust/crates/mecab-ko-core/
├── Cargo.toml                     # 5개 벤치마크 등록
├── BENCHMARKS.md                  # 종합 벤치마크 가이드
├── benches/
│   ├── README.md                  # 벤치마크 디렉토리 README
│   ├── tokenizer_bench.rs         # 토크나이저 벤치마크
│   ├── lattice_bench.rs           # Lattice 벤치마크
│   ├── viterbi_bench.rs           # Viterbi 벤치마크
│   ├── memory_bench.rs            # 메모리 벤치마크
│   └── comparison_bench.rs        # 비교 벤치마크
└── scripts/
    ├── run_all_benchmarks.sh      # 전체 실행
    ├── run_specific_benchmark.sh  # 특정 벤치마크 실행
    ├── flamegraph.sh              # Flamegraph 생성
    └── compare_benchmarks.sh      # 결과 비교
```

## 벤치마크 특징

### Criterion.rs 기반
- 통계적으로 유의미한 결과
- HTML 리포트 자동 생성
- 회귀 감지
- 다양한 출력 형식 지원

### 다양한 입력 크기
- **짧은 텍스트**: ~5자 (검색 쿼리)
- **중간 텍스트**: ~50자 (채팅 메시지)
- **긴 텍스트**: ~200자 (문단)
- **대용량**: 1KB, 10KB, 100KB (문서)

### 텍스트 유형별
- **뉴스**: 표준 문어체
- **SNS**: 구어체, 이모티콘, 줄임말
- **기술문서**: 영어 혼용, 전문 용어
- **법률**: 격식체, 긴 문장

### 실제 사용 시나리오
- 검색 엔진 쿼리 분석
- 문서 색인 생성
- 실시간 채팅 분석
- 감성 분석
- 키워드 추출

## 성능 목표

| 메트릭 | 목표 | 측정 방법 |
|--------|------|-----------|
| 짧은 텍스트 | < 20 µs | tokenizer_bench/short |
| 중간 텍스트 | < 100 µs | tokenizer_bench/medium |
| 긴 텍스트 | < 1 ms | tokenizer_bench/long |
| 처리량 | > 3 MiB/s | throughput_by_size |
| 메모리 오버헤드 | < 2x | memory_bench |

## 사용 방법

### 기본 실행
```bash
cd rust/crates/mecab-ko-core
cargo bench
```

### 특정 벤치마크
```bash
# 토크나이저만
cargo bench --bench tokenizer_bench

# 특정 테스트만
cargo bench --bench tokenizer_bench -- tokenize_basic

# 빠른 테스트 모드
cargo bench --bench tokenizer_bench -- --test
```

### HTML 리포트 확인
```bash
open target/criterion/report/index.html
```

### Flamegraph 생성
```bash
cargo install flamegraph
./scripts/flamegraph.sh tokenizer_bench
open flamegraph-tokenizer_bench.svg
```

## 프로파일링 통합

### CPU 프로파일링
- **Flamegraph**: 함수별 CPU 사용량 시각화
- **perf**: Linux 상세 프로파일링
- **Criterion**: 실행 시간 통계

### 메모리 프로파일링
- **Valgrind (Massif)**: 힙 메모리 사용량
- **memory_bench**: 할당 패턴 분석
- **heaptrack**: 메모리 누수 검사

## CI/CD 통합

벤치마크를 GitHub Actions에 통합하여 자동 회귀 테스트:

```yaml
- name: Run benchmarks
  run: |
    cd rust/crates/mecab-ko-core
    cargo bench --no-fail-fast
```

## 최적화 가이드

### 1. 메모리 할당 최소화
- Vec::with_capacity() 사용
- 토크나이저 재사용
- 문자열 interning

### 2. 적절한 분석 모드
- 표면형만 필요 → wakati()
- 명사만 필요 → nouns()
- 전체 정보 필요 → tokenize()

### 3. 배치 처리
- 여러 짧은 텍스트는 합쳐서 처리
- Lattice 재사용 효과

## 알려진 제한사항

### 사전 파일 필요
대부분의 벤치마크는 실제 MeCab-Ko 사전이 필요합니다:
- `MECAB_DICDIR` 환경변수 설정
- 또는 기본 경로에 사전 설치

사전 없이 실행하면 "Tokenizer creation failed" 경고 표시

### 시스템 의존성
- CPU governor 설정 영향
- 다른 프로세스의 영향
- 디스크 I/O 영향 (사전 로딩)

## 향후 계획

### Phase 6 계속 (추가 작업)
- [ ] 실제 경쟁 라이브러리 비교 (KoNLPy, kiwipiepy)
- [ ] 병렬 처리 벤치마크 (Rayon)
- [ ] 비동기 API 벤치마크 (Tokio)
- [ ] SIMD 최적화 효과 측정
- [ ] 메모리 풀링 효과 측정

### Phase 7: 배포 및 에코시스템
- crates.io 배포 준비
- 패키지 크기 최적화
- 문서 완성도 향상

## 결론

Phase 6의 벤치마크 확장 작업을 통해:

✅ **포괄적인 벤치마크 스위트 구축**
- 5개 벤치마크 파일, 50+ 개별 테스트

✅ **다양한 시나리오 커버**
- 입력 크기, 텍스트 유형, 사용 사례

✅ **프로파일링 도구 통합**
- Flamegraph, perf, Valgrind 가이드

✅ **자동화 스크립트 제공**
- 실행, 비교, 프로파일링 스크립트

✅ **종합 문서화**
- BENCHMARKS.md 가이드
- 성능 목표 및 최적화 팁

이제 mecab-ko-core의 성능을 객관적으로 측정하고 개선할 수 있는 인프라가 완비되었습니다!
