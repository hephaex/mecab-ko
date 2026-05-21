# Sprint 155 — Dict 확장 시도 후 회귀 → rollback

> **결과**: NNG→NNP 진단으로 호스트 76건 등 외래어 후보 식별. 9건 NNP 추가 → KLUE -0.2pp 회귀. 호스트 단독도 회귀. cost=-5000 cascade 영향 추정. rollback. Sprint 138 패턴 재현.

---

## 1. Sprint 155 B — 진단 (성공)

`test_klue_dp_real_error_analysis` 출력에서 POS mismatch 패턴 추출:

| Gold | Pred | 건수 | Top surface |
|------|------|------|------------|
| NNG | NNP | 155 | 호스트 (76) |
| NNB | NNG | 153 | 일 (53), 월 (19), 명 (19) — practical 동치 ✓ |
| NNP | NNG | 78 | 분산 (max 2건) |
| MMD | MM | 76 | 그 (?), 이 (?) |
| SL | NNP | 52 | 분산 |
| VV | NNG | 47 | 분산 |
| VA | VV | 46 | 있 (46) ← 이미 practical 동치 |
| MAG | MAJ | 45 | 다만 (20), 및 (13) |
| MMN | MM | 39 | 모든 (23), 두 (6) |
| NNG | NNB | 36 | 때 (18), 시 (5), 양 (4) — practical 동치 ✓ |

### 의미

- **NNB↔NNG 153+36건**: 이미 `TAG_EQUIVALENCE_GROUPS_PRACTICAL`에 포함 → practical 평가 무영향
- **VA↔VV 46건**: 이미 practical 동치 (Sprint 136)
- **NNG→NNP 155건**: dict 확장 후보, 호스트가 압도적

### 호스트 분석

- KLUE 도메인: Airbnb 후기 다수
- "호스트" = Airbnb host (NNP-like proper role)
- 76건 모두 KLUE Airbnb 컨텍스트
- mecab 분류: NNG (일반명사), gold: NNP

---

## 2. Sprint 155 A — Dict 확장 시도 (실패)

### 2.1 1차 시도: 9개 entries

`klue-domain.csv`에 추가 (cost=-5000):
- 호스트 (76건), 와이파이 (5), 트램 (5), 하우스 (4)
- 메트로 (3), 게스트 (2), 메인 (2), 브랜드 (2), 브런치 (2)
- 잠재 lift: 101 morphemes

**결과 (전체 9건)**:
| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv | 100.0%/99.9% | 100.0%/99.9% | 무회귀 ✓ |
| KLUE morph strict | 66.9% | 66.7% | **-0.2pp** |
| KLUE eojeol practical | 5283 | 5235 | **-48** |
| UD Kaist | 영향 없음 | — | — |
| UD GSD | 영향 없음 | — | — |

KLUE 회귀 발생.

### 2.2 2차 시도: 호스트 단독 (가장 큰 후보)

다른 8건 제거, 호스트만 유지.

**결과 (호스트 단독)**:
| Metric | Before | After | Δ |
|--------|--------|-------|---|
| KLUE morph strict | 66.9% | 66.7% | **-0.2pp** |
| KLUE eojeol practical | 5283 | 5239 | **-44** |

호스트 단독도 회귀. 76건 lift 기대 → 오히려 -44 eojeols 감소.

### 2.3 Rollback

Sprint 138 정책 (회귀 시 즉시 rollback) 적용. klue-domain.csv 원복.

**Baseline 복원 확인**:
- KLUE morph strict: 66.9%
- KLUE eojeol practical: 5283
- sample.tsv: 100.0%/99.9%

---

## 3. 회귀 원인 가설

### Cascade effect

`cost=-5000`은 매우 강한 cost. mecab Viterbi가 호스트를 무조건 NNP path 선택. 영향:
1. **합성어 분해 강제**: "호스트가족"이 있을 때 → "호스트/NNP + 가족/NNG"로 분해 (gold가 단일 단어를 요구할 수 있음)
2. **연결 cost 변화**: NNP는 NNG와 다른 connection table 적용 → 후속 토큰 분석 영향
3. **dict cost 메커니즘**: -5000이 절대값 아닌 상대값. mecab dict의 NNG base cost와 NNP base cost 차이를 무시할 수 있음

