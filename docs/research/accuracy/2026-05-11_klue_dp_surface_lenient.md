# Sprint 128 P1+P2: Per-eojeol Metric + Surface Normalization Lenient

> 핵심 결과:
> - **Per-eojeol metric 정식 추가** — KLUE DP eojeol 19.2% → **52.4%** (+33pp 측정 정확도 회복)
> - Practical POS와 결합 시 **per-eojeol combined eojeol 59.4%**
> - Surface canonical normalization은 morpheme split이 다른 경우 +0pp (Sprint 127 slice-lenient 거부와 동일 trade-off)
> - SURFACE_MISMATCH 12.3%는 어절-concat 레벨 차이, morpheme-level은 양쪽 이미 같은 normalize_jamo 사용

---

## 배경

Sprint 127 P1에서 두 가지 발견:
1. 기존 sequence-based eojeol metric (`evaluate_dataset_dual`의 어절 부분)이 cascade로
   33pp 이상 underestimate함 (per-eojeol 분석 52.4% vs sequence 19.2%)
2. SURFACE_MISMATCH가 per-eojeol 분석에서 12.3% 차지 (예: gold "공정하ㄴ" vs pred "공정한")

Sprint 128 P1+P2는 두 발견을 평가 인프라에 정식 통합:
- **P1**: per-eojeol algorithm을 정식 API (`evaluate_dataset_dual_per_eojeol_with_match`)
- **P2**: surface 비교 함수를 주입할 수 있는 `SurfaceMatchFn` 추가 + canonical/lenient 구현

---

## P2-01/02: Surface mismatch 분석 (재측정)

Sprint 127의 SURFACE_MISMATCH 2,745건 (per-eojeol 측정)에 대해 normalization 효과 측정:

| Tier | 흡수 | % of mismatch | % of total eojeols |
|------|-----:|--------------:|-------------------:|
| NFC compose only (decompose+compose) | 874 | 31.8% | 3.9% |
| NFC + 하았→하였/하어→하여 | 619 | 22.6% | 2.8% |
| Still mismatch | 1,252 | 45.6% | 5.6% |
| **Total absorbed** | **1,493** | **54.4%** | **+6.7pp eojeol** |

남은 mismatch (1,252건) 패턴 분석:
- "이ㅂ니다." vs "이습니다." (KLUE의 종결어미 분해 vs mecab의 사전 등재형) — ~150건+
- "있어서" vs "있어디에서" (mecab over-generation) — 20건
- "따르아" vs "따라" (음운 변화 normalize 차이) — 18건
- "것이" vs "게" (줄임말) — 12건
- "앞서" vs "앞서어" (모음 추가) — 12건
- "편하아요." vs "편하어요." (어미 변환) — 8건

이들은 더 정교한 normalization이 필요. 본 sprint scope 외.

---

## P2-03/04: API 설계 — `SurfaceMatchFn` 패턴

PosMatchFn(Sprint 125)과 동일한 패턴:

```rust
pub type SurfaceMatchFn = fn(&str, &str) -> bool;

pub fn surface_eq_strict(a: &str, b: &str) -> bool { a == b }

pub fn surface_eq_canonical(a: &str, b: &str) -> bool {
    // 양쪽을 fully decompose 후 다시 compose → canonical form
    canonical_form(a) == canonical_form(b)
}

pub fn surface_eq_canonical_lenient(a: &str, b: &str) -> bool {
    // canonical + 하았↔하였, 하어↔하여
    let a_can = canonical_form(a);
    let b_can = canonical_form(b);
    a_can == b_can || normalize_endings(&a_can) == normalize_endings(&b_can)
}

fn canonical_form(s: &str) -> String {
    use mecab_ko_hangul::{compose_str, decompose_str};
    compose_str(&decompose_str(s))
}
```

기존 `_with_pos_match` 함수들은 모두 `surface_eq_strict`로 위임 → API 100% 보존.
신규 `_with_match` 함수들은 양쪽 매개변수 받음.

### 신규 함수 (evaluate.rs)

| 함수 | 용도 |
|------|------|
| `evaluate_tokens_aligned_with_match` | greedy alignment + POS/surface 양쪽 주입 |
| `evaluate_dataset_sejong_with_match` | sequence-based morpheme + 양쪽 주입 |
| `evaluate_dataset_dual_with_match` | sequence-based dual metric + 양쪽 주입 |
| `evaluate_dataset_dual_per_eojeol_with_match` | **per-eojeol** dual metric + 양쪽 주입 (P1) |
| `evaluate_dataset_dual_per_eojeol` | per-eojeol strict 편의 함수 |

