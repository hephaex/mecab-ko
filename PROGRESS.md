# PROGRESS — mecab-ko Sprint 168 (Track B 공식 종료 + Sprint Cycle 종합)

> 마지막 업데이트: 2026-05-27

## Sprint 168 — Track B 공식 종료

| Task | 상태 | 결과 |
|------|------|------|
| S168-1: 사용자 confirm 요청 | ✅ 완료 | Option D 선택 |
| S168-2: Track B 종료 선언 | ✅ 완료 | docs 작성 |
| S168-3: Sprint cycle 종합 정리 | ✅ 완료 | sprint168_track_b_termination.md |
| S168-4: 향후 옵션 명시 | ✅ 완료 | NIKL Modu / Sejong / 다른 영역 |

## 핵심 결과

### Track B 종료 결정

사용자 confirm: **Option D — Track B 종료 (권고 채택)**.

근본 원인: 학습 데이터 features 부족 (POS only, 나머지 8 fields `*`).
추가 sprint로 해결 어려움:
- Option A (Self-training): self-amplification
- Option B (corpus 확장): leakage
- Option C (Sejong): 라이선스 + 자동화 불가

### Track B 자산 보존

미완성이지만 재사용 가능:
- legacy/ macOS arm64 빌드 (Sprint 164)
- tools/to_mecab_tagged.py (Sprint 165)
- 4단계 학습 파이프라인 (Sprint 166, 62.6초)
- 격리 메커니즘 (Sprint 167)

Sejong 코퍼스 입수 시 즉시 재활용 가능.

## 정확도 Sprint Cycle 종합 (Sprint 122 → 167)

### 누적 진척

| Metric | Sprint 122 | Sprint 167 | Δ |
|--------|-----------|-----------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (baseline 보존) |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new silver) |
| UD GSD morph practical | — | 71.8% | (new silver) |

### Sprint 분류 (46 sprints, 약 1개월)

| 유형 | 건수 | 효과 |
|------|------|------|
| Lift sprints | 11 | +6.3pp 누적 |
| Infrastructure | 9 | 5-gate CI, 3 silver |
| Rollback (viterbi/CRF) | 4 | Sprint 138/145/155/167 |
| 비이슈 확인 | 3 | mecab dict 처리 |
| 정리 | 5 | -2557 줄, 28 docs archive |

### 영역 소진 매트릭스 (확정)

| 영역 | 결과 |
|------|------|
| Splitter rule | ❌ Sprint 154 소진 |
| Dict cost 확장 | ❌ Sprint 155 회귀 |
| CRF matrix | ❌ Sprint 138 회귀 |
| **CRF Full Retrain** | ❌ **Sprint 167 회귀 → Track B 종료** |
| Surface normalization | ✅ 누적 +6pp |
| PRACTICAL 동치 | ✅ 누적 +6.3pp |
| Silver dataset | ✅ 5-gate 완성 |

## 메타 학습

### viterbi/CRF 변경 = 위험 (4번째 확인)

| Sprint | 시도 | 회귀 |
|--------|------|------|
| 138 | matrix.def cost | -0.9pp |
| 145 D | multi-syllable VV+ETM | -1 sentence |
| 155 A | dict cost=-5000 NNP | -0.2pp |
| **167** | **CRF retrain (POS only)** | **-37.8pp** |

→ viterbi/CRF mechanism 직접 변경은 cascade 회귀 위험 매우 큼.

### 안전 영역 (검증됨)

- normalize_endings (평가 함수)
- TAG_EQUIVALENCE_GROUPS_PRACTICAL (메트릭 동치)
- 새 silver dataset 추가 (coverage 확장)

### mecab dict의 강력함

decomposition fallback이 ㅂ/ㄹ/ㅎ 불규칙까지 처리 (Sprint 148/153/154 확인).

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 411 pass (변경 없음)
- 5-gate sample.tsv: 100.0%/99.9% (baseline 유지)
- Track B 학습 산출물 보관 (재사용 가능)

## 변경 파일

- `docs/research/accuracy/2026-05-27_sprint168_track_b_termination.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## 다음 단계

### 잔여 작업 (사용자 액션 대기)

1. **NIKL Modu** (Sprint 159~163 인프라 완료)
   - https://kli.korean.go.kr 학술 등록 + 다운로드
   - 완료 시 `./tools/nikl_modu_setup.sh <json>` 한 줄 실행

2. **Sejong 코퍼스** (Track B 재시도 시)
   - 국립국어원 또는 KAIST
   - 입수 시 Sprint 164~167 파이프라인 즉시 재활용

### 정확도 외 영역 (사용자 결정 시)

- 성능 최적화
- 언어 바인딩 강화
- 사용자 기능
- 유지보수 모드

## 정확도 lift sprint cycle 마무리

Sprint 122부터 약 1개월간의 정확도 lift sprint cycle 공식 종료.
- 안전 영역: 완전 소진
- 위험 영역: Track B 1차 시도 → 종료
- 외부 입수 필요: 별도 사용자 작업

다음 sprint-run은 새 방향 (사용자 결정 또는 다른 영역).
