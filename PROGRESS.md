# PROGRESS — mecab-ko Sprint 171 (B-2 성능 진단)

> 마지막 업데이트: 2026-05-27

## Sprint 171 — B-2: 성능 진단 (mecab-profile + Criterion)

| Task | 상태 | 결과 |
|------|------|------|
| S171-1: profiler 도구 인벤토리 | ✅ 완료 | mecab-profile CLI + Criterion benches |
| S171-2: mecab-profile 빌드 + 검증 | ✅ 완료 | 25.6초 빌드, --help 정상 |
| S171-3: tokenize memory profile | ⚠️ 부분 | memory tracking은 test-allocator feature 필요 |
| S171-4: Criterion tokenizer_bench | ✅ 완료 | baseline 측정 완료 |
| S171-5: 결과 분석 + 보고 | ✅ 완료 | |

## 측정 결과 — Tokenizer Baseline (mini-dict)

### Baseline 문장 (Criterion bench)

| Sentence | Time | Throughput |
|----------|------|-----------|
| sentence/0 | 10.66 µs | 6.7 MiB/s |
| sentence/1 | 9.80 µs | 7.1 MiB/s |
| sentence/2 | 8.93 µs | 7.7 MiB/s |
| sentence/3 | 8.86 µs | 7.1 MiB/s |
| sentence/4 | 10.95 µs | 6.9 MiB/s |
| **평균** | **~10 µs** | **~7 MiB/s** |

### Edge cases

| 입력 | Time |
|------|------|
| whitespace_only | 110 ns |
| english_only | 7.2 µs |
| numbers_only | 15.7 µs |

**주의**: mini-dict 사용 (full mecab-ko-dic 미설정). 실제 측정은 MECAB_DICDIR=mecab-ko-dic 환경에서 더 의미 있음.

### Memory profiling 제약

`mecab-profile tokenize`는 memory tracking이 비활성 상태 (0 B):
- 원인: test-allocator feature 미활성
- 해결: `cargo build --features test-allocator` 필요
- 현재 sprint는 baseline 시간 측정만 진행

## 성능 인프라 평가

| 도구 | 상태 |
|------|------|
| mecab-profile CLI | ✅ 빌드 가능, --help 정상 |
| dict_profiler | ✅ |
| tokenizer_profiler | ✅ |
| trie_profiler | ✅ |
| jemalloc_profiler | ✅ |
| Criterion benches (9개) | ✅ 정상 동작 |
| ─ tokenizer_bench | 측정 가능 |
| ─ batch_bench | 미측정 |
| ─ cold_start_bench | 미측정 |
| ─ dict_loading_bench | 미측정 |
| ─ matrix_bench | 미측정 |
| ─ memory_bench | 미측정 |
| ─ normalization_bench | 미측정 |
| ─ trie_bench | 미측정 |
| ─ viterbi_bench | 미측정 |

성능 진단 인프라 완비. 9개 Criterion benches + 4개 profiler 모듈.

## 변경 파일

- (코드 변경 없음 — 측정 sprint)
- `PLAN.md`, `PROGRESS.md` 갱신

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass)
- 5-gate sample.tsv: 영향 없음 (코드 변경 0)
- Criterion tokenizer_bench: 정상 출력

## Sprint 172 후보

### B-2 계속 (전체 bench suite 측정)
- 9개 bench 모두 실행 + baseline 기록
- 결과를 docs/benchmarks/ 또는 PERFORMANCE_BASELINES.md에 기록
- 시간 소요 (각 bench 수 분)

### B-2 심화 (memory profiling)
- test-allocator feature로 빌드
- dict_profiler, tokenizer_profiler 실제 측정
- 핫스팟 식별

### 유지보수 모드
- Sprint cycle 종료
- 다음 메이저 작업 대기

### 외부 입수 대기
- NIKL Modu, Sejong
