# 벤치마크 실행 예시

이 문서는 mecab-ko-core 벤치마크의 실제 실행 예시와 예상 출력을 보여줍니다.

## 빠른 시작

### 1. 전체 벤치마크 실행

```bash
cd rust/crates/mecab-ko-core
./scripts/run_all_benchmarks.sh
```

**예상 소요 시간**: 5-10분 (시스템 사양에 따라 다름)

### 2. 특정 벤치마크만 실행

```bash
# 토크나이저 벤치마크만 (약 1-2분)
cargo bench --bench tokenizer_bench

# Lattice 벤치마크만 (약 30초-1분)
cargo bench --bench lattice_bench
```

### 3. 빠른 테스트 모드

```bash
# 빠르게 동작 확인 (각 벤치마크 10초 이내)
cargo bench --bench tokenizer_bench -- --test
```

## 벤치마크별 예시

### tokenizer_bench

#### 실행 명령
```bash
cargo bench --bench tokenizer_bench
```

#### 예상 출력
```
Benchmarking tokenize_basic/short
Benchmarking tokenize_basic/short: Warming up for 3.0000 s
Benchmarking tokenize_basic/short: Collecting 100 samples in estimated 5.0000 s
Benchmarking tokenize_basic/short: Analyzing
tokenize_basic/short    time:   [15.234 µs 15.567 µs 15.912 µs]
                        change: [-2.3421% -0.8765% +0.5432%] (p = 0.23 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking tokenize_basic/medium
tokenize_basic/medium   time:   [45.123 µs 46.234 µs 47.456 µs]
                        change: [-1.2345% +0.4567% +2.1234%] (p = 0.45 > 0.05)
                        No change in performance detected.

Benchmarking tokenize_basic/long
tokenize_basic/long     time:   [180.23 µs 185.67 µs 191.23 µs]
                        change: [-3.4567% -1.2345% +1.1234%] (p = 0.15 > 0.05)
                        No change in performance detected.

Benchmarking throughput_by_size/1KB
throughput_by_size/1KB  time:   [234.56 µs 245.67 µs 256.78 µs]
                        thrpt:  [3.8942 MiB/s 4.0712 MiB/s 4.2634 MiB/s]
                        change: [-5.1234% -2.3456% +0.5678%] (p = 0.08 > 0.05)
                        No change in performance detected.

Benchmarking by_text_type/news
by_text_type/news       time:   [98.234 µs 101.23 µs 104.56 µs]

Benchmarking by_text_type/social
by_text_type/social     time:   [87.123 µs 89.456 µs 91.789 µs]
```

**해석**:
- **time**: 측정된 실행 시간 [하한, 중앙값, 상한]
- **change**: 이전 실행 대비 변화율
- **p-value**: 통계적 유의성 (< 0.05면 유의미한 변화)
- **thrpt**: 처리량 (MiB/s)
- **outliers**: 이상치 개수

### lattice_bench

#### 실행 명령
```bash
cargo bench --bench lattice_bench
```

#### 예상 출력
```
Benchmarking lattice_creation/short
lattice_creation/short  time:   [2.1234 µs 2.2345 µs 2.3456 µs]

Benchmarking lattice_creation/medium
lattice_creation/medium time:   [5.4321 µs 5.6789 µs 5.9123 µs]

Benchmarking node_addition/single_node
node_addition/single_node
                        time:   [123.45 ns 125.67 ns 127.89 ns]

Benchmarking node_addition/multiple_nodes
node_addition/multiple_nodes
                        time:   [1.2345 µs 1.2678 µs 1.3012 µs]

Benchmarking lattice_reset/reset
lattice_reset/reset     time:   [3.4567 µs 3.5678 µs 3.6789 µs]
```

**해석**:
- Lattice 생성은 매우 빠름 (수 마이크로초)
- 노드 추가는 나노초 단위로 매우 효율적
- Reset은 생성보다 빠름 (재사용 효과)

### viterbi_bench

#### 실행 명령
```bash
cargo bench --bench viterbi_bench
```

