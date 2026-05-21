# PLAN — mecab-ko Sprint 164 (사용자 NIKL Modu 다운로드 대기)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 163 — NIKL Modu 인프라 보강

### Sprint 163 결과

- `tools/nikl_modu_setup.sh` — 원샷 변환+평가 스크립트
- `docs/eval/nikl_modu_setup.md` — 트러블슈팅 + 원샷 사용법 추가

### NIKL Modu 인프라 현황

| 구성요소 | 상태 |
|----------|------|
| 변환 스크립트 | ✅ `tools/convert_nikl_modu.py` (Sprint 159 F) |
| Accuracy test | ✅ `test_nikl_modu_dual_metric` (skip 패턴, Sprint 159 F) |
| 다운로드 가이드 | ✅ `docs/eval/nikl_modu_setup.md` (Sprint 159 F + 163 보강) |
| 원샷 스크립트 | ✅ `tools/nikl_modu_setup.sh` (Sprint 163) |
| `.gitignore` 보호 | ✅ Sprint 159 F |
| **사용자 다운로드** | ⏸ 대기 중 |

## Sprint 164 시나리오

### A: 다운로드 완료 시 (사용자 알림)

```bash
# 사용자가 실행:
./tools/nikl_modu_setup.sh ~/Korpora/NIKL_MP/NXMP*.json
```

→ Sprint 164 자동 진행:
- 결과 PROGRESS.md 기록
- POS mismatch 분석
- 추가 동치/normalize 후보 발굴
- 5-gate에 6번째 gate 추가 검토

### B: 다운로드 보류 — 다른 영역 결정

NIKL Modu 다운로드가 어려운 상황이면:
1. **Full CRF Retrain (Track B)** — 비가역 confirm 필요
2. **언어 바인딩 강화** — Python/WASM/Node
3. **성능 최적화** — 프로파일링 sprint
4. **유지보수 모드** — 정확도 sprint 종료

## 누적 진척 (Sprint 122 → 163)

| Metric | Baseline | 현재 |
|--------|---------|------|
| sample.tsv | 100%/99.9% | 100%/99.9% (보존) |
| KLUE morph practical | ~65.8% | **72.1%** (+6.3pp) |
| KLUE surface canonical_lenient | ~89% | **95.6%** (+6pp) |
| UD Kaist morph practical | — | 68.6% |
| UD GSD morph practical | — | 71.8% |

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
