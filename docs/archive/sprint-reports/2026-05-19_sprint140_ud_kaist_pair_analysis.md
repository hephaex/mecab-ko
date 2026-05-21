# Sprint 140 A — UD Korean-Kaist SPLIT_DIFFERENT Pair Analysis

> Sprint 137(KLUE DP) 분석을 UD Kaist에도 적용하여 도메인 독립적 vs 도메인 특화 problematic pairs 식별. + Sprint 140 C에서 CI 게이트로 추가.

---

## 1. 측정 결과

### 1.1 데이터셋 비교

| 메트릭 | KLUE DP (Sprint 137) | UD Kaist (Sprint 140) |
|--------|---------------------|----------------------|
| Sentences | 1,995 | 1,638 |
| Total eojeols | 22,404 | 19,241 |
| SPLIT_DIFFERENT | 2,237 (10.0%) | 1,755 (9.1%) |
| Unique pairs | 570 | 479 |
| Pair occurrences | 4,330 | 3,134 |

### 1.2 상위 10 패턴 비교

| Rank | Pair | KLUE | UD Kaist | 카테고리 |
|------|------|------|----------|---------|
| 1 | (3534, 0) NNG-T → BOS/EOS | 298 | 227 | **도메인 독립** |
| 2 | (0, 1780) BOS/EOS → NNG | 264 | 204 | **도메인 독립** |
| 3 | (3533, 0) NNG-F → BOS/EOS | 196 | 257 | **도메인 독립** |
| 4 | (3534, 1780) NNG-T → NNG | 109 | 104 | **도메인 독립** |
| 5 | (3533, 1780) NNG-F → NNG | 129 | 76 | **도메인 독립** |
| 6 | (0, 0) BOS/EOS → BOS/EOS | 162 | 84 | 도메인 독립 (smaller in UD) |
| 7 | (5, 1794) EF → SF | 166 | 16 | **KLUE 특화** (구어/뉴스 문장체) |
| 8 | (3561, 1780) SH → NNG | 134 | 28 | **KLUE 특화** ("100여명" 수치 패턴) |
| 9 | (3584, 0) XR-T → BOS | 130 | 43 | KLUE 특화 (XR + 한 활용) |
| 10 | (8, 3) EP → EF | 114 | 38 | KLUE 특화 (어말어미 분리) |

### 1.3 UD Kaist 특화 상위 패턴 (KLUE 대비 새로움)

| Pair | UD count | KLUE count | 의미 |
|------|----------|-----------|------|
| (3777, 2240) XSN-T(적) → VCP(인) | 92 | 27 | "X적인" — 학술/역사적 단정 |
| (3533, 2609) NNG-F → XSN(적) | 60 | 14 | "역사적", "구체적", "체계적" |
| (3534, 2609) NNG-T → XSN(적) | 59 | 42 | "실질적", "특징적", "전통적" |
| (3777, 0) XSN-T(적) → BOS/EOS | 59 | 41 | "적으로" 종결 |
| (0, 1783) BOS/EOS → NNG(정적사태) | 68 | 36 | "한다", "온다" 동사파생 |
| (3539, 588) NNG(정적사태) → JX(는) | 34 | 29 | "다는" — 단정 보조사 결합 |

---

## 2. 핵심 발견

### 2.1 NNG 분해 패턴은 도메인 독립적

상위 1-6위 패턴 (NNG ↔ BOS/EOS, NNG ↔ NNG) 모두 두 데이터셋 공통.
이는 mecab-ko의 NNG 처리 자체가 어절 경계 처리에서 cost 분포가 split을 선호한다는 의미.

**Sprint 138 결론 재확인**: 이 NNG 패턴들의 cost 조정은 어절 경계 처리 일관성을 깨므로 sample.tsv 회귀를 야기. 도메인 무관하게 적용 불가.

### 2.2 KLUE 특화 패턴: 현대 뉴스 텍스트 특성

