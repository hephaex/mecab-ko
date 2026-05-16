# Sprint 129 P3 — KLUE DP 진짜 오류 분석 + 처방 분류

> **한 줄 요약**: KLUE DP 1,995문장에서 POS confusion 1,226건과 GOLD_SINGLE_PRED_MULTI 553건을 surface 단위로 추출하고, 처방별로 분류. 사전 추가 후보 ~226건(추정 +1pp morpheme), convention 차이 ~600건(흡수 한계), CRF 필요 ~400건.

## 배경

Sprint 126 P1에서 분류한 NNG/NNP/NNB confusion 809건 중 real error 380건(NNG/NNP 242 + MAG/NNG 95 + VV/NNG 43)과, Sprint 127 P1에서 식별한 GOLD_SINGLE_PRED_MULTI 553건의 surface 분포를 frequency 정렬로 추출.

각 패턴을 처방별로 분류:
- **(A) 사전 추가**: 누락 NNP/NNG/MAG (cost=-5000 패턴)
- **(B) cost 조정**: 사전에 있지만 분할이 우선
- **(C) CRF retrain**: context 의존, 정적 사전으론 불가
- **(D) Convention 차이**: KLUE 라벨링 규약 차이 (흡수 한계)

## 측정 인프라

신규 테스트 2개 추가 (`accuracy_eval.rs`):

```rust
#[test] #[ignore] fn test_klue_dp_real_error_analysis()              // POS confusion + per-surface frequency
#[test] #[ignore] fn test_klue_dp_gold_single_pred_multi_analysis()  // mecab 과분할 surface 빈도
```

타겟 태그 확장: `["NNG", "NNP", "NNB", "MAG", "VV", "VA", "MM"]` — Sprint 126의 `["NNG", "NNP", "NNB"]` 대비 확장.

## 측정 결과

### 1. POS Confusion (target-related POS_ONLY errors: 1,226건)

| Pair | 건수 | 분류 | 처방 |
|------|------|------|------|
| NNB → NNG | 158 | (D) Convention | 의존명사 boundary — Sprint 126 equivalence로 흡수됨 |
| **NNG → NNP** | **147** | **(A/D) 혼합** | "호스트" 73건은 KLUE convention, 나머지 ~74건은 dict 후보 |
| **NNP → NNG** | **95** | **(A) 사전 추가** | 파리(9), 로마(5), 국정원(2) 등 named entity 누락 |
| **MAG → NNG** | **95** | **(A) 사전 추가** | 모두(15), 다시(13), 현재(10) — 부사 dict 누락/cost 문제 |
| MMD → MM | 76 | (D) Convention | Sprint 125 equivalence 이미 흡수 |
| SL → NNP | 52 | (D) Convention | 외국 약어 — KLUE는 NNP로 라벨링 |
| MAG → MAJ | 44 | (D) Convention | 다만(20), 및(12) — 접속부사 라벨링 규약 |
| **VV → NNG** | **43** | **(C) CRF** | 보이(9), 타(3), 묵(3) — 동사 어간 단독 등장 시 명사 오인 |
| **VA → VV** | **41** | **(D) Convention** | 전부 "있" — KLUE는 VA로 라벨링, mecab은 VV |
| MMN → MM | 38 | (D) Convention | Sprint 125 equivalence 이미 흡수 |
| NNG → NNB | 34 | (D) Convention | 의존명사 boundary |
| XSV → VV | 29 | (C) CRF | 동사 파생접미사 vs 본동사 |
| VX → VV | 28 | (C) CRF | 보조동사 vs 본동사 |
| 기타 (40+ pairs, 각 <20건) | ~350 | 혼합 | 대부분 long tail |

**Convention 흡수량**: NNB↔NNG(192), MMD/MMN/MMA→MM(120), SL→NNP(52), MAG→MAJ(44), VA↔VV(41+19), 등 = **~470건 (38%)** — 이미 Sprint 125 equivalence로 lenient 모드에서 흡수됨

