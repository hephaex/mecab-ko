# PROGRESS — mecab-ko Sprint 136

> 마지막 업데이트: 2026-05-19

## Sprint 136 완료 작업

| Task | 상태 | 비고 |
|------|------|------|
| S136-P3: VA↔VV lenient 동치 추가 | ✅ 완료 | `TAG_EQUIVALENCE_GROUPS_PRACTICAL`에 `["VA","VV"]` 추가. Practical eojeol 22.7% → 23.5% (+0.8pp). Conservative는 VA/VV 구분 유지 (정밀 평가용). |
| S136-P3a: 따르아→따라 normalize (ㄹ불규칙) | ✅ 완료 | `normalize_endings`에 9개 ㄹ불규칙 동사 단방향 패턴 추가 (따르/모르/다르/부르/흐르/오르/자르/누르/고르). canonical_lenient surface 95.4% → 95.5% (+0.1pp). |
| S136-P1: CRF retrain 인프라 조사 | ✅ 완료 | docs/research/accuracy/2026-05-19_sprint136_crf_retrain_infra.md. Track A(connection cost 부분 조정)/B(full retrain)/C(Inflect.csv) 권고. Sprint 137 진입점 Track A. |

## 측정값 (Sprint 135 → Sprint 136)

| 메트릭 | Sprint 135 | Sprint 136 | Δ |
|--------|-----------|-----------|---|
| Morph strict | 66.8% | 66.8% | — (변경 없음) |
| Morph lenient (conservative) | 70.3% | 70.3% | — |
| Morph practical | 70.5% | **71.6%** | **+1.1pp** (VA/VV) |
| Per-eojeol strict | 20.7% | 20.7% | — |
| Per-eojeol lenient | 22.6% | 22.6% | — |
| Per-eojeol practical | 22.7% | **23.5%** | **+0.8pp** (VA/VV) |
| Surface canonical_lenient | 95.4% | **95.5%** | **+0.1pp** (ㄹ불규칙) |
| Sample.tsv Token | 100.0% | 100.0% | — (무회귀) |
| Sample.tsv Sentence | 99.9% | 99.9% | — (무회귀) |

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib` : all pass / 0 fail (392+118+... tests)
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` : clean
- `cargo test ... test_klue_dp_dual_metric_lenient -- --ignored` : pass
- `cargo test ... test_klue_dp_eojeol_surface_only -- --ignored` : pass
- `cargo test ... test_full_accuracy_evaluation -- --ignored` : pass (sample.tsv 100%/99.9%)

## 변경 파일

- `rust/crates/mecab-ko-core/src/evaluate.rs`:
  - `TAG_EQUIVALENCE_GROUPS_PRACTICAL`에 `["VA","VV"]` 추가
  - `normalize_endings()`에 ㄹ불규칙 step 3 추가
  - `R_IRREGULAR_PATTERNS` const (9 entries) 신규
  - 단위 테스트: `test_pos_tags_equivalent_practical_includes_va_vv`, `test_surface_eq_canonical_lenient_r_irregular`, `test_surface_eq_canonical_lenient_r_irregular_does_not_overcorrect` 추가
- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`:
  - practical 출력 텍스트 업데이트 (NNB/NNG + VA/VV 명시)
- `docs/research/accuracy/2026-05-19_sprint136_crf_retrain_infra.md`: 신규
- `PLAN.md`: Sprint 136 완료 표시 + Sprint 137 후보 추가
- `PROGRESS.md`: 신규

## Sprint 137 후보 (Sprint 136 P1 조사 기반)

- Track A: Connection cost 부분 조정 실험 (problematic 5-10쌍 식별 + 수동 조정)
- Track C: Inflect.csv ㄹ불규칙 활용형 정적 추가
- Track B (escalation): Full CRF retrain (mecab-cost-train 사용)
- 후속: P2 Noisy data, P4 borderline NNG↔NNP
