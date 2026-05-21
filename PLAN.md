# PLAN — mecab-ko Sprint 161 (next)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 160 — 문서 정리 (sprint 보고서 archive)

### 결과

- 23개 sprint 보고서 (132~158) → `docs/archive/sprint-reports/` 이동
- `SPRINT_LEARNINGS.md` 신규 (종합 학습)
- archive README (분류표 + 메타 학습)

### 디렉토리 정리

`docs/research/accuracy/`:
- Before: 34 파일 (일반 11 + sprint 23)
- After: 12 파일 (일반 11 + SPRINT_LEARNINGS.md)

## Sprint 161 후보 (자동 결정)

### 시나리오 B 계속 (정확도 외 영역, NIKL Modu 다운로드 보류 중)

#### B-1: CLI/API 사용성 개선
- mecab-ko-cli 옵션 추가/정리
- 사용자 친화적 출력 형식
- 진단: `mecab-ko-cli --help` 현재 상태 분석

#### B-2: 성능 최적화
- 프로파일링 (mecab-ko-profiler 활용)
- 핫스팟 식별
- 최적화 적용 (안전한 범위)

#### B-3: 언어 바인딩 강화
- Python (mecab-ko-python) 통합 강화
- WASM (mecab-ko-wasm) 사용성
- Node (mecab-ko-node) 통합

#### B-4: 추가 문서 정리
- docs/ 트리 추가 정리 (예: optimization/, phase6/ 등 오래된 디렉토리)
- README 업데이트
- 새 사용자 가이드

### 시나리오 A (NIKL Modu 다운로드 완료 시)
- 측정 + 분석

### 시나리오 C (사용자 confirm)
- Full CRF Retrain

## 결정 프로세스

규칙 5: 전문가 리뷰 → Top 권고 → 자동 채택.
- B 시나리오 sub-options: 자동 선택
- C (CRF Retrain): 비가역 → confirm 필요

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