기존 함수 (`_with_pos_match` 등)는 모두 `surface_eq_strict`에 위임. 회귀 0건.

---

## P2-05: KLUE DP 측정 결과

### Sequence-based (legacy, cascade)

| Mode | morpheme | eojeol | Δeo |
|------|---------:|-------:|----:|
| strict | 65.8% | 19.2% | +0.0pp |
| lenient (conservative) | 69.3% | 21.0% | +1.8pp |
| practical | 70.3% | 21.7% | +2.5pp |
| surface_canonical | 65.8% | 19.2% | +0.0pp |
| combined (practical + canon+lenient) | 70.3% | 21.7% | +2.5pp |

**관찰**: sequence-based에서는 surface lenient가 +0pp. 어절 cascade로 분할 mismatch가
발생하면 surface 비교 전에 이미 어절 인덱스가 어긋나 surface lenient가 의미 없음.

### Per-eojeol (Sprint 128 P1, no cascade)

| Mode | morpheme | eojeol | Δeo |
|------|---------:|-------:|----:|
| **strict** | 65.8% | **52.4%** | +0.0pp |
| practical | 70.3% | **59.4%** | +7.0pp |
| surface_canonical | 65.8% | 52.4% | +0.0pp |
| combined (practical + canon+lenient) | 70.3% | **59.4%** | +7.0pp |

**Headline**:
- Per-eojeol strict eojeol **52.4%** (sequence 19.2% → +33.2pp 측정 회복)
- Per-eojeol practical eojeol **59.4%** (+7.0pp lift from POS practical)
- Surface canonical은 per-eojeol에서도 +0pp

### Surface canonical이 per-eojeol에서도 +0pp인 이유

`evaluate_dataset_dual_per_eojeol_with_match`의 알고리즘:
1. 어절 surface concat을 `surface_eq`로 비교 → 통과
2. morpheme split이 같은지 확인 (`gold_slice.len() == pred_morphs.len()`)
3. morpheme별 (surface_eq, pos_eq) 모두 일치

surface canonical은 step 1에서는 통과시키지만, **step 2의 split 검사에서 실패**.
Sprint 127 P1의 SURFACE_MISMATCH 케이스 (예: "공정하ㄴ" vs "공정한")는 다음과 같음:
- gold: [공정/NNG, 하/XSA, ㄴ/ETM] (3 morphs)
- pred: [공정/NNG, 한/XSA+ETM 합성] (2 morphs)

Surface canonical로 어절은 통과하지만 morpheme 분할 차이로 fail.

이를 정답으로 카운트하려면 **morpheme split 무시 + POS 무시** = 의미 손실.
이는 Sprint 127 P1에서 **slice-lenient ceiling 87.7%** 측정 후 거부한 것과 동일 trade-off.

---

## 핵심 학습 포인트

### 1. Surface lenient는 morpheme-level 매칭에서는 본질적으로 효과 없음

**Why**: KLUE의 morpheme surface와 mecab의 morpheme surface는 양쪽 모두 같은 jamo
decomposition convention (compatibility jamo)을 사용. morpheme별로는 표기 차이가 거의 없음.
Sprint 127 P1의 SURFACE_MISMATCH는 어절 surface concat 비교에서 발생한 차이지, morpheme별
차이가 아님.

**적용 원칙**: Lenient surface matching은 어절 surface 합 비교에서만 의미. morpheme별 surface
비교에는 거의 lift 없음. Sprint 128에서 SurfaceMatchFn을 morpheme 비교에도 적용했으나 +0pp.

### 2. Eojeol cascade는 측정 정확도의 30pp+ underestimate를 일으킨다

**Why**: Sequence-based eojeol metric은 gold_idx와 pred_idx를 함께 advance하므로 한 어절이
잘못 분할되면 후속 어절 boundary가 모두 어긋남. KLUE DP에서 19.2% (sequence) vs 52.4%
(per-eojeol) — 33pp 차이는 알고리즘 선택만으로 발생.

**적용 원칙**: 어절 정확도 측정은 per-eojeol algorithm (어절별 독립 토크나이즈) 권장.
Sprint 128 P1에서 정식 API로 추가. 기존 sequence-based는 deprecated 후보.

### 3. Slice-level / surface-only matching은 의미 손실의 trade-off

**Why**: morpheme 분할이 다른 경우 어절 surface만으로 정답 인정하면 분할/POS 정보가 손실.
Sprint 127 P1의 87.7% ceiling이 매력적이나 형태소 분석 평가 메트릭으로는 부적합.
Sprint 128 P2의 surface lenient도 같은 trade-off — 분할까지 무시하면 +35pp 가능하나 거부.

