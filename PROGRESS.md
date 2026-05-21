# PROGRESS — mecab-ko Sprint 161 (docs 추가 정리)

> 마지막 업데이트: 2026-05-21

## Sprint 161 B-4 — docs/ phase6 + optimization archive

| Task | 상태 | 결과 |
|------|------|------|
| S161-1: NIKL Modu 다운로드 확인 | ⏸ 미다운로드 | 시나리오 B 계속 |
| S161-2: CLI 현재 상태 진단 | ✅ 완료 | 8 commands, 7 formats, 잘 갖춰짐 |
| S161-3: docs/ 오래된 디렉토리 식별 | ✅ 완료 | PHASE6_*.md (4) + phase6/ + optimization/ |
| S161-4: archive 디렉토리 생성 + 이동 | ✅ 완료 | docs/archive/phase6-and-old-optimization/ |
| S161-5: README 작성 | ✅ 완료 | |
| S161-6: 검증 | ✅ 완료 | 코드 변경 없음 |

## 변경 내용

### docs/ 정리

**이동 (docs/ → docs/archive/phase6-and-old-optimization/)**:
- `PHASE6_SUMMARY.md` (Phase 6 종합, 2026-01-27 완료)
- `PHASE6_IMPLEMENTATION.md`
- `PHASE6_BENCHMARKS.md`
- `PHASE6_SIMD_SUMMARY.md`
- `phase6/` (디렉토리)
- `optimization/` (Sprint 58 분석)

**유지 (현재도 유효)**:
- `SIMD_OPTIMIZATION.md` — 사용 가이드
- `PERFORMANCE_BASELINES.md` — 현재 baseline
- `benchmarks/` — 최신 벤치마크

### docs/ 트리 size

| 항목 | Before | After |
|------|--------|-------|
| docs/ 최상위 파일 | 52 | 46 |
| docs/archive/ 디렉토리 | 1 | 2 (sprint-reports + phase6-and-old-optimization) |

## CLI 상태 (Sprint 161 진단)

`mecab-ko-cli` 이미 잘 갖춰져 있음 (Sprint 161 추가 작업 불필요):

| 항목 | 상태 |
|------|------|
| Commands | 8개 (parse, dict, evaluate, sync, collect, collect-unknown, version, completions) |
| Output formats | 7개 (default, wakati, dump, pos, json, simple, csv) |
| User dict | 지원 (`-u, --user-dic`) |
| REPL mode | 지원 (`--repl`) |
| Batch processing | 지원 (`-i`, `-o`) |
| Help | 94줄 (한국어, 예제 포함) |

기능적으로 충분. 향후 lift 작업은 사용자 피드백 기반으로.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass)
- 5-gate sample.tsv: 영향 없음
- CLI build 정상

## 변경 파일

- `docs/archive/phase6-and-old-optimization/` (신규)
  - PHASE6_*.md (4) + phase6/ + optimization/ + README.md
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 162 후보

### 시나리오 B 계속

남은 B 옵션:
- B-1: CLI/API — 이미 충분
- B-2: 성능 최적화 — 진단 sprint 필요
- B-3: 언어 바인딩 강화

남은 docs 정리:
- ISSUE_BACKLOG.md (오래되었을 가능성)
- LessonLearn/ 디렉토리
- DIC-010-SUMMARY.md (오래된 dict 관련)

### 시나리오 A (NIKL Modu) 또는 C (CRF Retrain)

여전히 사용자 작업 / confirm 대기.
