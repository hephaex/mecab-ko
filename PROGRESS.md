# PROGRESS — mecab-ko Sprint 155 (Dict 확장 시도 → 회귀 → rollback)

> 마지막 업데이트: 2026-05-21

## Sprint 155 — B (진단) + A (실패) + rollback

| Task | 상태 | 결과 |
|------|------|------|
| S155-B1: 전문가 리뷰 → B+A 2단계 권고 | ✅ 완료 | rust-pro agent |
| S155-B2: test_klue_dp_real_error_analysis 활용 | ✅ 완료 | 10 POS mismatch 패턴 추출 |
| S155-B3: 호스트 76건 등 NNG→NNP 식별 | ✅ 완료 | Top: 호스트 (Airbnb 도메인) |
| S155-A1: 외래어 NNP 9건 추가 시도 | ❌ 실패 | KLUE -0.2pp 회귀 |
| S155-A2: 호스트 단독 시도 | ❌ 실패 | -44 eojeols |
| S155-A3: Rollback (Sprint 138 정책) | ✅ 완료 | Baseline 복원 |
| S155-A4: 회귀 원인 분석 | ✅ 완료 | viterbi cascade 영향 가설 |

## 측정 결과

### 1차 (9개 entries)

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv | 100.0%/99.9% | 100.0%/99.9% | ✓ |
| **KLUE morph strict** | 66.9% | 66.7% | **-0.2pp 회귀** |
| KLUE eojeol practical | 5283 | 5235 | -48 |

### 2차 (호스트 단독)

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| KLUE morph strict | 66.9% | 66.7% | -0.2pp |
| KLUE eojeol practical | 5283 | 5239 | -44 |

### Rollback 후

KLUE morph strict 66.9% / eojeol practical 5283 / sample.tsv 100% — **baseline 복원** ✓

## 핵심 발견

### Viterbi cascade 회귀 패턴

3 sprint 누적 패턴:

| Sprint | 시도 | 결과 |
|--------|------|------|
| 138 | matrix.def cost 조정 | -0.9pp sample.tsv |
| 145 D | multi-syllable VV+ETM split | -1 sample.tsv sentence |
| **155 A** | **dict cost=-5000 NNP** | **-0.2pp KLUE** |

**공통점**: viterbi/CRF 영향 변경 → cascade 회귀.

### cost=-5000은 위험한 값

너무 강한 cost가 viterbi path를 강제 변경. 다른 안전 path 차단.

**가설**: 호스트 NNP 강제 → 합성어 분해, connection cost 변화, cascade 영향.

### 안전한 lift 영역 — 재정의

| 영역 | 상태 |
|------|------|
| Splitter rule 추가 | ❌ 영역 소진 (Sprint 154) |
| dict cost=-5000 확장 | ❌ Sprint 155 회귀 |
| CRF matrix 조정 | ❌ Sprint 138 회귀 |
| **평가 메트릭 동치 그룹** | ✅ Sprint 147 패턴 |
| **Surface normalization** | ✅ Sprint 134 패턴 |
| Full CRF Retrain | ⏸ 비가역 (confirm 필요) |
| 새 silver dataset | ⏸ coverage only |

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (rollback 완료)
- 5-gate: baseline 복원 (Sprint 150 A 수준)
- klue-domain.csv: rollback 완료

## 변경 파일

- `data/user-dict/klue-domain.csv`: 시도 후 rollback (rollback 코멘트 추가)
- `docs/research/accuracy/2026-05-21_sprint155_dict_expansion_rollback.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 156 후보

- **C**: Surface normalization 확장 (Sprint 134 패턴)
- **G**: 추가 practical 동치 그룹 (보수적)
- **F [confirm]**: 새 silver dataset (NIKL Modu)
- **E [confirm]**: Full CRF Retrain
