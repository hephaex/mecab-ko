# PLAN — mecab-ko Sprint 144 (next)

> 마지막 업데이트: 2026-05-20

## 완료: Sprint 143 C — UD Korean-GSD silver baseline 통합

### 변환 결과
- `tools/convert_ud_gsd.py` (identity mapping — GSD XPOS는 Sejong 태그 직접 사용)
- 1,904 silver sentences 변환 (971 test + 933 dev, 98.2% 변환률)
- 신규: `data/eval/ud_gsd_{test,dev}.tsv`, `test_ud_gsd_dual_metric`

### Baseline 측정 (3 silver 비교)

| Metric | KLUE DP | UD Kaist | UD GSD |
|--------|---------|----------|--------|
| Morph strict | 66.8% | 66.3% | **67.4%** |
| Morph practical | 71.6% | 68.1% | 71.3% |
| Per-eojeol strict | 20.7% | 20.7% | **23.1%** |
| Per-eojeol practical | 23.5% | 21.8% | **25.8%** |

**핵심 발견**: GSD가 KLUE에 가장 가까움 (현대 뉴스/web). Kaist 학술 텍스트로 낮음.

### 보고서
`docs/research/accuracy/2026-05-20_sprint143_ud_gsd.md`

## 다음 스프린트: Sprint 144 (미정 — 사용자 선택)

### 후보 A: accuracy-gate CI에 UD GSD 추가 (4 → 5 gate)

Sprint 140 C 패턴 재사용. step 추가 + PR comment 섹션.

**Floor**: morph strict ≥ 60% (silver, KLUE 동일 기준)
**비용**: 0.5 sprint
**위험**: 낮음

### 후보 B [메인]: Full CRF Retrain (Track E)

**선행 충족**: Sprint 142 dict-builder fix.

**작업 (3-5 sprint 예상)**:
1. 학습 코퍼스 준비 (Sejong + KLUE train + UD Kaist train + UD GSD train)
2. `legacy/src/mecab-cost-train` (C++) 빌드 + 실행
3. 새 `model.def` → `matrix.def` + `left/right-id.def` 재생성
4. `mecab-ko-dict-builder` 재실행 → binary
5. 5-gate 회귀 검증 + lift 측정

**리스크**: 학습 코퍼스 라이선스 (Sejong 비공개), binary 호환성, 학습 시간 (수 시간).

### 후보 C: NIKL Modu 수동 다운로드 + 평가

Academic license, 로컬 only. 0.5-1 sprint.

### 후보 D: 다른 mecab 결합 토큰 패턴 (Sprint 141 연장)

NNG+VCP+EC, VV+EP+EF 등. 0.5-1 sprint.

## 백로그

- P4 (borderline NNG↔NNP): Sprint 132 보류 5 entries
- UD Korean-PUD (1,000 sentences 더 추가)

## 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- 4-gate CI 통과 (sample.tsv / KLUE morph / surface_only / UD Kaist) — UD GSD 추가 시 5-gate
