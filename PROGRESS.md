# PROGRESS — mecab-ko Sprint 145 (결합 POS 분석 + multi-syllable VV+ETM 실험 rollback)

> 마지막 업데이트: 2026-05-20

## Sprint 145 — PUD 보류 → D (결합 토큰 패턴 확장)

| Task | 상태 | 비고 |
|------|------|------|
| S145-E1: UD PUD 다운로드 + 형식 확인 | ✅ 완료 (PUD 보류) | Penn-Treebank XPOS + lemma 52% 누락 → 변환 부적합 |
| S145-D1: 결합 POS 빈도 분석 | ✅ 완료 | `test_compound_pos_frequency_analysis` (153 패턴, 13% tokens) |
| S145-D2: multi-syllable VV+ETM 실험 | ✅ 완료 (rollback) | sample.tsv -1 sentence 회귀 → rollback |
| S145-D3: 5-gate 검증 | ✅ 완료 | rollback 후 baseline 복구 |

## 핵심 발견

### PUD 보류 사유

UD Korean-PUD 다운로드 후 형식 분석:
- XPOS scheme: Penn-Treebank (NN/JJ/RB/CD/DT/VC/CM/...) — Sejong도 KAIST도 아님
- lemma `_` 비율: 52% (8,666/16,584 tokens) — morpheme 분해 정보 부족

→ PUD 통합 보류, D 후보로 전환.

### 결합 POS 빈도 분석 결과

- 3 silver 통합 측정 (KLUE val + UD Kaist test + UD GSD test)
- **104,885 tokens 중 13,611 (13.0%)이 결합 POS**
- **153 unique compound patterns**

상위 패턴 + splitter.rs 처리 여부:

| Pattern | Count | 처리 상태 |
|---------|-------|----------|
| VV+ETM | 5,376 | 1-syllable만 |
| VV+EC | 1,728 | 일부 |
| XSV+EC | 751 | 미처리 |
| VCP+ETM | 613 | **Sprint 141** |
| VA+ETM | 542 | 1-syllable만 |
| VV+EP | 542 | 미처리 |
| VV+EF | 513 | 일부 |
| XSV+EP | 413 | 미처리 |
| VCP+EC | 344 | **Sprint 141** |
| VCP+EF | 279 | **Sprint 132+** |

### multi-syllable VV+ETM 실험 → rollback

기존: 1-syllable만 (`surface.chars().count() == 1`)
확장: multi-syllable (예: "넘긴" → "넘기 + ㄴ")

**측정**:
- sample.tsv Sentence: 99.9% → 99.8% (-1 문장) ❌
- KLUE eo strict: 4642 → 4647 (+5건)
- UD Kaist eo strict: 3987 → 3993 (+6건)
- UD GSD eo strict: 2601 → 2610 (+9건)
- UD GSD morph: 67.4% → 67.3% (-0.1pp) ⚠️

**Pros**: 모든 silver eojeol +5~9 개선
**Cons**: sample.tsv 1 sentence 회귀 + UD GSD morph -0.1pp

**결정**: Rollback (Sprint 138 결론 적용 — baseline 회귀 금지). 1-syllable만 유지.

## 측정값 (변경 없음 — rollback)

| 메트릭 | Sprint 144 | Sprint 145 |
|--------|-----------|-----------|
| 모든 5-gate 메트릭 | 동일 | 동일 |

## 핵심 학습 포인트

### 1. PUD는 적합하지 않은 silver source

같은 UD framework이지만 PUD는 Penn-Treebank style + lemma 절반 누락. UD 전체가 동일 형식 가정 위험.

### 2. multi-syllable 확장은 1-syllable보다 위험

명시 surface 목록 패턴이 더 안전 (Sprint 141 VCP+EC처럼). multi-syllable은 false positive 위험.

### 3. 결합 POS 13% 비율 — 분리 가치 큼

전체 13%가 결합. 안전한 패턴 식별 → 누적 lift 가능. 단 각 패턴마다 false positive 검증 필수.

### 4. sample.tsv 1 sentence가 baseline 정의

1 sentence 회귀도 baseline 100%/99.9% 깸. 다른 silver +5~9 lift가 있어도 sample.tsv 우선.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: all pass / 0 fail
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `test_full_accuracy_evaluation`: PASS (rollback 후 sample.tsv 100.0%/99.9% 복구)

## 변경 파일

- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: `test_compound_pos_frequency_analysis` 추가 (~80줄)
- `rust/crates/mecab-ko-core/src/sejong/splitter.rs`: 주석 갱신 (multi-syllable 실험 + rollback 기록)
- `docs/research/accuracy/2026-05-20_sprint145_compound_pos_analysis.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 146 후보

- A: 명시 surface 목록 기반 안전 패턴 분리 (NP+JX "그는"/"이는" 등)
- B [메인]: Full CRF Retrain (Track E)
- C: NIKL Modu 수동 다운로드
