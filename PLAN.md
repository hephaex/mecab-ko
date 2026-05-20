# PLAN — mecab-ko Sprint 148 (next)

> 마지막 업데이트: 2026-05-20

## 완료: Sprint 147 A — VV/XSV practical 동치 추가

### 발견
- mecab "했/됐" = VV+EP (1 token)
- gold "하/XSV + 였/EP" (2 tokens)
- POS scheme 차이 → surface 분리 불가, practical 동치만 적절

### 구현
- `TAG_EQUIVALENCE_GROUPS_PRACTICAL`: `&["VA", "VV", "XSV"]`
- 단위 테스트 1개 + Conservative 보존

### Lift (3 silver 모두)
- KLUE practical morph: 71.6% → 71.9% (+0.3pp)
- UD Kaist practical morph: 68.1% → 68.3% (+0.2pp)
- UD GSD practical morph: 71.3% → 71.7% (+0.4pp)
- sample.tsv 무회귀

### 보고서
`docs/research/accuracy/2026-05-20_sprint147_xsv_practical_equivalence.md`

## 다음 스프린트: Sprint 148 (미정 — 사용자 선택)

### 후보 A: VV+EP 명시 동사 분리

VV+EP 542건. 명시 동사 surface 분리:
- "흘렸" → 흘리/VV + 었/EP
- "버렸" → 버리/VV + 었/EP
- "불탔" → 불타/VV + 았/EP

**복잡도**: stem 식별 필요 (regular/irregular conjugation). 명시 surface 목록 + 표준 활용 규칙.

**비용**: 0.5-1 sprint
**위험**: 중간 (false positive 위험)

### 후보 B [메인]: Full CRF Retrain (Track E)

3-5 sprint. 학습 데이터 + mecab-cost-train.

### 후보 C: NIKL Modu 수동 다운로드

Academic license, 구어/SNS 도메인 확장.

### 후보 D: ETM+ETM "라는" 조사

33건. mecab 비정상 출력. 분석 후 처리.

### 후보 E: 추가 practical 동치 후보 조사

Sprint 147 패턴 (POS scheme 차이) 분석. KLUE/UD/mecab 비교로 추가 convention 차이 발견.

## 백로그

- P4 (borderline NNG↔NNP)
- accuracy-gate CI에 UD eojeol gate 추가

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