- EF/SF, EP/EF 분리 (어말어미 + 마침표): 짧은 문장체
- SH/NNG (한자 수치 + 명사): "100여명", "1천명" — 뉴스 보도 표현
- XR-T/BOS ("탁월한", "비롯한"): 형용사 어근 + ㄴ 활용

### 2.3 UD Kaist 특화 패턴: 학술/역사 텍스트 특성

- **XSN(적) 패턴 다수**: "역사적", "구체적", "X적인" — 학술적 단정
- NNG(정적사태): 동사파생 명사 ("한다", "있다") — 학술 문체
- mecab은 이런 학술적 한자어 결합을 적극 분해하는 경향

이는 KAIST 코퍼스가 역사/철학/학술 텍스트를 포함한다는 사실과 일치 (UD project의 KAIST 데이터셋은 한국 근대사 텍스트 중심).

### 2.4 도메인 다양화의 가치

KLUE만으로 평가하면 NNG 분해가 주된 이슈로 보이지만, UD Kaist 추가 측정으로 **XSN(적) 처리**가 학술 도메인의 별도 이슈임을 노출.

향후 cost 조정 시도 시:
- NNG 5쌍 (Sprint 138 시도, sample.tsv 회귀로 실패): 도메인 무관 → 어절 경계 영향 회피 불가
- **XSN(적) 패턴**: 도메인 특화 → KLUE 회귀 적을 가능성. Sprint 141+ 실험 후보.

---

## 3. CI 게이트 (Sprint 140 C)

### 3.1 추가 step

`.github/workflows/accuracy-gate.yml`에 `Run UD Kaist silver gate` step 추가:
- 실행: `test_ud_kaist_dual_metric --ignored --nocapture`
- Floor: morph strict ≥ 60% (CI level, test 내부는 silver tolerance 40%)
- PR comment에 4번째 섹션 추가

### 3.2 효과

- 기존 3-gate (sample / KLUE / surface-only) → 4-gate
- Sprint 138 같은 cost 조정 회귀를 **두 도메인에서 동시 감지** 가능
- 도메인-편향 회귀 (KLUE 통과 / UD 회귀) 가능성도 격리 가능

### 3.3 baseline 측정값 (CI 기준)

| Metric | Value | Floor | Status |
|--------|-------|-------|--------|
| Morph strict | 66.3% | 60.0% | ✅ |
| Morph practical | 68.0% | — | (info only) |
| Per-eojeol strict | 20.7% | — | (info only) |
| Per-eojeol practical | 21.8% | — | (info only) |

---

## 4. Sprint 141 후보

### 후보 A: XSN(적) practical 동치 추가 검토

UD Kaist에서 92건 noise (XSN-T → VCP). KLUE에서는 27건 — 도메인 차이.
"적인" / "X적" 의 분해 패턴이 분류 측면 의미 있는지 검토.

### 후보 B: UD Kaist NNG/XSN cost 조정 실험

Sprint 138 NNG 5쌍보다 도메인 특화 패턴 (XSN 관련)부터 시도. KLUE에 회귀 영향 적을 가능성. 단 sample.tsv 영향 여전히 검증 필요.

### 후보 C: dict-builder CSV 버그 수정 (Track D 진입 선행)

Sprint 138 미해결. 학습 가능 인프라 갖추기 위해 필요.

### 후보 D: NIKL Modu 평가 추가 (manual download path)

또 다른 silver gold. 단 라이선스로 redistribute 불가. 로컬-only 평가.

---

## 5. 인프라

### 신규 테스트
- `test_ud_kaist_split_diff_connection_pairs` in `tests/accuracy_eval.rs`

### 신규 CI step
- `.github/workflows/accuracy-gate.yml`: `Run UD Kaist silver gate`

### 보고서
- 본 문서 (Sprint 140 A 분석)
- 비교 대상: `docs/research/accuracy/2026-05-19_sprint137_connection_cost_analysis.md` (KLUE pair 분석)

---

*작성: 2026-05-19 (Sprint 140 A+C)*
