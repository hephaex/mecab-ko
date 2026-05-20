# PROGRESS — mecab-ko Sprint 156 (ㄷ 불규칙 surface normalization)

> 마지막 업데이트: 2026-05-21

## Sprint 156 C — Surface normalization 확장

| Task | 상태 | 결과 |
|------|------|------|
| S156-1: 전문가 권고 (안전 영역 C) | ✅ 완료 | rust-pro agent |
| S156-2: surface mismatch 진단 실행 | ✅ 완료 | top 15 패턴 식별 |
| S156-3: 안전 후보 선택 | ✅ 완료 | ㄷ 불규칙 + 아우르다 (위험 패턴 제외) |
| S156-4: 구현 (R/D 패턴 확장) | ✅ 완료 | 10 패턴 추가 |
| S156-5: 단위 테스트 추가 | ✅ 완료 | 3 신규 테스트 |
| S156-6: 5-gate 검증 | ✅ 완료 | +0.1pp surface_only, 무회귀 |

## 변경 내용

### D_IRREGULAR_PATTERNS 신규 (Sprint 156)

```rust
const D_IRREGULAR_PATTERNS: &[(&str, &str)] = &[
    ("듣었", "들었"), ("듣어", "들어"), ("듣은", "들은"),  // 듣다 → 들
    ("묻었", "물었"), ("묻어", "물어"),                    // 묻다 → 물
    ("걷었", "걸었"), ("걷어", "걸어"),                    // 걷다 → 걸
    ("깨닫았", "깨달았"), ("깨닫아", "깨달아"),            // 깨닫다 → 깨달
];
```

### R_IRREGULAR_PATTERNS 확장

```rust
+ ("아우르어", "아울러"),  // 아우르다 + 어 (4건 KLUE)
```

### normalize_endings Step 4 추가

```rust
// Step 4: ㄷ불규칙 활용 (Sprint 156)
for (from, to) in D_IRREGULAR_PATTERNS {
    if out.contains(from) {
        out = out.replace(from, to);
    }
}
```

## 측정 결과

### 5-gate (Sprint 155 baseline → Sprint 156 C)

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv | 100.0%/99.9% | 100.0%/99.9% | 무회귀 ✓ |
| KLUE morph strict | 66.9% | 66.9% | — |
| KLUE morph practical | 71.9% | 71.9% | — |
| KLUE eojeol practical | 5283 | 5283 | — |
| KLUE surface strict | 87.8% | 87.8% | — |
| KLUE surface canonical | 91.6% | 91.6% | — |
| **KLUE surface canonical_lenient** | **95.5%** | **95.6%** | **+0.1pp (+30 eojeols)** |
| UD Kaist | 변경 없음 | — | — |
| UD GSD | 변경 없음 | — | — |

### 의미

- 21389 → 21419 eojeols canonical_lenient 매칭 (+30)
- normalize_endings는 surface_only metric에만 사용 → morph 무영향
- viterbi cascade 없음 (안전 영역 확인)

## 핵심 학습

### 1. 안전 영역의 진정한 가치

Sprint 155 회귀 → Sprint 156 안전 영역 전환 = 측정 가능 lift + 무회귀.
viterbi/CRF 변경 vs normalize_endings (평가 함수만) 영역 차이 명확.

### 2. 명시 stem 목록 vs 음운 분해

ㄷ 불규칙 일반화 시 false positive 위험 (예: "단어"는 ㄷ 시작 NNG, 변환 안 됨).
명시 stem 목록 (들/물/걸/깨달)만 안전 처리.

### 3. 누적 surface normalization 효과

| Sprint | 추가 | 효과 |
|--------|------|------|
| 128 | 하았/하어 | +22.6% mismatch 흡수 |
| 134 P3 | 이습니다 + 하아 | +1.0pp |
| 136 P3a | 르 불규칙 9 패턴 | +0.x pp |
| **156** | **ㄷ 불규칙 9 + 르 1 (10 패턴)** | **+0.1pp (+30 eojeols)** |

작아지지만 누적적으로 효과 명확.

### 4. 진단 → 안전 선택 (Sprint 148 D 패턴)

진단으로 후보 식별 → 위험 평가 → 안전 후보만 구현.
가장 큰 후보 (있어디에서 20건)는 mecab tokenizer 자체 이슈 → 정규화 영역 아님.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **409 passed / 0 failed** (406+3)
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings`: clean
- 5-gate sample.tsv: 100.0%/99.9% — **무회귀**
- KLUE surface canonical_lenient: **+0.1pp**

## 변경 파일

- `rust/crates/mecab-ko-core/src/evaluate.rs`:
  - D_IRREGULAR_PATTERNS 신규 (9)
  - R_IRREGULAR_PATTERNS 확장 (1)
  - normalize_endings Step 4 추가
  - 3 단위 테스트 신규
- `docs/research/accuracy/2026-05-21_sprint156_d_irregular_surface_normalization.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 157 후보

- 추가 surface normalization (작은 lift 가능)
- G: 평가 메트릭 동치 그룹 (MAG↔MAJ 검토)
- 영역 소진 시: F (NIKL Modu, confirm) / E (Full CRF Retrain, confirm)
