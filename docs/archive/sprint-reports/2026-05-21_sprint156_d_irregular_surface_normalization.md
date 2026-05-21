# Sprint 156 C — Surface normalization 확장 (ㄷ 불규칙 + 아우르다)

> **결과**: Sprint 155 회귀 후 안전 영역인 surface normalization으로 전환. ㄷ 불규칙 9 패턴 + 르 불규칙 1 추가. KLUE surface canonical_lenient 95.5% → 95.6% (+0.1pp, +30 eojeols). 다른 메트릭 무영향. sample.tsv 무회귀.

---

## 1. 안전 영역 선택 배경

Sprint 155 dict 확장 → rollback. viterbi 영향 변경 위험 확인.
잔여 안전 영역:
- ✅ 평가 메트릭 동치 그룹 (Sprint 147 패턴)
- ✅ Surface normalization (Sprint 134 패턴)

전문가 권고: **C** (Surface normalization 확장) — Sprint 134가 +1.0pp 큰 효과 냈음.

---

## 2. 진단 — surface mismatch 패턴

`test_klue_dp_surface_normalization_analysis` 출력 상위 후보:

| 빈도 | gold | pred | 패턴 |
|------|------|------|------|
| 20 | 있어서 | 있어디에서 | mecab over-split (복잡) |
| 18 | 따르아 | 따라 | 르 불규칙 (이미 처리됨) |
| 12 | 것이 | 게 | 축약 (위험) |
| 12 | 앞서 | 앞서어 | mecab over-split (복잡) |
| 9 | 가깝고 | 가깝시고 | mecab over-split |
| **6** | **들었습니다.** | **듣었습니다.** | **ㄷ 불규칙 ✓** |
| 6 | 살는 | 사는 | ㄹ 탈락 (위험) |
| 5 | 갑니다. | 가이습니다. | VCP 분해 |
| 5 | 모으고 | 모으게하고 | mecab over-split |
| 5 | 아니냐는 | 아니이냐는 | mecab 분해 |
| **4** | **아울러** | **아우르어** | **르 불규칙 ✓** |

### 안전한 후보 선택

- **ㄷ 불규칙 (들/물/걸/깨달)**: 명시 stem 목록만 처리, false positive 위험 낮음
- **아우르다 추가**: 기존 R_IRREGULAR_PATTERNS 패턴 1줄 추가

위험한 후보 제외:
- 있어디에서 → mecab tokenizer 자체 버그 영향, normalization 영역 아님
- 살는/사는 → ㄹ 탈락 일반 패턴 (false positive 위험)
- 게 → 것이 축약 → 다른 의미

---

## 3. 구현

### 3.1 D_IRREGULAR_PATTERNS 신규 추가

`evaluate.rs:1039`:

```rust
const D_IRREGULAR_PATTERNS: &[(&str, &str)] = &[
    // 듣다 (가장 빈번)
    ("듣었", "들었"),  // 6건 KLUE
    ("듣어", "들어"),
    ("듣은", "들은"),
    // 묻다 (질문)
    ("묻었", "물었"),
    ("묻어", "물어"),
    // 걷다
    ("걷었", "걸었"),
    ("걷어", "걸어"),
    // 깨닫다
    ("깨닫았", "깨달았"),
    ("깨닫아", "깨달아"),
];
```

### 3.2 R_IRREGULAR_PATTERNS 확장

```rust
const R_IRREGULAR_PATTERNS: &[(&str, &str)] = &[
    // 기존 9개 (Sprint 136 P3a)
    ...
    // Sprint 156 추가
    ("아우르어", "아울러"),  // 4건 KLUE
];
```

### 3.3 normalize_endings Step 4 추가

`evaluate.rs:1006`:

```rust
// Step 4: ㄷ불규칙 활용 (Sprint 156)
for (from, to) in D_IRREGULAR_PATTERNS {
    if out.contains(from) {
        out = out.replace(from, to);
    }
}
```

### 3.4 단위 테스트

