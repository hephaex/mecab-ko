# PLAN — mecab-ko Sprint 158 (next)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 157 G — MAG/MAJ practical 동치

### 결과 (3 silver 일관 lift)

| Dataset | morph practical | Δ | eojeol Δ |
|---------|----------------|---|---------|
| KLUE | 71.9% → **72.1%** | +0.2pp | +44 |
| UD Kaist | 68.4% → **68.6%** | +0.2pp | +65 |
| UD GSD | 71.6% → **71.8%** | +0.2pp | +15 |

**총 +124 eojeols 추가 매칭**, sample.tsv 무회귀, conservative 정밀 보존.

### 누적 PRACTICAL 동치 그룹

| Sprint | 추가 |
|--------|------|
| 126 | NNB↔NNG |
| 136 | VA↔VV |
| 147 | VV↔XSV |
| **157** | **MAG↔MAJ** |

## 현재 정확도 지표

| Metric | 현재 |
|--------|------|
| sample.tsv | 100.0%/99.9% |
| KLUE morph strict | 66.9% |
| **KLUE morph practical** | **72.1%** |
| KLUE eojeol practical | 5327 / 22404 |
| KLUE surface canonical_lenient | 95.6% |
| UD Kaist morph practical | 68.6% |
| UD GSD morph practical | 71.8% |

## Sprint 158 후보 — 안전 영역 거의 소진

### 잔여 안전 후보 (작은 lift)

#### 추가 surface normalization
- ㅎ 불규칙 (그렇다 → 그래)
- ㅂ 불규칙 (받침이 있는 stem + 모음)
- 예측: +0.05pp

#### 추가 진단 + PRACTICAL 동치
- Sprint 155 진단 남은 패턴: VV↔NNG (47건, 의미 차이 위험)
- MMD↔MM 76건 (이미 conservative)
- 예측 효과: 거의 없음

### 비가역 작업 (사용자 confirm 필요)

영역 소진 신호. 다음은 비가역 대규모:

#### F: NIKL Modu 도입
- Academic license 수동 다운로드
- 구어/SNS 도메인 silver dataset
- coverage 확장 (lift는 아님)
- **사용자 confirm 필요**

#### E: Full CRF Retrain (Track B)
- 3-5 sprint, 학습 데이터 + mecab-cost-train
- 잠재 lift +1~5pp
- Sprint 136에서 인프라 조사 완료
- **사용자 confirm 필요**

## 결정 프로세스

규칙 5: 전문가 리뷰 → Top 권고 → 자동 채택.

영역 소진 시 비가역 작업으로 진행:
1. 작은 잔여 (surface normalization 추가) → 시도 → 영향 미미하면 종료
2. F (NIKL Modu) 또는 E (CRF Retrain) confirm 요청
3. confirm 없으면 정확도 작업 종료 (다른 영역: 도구, 문서, 성능 등)

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
