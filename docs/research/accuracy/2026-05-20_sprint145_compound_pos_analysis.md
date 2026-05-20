# Sprint 145 D — mecab 결합 POS 빈도 분석 + multi-syllable VV+ETM 실험 (rollback)

> **결과**: PUD 보류(라이선스 외 형식 제약) → D로 전환. 결합 POS 빈도 분석으로 153 패턴 식별. multi-syllable VV+ETM 분리 확장 시도 → sample.tsv -1 sentence 회귀로 rollback. 분석 산출물 + 보고서만 commit. 다음 sprint 안전 후보 식별.

---

## 1. PUD 보류 사유

UD Korean-PUD 다운로드 후 형식 분석 결과:
- **XPOS scheme**: Penn-Treebank style (`NN`, `JJ`, `RB`, `CD`, `DT`, `VC`, `CM`, `NN+CM`, `NNP+CM` 등) — Sejong도 KAIST도 아님. ~25 태그 별도 매핑 필요.
- **lemma `_` 비율**: 52% (8,666 / 16,584 tokens) — morpheme 분해 정보 부족.

KAIST/GSD는 `lemma + xpos` 결합으로 morpheme 분해 가능. PUD는 lemma underscore가 절반 — `compound:NN+CM` 같은 분해 정보 추출 불가.

**결정**: PUD 통합 보류 (변환 복잡도 > 변환률).

대체 후보 D (결합 토큰 패턴 확장)로 전환.

---

## 2. 결합 POS 빈도 분석

### 2.1 인프라

신규 테스트: `test_compound_pos_frequency_analysis` in `tests/accuracy_eval.rs`
- 3 silver 통합 측정 (KLUE val + UD Kaist test + UD GSD test)
- `Token.pos`에 `+`가 포함된 패턴 빈도 집계 + surface 샘플

### 2.2 측정 결과

- **Total tokens: 104,885 / Compound: 13,611 (13.0%)**
- **Unique compound patterns: 153**

### 2.3 상위 패턴

| Rank | Pattern | Count | Samples | splitter.rs 처리? |
|------|---------|-------|---------|----------------|
| 1 | VV+ETM | **5,376** | 한, 된, 산 | 1-syllable만 |
| 2 | VV+EC | 1,728 | 대, 되, 넘겨 | 일부 (는다) |
| 3 | XSV+EC | 751 | 해, 하고, 타가 | 미처리 |
| 4 | VCP+ETM | 613 | 인, 냐는, 일 | **Sprint 141 처리** |
| 5 | VA+ETM | 542 | 어려울, 바른, 큰 | 1-syllable만 |
| 6 | VV+EP | 542 | 흘렸, 버렸 | 미처리 |
| 7 | VV+EF | 513 | 봤다, 합니다 | 일부 (causative) |
| 8 | XSV+EP | 413 | 했, 됐 | 미처리 |
| 9 | VCP+EC | 344 | 며, 어야, 고 | **Sprint 141 처리** |
| 10 | VCP+EF | 279 | 입니다, 다 | **Sprint 132+ 처리** |
| 11 | VV+EP+EF | 220 | 왔다, 했습니다 | 일부 (야겠다 한정) |
| 12 | NP+JX | 211 | 그는, 게다가, 난 | 미처리 |
| 13 | VA+EC | 164 | 빠르게, 이러, 크게 | 미처리 |
| 14 | EP+EC | 151 | 고, 도록, 면 | 일부 (며) |
| 15 | VCP+EP | 101 | 였 | 미처리 |

기존 미처리 큰 패턴: VV+ETM, VV+EP, XSV+EC, XSV+EP, NP+JX, VA+EC.

---

## 3. multi-syllable VV+ETM 실험 → rollback

### 3.1 가설

기존 splitter는 1-syllable VV+ETM/VA+ETM만 처리 (`surface.chars().count() == 1`).
- "한" → "하 + ㄴ" ✅
- "넘긴" → "넘긴" (분해 안 됨) ❌

`surface.chars().count() == 1` 조건 제거 + multi-syllable 확장:
- "넘긴" → "넘기 + ㄴ"
- "꺼진" → "꺼지 + ㄴ"

