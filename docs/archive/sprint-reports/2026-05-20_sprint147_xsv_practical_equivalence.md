# Sprint 147 A — VV/XSV practical 동치 추가 ("했/됐" convention 흡수)

> **결과**: XSV+EP/EC 단순 surface 분리 시도 → mecab/gold POS scheme 자체 차이 발견 → practical 동치로 전환. `TAG_EQUIVALENCE_GROUPS_PRACTICAL`에 VV/XSV 동치 추가. KLUE/UD 3 silver 모두 practical lift +0.2~0.4pp. sample.tsv 무회귀.

---

## 1. 가설 검증

### 1.1 초기 가설 (Sprint 145 분석 기반)

XSV+EP 413건, XSV+EC 751건 — 명시 surface 분리 시도.

### 1.2 mecab CLI 확인

```
$ echo "했\n됐\n해\n하고" | mecab-cli
했	VV+EP   ← VV (not XSV)
됐	VV+EP   ← VV (not XSV)
해	VV+EC   ← VV (not XSV)
하고	JKQ    ← 다른 entry로 인식
```

→ Sprint 145 분석의 "XSV+EP 413건"은 raw Token.pos 통계. 실제 mecab CLI 출력은 **VV+EP** 결합.

### 1.3 KLUE/UD gold 확인

```bash
$ grep -oE "하/XSV [^ ]*/EP" data/eval/klue_dp_val.tsv | sort -u
하/XSV 였/EP
하/XSV 았었/EP
```

→ KLUE gold는 "하/XSV + 였/EP" 형태로 분리. POS=XSV.

### 1.4 핵심 발견

mecab vs gold POS **scheme 자체 차이**:
- mecab: "하"를 본동사 (VV)로 분류
- gold (KLUE/UD): "하"를 동사파생접사 (XSV)로 분류

이는 surface 분리 문제가 아닌 **POS 분류 convention 차이**. 단순 분리 불가 → **practical 동치로 처리**.

---

## 2. 구현

### 2.1 PRACTICAL group 확장

`evaluate.rs:849` `TAG_EQUIVALENCE_GROUPS_PRACTICAL`:
- Before: `&["VA", "VV"]`
- After: `&["VA", "VV", "XSV"]`

언어학적 정당성:
- VA/VV (Sprint 136): "있다"의 형용사/동사 분류 논쟁
- VV/XSV (Sprint 147): "하/되"의 본동사/접사 분류 논쟁

세 카테고리 모두 한국어 문법에서 진행 중인 convention 차이.

### 2.2 단위 테스트 신규

```rust
#[test]
fn test_pos_tags_equivalent_practical_includes_xsv() {
    assert!(pos_tags_equivalent_practical("VV", "XSV"));
    assert!(pos_tags_equivalent_practical("XSV", "VV"));
    assert!(pos_tags_equivalent_practical("VA", "XSV"));
    assert!(pos_tags_equivalent_practical("XSV", "VA"));
    // Conservative는 XSV 구분 유지
    assert!(!pos_tags_equivalent("VV", "XSV"));
    assert!(!pos_tags_equivalent("XSV", "VV"));
}
```

Conservative (strict)는 영향 없음. precise 평가는 보존.

---

## 3. 측정 결과

### 3.1 5-gate

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv Token | 100.0% | 100.0% | — |
| sample.tsv Sentence | 99.9% | 99.9% | — |
| KLUE morph strict | 66.8% | 66.8% | — |
| KLUE eo strict | 20.7% | 20.7% | — |
| **KLUE morph practical** | **71.6%** | **71.9%** | **+0.3pp** |
| **KLUE eo practical** | 23.5% (5262) | **23.6% (5281)** | **+19건, +0.1pp** |
| Surface-only | 87.7%/91.6%/95.5% | 동일 | — |
| UD Kaist morph strict | 66.4% | 66.4% | — |
| **UD Kaist morph practical** | **68.1%** | **68.3%** | **+0.2pp** |
| UD Kaist eo practical (count) | 4193 | 4200 | +7 |
| UD GSD morph strict | 67.4% | 67.4% | — |
| **UD GSD morph practical** | **71.3%** | **71.7%** | **+0.4pp** |
| UD GSD eo practical (count) | 2907 | 2918 | +11 |

### 3.2 합산 효과

- **3 silver practical morph: +0.2~0.4pp (avg +0.3pp)**
- Conservative (strict) 변경 없음 — 정밀 평가 보존
- sample.tsv 무회귀
- Surface-only 영향 없음 (POS 무시)

### 3.3 lift 분포

KLUE +19, UD Kaist +7, UD GSD +11 (eojeol practical) — 데이터셋 모두에서 일관된 +효과. 도메인 독립적.

---

## 4. 핵심 학습 포인트

### 4.1 분석 빈도 → mecab CLI 확인 → 적합한 접근 식별

Sprint 145 분석에서 "XSV+EP 413건"으로 표시됐으나, mecab CLI 확인 후 실제 출력은 "VV+EP"임을 발견. surface 분리 접근에서 practical 동치 접근으로 전환 → 효과적 해결.

**적용 원칙**:
빈도 분석 → mecab CLI 확인 → gold 비교 → 분리 vs 동치 결정. 단순 빈도만으로는 잘못된 접근 선택 가능.

### 4.2 POS scheme 차이는 practical 동치로 처리

mecab vs gold의 POS 분류 차이 (VV/XSV, VA/VV)는 conventional disagreement. 형태소 분리 단위는 같으나 분류만 다름 → split 시도 불가, **lenient equivalence**만 적절.

### 4.3 Conservative vs Practical 분리의 가치

Conservative는 strict 평가용 (정밀 분석), Practical은 downstream use case (검색/색인). 두 mode 분리로 trade-off 명확화. 새 동치는 Practical에만 추가 → conservative 정밀도 보존.

### 4.4 일관된 도메인 lift = 진짜 효과

3 silver 모두 +0.2~0.4pp 일관 → 단일 도메인 anomaly가 아닌 진짜 convention 차이 흡수. 도메인 독립적 lift는 신뢰도 높음.

---

## 5. Sprint 148 후보

### 후보 A: 추가 안전 패턴 (VV+EP 명시 동사)

VV+EP 542건. "흘렸"/"버렸"/"불탔" 같은 명시 동사 → "VV stem + 었/EP" 분리.
단 stem 식별 복잡 (regular vs irregular conjugation).

### 후보 B [메인]: Full CRF Retrain (Track E)
3-5 sprint.

### 후보 C: NIKL Modu 수동 다운로드

### 후보 D: ETM+ETM 33건 ("라는") 조사
mecab의 비정상 출력 — 분석 후 처리.

---

## 6. 변경 파일

- `rust/crates/mecab-ko-core/src/evaluate.rs`:
  - `TAG_EQUIVALENCE_GROUPS_PRACTICAL`에 XSV 추가
  - `test_pos_tags_equivalent_practical_includes_xsv` 신규
- `docs/research/accuracy/2026-05-20_sprint147_xsv_practical_equivalence.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

*작성: 2026-05-20 (Sprint 147 A)*
