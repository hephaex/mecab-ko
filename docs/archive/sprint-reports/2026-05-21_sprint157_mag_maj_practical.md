# Sprint 157 G — MAG/MAJ practical 동치 추가

> **결과**: Sprint 155 진단의 MAG→MAJ 45건 (다만, 및, 역시 등)을 `TAG_EQUIVALENCE_GROUPS_PRACTICAL`에 추가. **3 silver 모두 morph practical +0.2pp 일관 lift** (KLUE +0.2pp, UD Kaist +0.2pp, UD GSD +0.2pp). sample.tsv 무회귀, conservative 무영향. Sprint 147 패턴 재현.

---

## 1. 배경

Sprint 155 B (진단)에서 식별된 POS mismatch 중:
- MAG → MAJ 45건 (다만 20, 및 13, 역시 3, 오히려 2, 아무튼 2, 이른바 2, 암튼 1, 보통 1, 또는 1)
- mecab: MAG (일반부사), gold: MAJ (접속부사) 분류 차이

언어학적 배경:
- MAG: 일반 부사 (예: very, slowly)
- MAJ: 접속 부사 (예: and, but, however, 다만)

mecab은 모두 MAG로 분류, KLUE는 접속 부사를 MAJ로 분리. **convention 차이**.

---

## 2. 구현

### 2.1 PRACTICAL 그룹 확장

`evaluate.rs:853`:

```rust
pub const TAG_EQUIVALENCE_GROUPS_PRACTICAL: &[&[&str]] = &[
    &["SP", "SC"],
    &["SS", "SY", "SSO", "SSC"],
    &["MM", "MMD", "MMN", "MMA"],
    &["SL", "NNP"],
    &["NNB", "NNG"],
    &["VA", "VV", "XSV"],
    &["MAG", "MAJ"],   // Sprint 157 G 신규
];
```

### 2.2 단위 테스트

`test_pos_tags_equivalent_practical_includes_mag_maj`:
- practical: MAG ↔ MAJ ✓
- conservative: MAG/MAJ 구분 유지 ✓

---

## 3. 측정 결과 — 3 silver 일관 lift

### 5-gate

| Metric | Sprint 156 | Sprint 157 G | Δ |
|--------|-----------|-------------|---|
| sample.tsv | 100.0%/99.9% | 100.0%/99.9% | 무회귀 ✓ |
| KLUE morph strict | 66.9% | 66.9% | — (정밀 보존) |
| **KLUE morph practical** | **71.9%** | **72.1%** | **+0.2pp** |
| **KLUE eojeol practical** | **5283** | **5327** | **+44** |
| UD Kaist morph strict | 66.4% | 66.4% | — |
| **UD Kaist morph practical** | **68.4%** | **68.6%** | **+0.2pp** |
| **UD Kaist eojeol practical** | **4197** | **4262** | **+65** |
| UD GSD morph strict | 67.3% | 67.3% | — |
| **UD GSD morph practical** | **71.6%** | **71.8%** | **+0.2pp** |
| **UD GSD eojeol practical** | **2926** | **2941** | **+15** |

### 일관된 도메인 lift

**3 silver 모두 +0.2pp morph practical**:
- 단일 anomaly 아님 (모든 도메인에 적용)
- 진짜 convention 차이 흡수
- Sprint 147 (VV/XSV 동치) 패턴 재현

### Eojeol 추가 매칭

- KLUE: +44 (다만, 및 등 접속부사가 포함된 eojeol)
- UD Kaist: +65 (가장 큰 효과 — UD Kaist 부사 사용 빈도 높음)
- UD GSD: +15

총 **+124 eojeols** 추가 매칭.

### Strict 정밀 보존

Conservative 그룹 변경 없음:
- KLUE morph strict 66.9% 유지
- 정밀 형태소 분석 평가 영향 없음
- precision 분석 보존 + downstream lift 분리

---

## 4. 누적 PRACTICAL 동치 진척

| Sprint | 추가 | 의미 |
|--------|------|------|
| 126 P1 | NNB↔NNG (counter words) | 158건 흡수 |
| 136 P3 | VA↔VV (있다 convention) | 41건 흡수 |
| 147 A | VV↔XSV (했/됐 convention) | 3 silver +0.2~0.4pp |
| **157 G** | **MAG↔MAJ (접속부사)** | **3 silver +0.2pp** |

언어학적으로 모두 한국어 문법의 진행 중 convention 차이:
- NNB/NNG: 의존명사 vs 일반명사 (counter words 분류 논쟁)
- VA/VV: 형용사 vs 동사 ("있다" 분류 논쟁)
- VV/XSV: 본동사 vs 동사파생접사 ("하/되" 분류 논쟁)
- MAG/MAJ: 일반부사 vs 접속부사 (담화 표지 분류 논쟁)

---

## 5. 핵심 학습 포인트

### 5.1 진단 결과의 다단계 활용

Sprint 155 B에서 진단한 POS mismatch 결과:
- A (dict 확장) → 실패 (Sprint 155)
- G (메트릭 동치) → 성공 (Sprint 157)

같은 진단 데이터로 두 가지 접근:
- dict 확장: viterbi cascade 위험
- 메트릭 동치: 안전 + 측정 가능 lift

**적용 원칙**: 진단 데이터는 여러 접근에 재활용. 안전한 접근 우선.

### 5.2 3 silver 일관 lift = 진짜 효과 (Sprint 147 패턴)

단일 도메인 anomaly가 아닌 3 silver 모두 +0.2pp는 신뢰도 높음:
- 도메인 독립적 효과
- 진짜 convention 차이 흡수
- 회귀 위험 낮음

### 5.3 Conservative vs Practical 분리의 가치

- Conservative: 정밀 평가 (strict 형태소 비교)
- Practical: downstream 사용 (검색, 색인)

새 동치는 Practical에만 추가 → trade-off 명확화.

### 5.4 안전한 lift의 진정한 가치

Sprint 155 회귀 후 안전 영역 작업:
- Sprint 156: surface normalization +0.1pp
- **Sprint 157: practical 동치 +0.2pp**

작아 보이지만 무회귀 + 누적적 효과. ROI 매우 높음.

---

## 6. 변경 파일

- `rust/crates/mecab-ko-core/src/evaluate.rs`:
  - `TAG_EQUIVALENCE_GROUPS_PRACTICAL`에 MAG/MAJ 추가
  - 단위 테스트 1개 신규
- `docs/research/accuracy/2026-05-21_sprint157_mag_maj_practical.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

## 7. Sprint 158 후보

PRACTICAL 동치 그룹의 추가 후보 (Sprint 155 진단 활용):
- MMD↔MM 76건 — 이미 CONSERVATIVE 동치 (`MM/MMD/MMN/MMA`) — 무영향
- MMN↔MM 39건 — 이미 CONSERVATIVE 동치 — 무영향
- VV↔NNG 47건 — 의미 분류 차이 (위험, 흡수 시 false positive 위험)
- NNG↔NNB 36건 — 이미 NNB/NNG 동치
- VA↔VV 46건 — 이미 동치

남은 안전 후보:
- 추가 surface normalization (작은 lift)
- 추가 진단 (다른 POS 패턴)

영역 소진 시:
- F (NIKL Modu, confirm)
- E (Full CRF Retrain, confirm)

---

*작성: 2026-05-21 (Sprint 157 G)*