### 3.2 측정 결과 (실험 적용)

| 메트릭 | Before | After (multi-syl) | Δ |
|--------|--------|-----------|---|
| **sample.tsv Sentence** | **99.9%** | **99.8%** | **-0.1pp ❌** (-1 문장) |
| sample.tsv Token | 100.0% | 100.0% | — |
| KLUE morph strict | 66.8% | 66.8% | — |
| KLUE eo strict (count) | 4642 | 4647 | +5 |
| UD Kaist morph strict | 66.3% | 66.4% | +0.1pp |
| UD Kaist eo strict (count) | 3987 | 3993 | +6 |
| UD GSD morph strict | 67.4% | 67.3% | -0.1pp ⚠️ |
| UD GSD eo strict (count) | 2601 | 2610 | +9 |
| Surface-only strict | 87.7% | 87.8% | +0.1pp |

**Pros**: 모든 silver eojeol +5~9 개선, surface +0.1pp.
**Cons**: sample.tsv 1 sentence 회귀, UD GSD morph -0.1pp.

### 3.3 결정: Rollback

Sprint 138 결론 ("baseline 회귀 금지") 적용. assertion(>=99.8%)은 통과하나 baseline 99.9% 깨짐. 형태론적 정확성 향상이 baseline 회귀를 정당화하기엔 lift가 작음 (eojeol +5~9, morph 변화 없음 또는 미미).

→ multi-syllable 확장 rollback. 1-syllable VV+ETM 처리만 유지.

**원인 추측**: multi-syllable에서 false positive — mecab이 "X+ETM"으로 출력했지만 실제 stem이 형태론적으로 다른 분해를 요구하는 케이스가 sample.tsv에 1건 존재.

---

## 4. 핵심 학습 포인트

### 4.1 PUD는 적합하지 않은 silver source

같은 UD framework이지만 PUD는 Penn-Treebank style + lemma 절반 누락. KAIST/GSD와 별개. **UD 전체가 동일 형식이라는 가정 위험**.

### 4.2 multi-syllable 확장은 1-syllable보다 위험

1-syllable VV+ETM은 surface 자체가 stem + 종성 (단순). multi-syllable은 더 긴 stem + 다양한 활용형이 섞여 false positive 가능. **명시 surface 목록 패턴이 더 안전** (Sprint 141 VCP+EC처럼).

### 4.3 결합 POS 13% 비율 — 분리 가치 큼

전체 tokens의 13%가 결합 POS. 안전한 패턴 식별 → 누적 lift 가능. 단 각 패턴마다 false positive 검증 필수.

### 4.4 sample.tsv 1 sentence가 baseline 정의

sample.tsv 1,100 sentence 중 1 sentence 회귀도 baseline 100%/99.9% 깸. 다른 silver +5~9 lift가 있어도 sample.tsv가 quality gate. **trade-off 시 sample.tsv 우선**.

---

## 5. Sprint 146 후보

### 후보 A: 명시 surface 목록 기반 결합 패턴 분리

상위 패턴 중 안전한 명시 surface 추가 (Sprint 141 VCP+EC 방식):
- NP+JX: "그는", "이는" (contraction 회피)
- VCP+EP: "였" → "이 + 었"
- VV+EP: 명시 동사 ("흘렸" → "흘리 + 었", "버렸" → "버리 + 었")

각 패턴마다 sample.tsv + 5-gate 검증.

### 후보 B: Full CRF Retrain (Track E)
3-5 sprint, 메인 목표.

### 후보 C: NIKL Modu 수동 다운로드
Academic license.

---

## 6. 변경 파일

- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: `test_compound_pos_frequency_analysis` 추가 (~80줄)
- `rust/crates/mecab-ko-core/src/sejong/splitter.rs`: 주석 갱신 (multi-syllable 실험 + rollback 기록)
- `docs/research/accuracy/2026-05-20_sprint145_compound_pos_analysis.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신
- 측정값 변경 없음 (rollback 완료)

---

*작성: 2026-05-20 (Sprint 145 D)*
