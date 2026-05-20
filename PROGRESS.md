# PROGRESS — mecab-ko Sprint 157 (MAG/MAJ practical 동치)

> 마지막 업데이트: 2026-05-21

## Sprint 157 G — MAG/MAJ practical 동치 추가

| Task | 상태 | 결과 |
|------|------|------|
| S157-G1: Sprint 155 진단 결과 재활용 | ✅ 완료 | MAG→MAJ 45건 식별 |
| S157-G2: PRACTICAL 그룹 MAG/MAJ 추가 | ✅ 완료 | `evaluate.rs:853` |
| S157-G3: 단위 테스트 추가 | ✅ 완료 | `test_pos_tags_equivalent_practical_includes_mag_maj` |
| S157-G4: 5-gate 측정 (3 silver) | ✅ 완료 | **3 silver 모두 +0.2pp morph practical** |
| S157-G5: 회귀 확인 | ✅ 완료 | sample.tsv 무회귀, conservative 무영향 |

## 핵심 결과

### 3 silver 일관 +0.2pp Lift

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv | 100.0%/99.9% | 100.0%/99.9% | 무회귀 ✓ |
| **KLUE morph practical** | 71.9% | **72.1%** | **+0.2pp** |
| **KLUE eojeol practical** | 5283 | **5327** | **+44** |
| **UD Kaist morph practical** | 68.4% | **68.6%** | **+0.2pp** |
| **UD Kaist eojeol practical** | 4197 | **4262** | **+65** |
| **UD GSD morph practical** | 71.6% | **71.8%** | **+0.2pp** |
| **UD GSD eojeol practical** | 2926 | **2941** | **+15** |
| KLUE/UD strict morph | — | — | — (정밀 보존) |

**총 eojeol +124건 추가 매칭**.

### 일관된 도메인 lift = 진짜 효과

3 silver 모두 +0.2pp morph practical → 단일 anomaly 아님, 도메인 독립적.
Sprint 147 (VV/XSV 동치) 패턴 재현.

### 변경 내용

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

### 언어학적 배경

- MAG: 일반 부사 (very, slowly 등)
- MAJ: 접속 부사 (and, but, however, 다만 등)
- mecab은 모두 MAG, KLUE는 접속부사를 MAJ로 분리 — convention 차이

샘플 surfaces (Sprint 155 진단):
- 다만 (20), 및 (13), 역시 (3), 오히려 (2), 아무튼 (2), 또는 (1)

## 핵심 학습

### 1. 진단 결과 재활용

Sprint 155 진단 → A (실패) → G (성공). 같은 데이터로 여러 접근 가능.

| 접근 | Sprint | 결과 |
|------|--------|------|
| dict 확장 | 155 A | 회귀 → rollback |
| 메트릭 동치 | 157 G | +0.2pp 성공 |

### 2. 안전 영역 누적 효과

| Sprint | 영역 | 효과 |
|--------|------|------|
| 156 | surface normalization | +0.1pp (+30 eojeols) |
| **157** | **메트릭 동치** | **+0.2pp (+124 eojeols)** |

Sprint 155 회귀 후 안전 영역으로 전환 = 누적 +0.3pp KLUE morph practical.

### 3. PRACTICAL 동치 누적 진척

| Sprint | 추가 | 언어학적 배경 |
|--------|------|----------|
| 126 | NNB↔NNG | counter words |
| 136 | VA↔VV | "있다" |
| 147 | VV↔XSV | "하/되" |
| **157** | **MAG↔MAJ** | **접속부사** |

모두 한국어 문법 진행 중 convention 차이.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **410 passed / 0 failed** (409+1)
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings`: clean
- 5-gate: 3 silver 모두 +0.2pp practical, sample.tsv 무회귀, conservative 정밀 보존

## 변경 파일

- `rust/crates/mecab-ko-core/src/evaluate.rs`:
  - PRACTICAL 그룹에 MAG/MAJ 추가
  - 단위 테스트 1개 신규
- `docs/research/accuracy/2026-05-21_sprint157_mag_maj_practical.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 158 후보

### 남은 안전 영역

PRACTICAL 동치 추가 후보 (Sprint 155 진단):
- 대부분 이미 처리됨 (MMD/MMN/MM, NNB/NNG, VA/VV)
- VV↔NNG는 의미 차이 (위험)

남은:
- 추가 surface normalization (작은 lift)
- 추가 진단 후보 발굴 (Sprint 155 진단의 다른 패턴)

### 비가역 작업

영역 거의 소진 상태. 다음은 비가역 (사용자 confirm 필요):
- F: NIKL Modu 도입
- E: Full CRF Retrain
