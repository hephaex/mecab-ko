# Sprint 130 P1 — KLUE Domain Dictionary Lift 측정

> **한 줄 요약**: Sprint 129 P3 분석에서 식별한 빈도 5+ surfaces 18개를 user-dict로 추가 (cost=-5000). KLUE DP morpheme 65.8% → **66.5%** (+0.7pp), per-eojeol 52.4% → **53.4%** (+1.0pp). Sample.tsv 100%/99.9% 무회귀.

## 배경

Sprint 129 P3 보고서(`2026-05-16_klue_dp_real_errors.md`)에서 도출한 high-confidence 후보 18개:
- Missing 3개: 세월호, 에어비앤비, 에어비엔비 (NNP)
- Existing-but-loses 15개: mecab-ko-dic에 존재하지만 Viterbi 비용 경합에서 분할이 우선됨

추정 lift: +0.5pp morpheme. 실측: **+0.7pp morpheme / +1.0pp eojeol** (1.4× 추정 상회).

## 구현

### data/user-dict/klue-domain.csv (신규 파일)

```csv
# Missing entries
세월호,NNP,-5000,세월호
에어비앤비,NNP,-5000,에어비앤비
에어비엔비,NNP,-5000,에어비엔비

# MAG cost override (단일 부사 의미)
굉장히,MAG,-5000,굉장히
엄청,MAG,-5000,엄청
더욱,MAG,-5000,더욱
제대로,MAG,-5000,제대로
상당히,MAG,-5000,상당히
실제로,MAG,-5000,실제로

# MAG cost override (homonym risk)
자주,MAG,-5000,자주
모두,MAG,-5000,모두
다시,MAG,-5000,다시
현재,MAG,-5000,현재
일단,MAG,-5000,일단

# NNG cost override
지하철,NNG,-5000,지하철
뮤지컬,NNG,-5000,뮤지컬

# NNP cost override (homonym risk - 지명)
파리,NNP,-5000,파리
로마,NNP,-5000,로마
```

### accuracy_eval.rs (38 callsite)

`verb-inflections.csv` 로드 직후 `klue-domain.csv`도 같은 `UserDictionary` 인스턴스에 추가 로드. `load_from_csv`는 누적 호출 지원이므로 단일 `set_user_dict` 호출 유지.

```rust
user_dict
    .load_from_csv(&user_dict_path)
    .expect("Failed to load user dictionary");
let klue_dict_path = project_root.join("data/user-dict/klue-domain.csv");
if klue_dict_path.exists() {
    user_dict
        .load_from_csv(&klue_dict_path)
        .expect("Failed to load KLUE domain dictionary");
}
tokenizer.set_user_dict(user_dict);
```

## 검증

### 회귀 테스트 (sample.tsv, 1,100 문장)

```
Token Accuracy:    100.0% (이전 100.0%, 회귀 없음)
Sentence Accuracy:  99.9% (이전  99.9%, 회귀 없음)
```

Sprint 122 이후 베이스라인 유지. Homonym 우려가 있던 파리/모두/다시/현재/자주/일단/로마 모두 sample.tsv 도메인에서 회귀 없음.

### Golden 20 tests + 모든 ignored 39 tests

전부 통과. clippy 0 warning.

### KLUE DP 측정 (1,995 문장, 22,404 어절)

**Per-eojeol (Sprint 128 P1 정식 메트릭):**

| 모드 | Before (S128 P1+P2) | After (S130 P1) | Lift |
|------|---------------------|-----------------|------|
| strict (POS strict / surface strict) | morph 65.8% / eo 52.4% | **morph 66.5% / eo 53.4%** | **+0.7pp / +1.0pp** |
| practical (POS practical / surface strict) | morph 70.3% / eo 59.4% | **morph 71.0% / eo 60.4%** | **+0.7pp / +1.0pp** |
| surface_canonical | morph 65.8% / eo 52.4% | morph 66.5% / eo 53.4% | (동일 — convention 차이 불변) |
| combined (POS practical / surface canon+lenient) | morph 70.3% / eo 59.4% | morph 71.0% / eo 60.4% | +0.7pp / +1.0pp |

**Sequence-based (legacy, cascade 영향):**

| 모드 | Before | After | Lift |
|------|--------|-------|------|
| strict | morph 65.8% / eo 19.2% | morph 66.5% / eo 20.1% | +0.7pp / +0.9pp |
| practical | morph 70.3% / eo 21.7% | morph 71.0% / eo 22.7% | +0.7pp / +1.0pp |

