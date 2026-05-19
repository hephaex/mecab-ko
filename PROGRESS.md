# PROGRESS — mecab-ko Sprint 137 (Track A 분석)

> 마지막 업데이트: 2026-05-19

## Sprint 137 Track A — Connection Cost Pair Analysis (분석-only)

| Task | 상태 | 비고 |
|------|------|------|
| S137-A1: SPLIT_DIFFERENT 오류 + left/right_id 매핑 | ✅ 완료 | `test_klue_dp_split_diff_connection_pairs` 추가. `Tokenizer::lattice()` getter 신설 (Viterbi 결과 lattice 접근). 2,237 어절 → 570 unique pairs / 4,330 occurrences. |
| S137-A2: problematic 쌍 식별 + 보고서 | ✅ 완료 | docs/research/accuracy/2026-05-19_sprint137_connection_cost_analysis.md. 상위 30 쌍 분석, Tier A(NNG 분해, +0.5-1.0pp 추정)/B(EF·SF·EP 유지)/C(보류) 분류. Sprint 138 실험 절차 명시. |

## 핵심 발견

**SPLIT_DIFFERENT 2,237 어절의 50.3% (1,126건)이 NNG 분해에 집중**:

| 패턴 | 빈도 | 예시 | 조치 |
|------|------|------|------|
| (NNG-T, BOS/EOS) | 298 | 공정성\|을, 돌\|입 | Tier A: +300 cost |
| (BOS/EOS, NNG) | 264 | 지\|검장, 주\|의 | Tier A: +300 cost |
| (NNG-F, BOS/EOS) | 196 | 대\|한, 위\|한 | Tier A: +300 cost |
| (EF, SF) | 166 | 다\|., 빠집니다\|. | Tier B: 형태론적 정확 |
| (BOS/EOS, BOS/EOS) | 162 | 한\|다면, 나\|갈 | Tier A: +300 cost |
| (SH, NNG) | 134 | 100\|여명, 1\|천명 | Tier C: 보류 |
| (XR-T, BOS/EOS) | 130 | 탁월\|한, 비롯\|한 | Tier C: 보류 |
| (NNG-F, NNG) | 129 | 테니스\|단, 국가\|보훈 | Tier A: +500 cost |
| (EP, EF) | 114 | 알려졌\|다, 옮겼\|다 | Tier B: 형태론적 정확 |
| (NNG-T, NNG) | 109 | 팝\|스타, 보훈\|처 | Tier A: +500 cost |

**Tier A 5쌍**: 1,126/4,330 ≈ 26% pair occurrence 흡수 잠재력. per-eojeol strict +0.5-1.0pp 추정 (sample.tsv 회귀 위험은 별도 검증 필요).

## 측정값 (변경 없음 — 분석만)

| 메트릭 | Sprint 136 | Sprint 137 | Δ |
|--------|-----------|-----------|---|
| Morph strict | 66.8% | 66.8% | — (분석-only) |
| Morph practical | 71.6% | 71.6% | — |
| Per-eojeol practical | 23.5% | 23.5% | — |
| Surface canonical_lenient | 95.5% | 95.5% | — |
| Sample.tsv Token | 100.0% | 100.0% | — |
| Sample.tsv Sentence | 99.9% | 99.9% | — |

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib` : all pass / 0 fail
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` : clean
- `cargo test ... --test accuracy_eval -- --ignored` : 41/41 pass (90s)

## 변경 파일

- `rust/crates/mecab-ko-core/src/tokenizer.rs`:
  - `pub const fn lattice(&self) -> &Lattice` 추가 (Viterbi 결과 접근)
- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`:
  - `test_klue_dp_split_diff_connection_pairs` 신규 (~115줄)
  - left-id.def/right-id.def 파서 + (right_id, left_id) 빈도 집계
- `docs/research/accuracy/2026-05-19_sprint137_connection_cost_analysis.md` 신규
- `PLAN.md`: Sprint 137 Track A 완료 + Sprint 138 실험 절차
- `PROGRESS.md`: 갱신

## Sprint 138 진입점

**Tier A matrix.def 수동 조정 실험**:
1. matrix.def 백업 (matrix.def.s137-baseline)
2. Tier A 5쌍 cost +300 ~ +500 수정 (스크립트)
3. dict-builder 재실행 → binary 재생성
4. 4-mode 회귀 검증 (sample.tsv 100% / KLUE morph 60%+ / surface_only canon 80%+ / per-eojeol practical ≥ lenient)
5. SPLIT_DIFFERENT 건수 측정 (test_klue_dp_split_diff_connection_pairs 재실행)
