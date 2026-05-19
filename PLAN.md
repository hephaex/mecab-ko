# PLAN — mecab-ko Sprint 138 (next)

> 마지막 업데이트: 2026-05-19

## 완료: Sprint 137 Track A — Connection Cost Pair Analysis (분석-only)

- `test_klue_dp_split_diff_connection_pairs` 추가 (~115줄)
- `Tokenizer::lattice()` getter 신설 (Viterbi 결과 lattice 접근)
- 측정: 2,237 SPLIT_DIFFERENT 어절 → 570 unique pairs / 4,330 occurrences
- 상위 10 패턴 중 6개 (1,126건, 50.3%)가 NNG 분해 (Tier A 후보)
- 보고서: `docs/research/accuracy/2026-05-19_sprint137_connection_cost_analysis.md`
- 회귀: 없음 (41/41 KLUE 테스트 pass, sample.tsv 100%/99.9% 유지)

### Sprint 138 Tier A 후보 (이미 식별)

| Pair | 의미 | 빈도 | 조정 |
|------|------|------|------|
| (3534, 0) | NNG-T → BOS/EOS | 298 | +300 cost |
| (0, 1780) | BOS/EOS → NNG | 264 | +300 cost |
| (3533, 0) | NNG-F → BOS/EOS | 196 | +300 cost |
| (3533, 1780) | NNG-F → NNG | 129 | +500 cost |
| (3534, 1780) | NNG-T → NNG | 109 | +500 cost |

**합계**: 996 pair occurrences 흡수 잠재 → per-eojeol strict +0.5-1.0pp 추정

## 다음 스프린트: Sprint 138 (matrix.def Tier A 실험)

### P1: Tier A matrix.def 수동 조정

**단계**:
1. `matrix.def` 백업 → `matrix.def.s137-baseline`
2. 5쌍 cost +300/+500 수정 (Python/sh 스크립트)
3. `cargo run --bin mecab-ko-dict-builder -- ...` 재실행 → binary 재생성
4. **4-mode 회귀 검증** (모두 통과 필수):
   - `test_full_accuracy_evaluation` (sample.tsv 100%/99.9%)
   - `test_klue_dp_dual_metric` (morph 60%+ / eo 15%+)
   - `test_klue_dp_dual_metric_lenient` (practical ≥ lenient)
   - `test_klue_dp_eojeol_surface_only` (strict 50%+ / canon 80%+)
5. `test_klue_dp_split_diff_connection_pairs` 재실행 → SPLIT_DIFFERENT 감소량 측정
6. 실패 시 cost 폭 조정 또는 부분 적용 (5쌍 중 일부만)

**예상 효과**: per-eojeol strict +0.5-1.0pp, surface canonical_lenient +0.1-0.3pp
**리스크**: 다른 어절(올바른 분해)에 회귀. 단계별 적용 + 매 단계 검증.

### P2 (대안): Track C — Inflect.csv ㄹ불규칙 정적 추가

P1이 회귀로 실패하거나 lift 미달 시 fallback. 
ㄹ불규칙 동사 활용형(따라/몰라/달라 등)을 Inflect.csv에 정적 추가 + dict-builder 재실행.
+0.1-0.3pp + normalize_endings rule 단순화 가능.

### P3 (escalation): Track B — Full CRF Retrain

P1+P2 lift 부족 시. mecab-cost-train (legacy/src) 사용. 3-5 sprint 비용.

### 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- accuracy-gate CI 통과 (sample.tsv 99.9%+ / KLUE DP morph 60%/eo 15% / surface_only strict 50%/canon 80%)

## 백로그 (deferred)

- P4: borderline NNG↔NNP normalization layer (Sprint 132 보류 5 entries + 호스트 73건)
- P5: Noisy data 추가 (KoBEST, KorQuAD, SNS 도메인)
