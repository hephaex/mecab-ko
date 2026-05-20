# PLAN — mecab-ko Sprint 155 (방향 전환)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 154 — 빈도 기반 영역 소진 선언

### 통합 진단 결과

| 패턴 | 빈도 | 미처리 |
|------|------|--------|
| EP+ETM | 86 | 0 |
| XSV+ETM | 72 | 0 |
| VX+EP | 25 | 0 |
| XSA+EP | 35 | 0 |

mecab dict decomposition features가 모든 케이스 처리.

### 영역 소진 결론

| Sprint | 빈도 후보 | 실효 lift |
|--------|----------|----------|
| 148 D | 33 | 0 (비이슈) |
| 150 A | 542 | 24 (+0.4pp) |
| 153 E | 38 | 0 (비이슈) |
| 154 | 218 | 0 (비이슈) |

총 833건 중 24건만 의미 (2.9%) → **splitter rule 영역 소진**.

## 새로운 방향 (Sprint 155 후보)

### 안전 작업 (sprint당 0.3~0.5)

#### A. dict 확장 — Sprint 130/132 재방문

- 도메인 NNG/NNP 추가 (KLUE/UD에서 미처리 단어)
- 빈도 측정 + 효과 검증
- 위험: 낮음, lift: 가능 (Sprint 130 +0.7pp, Sprint 132 +0.3pp 전례)

#### B. test_klue_dp_real_error_analysis 활용

- 실제 오분류 패턴 분석 (빈도가 아닌 오류 기반)
- 진단 → 작업 후보 식별
- 위험: 낮음 (분석 only sprint 가능)

#### C. surface normalization 확장 — Sprint 134 패턴

- normalize_endings 추가 후보 발굴
- canonical / canonical_lenient 평가 lift
- 위험: 낮음, 분석 sprint 필요

#### D. CRF Track A 재시도 — 좁은 범위

- Sprint 137 분석 + Sprint 138 rollback 교훈 반영
- 매우 좁은 범위 (1~3 pair) 수동 조정
- 위험: 중간 (Sprint 138 회귀 전례)

### 비가역 대규모 (사용자 confirm 필요)

#### E. Full CRF Retrain (Track B)

- 3-5 sprint
- 학습 데이터 + mecab-cost-train
- 잠재 lift +1~5pp

#### F. NIKL Modu 도입

- Academic license 다운로드
- 구어/SNS 도메인 확장
- 새 silver dataset → CI gate 추가

## 다음 결정 프로세스

규칙 5에 따라:
1. 전문가 리뷰 → A/B/C/D 중 Top 권고
2. 자동 채택 → 진단 또는 구현
3. 비가역 작업 (E/F)은 사용자 confirm

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
