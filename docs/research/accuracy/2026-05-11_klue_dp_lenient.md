# Sprint 125 P1: Tag Equivalence Map + Lenient Evaluator

> 핵심 발견: tag equivalence가 **morpheme +3.2pp, eojeol +1.6pp** 상승.
> Phase 1 진단(31% tag scheme)이 시사한 것보다 작은 효과 — 대부분 errors가
> tag scheme + 다른 issue들이 결합된 cascade임을 보여줌.

---

## 배경

Sprint 124 Phase 1 진단 결과 KLUE DP 65.8% morpheme accuracy의 ~31%가
tag scheme 차이(SP/SC, SS/SY/SSO/SSC, MMD/MMN/MMA→MM)로 분류됨.
Sprint 125 P1은 이 차이를 흡수하는 **lenient evaluator**를 구현하여
"진짜 분석 정확도"를 분리 측정.

---

## 구현

### TAG_EQUIVALENCE_GROUPS (`evaluate.rs`)

```rust
pub const TAG_EQUIVALENCE_GROUPS: &[&[&str]] = &[
    &["SP", "SC"],                      // 구두점/공백
    &["SS", "SY", "SSO", "SSC"],        // 괄호/따옴표/기호
    &["MM", "MMD", "MMN", "MMA"],       // 관형사 (KLUE 세분 vs mecab 통합)
];
```

**포함하지 않는 그룹** (의미적으로 다른 태그):
- NNG/NNP/NNB — 일반/고유/의존명사는 진짜 구분
- VV/VA — 동사/형용사
- EC/EF — 연결/종결어미

### Function pointer 패턴

코드 중복 없이 strict/lenient 양쪽 지원:

```rust
pub type PosMatchFn = fn(&str, &str) -> bool;
pub fn pos_eq_strict(a: &str, b: &str) -> bool { a == b }
pub fn pos_tags_equivalent(a: &str, b: &str) -> bool { ... }

// Strict (기본 wrapper)
pub fn evaluate_dataset_dual(t, d) -> DualMetricResult {
    evaluate_dataset_dual_with_pos_match(t, d, pos_eq_strict)
}

// Lenient (Sprint 125 신규)
pub fn evaluate_dataset_dual_lenient(t, d) -> DualMetricResult {
    evaluate_dataset_dual_with_pos_match(t, d, pos_tags_equivalent)
}

// 내부 (실제 구현)
pub fn evaluate_dataset_dual_with_pos_match(t, d, pos_eq) -> DualMetricResult { ... }
```

같은 패턴이 `evaluate_tokens_aligned` / `evaluate_dataset_sejong`에도 적용.
**기존 strict API 시그니처 100% 호환** — 모든 기존 caller는 변경 불필요.

### Lenient 적용 범위

morpheme + eojeol **양쪽 메트릭 모두**에 동일 `pos_eq` 함수 적용.
- `evaluate_tokens_aligned_with_pos_match`: 형태소 정렬 시 POS 비교
- `evaluate_dataset_sejong_with_pos_match`: 품사별 통계 + Sejong 변환 후 비교
- `evaluate_dataset_dual_with_pos_match`: 어절 슬라이스 비교

---

## 측정 결과 (KLUE DP val, 1,995 sentences)

### Strict vs Lenient

| 메트릭 | Strict | Lenient | Δ |
|--------|--------|---------|---|
| Morpheme (greedy aligned) | 65.8% | **69.0%** | **+3.2pp** |
| Eojeol (strict per-eojeol) | 19.2% (4,299) | **20.8% (4,667)** | **+1.6pp** |

### 해석

**Morpheme +3.2pp** = 약 1,633개 토큰이 새로 정답 처리.
- Phase 1 진단 분류에서 SP/SC, SS/SY/SSO/SSC, MMD/MMN/MMA→MM 등 ~615 unique
  confusion patterns × 평균 빈도 ≈ 1,600 morpheme matches.
- 진단 추정과 실측이 일치.

**Eojeol +1.6pp** = 368개 어절이 새로 정답 처리.
- 어절 metric은 strict — 한 어절 내 모든 형태소가 맞아야 함
- tag equivalence가 한 morpheme 만 고치면 그 morpheme이 마지막 차이였던
  어절만 새로 정답 처리됨
