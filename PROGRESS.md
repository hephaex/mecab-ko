# PROGRESS — mecab-ko Sprint 174 (Sprint Cycle 종료 + 유지보수 모드)

> 마지막 업데이트: 2026-05-27

## Sprint 174 — Cycle 종료 + 유지보수 모드 선언

| Task | 상태 | 결과 |
|------|------|------|
| S174-1: NIKL Modu 7번째 체크 | ⏸ 미다운로드 | cycle 종료 진행 |
| S174-2: Sprint cycle 총결산 작성 | ✅ 완료 | sprint174_cycle_termination.md |
| S174-3: 유지보수 모드 선언 | ✅ 완료 | sprint-run 정지 (트리거 조건 충족 시 재개) |

## 핵심: 유지보수 모드 진입

자동 진행 가능 영역 모두 소진. 사용자 결정 또는 외부 의존 작업 필요.

### sprint-run 재개 트리거

1. **NIKL Modu 다운로드 완료** — `./tools/nikl_modu_setup.sh <json>` 실행
2. **Sejong 코퍼스 입수** — Track B 재시도 (인프라 즉시 활용)
3. **사용자 명시 신규 영역** — 특정 기능/버그/바인딩

## Sprint Cycle 총결산 (Sprint 122 → 173)

### 정확도 누적 진척

| Metric | Baseline | 현재 | Δ |
|--------|---------|------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (보존) |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new silver) |
| UD GSD morph practical | — | 71.8% | (new silver) |

### 인프라 누적 진척

| 영역 | Δ |
|------|---|
| accuracy_eval.rs | 4963 → 2406 줄 (-51%) |
| WASM tests | 5 → 11 (+120%) |
| Docs archive | 28+ 파일 정리 |
| CI gate | sample.tsv → 5-gate |
| 성능 baseline | v0.7.2 5/9 benches 측정 |

### Sprint 분류 (52 sprints, ~1.5 개월)

| 유형 | 건수 |
|------|------|
| Lift sprints | 11 |
| Infrastructure | 12 |
| Rollback | 4 |
| 비이슈 확인 | 3 |
| 정리 | 8 |
| 진단 only | 6 |
| 외부 인프라 | 1 |
| Track B (실패) | 5 |
| 종합 정리 | 2 |

## 검증된 영역 매트릭스 (확정)

### ✅ 안전 영역
- PRACTICAL 동치 그룹 (NNB/NNG, VA/VV, VV/XSV, MAG/MAJ)
- Surface normalization (하았/이습니다/르/ㄷ 불규칙 + 명시 어구)
- Splitter rule (제한적, mecab dict 미처리만)
- Silver dataset 통합

### ❌ 위험 영역 (회귀 4회)
- matrix.def cost (S138)
- multi-syllable VV+ETM (S145)
- dict cost=-5000 (S155)
- CRF retrain (POS only) (S167)

→ viterbi/CRF 직접 변경 = cascade 회귀 매우 큼.

### ⏸ 비이슈 (mecab dict 처리)
- ETM+ETM, XSA+ETM, EP+ETM 등 833건 빈도 → 실효 24건만 (2.9%)

## 메타 학습

1. mecab dict의 압도적 강력함 (decomposition fallback)
2. 빈도 ≠ 실효 lift (변환 후 측정 필수)
3. 3 silver 일관 lift = 진짜 효과
4. Rollback 신속화 (Sprint 138 정책)
5. 격리의 가치 (별도 dict + 환경 변수)
6. 진단 데이터 재활용 (다른 접근 가능)
7. 규칙 5 자동 트랙 선택 (전문가도 측정으로 검증)
8. 작은 corpus + sparse features = overfit 함정

## Track B 자산 보존

Sejong 코퍼스 입수 시 즉시 재활용:
- legacy/ macOS arm64 빌드 (S164)
- tools/to_mecab_tagged.py (S165)
- 4단계 학습 파이프라인 62.6초 (S166)
- 격리 메커니즘 (S167)

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 411 pass (변경 없음)
- 5-gate sample.tsv: 100.0%/99.9% (baseline 유지)
- 성능 회귀 없음 (v0.3.0 → v0.7.2)

## 변경 파일

- `docs/research/accuracy/2026-05-27_sprint174_cycle_termination.md` (신규)
- `PLAN.md`, `PROGRESS.md` 유지보수 모드 갱신

## Sprint 175+ (트리거 조건 발생 시 재개)

자동 sprint-run 정지 상태. 다음 명시 작업 발생 시 즉시 재개 가능.
