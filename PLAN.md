# PLAN — mecab-ko Sprint 163 (사용자 결정 대기)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 162 — 마지막 잔여 정리 + 영역 소진 선언

### Sprint 162 결과

- DIC-010-SUMMARY.md (완료 보고서) → archive
- ISSUE_BACKLOG.md, LessonLearn/ 검토 → 유지
- docs/ 최상위: 46 → 45 파일

### 안전 영역 소진 (Sprint 156~162 누적 7 sprint)

- Surface normalization: +0.15pp 누적
- PRACTICAL 동치: +0.2pp 3 silver
- NIKL Modu 인프라 준비
- docs 28 파일 archive

## 누적 진척 (Sprint 122 → 162)

| Metric | Baseline | 현재 |
|--------|---------|------|
| sample.tsv | 100%/99.9% | 100%/99.9% |
| **KLUE morph practical** | ~65.8% | **72.1%** |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** |
| UD Kaist morph practical | — | 68.6% |
| UD GSD morph practical | — | 71.8% |
| accuracy_eval.rs 줄 수 | 4963 | 2406 (-51%) |

## Sprint 163 — 사용자 결정 필수

자동 진행 불가능 상태. 4 옵션 중 사용자 선택:

### 옵션 1: NIKL Modu 다운로드 진척
- 학술 등록 진행 중인지?
- 완료 시 시나리오 A (측정 + 분석)

### 옵션 2: Full CRF Retrain (Track B)
- 3-5 sprint 비가역 대규모
- 잠재 lift +1~5pp
- **사용자 confirm 필요**

### 옵션 3: 정확도 작업 종료 + 다른 영역
- 정확도 lift 영역 소진
- 언어 바인딩 / 성능 / 사용자 기능 등
- 새 우선순위 사용자 결정 필요

### 옵션 4: 유지보수 모드
- 정확도 lift sprint 종료
- 버그 픽스, 의존성 업데이트만
- 다음 메이저 작업 대기

## 검증 기준 (모든 옵션 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
