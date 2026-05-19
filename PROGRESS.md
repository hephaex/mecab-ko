# PROGRESS — mecab-ko Sprint 138 (Tier A 실험 → rollback)

> 마지막 업데이트: 2026-05-19

## Sprint 138 결과 요약

**P1 결과**: ❌ Tier A matrix.def 수동 cost 조정 — sample.tsv baseline 회귀 회피 불가. **완전 rollback**.
**P2 결과**: ⏭️ ㄹ불규칙 활용형이 이미 Inflect.csv에 모두 존재 (skip).
**P1 인프라**: ✅ matrix.bin 단독 변환 도구 신설 (dict-builder CSV 버그 우회).

## 실험 상세

| Task | 상태 | 비고 |
|------|------|------|
| S138-P2-1: Inflect.csv 분석 | ✅ 완료 | 따라/달라/몰라/불러/흘러/올라/잘라/눌러/골라 모두 entries 존재 — 추가 작업 불요 |
| S138-P1-1: matrix.def 백업 + 5쌍 수정 | ✅ 완료 | 신규 `matrix_def_to_bin` example로 matrix.bin.zst 갱신 |
| S138-P1-2: 4-mode 회귀 검증 | ❌ FAIL | 5쌍: sample.tsv Token -0.9pp / NNG+NNG만: Sentence -0.2pp. 둘 다 rollback |

## P1 실험 결과 (모두 rollback)

### 실험 1 — 5쌍 모두 (cost +300~+500)

| 메트릭 | Before | After | Δ |
|--------|--------|-------|---|
| **sample.tsv Token** | **100.0%** | **99.1%** | **-0.9pp ❌** |
| KLUE morph strict | 66.8% | 66.9% | +0.1pp |
| KLUE eo strict | 20.7% | 21.0% | +0.3pp |
| KLUE eo practical | 23.5% | 23.7% | +0.2pp |

### 실험 2 — NNG+NNG 2쌍만 (cost +500)

| 메트릭 | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv Token | 100.0% | 99.9% | -0.1pp ⚠️ |
| **sample.tsv Sentence** | **99.9%** | **99.7%** | **-0.2pp ❌** |

## 핵심 학습 포인트

1. **matrix.def cost 조정은 너무 거친 도구**: 한 (right_id, left_id) 쌍의 cost는 모든 발생에 적용. 어절 내부 split 회피 의도가 어절 경계 처리에도 영향 → trade-off 자동 해결 불가.
2. **dict-builder CSV 파싱 버그 발견**: entries.csv/Symbol.csv의 쉼표 surface 행에서 fail. Sprint 139 선행 작업 필요.
3. **CRF retrain만이 trade-off 자동 해결 경로**: 학습 데이터 분포 기반 최적화로 sample.tsv 회귀 회피.

## 측정값 (변경 없음 — rollback)

| 메트릭 | Sprint 137 | Sprint 138 |
|--------|-----------|-----------|
| morph strict | 66.8% | 66.8% |
| morph practical | 71.6% | 71.6% |
| per-eojeol practical | 23.5% | 23.5% |
| surface canonical_lenient | 95.5% | 95.5% |
| sample.tsv Token | 100.0% | 100.0% |
| sample.tsv Sentence | 99.9% | 99.9% |

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: all pass / 0 fail
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `test_full_accuracy_evaluation`: PASS (rollback 후 baseline 복구)

## 변경 파일

- `rust/crates/mecab-ko-dict/examples/matrix_def_to_bin.rs`: 신규 (matrix.def → matrix.bin.zst 단독 변환)
- `docs/research/accuracy/2026-05-19_sprint138_tier_a_experiment.md`: 신규 실험 보고서
- `PLAN.md`: Sprint 138 완료 (실패) + Sprint 139 권고
- `PROGRESS.md`: 갱신
- matrix.def, matrix.bin.zst: rollback (변경 없음)

## Sprint 139 진입점

| Track | 비용 | 우선순위 |
|-------|------|---------|
| C: dict-builder CSV 버그 수정 | 0.5-1 sprint | 선행 (Track B 진입 전 필요) |
| B: Full CRF retrain | 3-5 sprint | 메인 — trade-off 자동 해결 |
| A: 세분화 cost 분석 | 1 sprint | 보류 — mecab matrix는 위치 정보 없어 본질적 한계 |
