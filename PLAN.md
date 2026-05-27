# PLAN — mecab-ko Sprint 173 (next)

> 마지막 업데이트: 2026-05-27

## 완료: Sprint 172 — B-2 계속 (cold_start + viterbi)

### 결과

- cold_start: 90~280 µs (reuse 30x 가속 발견)
- viterbi_search: 758 ns ~ 9.54 µs (매우 빠름)
- v0.3.0 → v0.7.2 회귀 없음 확인
- PERFORMANCE_BASELINES.md v0.7.2 섹션 추가

## 성능 측정 진척 (Sprint 171~172)

| Bench | 상태 |
|-------|------|
| tokenizer_bench | ✅ (S171) |
| cold_start_bench | ✅ (S172) |
| viterbi_bench | ✅ (S172) |
| batch_bench | 미측정 |
| dict_loading_bench | 미측정 |
| matrix_bench | 미측정 |
| memory_bench | 미측정 |
| normalization_bench | 미측정 |
| trie_bench | 미측정 |

3/9 측정 완료 (핵심 영역).

## Sprint 173 후보

### B-2 계속: 남은 6 benches
- 시간 절약: dict_loading + normalization 우선
- 또는 batch + trie

### B-2 심화: memory profiling
- test-allocator feature 빌드
- 실제 메모리 측정

### 유지보수 모드
- Sprint cycle 공식 종료

### 외부 입수 대기
- NIKL Modu / Sejong

## 결정 프로세스

규칙 5 자동 채택. 시간 절약을 위해 핵심 benches 우선.

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