### 단순화 검증 불가능 — viterbi 전체 path 추적 필요

mecab Viterbi의 빔 서치 결과를 직접 디버그하지 않는 한 정확한 회귀 원인 파악 어려움. dict 확장은 viterbi 전체 path에 영향 → 단순 빈도 lift 예측 불가.

---

## 4. 누적 회귀 패턴

| Sprint | 시도 | 회귀 | 공통점 |
|--------|------|------|--------|
| 138 | matrix.def 수동 cost 조정 | -0.9pp sample.tsv | viterbi 전체 영향 |
| 145 D | multi-syllable VV+ETM split | -1 sentence sample.tsv | false positive cascade |
| **155 A** | **dict cost=-5000 NNP 9건** | **-0.2pp KLUE** | **viterbi path cascade** |

**핵심 패턴**: viterbi/CRF 메커니즘에 영향을 주는 변경은 cascade 회귀 위험 매우 높음.

---

## 5. 안전한 lift 경로 — 재정의

영역 소진 표:

| 영역 | 상태 |
|------|------|
| Splitter rule 추가 | ❌ 소진 (Sprint 154) |
| Splitter ㅂ/ㄹ 불규칙 | ❌ mecab dict이 처리 |
| Compound POS 동치 그룹 (practical) | ✅ Sprint 147 패턴 적용 가능 |
| Surface normalization | ✅ Sprint 134 패턴 적용 가능 |
| dict 확장 (NNP cost) | ❌ Sprint 155 회귀 |
| CRF matrix 수동 조정 | ❌ Sprint 138 회귀 |
| Full CRF Retrain | ⏸ 비가역 (3-5 sprint, confirm) |
| 새 silver dataset | ⏸ coverage only, lift 아님 |

### 남은 안전 후보

#### 1. Practical 동치 그룹 확장 (Sprint 147 패턴)

실제 mismatch 패턴 (MAG↔MAJ, MMN↔MM, MMD↔MM):
- MAG↔MAJ 45건 → 동치 그룹 후보
- MMN↔MM 39건 → 이미 MM/MMD/MMN/MMA 동치 (CONSERVATIVE)
- MMD↔MM 76건 → 동치 (CONSERVATIVE)

→ 이미 처리되거나 추가 분석 필요.

#### 2. Surface normalization 확장 (Sprint 134 패턴)

normalize_endings 추가 후보 → canonical_lenient 평가 lift.
영향: morph strict 무관, surface_only metric 개선.

---

## 6. 핵심 학습 포인트

### 6.1 빈도 ≠ 실효 lift (재확인)

호스트 76건 raw mismatch가 dict 확장으로 즉시 lift되지 않음. viterbi cascade 영향 고려 필요.

### 6.2 dict cost=-5000은 위험한 값

너무 강한 cost는 viterbi path를 강제로 변경. 다른 안전한 path를 차단할 수 있음. 향후 dict 추가 시 cost=-1000~-3000 범위로 시작 권장.

### 6.3 Sprint 138, 145, 155 — viterbi 영향 변경의 위험성

3 sprint 모두 viterbi/CRF 영향 변경 시도 → 회귀. 안전한 변경 영역은:
- 평가 메트릭 (동치 그룹, surface normalization)
- splitter post-processing (분석 결과 변환만)

### 6.4 Rollback 의사결정 신속화

회귀 발견 즉시 rollback (Sprint 138 정책). 분석은 rollback 후. 시간 절약.

---

## 7. 변경 파일

- `data/user-dict/klue-domain.csv`: rollback (시도 → 원복)
- `docs/research/accuracy/2026-05-21_sprint155_dict_expansion_rollback.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

## 8. Sprint 156 방향

남은 안전 후보:
- **C**: Surface normalization 확장 (Sprint 134 패턴)
- **G**: 평가 메트릭 추가 동치 그룹 (보수적, 분석 필요)
- 또는 **F**: 새 silver dataset (coverage 확장, confirm 필요)

비가역 작업:
- E: Full CRF Retrain (confirm 필요)

---

*작성: 2026-05-21 (Sprint 155 B + A 시도)*
