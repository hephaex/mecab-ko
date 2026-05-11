# Sprint 126 P1: 사전 정책 차이 정량화 + 이중 Equivalence

> 핵심 결과: 3-mode 측정으로 morpheme 65.8% → 70.3% (+4.5pp 누적), eojeol 19.2% → 21.7% (+2.5pp).
> NNB/NNG counter convention 추가가 morpheme +1.0pp / eojeol +0.7pp의 추가 lift.
> 진단 추정(85-90%)에 비해 작음 — 데이터셋의 진짜 분석 오류가 25%+ 차지.

---

## 배경

Sprint 125 P1에서 conservative tag equivalence 적용 후 morpheme +3.2pp / eojeol +1.6pp.
진단 추정(31% errors are tag scheme)보다 작은 lift였음. Sprint 126 P1은 **사전 정책 차이**
(NNG/NNP/NNB confusions) 정량화 + 추가 equivalence 결정.

---

## P1-1: NNG/NNP/NNB 혼동 케이스 추출

`test_klue_dp_nng_nnp_analysis` 추가. KLUE DP 평가 실패에서 NNG/NNP/NNB 관련
POS_ONLY 케이스만 추출, confusion direction별 샘플 출력.

### 결과 (Top patterns)

| Confusion | 건수 | 샘플 |
|-----------|------|------|
| NNB → NNG | **158** | 씨, 일, 명, 회, 달러 (counter words) |
| NNG → NNP | **147** | 한국어, 당, 작사가, 시리얼, 이미지 |
| MAG → NNG | 95 | 현재, 지금, 통상, 모두, 우선 |
| NNP → NNG | **95** | 어벤져스, 카투사, 외무부, 아바, 강 |
| SL → NNP | **52** | NXC, CCTV, tvN, IOC, NASA, MBC, LG |
| VV → NNG | 43 | 드리, 벌이, 외치, 갖, 보이 |

총 NNG/NNP/NNB 관련 POS_ONLY: 809건 (전체 1,974 errors 중 41%).

---

## P1-2: 케이스 분석 (real error vs convention)

### 분류 결과

| 패턴 | 건수 | 분류 | 근거 |
|------|------|------|------|
| **NNB → NNG** | 158 | **Convention** | 거의 전부 counter words. KLUE는 NNB(의존명사), mecab은 NNG. 둘 다 internally consistent. |
| NNG → NNP | 147 | **Real error** | "한국어", "당", "이미지" 등이 NNP가 아님. mecab의 over-tagging. |
| MAG → NNG | 95 | Real disambig | "현재", "지금"은 문맥 의존 (부사 vs 명사). mecab의 disambiguation 부족. |
| NNP → NNG | 95 | **Real error** | "어벤져스", "외무부" 등 명백한 고유명사를 NNG로 under-tagging. |
| **SL → NNP** | 52 | **Convention** | 영문 약어: KLUE는 SL(외국어), mecab은 NNP(고유명사). |
| VV → NNG | 43 | Real error | 동사 어간을 NNG로 잘못 분석. |

### 사용자 결정 (사용자에게 물어봄)

- Conservative equivalence: **{SL, NNP} 추가** (이전 그룹 + 영문 약어 convention)
- Practical equivalence (별도 그룹): **+ {NNB, NNG}** 추가 (counter words convention)
- NNG/NNP는 real error라 동치 안 함

---

## P1-3: 이중 Equivalence 구현

### 두 상수 정의

```rust
pub const TAG_EQUIVALENCE_GROUPS: &[&[&str]] = &[
    &["SP", "SC"],
    &["SS", "SY", "SSO", "SSC"],
    &["MM", "MMD", "MMN", "MMA"],
    &["SL", "NNP"],            // Sprint 126 추가
];

pub const TAG_EQUIVALENCE_GROUPS_PRACTICAL: &[&[&str]] = &[
    /* conservative + */
    &["NNB", "NNG"],           // Sprint 126 추가 (counter words)
];
```

### API

- `pos_tags_equivalent(a, b)` — conservative
- `pos_tags_equivalent_practical(a, b)` — practical
- 헬퍼: `pos_tags_equivalent_in(a, b, groups)` — 임의 그룹

### Trade-off (practical)

언어학적으로 NNB(의존명사)와 NNG(일반명사)는 다른 범주.
Practical mode는 KLUE-vs-mecab convention 차이를 흡수하는 대신 진짜 NNB/NNG 의미적
오류도 함께 흡수. **검색/검색 인덱싱 등 downstream 사용**에는 NNB/NNG 구분이 중요하지
않은 경우가 많아 practical 유용. **정밀한 형태소 분석 평가**에는 conservative 권장.

---

## P1-3: 측정 결과 (3-mode 비교)

| Mode | Morpheme | Δ vs prev | Eojeol | Δ vs prev |
|------|----------|-----------|--------|-----------|
| Strict | 65.8% | — | 19.2% (4,299) | — |
| Lenient (conservative) | 69.3% | +3.4pp | 21.0% (4,708) | +1.8pp |
| Lenient (practical) | 70.3% | +1.0pp | 21.7% (4,868) | +0.7pp |