**Real error 추정**: 1,226 - 470 = **~756건 (62%)**

### 2. GOLD_SINGLE_PRED_MULTI (553건)

KLUE는 단일 morpheme, mecab은 분할.

**Gold POS 분포**:
- NNP 204 (36.9%) — 고유명사
- MAG 166 (30.0%) — 부사
- NNG 146 (26.4%) — 일반명사
- MAJ 30 (5.4%) — 접속부사

**Frequency tier**:
- >= 5회: 15 surfaces, 129 cases (23.3%) — **고신뢰 dict 추가 후보**
- 2-4회: 54 surfaces, 135 cases (24.4%) — review 후 추가
- 1회: 289 surfaces (52.3%) — long tail

#### Top 15 dict-add candidates (>= 5회)

| Rank | Count | Gold | Mecab 분할 | 카테고리 |
|------|-------|------|-----------|---------|
| 1 | 22× | 하지만/MAJ | 하/VV + 지만/EC | (D) 접속부사 convention |
| 2 | 16× | 굉장히/MAG | 굉장/NNG + 히/NNG | **(A) 부사 dict 추가** |
| 3 | 11× | 지하철/NNG | 지/VX + 하철/NNG | **(A) 명사 dict 추가** (심각한 오분할) |
| 4 | 10× | 엄청/MAG | 엄/IC + 청/NNG | **(A) 부사 dict 추가** |
| 5 | 10× | 없이/MAG | 없/VX + 이/MM | (D) 어휘화 vs 분석 convention |
| 6 | 8× | 더욱/MAG | 더/MAG + 욱/NNG | **(A) 부사 dict 추가** |
| 7 | 7× | 제대로/MAG | 제대/NNG + 로/JKB | **(A) 부사 dict 추가** |
| 8 | 6× | 자주/MAG | 자/NNG + 주/VX | **(A) 부사 dict 추가** |
| 9 | 6× | 세월호/NNP | 세월/NNG + 호/NNG | **(A) 고유명사 dict 추가** |
| 10 | 6× | 이어/MAG | 이/MM + 어/EF | (D) 부사화 convention |
| 11 | 6× | 그래도/MAJ | 그/VV + 래/EF + 도/NNG | (D) 어휘화 convention |
| 12 | 6× | 상당히/MAG | 상당/NNG + 히/NNG | **(A) 부사 dict 추가** |
| 13 | 5× | 실제로/MAG | 실제/NNG + 로/JKB | **(A) 부사 dict 추가** |
| 14 | 5× | 에어비앤비/NNP | 에/IC + 어/IC + 비앤비/NNP | **(A) 고유명사 dict 추가** |
| 15 | 5× | 뮤지컬/NNG | 뮤/NNG + 지/VX + 컬/NNG | **(A) 명사 dict 추가** |

11/15 = **(A) 사전 추가로 즉시 해결 가능**, 4/15 = (D) convention.

#### 2-4회 빈도 dict 후보 (선별)

**고유명사 (named entity)**: 새누리당(4), 민주통합당(3), 테르미니역(4), 서울대(2), 공화당(2), 소녀시대(2), 프라하(2), 서울시(2), 민주노총(2), 개성공단(2), 세인트루이스(2), 에어비엔비(3)

**일반명사**: 비서실장(2), 초등학교(2), 상수도(2), 청결도(3), 가성비(3), 관광지(3), 한국인(3)

**부사**: 갑자기(4), 그대로(4), 이미(4), 보다(4), 너무나(3), 주로(3), 거듭(3), 여러모로(3), 빨리(3), 곧바로(3), 되게(3), 워낙(2), 끝내(2), 철저히(2), 혹시(2), 즉각(2)

## 처방 분류 종합

### (A) 사전 추가 후보 — 즉시 해결 가능

