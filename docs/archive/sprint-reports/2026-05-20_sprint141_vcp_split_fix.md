# Sprint 141 A — VCP+ETM/EC 복합 토큰 분리 (SejongConverter splitter)

> **결론**: 초기 가설(XSN/VCP practical 동치)이 무효 — 두 데이터셋이 동일 분류 사용. 실제 문제는 mecab의 VCP+ETM 결합 토큰 출력. SejongConverter splitter에 분리 규칙 추가 → UD morph +0.1pp lift, KLUE 무변경, sample.tsv 무회귀.

---

## 1. 가설 검증 (가설 → 폐기)

### 초기 가설 (Sprint 140 분석 기반)

UD Kaist에서 (3777, 2240) XSN(적) → VCP(인) 패턴 92건. → `TAG_EQUIVALENCE_GROUPS_PRACTICAL`에 XSN/VCP 동치 추가하면 lift?

### 데이터 확인 후 폐기

mecab 출력:
```
특징적인 → 특징/NNG + 적/XSN + 인/VCP+ETM    (3 tokens)
역사적이다 → 역사/NNG + 적/XSN + 이/VCP + 다/EF  (4 tokens)
```

UD gold annotation (test):
```
특징적인 → 특징/NNG + 적/XSN + 이/VCP + ㄴ/ETM  (4 tokens)
```

KLUE gold annotation (val):
```
공정적인 → 공정/NNG + 적/XSN + 이/VCP + ㄴ/ETM  (4 tokens, 동일 패턴)
```

**핵심 발견**:
- mecab과 UD/KLUE는 모두 같은 분해 방식 (XSN + VCP + ETM)
- 차이는 mecab이 `이/VCP + ㄴ/ETM`을 한 결합 토큰 `인/VCP+ETM`으로 출력한다는 점
- XSN/VCP는 진짜 의미적 차이 (동치 아님). 동치 추가 시 다른 의미 손실.

→ 가설 폐기. 실제 해결책은 **SejongConverter splitter에 VCP+ETM 분리 규칙 추가**.

---

## 2. 구현 — splitter.rs 패턴 추가

### 추가된 패턴

```rust
// VCP+ETM 복합 분리
if pos == "VCP+ETM" {
    if surface == "인" → 이/VCP + ㄴ/ETM
    if surface == "일" → 이/VCP + ㄹ/ETM
    if surface == "라는" → 이/VCP + 라는/ETM
}

// VCP+EC 복합 분리
if pos == "VCP+EC" {
    if surface ∈ {라, 며, 라서, 라고, 라며, 라면, 라야, 라든지} →
        이/VCP + surface/EC
}
```

### 단위 테스트 (4개 신규)

- `test_split_morpheme_vcp_etm_in`: "인" → "이/VCP + ㄴ/ETM" 검증
- `test_split_morpheme_vcp_etm_il`: "일" → "이/VCP + ㄹ/ETM" 검증
- `test_split_morpheme_vcp_ec_la`: "라" → "이/VCP + 라/EC" 검증
- `test_split_morpheme_vcp_etm_unrelated_surface_no_split`: overcorrect 방지 (명시 목록 외 surface는 분리 안 함)

---

## 3. 측정 결과

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv Token | 100.0% | 100.0% | — |
| sample.tsv Sentence | 99.9% | 99.9% | — |
| KLUE morph strict | 66.8% | 66.8% | — |
| KLUE morph practical | 71.6% | 71.6% | — |
| KLUE eo strict | 20.7% | 20.7% | — |
| KLUE eo practical | 23.5% | 23.5% | — |
| **UD Kaist morph strict** | **66.3%** | **66.4%** | **+0.1pp** |
| **UD Kaist morph practical** | **68.0%** | **68.1%** | **+0.1pp** |
| UD Kaist eo strict | 20.7% (3989) | 20.7% (3987) | -2 (-0.01pp) |
| UD Kaist eo practical | 21.8% (4194) | 21.8% (4193) | -1 |

### 분석

**Positive**:
- UD morph +0.1pp (형태론적 정확성 향상)
- VCP+ETM 패턴이 KLUE에는 적게 등장 (27건 vs UD 92건) → KLUE 측정값 변화 미미
- 형태론적으로 표준 분해 방식. mecab 결합 출력을 standardize.

**Neutral/Negative**:
- UD eojeol -2건 회귀 (-0.01pp): 일부 어절에서 mecab의 결합 토큰이 gold의 다른 분해 방식과 우연히 일치했던 케이스
- KLUE 무변화: morpheme token 단위로 27건 분리해도 22,404 어절 중 morph token 단위 비율 차이는 무시할 정도
- VCP+EC 패턴 추가 효과 없음 (mecab이 VCP+EC를 별도 출력하는 빈도 매우 낮음)

### 결정

**유지** — 형태론적 정확성 향상이 미미한 eojeol 회귀(-0.01pp)를 정당화. downstream 일관성 향상.

---

## 4. 핵심 학습 포인트

### 4.1 가설은 데이터로 검증해야 함

Sprint 140 분석의 (3777, 2240) pair는 SPLIT_DIFFERENT로 보였지만, 실제 mecab/gold 분해 방식을 직접 확인하니 분류 차이가 아니라 **mecab의 결합 출력 표기 차이**임이 밝혀짐.

**적용 원칙**:
"같은 pair가 자주 등장 → 동치 처리" 같은 단순 추론 전에 실제 mecab 출력 vs gold 출력을 비교. POS 명칭이 같아도 token 단위 boundary가 다를 수 있음.

### 4.2 SejongConverter splitter는 형태론 정규화의 진짜 진입점

matrix.def cost 조정(Sprint 138 실패)은 너무 거친 도구. splitter 패턴 추가는:
- 형태론적으로 정확한 변환
- KLUE/UD 양쪽에 일관 적용
- sample.tsv 회귀 위험 거의 없음 (mecab 결합 토큰만 분리, 다른 출력 변화 없음)

**적용 원칙**:
mecab 결합 토큰 (VCP+ETM, VV+EF, EP+EF 등)의 표준 분해 패턴은 splitter에 추가. 정확성 향상이 점진적이지만 안전.

### 4.3 도메인별 빈도 차이는 패턴 분포의 의미

VCP+ETM 패턴: UD 92건, KLUE 27건. → 학술 텍스트("X적인")에서 더 많이 등장.
이는 splitter 변경의 KLUE 영향이 작다는 의미. **도메인 특화 변경이 도메인 독립 변경보다 위험 낮음.**

---

## 5. Sprint 142 후보

### A: 다른 mecab 결합 토큰 패턴 조사

VCP+ETM/EC 외 다른 결합 패턴:
- VV+EC+EP+EF (이미 처리됨, "야겠다" 패턴)
- NNG+VCP+EC (mecab "이고" → "NNG+VCP+EC" 한 토큰 출력)
- 그 외 빈도 분석 필요

### B: dict-builder CSV 버그 수정 (Track D 선행)

Sprint 138 미해결. Full CRF retrain 진입 전 필요.

### C: NIKL Modu 또는 OpenKorPOS 추가

Sprint 123 보고서의 다른 silver 데이터셋.

---

## 6. 변경 파일

- `rust/crates/mecab-ko-core/src/sejong/splitter.rs`: VCP+ETM / VCP+EC 패턴 + 4 단위 테스트
- `docs/research/accuracy/2026-05-20_sprint141_vcp_split_fix.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

*작성: 2026-05-20 (Sprint 141 A)*