#### 예상 출력
```
Benchmarking viterbi_search/short
viterbi_search/short    time:   [12.345 µs 12.678 µs 13.012 µs]

Benchmarking viterbi_search/medium
viterbi_search/medium   time:   [38.234 µs 39.567 µs 40.891 µs]

Benchmarking space_penalty/no_penalty
space_penalty/no_penalty
                        time:   [35.123 µs 36.456 µs 37.789 µs]

Benchmarking space_penalty/korean_default
space_penalty/korean_default
                        time:   [36.234 µs 37.567 µs 38.891 µs]
                        change: [+2.5% +3.1% +3.8%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

**해석**:
- Viterbi 탐색은 Lattice 크기에 비례
- 띄어쓰기 패널티는 약간의 오버헤드 추가 (3-5%)
- Performance regression 경고는 정상 (패널티 계산 비용)

### memory_bench

#### 실행 명령
```bash
cargo bench --bench memory_bench
```

#### 예상 출력
```
Benchmarking token_clone/single_clone
token_clone/single_clone
                        time:   [45.123 ns 46.234 ns 47.345 ns]

Benchmarking token_clone/vec_clone_100
token_clone/vec_clone_100
                        time:   [4.5123 µs 4.6234 µs 4.7345 µs]

Benchmarking vec_allocation/vec_new
vec_allocation/vec_new  time:   [1.2345 µs 1.2678 µs 1.3012 µs]

Benchmarking vec_allocation/vec_with_capacity
vec_allocation/vec_with_capacity
                        time:   [987.65 ns 1.0123 µs 1.0456 µs]
                        change: [-25.3% -21.5% -17.8%] (p = 0.00 < 0.05)
                        Performance has improved.

Benchmarking tokenizer_reuse/reuse_tokenizer
tokenizer_reuse/reuse_tokenizer
                        time:   [234.56 µs 245.67 µs 256.78 µs]

Benchmarking tokenizer_reuse/create_new_each_time
tokenizer_reuse/create_new_each_time
                        time:   [12.345 ms 12.678 ms 13.012 ms]
                        change: [+4800% +5000% +5200%] (p = 0.00 < 0.05)
```

**해석**:
- Vec::with_capacity()가 Vec::new()보다 20% 빠름
- 토크나이저 재사용은 재생성보다 50배 이상 빠름
- 메모리 할당 최적화의 중요성

### comparison_bench

#### 실행 명령
```bash
cargo bench --bench comparison_bench
```

#### 예상 출력
```
Benchmarking analysis_modes/full_tokenize
analysis_modes/full_tokenize
                        time:   [156.23 µs 160.45 µs 164.67 µs]

Benchmarking analysis_modes/wakati
analysis_modes/wakati   time:   [145.67 µs 149.89 µs 154.12 µs]
                        change: [-8.5% -6.5% -4.5%] (p = 0.00 < 0.05)
                        Performance has improved.

Benchmarking real_world_scenarios/search_query
real_world_scenarios/search_query
                        time:   [67.234 µs 69.567 µs 71.890 µs]

Benchmarking real_world_scenarios/document_indexing
real_world_scenarios/document_indexing
                        time:   [1.2345 ms 1.2678 ms 1.3012 ms]

Benchmarking edge_cases/empty_string
edge_cases/empty_string time:   [12.345 ns 12.678 ns 13.012 ns]
```

**해석**:
- wakati()가 full tokenize()보다 약간 빠름 (6-8%)
- 빈 문자열 처리는 매우 빠름 (나노초 단위)
- 실제 시나리오별 성능 특성 확인 가능

## HTML 리포트 확인

### 리포트 생성
```bash
cargo bench
```

### 리포트 열기
```bash
# macOS
open target/criterion/report/index.html

# Linux
xdg-open target/criterion/report/index.html

# Windows
start target/criterion/report/index.html
```

### HTML 리포트 내용
- **Summary**: 모든 벤치마크 요약
- **Individual Plots**: 각 벤치마크 상세 그래프
- **History**: 시간 경과에 따른 성능 변화
- **Violin Plots**: 측정값 분포
- **PDF Export**: 결과 내보내기

## Flamegraph 프로파일링

### Flamegraph 생성
```bash
# cargo-flamegraph 설치 (최초 1회)
cargo install flamegraph

