# PLAN — mecab-ko Sprint 156 (next)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 155 — Dict 확장 실패 → rollback

### 결과

- B (진단): 호스트 76건 등 NNG→NNP 후보 식별 ✓
- A (구현): cost=-5000 NNP 추가 → -0.2pp KLUE 회귀 ❌
- Rollback: baseline 복원 ✓

### Sprint 138/145/155 회귀 패턴

| Sprint | 시도 | 결과 |
|--------|------|------|
| 138 | matrix.def 조정 | rollback |
| 145 D | multi-syllable VV+ETM | rollback |
| 155 A | dict NNP cost=-5000 | rollback |

**공통점**: viterbi/CRF 영향 변경 → cascade 회귀.

## 영역 소진 / 잔여 안전 영역

| 영역 | 상태 |
|------|------|
| Splitter rule | ❌ Sprint 154 소진 |
| dict cost 확장 | ❌ Sprint 155 회귀 |
| CRF matrix 조정 | ❌ Sprint 138 회귀 |
| **평가 메트릭 동치** | ✅ 안전 |
| **Surface normalization** | ✅ 안전 |
| Full CRF Retrain | ⏸ 비가역 |
| 새 silver dataset | ⏸ coverage only |

## Sprint 156 후보

### C [권고]: Surface normalization 확장 (Sprint 134 패턴)

- normalize_endings 추가 후보 발굴
- canonical/canonical_lenient 평가 lift
- 위험: 매우 낮음 (메트릭 변경만)
- Sprint 134 전례: +1.0pp surface_only

### G: 평가 메트릭 추가 동치 그룹

Sprint 155 진단 결과:
- MAG↔MAJ 45건 (다만, 및, 역시) — 부사 분류 차이
- MAG/MAJ 동치 추가 검토
- Sprint 147 패턴 (XSV practical 동치)
- 위험: 낮음 (메트릭만)

### F [confirm]: 새 silver dataset (NIKL Modu)

- Academic license, 구어/SNS 도메인
- coverage 확장, lift는 아님
- 비가역 (수동 다운로드 필요)
- **사용자 confirm 필요**

### E [confirm]: Full CRF Retrain

- 3-5 sprint, 학습 데이터 + mecab-cost-train
- 잠재 lift +1~5pp
- 비가역 대규모
- **사용자 confirm 필요**

## 다음 결정 프로세스

규칙 5: 전문가 리뷰 → Top 권고 → 자동 채택.
- 안전 후보 (C, G): 자동 진행
- 비가역 (F, E): 사용자 confirm 필요

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
