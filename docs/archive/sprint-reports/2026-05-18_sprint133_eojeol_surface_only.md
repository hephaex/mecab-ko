# Sprint 133 P2 — Eojeol Surface-only Metric

> **한 줄 요약**: 검색/인덱싱 use case 전용 메트릭 신규. POS와 split 무시, surface concat 일치만으로 정답 판정. KLUE DP strict **87.7%** / canonical **91.6%** / canonical_lenient **94.4%**. Sprint 127 P1의 추정 ceiling 87.7%와 정확히 일치.

## 배경

Sprint 127 P1 분석에서 추정한 "slice-lenient ceiling 87.7%"를 별도 메트릭으로 노출. 다음 use case 대상:

1. **검색 색인 빌드**: 어절 단위 surface 보존이 중요, POS는 무관
2. **부분 일치 검색 baseline**: 사용자 입력과 색인된 어절의 surface 매칭
3. **천장 추정**: 형태소 분석 정확도와 다른 차원의 측정값

기존 메트릭(`evaluate_dataset_sejong`, `evaluate_dataset_dual_per_eojeol`)은 형태소 분석 품질을 측정하므로 검색 use case에서는 과도하게 엄격함.

## 구현

### evaluate.rs (+~110 lines)

```rust
pub struct EojeolSurfaceResult {
    pub correct: usize,
    pub total: usize,
    pub accuracy: f64,
}

pub fn evaluate_dataset_eojeol_surface_only_with_match(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
    surface_eq: SurfaceMatchFn,
) -> EojeolSurfaceResult { ... }

pub fn evaluate_dataset_eojeol_surface_only(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
) -> EojeolSurfaceResult {
    evaluate_dataset_eojeol_surface_only_with_match(tokenizer, dataset, surface_eq_strict)
}
```

알고리즘 (per-eojeol, no cascade):
1. `text.split_whitespace()` → 어절 리스트
2. 각 어절: gold morphs surface concat ↔ 어절별 토크나이즈 pred surface concat
3. `surface_eq`로 비교 (strict / canonical / canonical_lenient 주입)

POS와 inner split boundary 모두 무시 — Sprint 128 `evaluate_dataset_dual_per_eojeol`의 surface concat 비교 부분만 추출.

### 단위 테스트 + 통합 테스트

- 2 unit tests: `EojeolSurfaceResult::format_report` (빈 / 채워진)
- 1 integration test: `test_klue_dp_eojeol_surface_only` (KLUE DP 측정 + floor 검증)

## 측정 결과 (KLUE DP val, 1,995 sentences, 22,404 어절)

| 모드 | Accuracy | Correct | Δ vs strict |
|------|----------|---------|------------|
| **strict** | **87.7%** | 19,655 | — |
| **canonical** | **91.6%** | 20,528 | +3.9pp |
| **canonical_lenient** | **94.4%** | 21,147 | +6.7pp |

### Sprint 127 P1 ceiling 일치 확인

Sprint 127 P1 보고서에서 추정한 "slice-lenient ceiling 87.7%"와 정확히 일치 — 같은 데이터셋, 같은 정의(surface concat lenient), 같은 알고리즘. 본 sprint는 그 측정을 **정식 API + 회귀 게이트**로 노출.

### Normalization absorption

- NFC compose (canonical): +3.9pp = ~870 어절. KLUE의 jamo decomposition convention vs mecab의 syllable convention 차이를 흡수.
- Endings (canonical_lenient): canonical 대비 +2.8pp = ~619 어절. "하았→하였", "하어→하여" 변환을 흡수.

Sprint 128 P2에서 측정한 SURFACE_MISMATCH absorption rate(NFC 31.8% + endings 22.6% = 54.4%)와 일관된 수치.

### 다른 메트릭과 비교 (KLUE DP)

| 메트릭 | Strict | Practical | Lift via surface_only |
|--------|--------|-----------|----------------------|
| Per-eojeol (POS + surface) | 53.9% | 61.0% | — |
| **Eojeol surface-only** | **87.7%** | **94.4%** | +33.8pp / +33.4pp |

검색 use case는 형태소 분석 use case보다 ~34pp 높은 정확도를 달성 가능 — POS/split 무시의 trade-off가 명확.

## 핵심 학습 포인트

### 1. Use case별 메트릭 분리의 가치

기존 `evaluate_dataset_sejong` / `evaluate_dataset_dual_per_eojeol`은 형태소 분석 정확도 (POS + split + surface). 검색 시스템 평가에 적용하면 87.7% 가능한 시스템을 53.9%로 underestimate. **다운스트림 use case와 메트릭이 일치해야 의사결정에 유용**.

### 2. Function pointer 패턴 일관성

Sprint 125의 `PosMatchFn`, Sprint 128의 `SurfaceMatchFn`을 그대로 활용. 새 메트릭이 동일 패턴을 따르므로 확장성 보장. `surface_eq` 함수만 바꾸면 strict/canonical/lenient 자동 지원.

### 3. Sprint 127 분석을 메트릭화

Sprint 127 P1은 "분석"이었음 — 한 번 측정하고 보고서로 남김. Sprint 133은 그 측정을 **CI 게이트로 자동화** — floor 위반 시 PR 차단. 분석 결과가 영속적 안전망이 되는 transformation.

### 4. 의미 손실의 명시적 문서화

API doc에 "의미 손실" 섹션 명시:
- 형태소 분석 품질은 측정하지 않음
- 빈도/품사/동의어 처리 다운스트림 사용 시 부적합
- 검색 색인 baseline 또는 ceiling 추정에만 사용

오해 방지를 위한 사전 가드 — Sprint 124의 21.7% sequence eojeol "오해 시점"과 같은 misinterpretation 위험을 차단.

## Sprint 134 권고

### CI 통합 (선택)

`test_klue_dp_eojeol_surface_only`를 `.github/workflows/accuracy-gate.yml`에 추가 검토. 회귀 시 PR 차단. 다만 본 메트릭은 형태소 분석 변경에 둔감하므로 기존 KLUE DP 3-mode 게이트로 충분할 수 있음.

### 후속 트랙

Sprint 132에서 결정된 "사전 트랙 종료"는 유지. Sprint 134 후보:
- **P1**: Noisy data 추가 (Sprint 131 deferred)
- **P3**: 종결어미 normalization 확장
- **P4**: CRF retrain 인프라 조사

## 관련 문서

- [Sprint 127 P1 — 복합명사 분할](./2026-05-11_klue_dp_compound_noun.md) — 87.7% ceiling 원천
- [Sprint 128 P2 — Surface lenient](./2026-05-11_klue_dp_surface_lenient.md) — SurfaceMatchFn 패턴
- [Sprint 132 P1 — Dict 확장](./2026-05-18_sprint132_dict_expansion.md) — dict 트랙 종료 결정

---

*작성: 2026-05-18*
