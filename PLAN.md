# PLAN — mecab-ko Sprint 174 (next)

> 마지막 업데이트: 2026-05-27

## 완료: Sprint 173 — B-2 계속 (dict_loading + normalization)

### 결과

- dict_loading: 182 ns (cached) ~ 334 µs (complex)
- normalization: 1.1~27 µs (Sprint 156/158 추가 후 회귀 없음)
- 5/9 benches 측정 완료

## 성능 측정 진척 (S171~S173)

| 측정 완료 (5) | 미측정 (4) |
|--------------|------------|
| tokenizer_bench (S171) | batch_bench |
| cold_start_bench (S172) | matrix_bench |
| viterbi_bench (S172) | memory_bench |
| dict_loading_bench (S173) | trie_bench |
| normalization_bench (S173) | |

**핵심 영역 모두 측정 완료**.

## Sprint 174 후보

### 유지보수 모드 (권고)

남은 4 benches는 marginal value:
- batch (derivative of tokenizer)
- matrix (derivative of viterbi)
- memory (test-allocator feature 필요)
- trie (derivative of dict_loading)

B-2 영역 사실상 완료. 유지보수 모드 권고.

### B-2 심화 (선택)

memory profiling — test-allocator feature 빌드 + 실제 측정.
시간 소요. ROI 검토 필요.

### 외부 입수 대기

NIKL Modu / Sejong → 정확도 재개.

### 사용자 명시 신규 영역

## 결정 프로세스

규칙 5 자동 채택. 현재 진행 가능한 안전 작업 거의 모두 완료.

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
