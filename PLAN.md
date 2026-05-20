# PLAN — mecab-ko Sprint 157 (next)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 156 C — Surface normalization 확장

- ㄷ 불규칙 9 패턴 + 르 불규칙 1 (총 10) 추가
- normalize_endings Step 4 (ㄷ 불규칙) 신규
- KLUE surface canonical_lenient: 95.5% → 95.6% (+0.1pp, +30 eojeols)
- sample.tsv 무회귀, morph 무영향

### 안전 영역 검증

normalize_endings (평가 함수만) = viterbi cascade 없음 → 안전. Sprint 155 회귀 후 진정으로 안전한 lift 영역 확인.

## 누적 정확도 지표 (Sprint 156 후)

| Metric | 현재 |
|--------|------|
| sample.tsv | 100.0%/99.9% (baseline) |
| KLUE morph strict | 66.9% |
| KLUE morph practical | 71.9% |
| KLUE eojeol practical | 5283 / 22404 |
| KLUE surface strict | 87.8% |
| KLUE surface canonical | 91.6% |
| **KLUE surface canonical_lenient** | **95.6%** |
| UD Kaist morph practical | 68.4% |
| UD GSD morph practical | 71.6% |

## Sprint 157 후보

### 안전 후보

#### 잔여 surface normalization 후보
Sprint 156 진단에서 식별된 미처리 패턴:
- 들었/걸었/물었 외 ㄷ 불규칙 (싣다 → 실 등)
- ㅎ 불규칙 (그렇다 → 그래)
- 예측 효과: +0.05pp 정도

#### G: 평가 메트릭 동치 그룹 (Sprint 147 패턴)

- MAG↔MAJ 45건 (다만, 및, 역시) — 부사 대분류 동치
- 위험: 낮음 (메트릭 영역)
- 예측 효과: KLUE practical morph +0.1~0.2pp

### 비가역 작업 (사용자 confirm)

#### F: 새 silver dataset (NIKL Modu)
- 구어/SNS 도메인 확장
- Academic license + 수동 다운로드

#### E: Full CRF Retrain (Track B)
- 3-5 sprint, mecab-cost-train
- 잠재 lift +1~5pp

## 다음 결정 프로세스

규칙 5: 전문가 리뷰 → Top 권고 → 자동 채택.

영역 소진 진단:
- Surface normalization: 점점 작아지나 아직 잔여 있음
- 평가 메트릭 동치: G로 시도 가능
- 둘 다 소진 시: 비가역 작업 confirm 요청

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
