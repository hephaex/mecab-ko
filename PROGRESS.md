# PROGRESS — mecab-ko Sprint 162 (안전 영역 소진 — 사용자 결정 필요)

> 마지막 업데이트: 2026-05-21

## Sprint 162 — 마지막 잔여 정리 + 영역 소진 선언

| Task | 상태 | 결과 |
|------|------|------|
| S162-1: NIKL Modu 다운로드 확인 | ⏸ 미다운로드 (3rd time) | 시나리오 B 계속 |
| S162-2: ISSUE_BACKLOG.md 검토 | ✅ 검토 | 840줄, 검토 필요 (단순 archive 안됨) → 유지 |
| S162-3: DIC-010-SUMMARY.md archive | ✅ 완료 | 완료된 작업 보고서 |
| S162-4: LessonLearn/ 검토 | ✅ 검토 | 18줄, 유지 |

## 변경 내용

- `docs/DIC-010-SUMMARY.md` → `docs/archive/` (완료 보고서)
- docs/ 최상위: 46 → 45 파일

## 핵심 결론: 안전 영역 완전 소진

Sprint 156~162 (7 sprint) 누적 안전 영역 작업:

| 영역 | Sprint | 효과 |
|------|--------|------|
| Surface normalization (ㄷ 불규칙) | 156 | +0.1pp surface |
| PRACTICAL 동치 (MAG/MAJ) | 157 | +0.2pp 3 silver |
| 명시 어구 정규화 | 158 | +0.05pp surface |
| NIKL Modu 인프라 | 159 F | 인프라 (다운로드 대기) |
| Sprint 보고서 archive | 160 | 23 파일 정리 |
| PHASE6/optimization archive | 161 | 6 파일 정리 |
| DIC-010 archive | **162** | 1 파일 정리 |

**누적 정확도 lift (안전 영역만): +0.35pp KLUE morph + +0.15pp surface + 28 파일 archive**.

## 다음 단계 — 사용자 결정 필수

### 옵션 1: NIKL Modu 다운로드 진척 보고
- 사용자가 kli.korean.go.kr 등록/다운로드 진행 중인가?
- 완료 시 Sprint 163 = 측정 + 분석 (시나리오 A)
- 보류면 옵션 2/3으로

### 옵션 2: Full CRF Retrain (Track B) — 비가역 confirm
- 3-5 sprint 장기 작업
- 학습 데이터 + mecab-cost-train
- 잠재 lift +1~5pp KLUE morph
- Sprint 136에서 인프라 조사 완료
- **사용자 confirm 필요**

### 옵션 3: 정확도 작업 종료 + 다른 영역 전환
- 정확도 lift는 영역 소진
- 다른 영역으로 전환:
  - 언어 바인딩 강화 (Python/WASM/Node)
  - 성능 최적화 (프로파일링 sprint)
  - 사용자 피드백 기반 기능 추가

### 옵션 4: 정확도 작업 종료 + 프로젝트 안정화
- 정확도 lift sprint 종료
- 유지보수 모드 (버그 픽스, 의존성 업데이트)
- 다음 메이저 작업 대기

## 현재 정확도 (Sprint 122 → 162 누적)

| Metric | Baseline | 현재 | Δ |
|--------|---------|------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new silver) |
| UD GSD morph practical | — | 71.8% | (new silver) |

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass)
- 5-gate sample.tsv: 영향 없음
- docs/ 정리 진행

## 변경 파일

- `docs/DIC-010-SUMMARY.md` → archive
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 163 — 사용자 결정 대기

자동 진행 불가능 상태. 다음 4 옵션 중 사용자 선택 필수.