# Flamegraph 생성
./scripts/flamegraph.sh tokenizer_bench

# 생성된 파일 열기
open flamegraph-tokenizer_bench.svg
```

### Flamegraph 읽는 법
- **X축**: CPU 시간 비율
- **Y축**: 호출 스택 깊이
- **색상**: 함수 구분 (의미 없음)
- **넓이**: 해당 함수가 사용한 CPU 시간

**주목할 부분**:
- 넓은 블록: CPU 시간을 많이 사용하는 함수
- 깊은 스택: 복잡한 호출 체인
- 반복 패턴: 루프 내부 호출

## 성능 회귀 감지

### Baseline 저장
```bash
# 현재 main 브랜치 성능을 baseline으로 저장
git checkout main
cargo bench --bench tokenizer_bench -- --save-baseline main
```

### 변경 후 비교
```bash
# feature 브랜치로 이동
git checkout feature-branch

# baseline과 비교
cargo bench --bench tokenizer_bench -- --baseline main
```

### 비교 출력 예시
```
tokenize_basic/short    time:   [15.234 µs 15.567 µs 15.912 µs]
                        change: [+8.5% +10.2% +12.1%] (p = 0.00 < 0.05)
                        Performance has regressed. ⚠️
```

**해석**:
- **change > +5%**: 성능 저하 주의
- **p < 0.05**: 통계적으로 유의미
- **Performance has regressed**: 성능 회귀 발생

## 시스템 최적화

### CPU Governor 설정 (Linux)
```bash
# 현재 설정 확인
cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# performance로 변경 (재부팅 시 초기화됨)
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

### 벤치마크 환경 준비
```bash
# 다른 프로세스 최소화
# 브라우저, IDE 등 닫기

# 벤치마크 실행
cargo bench

# 여러 번 실행하여 일관성 확인
cargo bench
cargo bench
cargo bench
```

## 자주 묻는 질문

### Q: 벤치마크가 "Tokenizer creation failed"라고 나옵니다
**A**: MeCab-Ko 사전 파일이 필요합니다.
```bash
# 사전 경로 설정
export MECAB_DICDIR=/path/to/mecab-ko-dic

# 또는 기본 경로에 설치
# /usr/local/lib/mecab/dic/mecab-ko-dic
```

### Q: 벤치마크가 너무 오래 걸립니다
**A**: 샘플 크기를 줄이거나 특정 테스트만 실행하세요.
```bash
# 샘플 크기 줄이기
cargo bench -- --sample-size 10

# 특정 테스트만
cargo bench --bench tokenizer_bench -- short
```

### Q: 결과가 매번 다릅니다
**A**: 시스템 노이즈를 줄이세요.
- CPU governor를 performance로 설정
- 다른 프로세스 종료
- 여러 번 실행하여 평균 확인

### Q: Flamegraph가 생성되지 않습니다
**A**: 권한 또는 도구 설치 문제입니다.
```bash
# cargo-flamegraph 재설치
cargo install --force flamegraph

# Linux: perf 권한 설정
echo 0 | sudo tee /proc/sys/kernel/perf_event_paranoid

# macOS: DTrace 권한 필요
sudo dtruss -c cargo bench
```

## 추가 도구

### cargo-criterion
더 나은 리포트와 히스토리 관리:
```bash
cargo install cargo-criterion
cargo criterion
```

### hyperfine
명령줄 도구 벤치마크:
```bash
cargo install hyperfine
hyperfine 'cargo run --release -- tokenize "텍스트"'
```

### valgrind (메모리 프로파일링)
```bash
valgrind --tool=massif \
    target/release/deps/tokenizer_bench-* \
    --bench --profile-time 5

ms_print massif.out.*
```

## 결론

이 벤치마크 스위트를 통해:
- ✅ 성능을 객관적으로 측정
- ✅ 최적화 효과를 정량적으로 평가
- ✅ 성능 회귀를 자동으로 감지
- ✅ 병목 지점을 시각적으로 파악

정기적으로 벤치마크를 실행하여 성능을 모니터링하세요!
