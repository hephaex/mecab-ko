# PROGRESS — mecab-ko Sprint 172 (B-2 계속: cold_start + viterbi 측정)

> 마지막 업데이트: 2026-05-27

## Sprint 172 — B-2 성능 측정 (cold_start + viterbi)

| Task | 상태 | 결과 |
|------|------|------|
| S172-1: PERFORMANCE_BASELINES.md 검토 | ✅ 완료 | v0.3.0 baseline 존재 |
| S172-2: cold_start_bench 측정 | ✅ 완료 | 90~280 µs 범위 |
| S172-3: viterbi_bench 측정 | ✅ 완료 | 758 ns ~ 9.54 µs |
| S172-4: PERFORMANCE_BASELINES.md 갱신 | ✅ 완료 | v0.7.2 섹션 추가 |

## 측정 결과

### Cold Start

| Bench | Time (median) |
|-------|--------------|
| cold_start (init) | 90~100 µs |
| cold_start_reuse/recreate_each_time | 100 µs |
| **cold_start_reuse (instance reuse)** | **3.3 µs** ← 30x 빠름 |
| cold_start_complex | 275~280 µs |

핵심 발견: **Tokenizer instance reuse 30x 가속**. 권장 사용 패턴.

### Viterbi Algorithm

| Bench | Time (median) |
|-------|--------------|
| viterbi_search/small_zero_cost | 758 ns |
| viterbi_search/small_with_matrix | 768 ns |
| viterbi_search/medium | 955 ns |
| viterbi_search/large | 9.54 µs |
| viterbi_space_penalty/* | 132~133 ns |
| viterbi_nbest/1 | 965 ns |

핵심 발견: **Viterbi 매우 빠름** (대부분 µs 미만). matrix 적용 cost 거의 없음 (758→768 ns, +1.3%).

### v0.3.0 → v0.7.2 비교

| Metric | v0.3.0 baseline | v0.7.2 측정 | 평가 |
|--------|----------------|------------|------|
| tokenize (medium 75chars) | 11.49 µs | ~10 µs | 유사 또는 미세 개선 |
| Cold start | < 1 ms | 90~280 µs | OK |

회귀 없음. 30+ sprint 정확도 작업으로 인한 성능 저하 없음.

## 변경 파일

- `docs/PERFORMANCE_BASELINES.md`: v0.7.2 측정 섹션 추가
- `PLAN.md`, `PROGRESS.md` 갱신

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass)
- 5-gate sample.tsv: 영향 없음 (코드 변경 0)
- Criterion benches: 정상 출력

## 누적 성능 측정 (Sprint 171/172)

| 영역 | 측정 |
|------|------|
| tokenizer_bench | ✅ Sprint 171 |
| cold_start_bench | ✅ Sprint 172 |
| viterbi_bench | ✅ Sprint 172 |
| batch_bench | 미측정 |
| dict_loading_bench | 미측정 |
| matrix_bench | 미측정 |
| memory_bench | 미측정 |
| normalization_bench | 미측정 |
| trie_bench | 미측정 |

3/9 benches 측정. 핵심 (tokenizer/viterbi/cold_start) 완료.

## Sprint 173 후보

### B-2 계속 (남은 6 benches)
- batch/dict_loading/matrix/memory/normalization/trie
- 각 수 분 소요

### B-2 심화 (memory profiling)
- test-allocator feature 활성
- 실제 메모리 사용 측정

### 유지보수 모드
- Sprint cycle 종료
