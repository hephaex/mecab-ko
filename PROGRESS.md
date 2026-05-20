# PROGRESS — mecab-ko Sprint 154 (빈도 영역 소진 선언)

> 마지막 업데이트: 2026-05-21

## Sprint 154 — 4 후보 통합 진단 (빈도 영역 소진)

| Task | 상태 | 결과 |
|------|------|------|
| S154-1: 통합 진단 테스트 작성 | ✅ 완료 | `test_sprint154_unified_diagnosis` |
| S154-2: 4개 패턴 동시 측정 | ✅ 완료 | EP+ETM/XSV+ETM/VX+EP/XSA+EP |
| S154-3: 결과 분석 | ✅ 완료 | 218/218 처리됨 (100%) |
| S154-4: SPLIT_DIFFERENT 측정 | ✅ 완료 | KLUE 2237/22404 ≈ 10% |
| S154-5: 작업 영역 소진 선언 | ✅ 완료 | 빈도 기반 splitter rule 종료 |
| S154-6: 연구 문서 작성 | ✅ 완료 | sprint154_frequency_exhausted.md |

## 핵심 결과

### 4 후보 모두 비이슈

| 패턴 | Raw | Split OK | 미처리 |
|------|-----|----------|--------|
| EP+ETM | 86 | 86 (100%) | 0 |
| XSV+ETM | 72 | 72 (100%) | 0 |
| VX+EP | 25 | 25 (100%) | 0 |
| XSA+EP | 35 | 35 (100%) | 0 |
| **합계** | **218** | **218** | **0** |

mecab dict decomposition features가 ㅂ/ㄹ/ㅎ 불규칙까지 모두 처리.

### 누적 빈도 진단 결과

| Sprint | 패턴 | 빈도 | 미처리 | 결론 |
|--------|------|------|--------|------|
| 148 D | ETM+ETM | 33 | 0 | 비이슈 |
| 150 A | VA+ETM | 542 | 24 | lift (+0.4pp) |
| 153 E | XSA+ETM | 38 | 0 | 비이슈 |
| **154** | **4개 (218)** | **218** | **0** | **비이슈** |

총 빈도 833건 중 실효 작업 24건 (Sprint 150 A) — **2.9%만 의미 있음**.

### 진짜 mismatch 위치 식별

`test_klue_dp_split_diff_connection_pairs`:
- SPLIT_DIFFERENT eojeols: 2237 / 22404 (~10%)
- Top patterns은 CRF connection cost 이슈 (NNG↔BOS/EOS, EF↔SF 등)
- → splitter rule 영역이 아님

### 빈도 기반 작업 영역 소진 선언

- splitter rule은 mecab dict decomposition으로 대체됨
- 남은 mismatch는 CRF/dict cost / 새 도메인 영역
- 다음 단계 전환 신호

## 핵심 학습

### 1. mecab dict의 위력
`SejongConverter::convert_token` decomposition fallback이 ending_rules보다 먼저 시도되어 사전 features를 활용. ㅂ/ㄹ/ㅎ 불규칙 stem 복원까지 dict이 처리.

### 2. 빈도 분석의 진정한 가치
빈도 = 잠재 영역 식별. **반드시 splitter+converter 변환 후 진단**으로 실제 미처리 측정 필요.

### 3. 통합 진단의 효율성
이전: 후보당 sprint 1회 → 현재: 4개를 1 sprint로 통합 진단 → 영역 빠르게 소진 확인.

### 4. 작업 영역 전환 시점
빈도 후보 5번 시도 후 1번만 lift (20%) → 영역 소진 신호. 새 방향 필요.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (테스트만 추가)
- 5-gate sample.tsv: 영향 없음 (코드 변경 없음)
- 통합 진단 테스트: PASS

## 변경 파일

- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: 통합 진단 테스트 추가
- `docs/research/accuracy/2026-05-21_sprint154_frequency_exhausted.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 155 방향 (전문가 리뷰 필요)

빈도 기반 영역 소진 → 새 방향:
- A: dict 확장 (Sprint 130/132 재방문)
- B: test_klue_dp_real_error_analysis 활용 (오류 기반 작업)
- C: surface normalization 확장 (Sprint 134 패턴)
- D: CRF Track A 재시도 (좁은 범위)
- E [대규모, confirm]: Full CRF Retrain
- F [확장, confirm]: NIKL Modu 도입
