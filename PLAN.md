# PLAN — mecab-ko Sprint 146 (next)

> 마지막 업데이트: 2026-05-20

## 완료: Sprint 145 — PUD 보류 → D (결합 POS 분석, multi-syllable VV+ETM rollback)

### PUD 보류
- Penn-Treebank XPOS (NN/JJ/RB/...) — Sejong 아님
- lemma `_` 52% (8,666/16,584) — morpheme 분해 부족
- → 보류, D로 전환

### D: 결합 POS 빈도 분석
- 신규 `test_compound_pos_frequency_analysis`
- 153 patterns / 13,611 occurrences (13% tokens)
- 상위: VV+ETM 5,376 / VV+EC 1,728 / XSV+EC 751 / VCP+ETM 613

### multi-syllable VV+ETM 실험 → rollback
- sample.tsv Sentence 99.9% → 99.8% (-1 문장) → rollback
- silver eojeol +5~9 lift는 sample.tsv 회귀를 정당화 못함
- 1-syllable 처리만 유지

### 보고서
`docs/research/accuracy/2026-05-20_sprint145_compound_pos_analysis.md`

## 다음 스프린트: Sprint 146 (미정 — 사용자 선택)

### 후보 A: 명시 surface 목록 기반 안전 패턴 분리

Sprint 141 VCP+EC 방식 — 명시 surface 목록만 분리 (false positive 방지).

**후보 패턴**:
- NP+JX 211건: "그는" → "그/NP + 는/JX", "이는" → "이/NP + 는/JX"
  * **회피**: "난" (contraction)
- VCP+EP 101건: "였" → "이/VCP + 었/EP"
- VV+EP 542건: 명시 동사 ("흘렸" → "흘리/VV + 었/EP", "버렸" → "버리/VV + 었/EP")
  * **회피**: 사동/피동 동사 (이미 splitter에 처리됨)

**각 패턴마다**: 단위 테스트 + 5-gate 검증.

**비용**: 0.5-1 sprint
**위험**: 낮음 (명시 surface)
**예상 lift**: 누적 +0.1-0.3pp (silver 데이터셋)

### 후보 B [메인]: Full CRF Retrain (Track E)

3-5 sprint. 학습 데이터 (KLUE + UD Kaist + UD GSD train) + mecab-cost-train.

### 후보 C: NIKL Modu 수동 다운로드

Academic license, 로컬 only.

## 백로그

- P4 (borderline NNG↔NNP): Sprint 132 보류
- accuracy-gate CI에 UD Kaist + GSD eojeol gate 추가 (현재 morph만)

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지** (Sprint 138 결론)