- `test_surface_eq_canonical_lenient_r_irregular_aureo`: 아우르어 → 아울러
- `test_surface_eq_canonical_lenient_d_irregular`: 6개 ㄷ 불규칙 case
- `test_surface_eq_canonical_lenient_d_irregular_does_not_overcorrect`: false positive 가드

---

## 4. 측정 결과

### 5-gate

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv | 100.0%/99.9% | 100.0%/99.9% | 무회귀 ✓ |
| KLUE morph strict | 66.9% | 66.9% | — |
| KLUE morph practical | 71.9% | 71.9% | — |
| KLUE eojeol practical | 5283 | 5283 | — |
| KLUE surface strict | 87.8% | 87.8% | — |
| KLUE surface canonical | 91.6% | 91.6% | — |
| **KLUE surface canonical_lenient** | **95.5%** | **95.6%** | **+0.1pp** (+30) |
| UD Kaist | 영향 없음 | — | — |
| UD GSD | 영향 없음 | — | — |

### 의미

- Surface canonical_lenient 21389 → 21419 = **+30 eojeols 매칭 추가**
- ㄷ 불규칙 (들었/물었/걸었/깨달았) + 아우르어 변환 효과
- morph metric은 무영향 (정상 — normalize_endings는 surface_only에만 사용)

---

## 5. 핵심 학습 포인트

### 5.1 안전 영역의 진정한 가치

Sprint 155 회귀 후 surface normalization으로 전환 → 안전 + 측정 가능한 lift.
viterbi/CRF 변경과 달리 normalize_endings는 평가 함수에만 사용 → cascade 없음.

### 5.2 진단 → 안전 후보 식별 → 구현 (Sprint 148 D 패턴)

빈도 분석은 안전한 변환 패턴 식별에도 유효:
- 가장 큰 mismatch가 위험하지 않으면 (ㄷ 불규칙) 안전
- 가장 큰 mismatch가 위험하면 (있어디에서) 보류

### 5.3 명시 stem 목록 vs 자동 음성 분해

ㄷ/ㄹ/르 불규칙 모두 명시 stem 목록만 처리:
- 자동 음성 분해 (예: ㄷ → ㄹ 변환 일반화) = false positive 위험
- 명시 stem (들었, 물었, 걸었) = 정확하지만 커버리지 제한

trade-off: 안전 + 작은 lift vs 위험 + 큰 lift. Sprint 155 학습 반영하여 안전 선택.

### 5.4 누적 surface normalization 효과

| Sprint | 추가 패턴 | 효과 |
|--------|----------|------|
| 128 | 하았/하어 (3 패턴) | +22.6% SURFACE_MISMATCH 흡수 |
| 134 P3 | 이습니다 + 하아 (2 패턴) | +1.0pp surface_only |
| 136 P3a | 르 불규칙 9 패턴 | +0.x pp |
| **156** | **ㄷ 불규칙 9 + 르 1 (총 10 패턴)** | **+0.1pp (+30 eojeols)** |

surface normalization은 sprint마다 작아지지만 누적적으로 +5pp 정도 효과.

---

## 6. 변경 파일

- `rust/crates/mecab-ko-core/src/evaluate.rs`:
  - `R_IRREGULAR_PATTERNS`: 아우르어 추가 (1줄)
  - `D_IRREGULAR_PATTERNS`: 신규 (9줄)
  - `normalize_endings`: Step 4 추가
  - 단위 테스트 3개 신규
- `docs/research/accuracy/2026-05-21_sprint156_d_irregular_surface_normalization.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

## 7. Sprint 157 후보

남은 안전 영역:
- 추가 surface normalization 패턴 (작은 lift 가능)
- 평가 메트릭 동치 그룹 확장 (Sprint 147 패턴, MAG↔MAJ 검토)

영역 소진 시:
- 비가역 작업 (Full CRF Retrain, NIKL Modu) confirm 요청

---

*작성: 2026-05-21 (Sprint 156 C)*