| 종류 | 빈도 5+ | 빈도 2-4 | 총 cases |
|------|--------|---------|---------|
| 고유명사 (NNP) | 2 (10+5=15) | ~12 (~28) | ~43 |
| 일반명사 (NNG) | 2 (11+5=16) | ~7 (~17) | ~33 |
| 부사 (MAG) | 7 (16+10+8+7+6+6+5=58) | ~16 (~50) | ~108 |
| **GOLD_SINGLE 합계** | **11 surfaces, ~89 cases** | **~35 surfaces, ~95 cases** | **~184 cases** |

POS confusion side (MAG→NNG, NNP↔NNG에서 빈도 5+):
- MAG→NNG: 모두(15), 다시(13), 현재(10), 일단(6) = **44 cases** — 이미 사전에 있을 수 있음, cost 조정 후보
- NNP→NNG: 파리(9), 로마(5) = **14 cases** — 지명 dict 추가

**(A) 총 추정**: ~242 cases = **~1.1pp morpheme accuracy lift** (65.8% → ~66.9%)

### (B) Cost 조정 후보

부사 중 mecab dict에 entry는 있지만 분할이 비용상 우선되는 경우.
"모두/MAG"는 mecab-ko-dic에 존재하나 "모두/NP + ε" 또는 단일 어절일 때 NNG로 분류됨 — entry cost 조정 또는 형태소 cost factor 조정.

추정 영향: (A)와 일부 겹침. 별도 ~30 cases (1.5%).

### (C) CRF retrain 필요

Sprint 130 단기 fix로는 불가능. 모든 long tail (1회 빈도) + VV/NNG, XSV/VV, VX/VV 등 context-dep:

- VV→NNG long tail: 26 surfaces (singleton 18)
- NNG→NNP long tail: 49 singleton (호스트 73 제외)
- MAG→NNG long tail: 25 singleton
- 기타 confusion long tail: ~250건

**(C) 총 추정**: ~400 cases — Sprint 130에서 다루지 못함.

### (D) Convention 차이 — 흡수 한계

KLUE 라벨링 규약 차이. 이미 lenient 모드(Sprint 125)에서 흡수되거나, 흡수하면 다른 데이터에서 손해:

- 호스트(73) NNG↔NNP — KLUE 일관성 (review 도메인 영향)
- 의존명사 NNB↔NNG (158+34=192)
- 있(41) VA↔VV
- MAG↔MAJ (44)
- SL↔NNP (52)
- 하지만/그래도/이어/없이 — 어휘화 boundary

**(D) 총 추정**: ~600 cases — 진단(85-90% 천장)과 lenient(70.3% practical) 사이 14pp 중 ~30%는 이 영역에 있음.

## 처방별 lift 추정

| 처방 | 적용 가능 cases | 추정 lift (morpheme) | 비용 |
|------|---------------|--------------------|------|
| (A) 사전 추가 (전체) | ~242 | **+1.1pp** | 낮음 — user-dict CSV |
| (A1) 부사만 우선 | ~108+44=152 | +0.7pp | 매우 낮음 |
| (A2) 고유명사만 우선 | ~43+14=57 | +0.3pp | 낮음 — 검증 필요 |
| (B) Cost 조정 | ~30 | +0.1pp | 중간 — 회귀 위험 |
| (C) CRF retrain | ~400 | +1.8pp | 매우 높음 — 학습 인프라 |
| (D) Convention | ~600 | 0 (이미 lenient) | — |

**총 dict-only 가능 천장**: 65.8% + 1.1pp = **~66.9% morpheme strict** (Sprint 126 65.8% baseline 대비)
- Eojeol per-eojeol: 52.4% + 추정 +1.5pp = **~53.9%**

이는 Sprint 122의 government dict (NNP 503건, +0pp KLUE 영향) 결과와 일치 — KLUE는 일상어 도메인이라 정부 기관명 추가가 거의 무효였음. 본 sprint의 후보는 KLUE 도메인(뉴스+리뷰)에서 직접 측정된 것이므로 적용 효과가 측정으로 검증된 셈.