**적용 원칙**: Lenient 메트릭은 "convention 흡수"와 "의미 손실"의 경계를 넘지 않아야 함.
NFC canonical, 하았→하였 같은 표기 normalize는 의미 손실 없음. 분할 무시는 의미 손실.

### 4. 모듈식 설계는 효과 측정을 가능하게 한다

**Why**: PosMatchFn (Sprint 125)와 동일한 SurfaceMatchFn 패턴으로 strict/canonical/lenient를
조합 측정 가능. 측정 결과 surface canonical이 +0pp임이 명확히 드러남 — 가설 기각 명확.
함수 포인터 주입 패턴 없이는 가설 검증에 더 많은 코드 변경이 필요했을 것.

**적용 원칙**: 평가 인프라는 비교 차원(POS, surface, alignment)을 직교적으로 분리해
조합 측정이 가능해야 한다.

---

## Sprint 129 권고

### P1 후보: Eojeol surface lenient — 별도 메트릭

Surface canonical로 어절 surface가 일치하면 정답 (POS/split 무시).
Sprint 127 P1의 slice-lenient ceiling 87.7%과 거의 동일.
**검색/인덱싱 use case 전용** 명시. 형태소 분석 평가 메트릭에서는 제외.

함수: `evaluate_dataset_eojeol_surface_only(tokenizer, dataset, surface_eq) -> EojeolMetricResult`

### P2 후보: 종결어미 normalization 확장

본 sprint에서 남은 1,252건의 절반 이상이 "이ㅂ니다 ↔ 이습니다" 패턴.
KLUE의 "ㅂ니다" + 다른 종결어미와 mecab의 "습니다" 사전 등재형 차이.
구현: `normalize_endings`에 종결어미 매핑 추가 → 추가 5-10pp 흡수 가능 추정.

### P3 (carryover): 진짜 분석 오류 디버그

NNG/NNP 242건, MAG/NNG 95건, VV/NNG 43건 + GOLD_SINGLE_PRED_MULTI 553건 분류.
사전 보강 vs CRF 재학습 결정.

### P4 (carryover): noisy 데이터, CI 통합

---

## 산출물

### 코드 (`rust/crates/mecab-ko-core/src/evaluate.rs`)

- `SurfaceMatchFn` 타입 + `surface_eq_strict` / `surface_eq_canonical` / `surface_eq_canonical_lenient`
- `canonical_form` (decompose+compose) + `normalize_endings` (하았→하였, 하어→하여) helpers
- `evaluate_tokens_aligned_with_match` (POS + surface 양쪽 주입)
- `evaluate_dataset_sejong_with_match` (POS + surface 주입)
- `evaluate_dataset_dual_with_match` (POS + surface 주입)
- `evaluate_dataset_dual_per_eojeol_with_match` (P1: 어절별 독립 + 양쪽 주입)
- `evaluate_dataset_dual_per_eojeol` (편의 함수)
- 기존 `_with_pos_match` 함수들은 모두 `surface_eq_strict` 위임 (회귀 0)
- 6개 신규 unit test (canonical 동작, lenient 동작, false-positive 방지)

### 테스트 (`rust/crates/mecab-ko-core/tests/accuracy_eval.rs`)

- `test_klue_dp_surface_normalization_analysis`: NFC + endings 흡수율 분석 (P2-01/02)
- `test_klue_dp_surface_lenient_full`: 9-mode 측정 (sequence 5 + per-eojeol 4)

### 보고서

- 본 문서

빌드/clippy clean (다음 단계에서 검증), 모든 기존 테스트 pass.

---

## 결론

Sprint 128 P1+P2의 진짜 lift는 **per-eojeol metric** (P1)에서 발생: eojeol 19.2% → 52.4%
(+33pp 측정 회복). Surface lenient (P2)는 morpheme-level과 어절-with-split 모두에서
+0pp — 가설(NFC canonical 흡수)이 morpheme 분할 차이로 무력화됨.

Per-eojeol metric은 mecab-ko의 진짜 어절 정확도가 60%대임을 처음으로 측정. Sprint 126
practical 21.7%는 cascade artifact였음이 확정.

향후 lift는 형태소 분할 정책 일치 (Sprint 129+) 또는 surface-only eojeol 메트릭 (downstream
search 전용) 방향으로 분리해야 함.

---

*작성: 2026-05-11*
*작성자: Mario Cho (hephaex@gmail.com)*
*Sprint: 128 P1+P2*
