# PLAN — mecab-ko Sprint 162 (next)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 161 B-4 — docs/ 추가 정리

### 결과

- PHASE6 보고서 4개 + phase6/ + optimization/ → archive
- docs/ 최상위 52 → 46 파일
- CLI 진단: 이미 충분 (8 commands, 7 formats)

## Sprint 162 후보 (자동 결정)

### 시나리오 B 계속 (NIKL Modu 미다운로드)

#### B-2: 성능 진단 sprint
- mecab-ko-profiler 활용
- 핫스팟 식별 (Viterbi/dict lookup)
- 측정 + 분석만 (구현은 별도)

#### B-3: 언어 바인딩 강화 (Python/WASM/Node)
- Python wheel 빌드 검증
- WASM bundler/web 사용성
- Node napi-rs 통합

#### B-4 추가 docs 정리
- ISSUE_BACKLOG.md, LessonLearn/, DIC-010-SUMMARY.md 등 검토
- 오래된 디렉토리 추가 식별

### 시나리오 A (NIKL Modu)
- 사용자 다운로드 완료 시 측정

### 시나리오 C (Full CRF Retrain)
- 비가역, 사용자 confirm 필요

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
