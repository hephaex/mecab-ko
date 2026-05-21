# PROGRESS — mecab-ko Sprint 160 (문서 정리 - sprint 보고서 아카이브)

> 마지막 업데이트: 2026-05-21

## Sprint 160 — docs/research/accuracy 정리

| Task | 상태 | 결과 |
|------|------|------|
| S160-1: NIKL Modu 다운로드 확인 | ⏸ 미다운로드 | 시나리오 B 전환 |
| S160-2: docs/research/accuracy 진단 | ✅ 완료 | sprint 보고서 23개 누적 |
| S160-3: docs/archive/sprint-reports/ 디렉토리 생성 | ✅ 완료 | |
| S160-4: sprint 132~158 보고서 23개 이동 | ✅ 완료 | git mv |
| S160-5: archive README 작성 | ✅ 완료 | 누적 진척 + 분류표 |
| S160-6: SPRINT_LEARNINGS.md 종합 작성 | ✅ 완료 | docs/research/accuracy/ |
| S160-7: 검증 (5-gate 무회귀) | ✅ 완료 | 코드 변경 없음 |

## 변경 내용

### 1. 디렉토리 구조 정리

**Before** (`docs/research/accuracy/`):
- 일반 분석 문서 11개 + sprint 보고서 23개 = 34개 파일

**After**:
- `docs/research/accuracy/`: 일반 분석 11개 + SPRINT_LEARNINGS.md 1개 = 12개
- `docs/archive/sprint-reports/`: sprint 132~158 보고서 23개 + README.md

### 2. SPRINT_LEARNINGS.md 신규

`docs/research/accuracy/SPRINT_LEARNINGS.md`:
- 30+ sprint 종합 학습 정리
- 영역별 접근 (안전/위험/비이슈)
- 메타 학습 (mecab dict 위력, 빈도 ≠ 실효 lift 등)
- 워크플로우 정착 (Sprint 154 이후 규칙 5)
- 누적 진척 표 (Sprint 122 → 158)

### 3. archive README

`docs/archive/sprint-reports/README.md`:
- 누적 진척표
- Lift sprints / Infrastructure sprints / Rollback sprints / 분석 sprints 분류
- 보고서 목록 (sprint 순)

## NIKL Modu 다운로드 상태

`data/eval/nikl_modu_*.tsv` 미존재 → 시나리오 B (정확도 외 영역) 전환.

사용자가 NIKL Modu 다운로드 완료 시 Sprint 161 시나리오 A로 전환 가능.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **411 passed / 0 failed** (변경 없음)
- 5-gate sample.tsv: 영향 없음 (코드 변경 없음)
- 문서 구조: 깔끔 (docs/research/accuracy 12개 파일)

## 변경 파일

- `docs/archive/sprint-reports/` (신규): sprint 132~158 23개 + README.md
- `docs/research/accuracy/SPRINT_LEARNINGS.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 161 후보

### 시나리오 B 계속 (정확도 외 영역)
- **CLI/API 사용성**: mecab-ko-cli 옵션 개선
- **성능 최적화**: 프로파일링 + 핫스팟 식별
- **언어 바인딩 강화**: Python/WASM/Node 통합

### 시나리오 A (NIKL Modu 다운로드 완료 시)
- 측정 → POS mismatch 분석
- 추가 동치/normalize 후보 발굴

### 시나리오 C (사용자 confirm 필요)
- Full CRF Retrain (Track B, 3-5 sprint)
