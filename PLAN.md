# PLAN — mecab-ko Sprint 145 (next)

> 마지막 업데이트: 2026-05-20

## 완료: Sprint 144 A — accuracy-gate CI 5번째 게이트 (UD GSD)

### 추가
- `Run UD GSD silver gate` step (test_ud_gsd_dual_metric 실행)
- Floor: morph strict ≥ 60%
- PR comment 5번째 섹션 + 5-gate summary

### 5-gate 시스템 완성

| Gate | Dataset | 도메인 |
|------|---------|--------|
| 1 | sample.tsv | curated quality |
| 2 | KLUE DP morph | 뉴스/리뷰 |
| 3 | KLUE DP surface-only | 검색/색인 |
| 4 | UD Korean-Kaist | 역사/철학/학술 |
| 5 | UD Korean-GSD | Google news/web |

### 보고서
`docs/research/accuracy/2026-05-20_sprint144_ud_gsd_ci_gate.md`

## 다음 스프린트: Sprint 145 (미정 — 사용자 선택)

### 후보 B [메인]: Full CRF Retrain (Track E)

**선행 충족**: Sprint 142 dict-builder fix + Sprint 143 UD GSD + Sprint 144 5-gate CI.

**작업 (3-5 sprint 예상)**:
1. 학습 코퍼스 준비:
   - KLUE train (CC BY-SA 4.0) — 10K sentences
   - UD Kaist train — 23K sentences
   - UD GSD train — 5K sentences
   - 라이선스 호환 (CC BY-SA 4.0 통합)
2. `legacy/src/mecab-cost-train` (C++) 빌드 + 실행
3. 새 `model.def` → `matrix.def` + `left/right-id.def` 재생성
4. `mecab-ko-dict-builder` 재실행 → binary
5. **5-gate 회귀 검증** + lift 측정

**리스크**:
- 학습 시간 (수 시간)
- left/right-id.def 변경 시 기존 binary 호환성
- 학습 데이터 비율 조정 필요할 수 있음

**기대 효과**: +1pp 이상 (지금까지 시도와 차원 다른 결과)

### 후보 C: NIKL Modu 수동 다운로드 + 평가

Academic license, 로컬 only. 6번째 silver. 0.5-1 sprint.

### 후보 D: 다른 mecab 결합 토큰 패턴 (Sprint 141 연장)

NNG+VCP+EC, VV+EP+EF 등. 0.5-1 sprint. 위험 낮음.

### 후보 E: UD Korean-PUD 추가

또 다른 silver (1,000 sentences). convert_ud_gsd.py 또는 새 변환기.
0.5 sprint.

## 백로그

- P4 (borderline NNG↔NNP): Sprint 132 보류 5 entries

## 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