- 다른 issue(분할/사전)와 결합된 어절은 여전히 fail

### 진짜 분석 정확도 추정

| 추정 | morpheme |
|------|----------|
| 측정 strict | 65.8% |
| Tag scheme 흡수 (lenient) | 69.0% |
| + 사전 정책 차이 흡수 (가설) | ~75-80%? |
| + 복합명사 분할 정책 흡수 (가설) | ~85-90%? |
| **= 순수 분석 정확도 (가설)** | **~85-90%** |

위는 추정. Sprint 125 후속에서 사전 정책/분할 차이도 정량화 필요.

---

## 핵심 학습 포인트

### 1. Tag equivalence의 한계

진단 분류로 "31% errors are tag scheme"이라 했지만, 실제 정확도 lift는
morpheme +3.2pp / eojeol +1.6pp에 그침. **31%는 unique error patterns 비율**이며
**이로 인한 positional 정확도 영향은 그보다 작다**.

특히 eojeol-level은 한 어절 내 모든 morpheme이 맞아야 하므로,
tag equivalence가 단독 issue를 가진 어절만 구제. 복합 issue 어절은 여전히
fail.

**적용 원칙**: 진단 분류 비율과 메트릭 lift는 다른 차원. 실측 전에 추정
overlift하지 말 것.

### 2. Function pointer로 확장 가능한 평가 인프라

`PosMatchFn = fn(&str, &str) -> bool` 패턴으로:
- 기존 strict API 100% 호환 유지
- 새 lenient API를 wrapper로 추가
- 미래 다른 매칭 전략(예: 도메인별 equivalence) 추가 용이

**적용 원칙**: 평가 인프라는 매칭 전략이 자주 변하므로 함수 포인터/closure
로 주입 가능하게 설계.

### 3. 단계적 정확도 분석의 가치

단일 숫자 "65.8%" 대신:
- Strict morpheme: 65.8%
- Lenient morpheme (tag eq): 69.0%
- Lenient eojeol: 20.8%

각 숫자가 다른 정보를 줌. 65.8 → 69.0의 +3.2pp가 "tag scheme만의 영향",
이걸 빼면 남은 31%(100-69)가 "분할/사전/disambiguation 합산".

**적용 원칙**: 정확도 보고는 strict + lenient + dimensional metric set으로.

---

## 산출물

- `rust/crates/mecab-ko-core/src/evaluate.rs`:
  - `TAG_EQUIVALENCE_GROUPS` 상수
  - `pos_tags_equivalent()`, `pos_eq_strict()`, `PosMatchFn` 타입
  - `evaluate_tokens_aligned_with_pos_match()` 내부 함수
  - `evaluate_dataset_sejong_with_pos_match()` + `_lenient()` wrapper
  - `evaluate_dataset_dual_with_pos_match()` + `_lenient()` wrapper
  - 4개 unit test
- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`:
  - `test_klue_dp_dual_metric_lenient` (strict vs lenient 비교)
- 본 보고서

빌드/clippy clean, 모든 기존 테스트 pass.

---

## Sprint 126 후보

### P1: 사전 정책 차이 정량화
- NNG↔NNP, NNB↔NNG, XSA↔XSV 등 사전 분류 차이 측정
- KLUE 기준 vs mecab-ko-dic 기준 차이 추정
- 가능하면 별도 equivalence map 또는 정답 보정 옵션

### P2: 복합명사 분할 정책 차이 측정
- 팝스타 (NNP) vs 팝+스타 (NNG+NNG) 같은 케이스 통계
- 양쪽 모두 정답으로 인정하는 매칭 룰

### P3 (Sprint 125 carryover): noisy 데이터 추가
- KLUE DP는 편집 register만
- 사용자 우선순위 "높음"

### P4: CI 통합
- accuracy-gate.yml에 KLUE DP dual-metric job

---

*작성: 2026-05-11*
*작성자: Mario Cho (hephaex@gmail.com)*
*Sprint: 125 P1*