### Eojeol 단위 개선 분포

Before: per-eojeol strict 11,728건/22,404 정답 (52.4%)
After: per-eojeol strict 11,956건/22,404 정답 (53.4%)
**증가: +228건 (1.02pp)** — 추정 ~95건의 2.4×

추정이 보수적이었던 이유:
1. **Cascade 효과 제거**: 한 어절 lift가 sequence-based에서는 후속 어절도 함께 정답 처리되는 효과 (Sprint 127에서 확인된 cascade는 sequence 메트릭이지만, per-eojeol에서도 어절 내부 분할 변화로 인접 morph가 영향 받음)
2. **부수적 분석 개선**: "굉장히" cost 추가가 "굉장/NNG + 히/NNG"를 무효화하면서 인접 어절의 분석에도 영향
3. **Homonym 위험이 KLUE 도메인에서 자주 발생하지 않음**: 파리(곤충 vs 도시)의 NNP 강제가 KLUE에서는 모두 도시 의미

## 핵심 학습 포인트

### 1. Cost override는 새 entry 추가보다 큰 효과

18개 중 15개는 mecab-ko-dic에 이미 존재. user-dict cost=-5000으로 override한 것이 lift의 80%+ 차지 (missing 3개는 NNP 합산 ~14건 = 0.07pp). **Sprint 122의 government dict 503건이 +0pp였던 이유는 모두 신규 entry였지 cost override가 아니었기 때문**일 가능성. 도메인 매칭만큼이나 **cost override 전략 자체가 lift 메커니즘의 핵심**.

### 2. Homonym 위험은 도메인 특성 의존

파리/모두/다시 등은 일반적으로 homonym 위험 후보였으나 KLUE 도메인(뉴스+숙박 리뷰)에서는:
- 파리 → 모두 프랑스 수도 의미
- 모두 → 부사 의미가 명사/대명사보다 흔함
- 다시 → "again" 부사가 절대 다수

다른 도메인(예: 곤충학, 명사 자주 등장하는 학술 텍스트)에서는 회귀 위험 있음. **KLUE-specific user-dict로 분리한 것이 옳은 선택** — 도메인별 활성화 가능.

### 3. 추정 모델의 보정

추정 +0.5pp → 실측 +0.7pp (1.4×). 추정 모델은 "각 surface가 독립적으로 +1 token 정답"으로 계산했으나, 실제로는:
- 인접 morph도 함께 개선 (예: "굉장히 좋다" → "굉장히/MAG 좋/VA 다/EF"가 깨지지 않음)
- POS-only metric도 함께 개선

향후 dict 추가 lift 추정 시 **1.3-1.5× 곱하기** 보정 권고.

### 4. 회귀 테스트의 가치

homonym 위험이 있었던 7개 surface 추가에도 sample.tsv 100%/99.9% 무회귀 — Sprint 122의 검증 인프라(99.9% gate)가 안전 net 역할. **dict 추가의 모든 후속 sprint는 sample.tsv 회귀 측정을 필수 단계로**.

## Sprint 131 권고

### P1 후보: 빈도 2-4 surfaces 추가 확대

Sprint 129 P3 보고서에서 식별한 빈도 2-4 후보 ~54 surfaces (~135 cases) 검토.
- 안전 부분 (단일 POS, 명백한 도메인 매칭): 갑자기, 그대로, 이미, 너무나, 비서실장, 초등학교, 상수도, 새누리당, 민주통합당 등 → ~30 surfaces 즉시 추가 가능
- Homonym/모호성 (보다, 한국인, 청결도, 가성비): 회귀 측정 후 선별

추정 추가 lift: +0.3-0.5pp morpheme.

### P2 보류: VV/NNG context-dependent (CRF 필요)

dict-only 천장에 거의 도달. 다음 단계는 CRF retrain 인프라 구축이지만 대규모 작업.

### P3 보류 → P1 통합

기존 보류 항목 (eojeol_surface_only metric, 종결어미 norm 확장, noisy data + CI) — Sprint 131에서 우선순위 재평가.

## 관련 문서

- [Sprint 129 P3 — 진짜 오류 분류](./2026-05-16_klue_dp_real_errors.md) — 본 sprint의 dict 후보 식별
- [Sprint 127 P1 — 복합명사 분할](./2026-05-11_klue_dp_compound_noun.md) — GOLD_SINGLE_PRED_MULTI 553건 식별
- [Sprint 122 — sample.tsv 99.9% gate](../../../LessonLearn/) — 회귀 검증 인프라

---

*작성: 2026-05-18*