**Strict → Practical 누적 lift**: morpheme +4.5pp, eojeol +2.5pp.

### 해석

- **Conservative 추가의 lift**: SL/NNP만 추가했는데 morpheme +0.2pp 정도 (이전 +3.2 → +3.4),
  이전 측정 대비 작은 변화. SL→NNP가 52건 정도라 expected.
- **Practical 추가의 lift**: NNB/NNG가 158건이지만 morpheme +1.0pp / eojeol +0.7pp만 증가.
  이유: 같은 어절 내 다른 issue(분할/사전)와 결합된 케이스가 많아 cascade 효과.
- **진단 추정(~85-90%)과의 차이**: 측정 70.3%로 추정에 못 미침. 데이터셋의 **진짜 분석 오류 25%+**
  존재 시사. NNG/NNP confusion(242건), MAG/NNG(95건), VV/NNG(43건) 등이 진짜 disambiguation 오류.

---

## 핵심 학습 포인트

### 1. 사전 정책 차이 vs 진짜 오류 구분 = 결정의 핵심

**Why**: NNG/NNP/NNB confusion 809건이 모두 같은 카테고리가 아님.
- Convention (NNB/NNG counter, SL/NNP foreign): 동치 처리 정당화 가능
- Real error (NNG/NNP, MAG/NNG, VV/NNG): 동치 처리하면 진짜 오류 은폐

샘플 inspection 없이 "808/809 동치 처리"는 잘못된 결정. 패턴별 case-by-case 분류 필수.

**적용 원칙**: 동치 그룹 추가 결정 시 항상 샘플 10-20건 inspection. 샘플에서 80%+가
convention이면 추가, 50% 미만이면 보류.

### 2. 이중 Equivalence (conservative + practical)의 가치

**Why**: 모든 use case에 같은 동치 그룹이 적합하지 않음.
- 정밀 평가: conservative (언어학적으로 명백한 표기/관용 차이만)
- Downstream 사용: practical (실용적 무차별 처리)

단일 equivalence map은 한쪽 use case에 부적합.

**적용 원칙**: 평가 인프라는 use case별 equivalence를 분리 제공. 사용자가 필요에 따라 선택.

### 3. Cascade 효과로 인한 작은 lift는 정상

**Why**: NNB/NNG 158건이 전부 lift로 이어지지 않음 (실제 +1.0pp morpheme = ~510 토큰).
같은 어절에 다른 issue가 결합되어 있으면 NNB/NNG 동치만으로는 어절이 정답이 안 됨.

**적용 원칙**: "X 패턴 N건 흡수 → +Npp lift" 추정은 과대추정. cascade로 인해
실제 lift는 N의 30-50%가 일반적.

### 4. 진단 추정 vs 측정 결과의 괴리는 진짜 분석 오류 비율을 시사

**Why**: 진단에서 ~31% tag scheme + ~22% 사전 정책 = ~53% convention. 만약 정확하면
strict 65.8% → 100 - (100-65.8)*(1-0.53) = 84% 정도 lift 기대. 실제 70.3%.
차이 14pp는 **진짜 분석 오류**가 25%+ 차지함을 의미 (cascade 효과 일부도 있음).

**적용 원칙**: 진단 추정이 목표치, 측정이 현실. 차이가 "다음 sprint의 진짜 작업 영역".

---

## Sprint 127 권고

### P1 후보: 복합명사 분할 정책 분석
- Phase 1 진단의 ~20% 추정 잔여 영역
- "팝스타/NNP" vs "팝/NNG + 스타/NNG" 같은 케이스 통계
- Slice-level matching (양쪽 인정) 메트릭 검토

### P2 후보: 진짜 분석 오류 디버그 (25%+ 추정)
- NNG↔NNP real errors (242건)
- MAG↔NNG disambiguation (95건)
- VV↔NNG (43건)
- 어떤 케이스가 mecab-ko-dic 사전 보강으로 해결 가능한지
- 어떤 케이스가 CRF 재학습 필요한지

### P3 후보: noisy 데이터 추가 (사용자 우선순위 "높음", carryover)
- 편집 register vs 구어/SNS register

### P4 후보: CI 통합
- HF 자동 다운로드 + KLUE DP gate (3-mode 보고)

---

## 산출물

- `rust/crates/mecab-ko-core/src/evaluate.rs`:
  - `TAG_EQUIVALENCE_GROUPS` 확장 (`SL/NNP` 추가)
  - `TAG_EQUIVALENCE_GROUPS_PRACTICAL` 신규 (`NNB/NNG` 추가)
  - `pos_tags_equivalent_practical()` 함수
  - 헬퍼 `pos_tags_equivalent_in()`
  - 4개 unit test (SL/NNP, practical NNB/NNG, conservative inheritance)
- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`:
  - `test_klue_dp_nng_nnp_analysis` 추가 (sample extraction)
  - `test_klue_dp_dual_metric_lenient` 확장 (3-mode 비교)
- 본 보고서

빌드/clippy clean, 모든 기존 테스트 pass.

---

*작성: 2026-05-11*
*작성자: Mario Cho (hephaex@gmail.com)*
*Sprint: 126 P1*