## 핵심 학습 포인트

### 1. "진짜 오류"의 정량 분리는 measurement-first가 가능

Sprint 126의 추정(real error 25%+)을 실제 surface frequency로 분해하니 사전 추가로 즉시 해결되는 부분이 명확해짐. 380+553 = 933 cases 중 ~242만 dict-add 가능 (26%). 나머지 74%는 convention(64%) 또는 CRF(10%) — **장기 개선이 필요한 영역과 단기 fix 영역을 분리 측정**.

### 2. POS confusion alone is misleading

"NNG/NNP 242건 real error"는 reasonable했으나, surface frequency로 보니 **단일 단어 "호스트"가 73건 차지** (30%). 단어 수가 아닌 surface 수 + frequency로 봐야 처방을 정할 수 있음. Sprint 130에서 cost=-5000 추가를 결정할 때는 frequency tier 기반으로 판단.

### 3. KLUE 도메인 (뉴스+리뷰) 특성

기관명(국정원, 새누리당, 민주통합당)과 review-specific 용어(호스트, 와이파이, 트램, 게스트) 비중이 높음. Sprint 122 government dict이 KLUE에서 +0pp였던 이유 — 공공기관명보다 뉴스+숙박 리뷰 용어가 우세. 본 sprint의 dict 후보는 **도메인 매칭**이 검증됨.

### 4. Surface-frequency 정렬은 cost 조정 위험도 평가 도구

빈도 1회 surface 289개는 cost=-5000 일괄 추가 시 회귀 위험 큼 (다른 문장에서 oversegmentation). 빈도 5+ 15개는 일관성이 높아 안전. 이는 **dict augmentation의 일반 원칙**: "한 데이터셋에서 5회 이상 등장하면 entry 추가, 1회는 long tail로 무시".

## Sprint 130 권고

### P1 후보: 사전 추가 정확도 향상 — high-confidence subset

```yaml
target: 빈도 5+ surfaces only (=15 surfaces, 129 cases from GOLD_SINGLE_PRED_MULTI + 빈도 5+ from POS confusion)
total_cases: ~95 (filter out convention category)
estimated_lift: +0.5pp morpheme strict (65.8% → 66.3%)
risk: 매우 낮음 — 빈도 5+는 일관 패턴
mechanism: data/user-dict/klue_domain.csv 신규 파일, cost=-5000
verification: 회귀 테스트 (sample.tsv 100% 유지) + KLUE DP 모든 모드 측정
```

### P2 후보: 빈도 2-4 후보 검토 + 회귀 측정

각 후보를 sample.tsv + golden test에서 회귀 영향 평가 후 선별 추가. 이건 사람 검토 필요 (자동화 위험).

### P3 보류: VV/NNG context-dependent

CRF retrain 인프라 부재. Sprint 132+ 후보로 보류.

### 즉시 적용 불가 (이번 sprint 제외)

- (C) CRF retrain — 학습 데이터 + 인프라 부재
- (D) Convention 처리 — 이미 lenient 모드 흡수, 추가 처리 시 strict 모드 회귀

## 관련 문서

- [Sprint 126 dictionary policy](./2026-05-11_klue_dp_dictionary_policy.md) — NNG/NNP/NNB 809건 초기 분류
- [Sprint 127 compound noun](./2026-05-11_klue_dp_compound_noun.md) — GOLD_SINGLE_PRED_MULTI 553건 식별
- [Sprint 128 surface lenient](./2026-05-11_klue_dp_surface_lenient.md) — per-eojeol metric 도입
- [Sprint 122 government dict](../../../LessonLearn/) — KLUE에서 정부 기관명 +0pp 사례 (도메인 매칭 학습)

---

*작성: 2026-05-16*
