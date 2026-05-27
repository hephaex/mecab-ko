# PROGRESS — mecab-ko Sprint 167 (Track B Step 4: 회귀 → Rollback)

> 마지막 업데이트: 2026-05-27

## Sprint 167 — Track B Step 4: Rust 통합 + 5-gate 검증

| Task | 상태 | 결과 |
|------|------|------|
| S167-B1: Rust dict-builder 실행 | ✅ 완료 | 25.66초, 816K entries |
| S167-B2: uncompressed 옵션 (-c 0) | ✅ 완료 | sys.dic + matrix.bin (uncompressed) |
| S167-B3: sample.tsv gate 측정 | ❌ **실패** | **Token 62.2% (-37.8pp catastrophic)** |
| S167-B4: 즉시 rollback (Sprint 138 정책) | ✅ 완료 | 환경 변수 unset → baseline 복원 |
| S167-B5: 원인 분석 | ✅ 완료 | 학습 데이터 features 부족 |
| S167-B6: 연구 문서 작성 | ✅ 완료 | sprint167_crf_retrain_rollback.md |

## 핵심 결과

### Catastrophic 회귀

| Metric | Baseline | 새 dict | Δ |
|--------|---------|---------|---|
| **sample.tsv Token** | 100.0% | **62.2%** | **-37.8pp** |
| **F1 Score** | 1.000 | 0.574 | -0.426 |

Sprint 138 hard rule (sample.tsv 무회귀) 위반 → 즉시 rollback.

### Rollback 확인

- 환경 변수 unset → 기존 mecab-ko-dic-2.1.1-20180720 사용
- sample.tsv: 100.0%/99.9% (baseline 복원)
- 코드 변경 없음 (격리 효과)

### 원인: 학습 데이터 features 부족

```
.tagged 형식: surface\tPOS,*,*,*,*,*,*,*,*
```

POS만 정확, 나머지 8 fields는 `*` wildcard.

mecab feature.def는:
```
UNIGRAM U01:%F[0],%F?[1]     # POS + semantic
BIGRAM B01:%L[0],%L?[1]/%R[0]
```

대부분 features `*` → CRF가 effective patterns 학습 못함 → overfit (F=0.99977 학습 데이터에만) → general regression.

### Sanity check 전조

Sprint 166의 sanity check가 이미 비정상 분석 보임:
```
안녕하세요 → 안녕하/VA + 세요/EP+EC+VCP  ← 비정상
```
기존: 안녕/NNG + 하/XSV + 세요/EP+EF

이는 catastrophic 회귀 전조였음.

## viterbi/CRF 변경 회귀 패턴 (4번째 사례)

| Sprint | 시도 | 회귀 크기 |
|--------|------|----------|
| 138 | matrix.def 수동 cost | -0.9pp sample.tsv |
| 145 D | multi-syllable VV+ETM | -1 sentence |
| 155 A | dict cost=-5000 NNP | -0.2pp KLUE |
| **167** | **CRF retrain** | **-37.8pp sample.tsv** |

CRF retrain은 가장 큰 회귀. 학습 데이터 quality가 절대적.

## Track B Step 5 (Sprint 168) 옵션

### Option A: Self-training (features 보강)
- 기존 mecab으로 KLUE/UD train tokenize → features 추출
- 단, self-amplification 위험

### Option B: 학습 corpus 확장 (leakage 문제)
- KLUE val + UD test 학습에 포함
- Evaluation leakage → 신뢰도 낮음

### Option C: Sejong 코퍼스 입수
- 원본 mecab-ko-dic 학습 데이터
- 라이선스 제약 (학술 전용)
- 입수 가능 시 가장 정확

### Option D: Track B 종료
- 학습 데이터 features 부족이 근본 원인
- 추가 sprint로 해결 어려움

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass, 코드 변경 0)
- 5-gate sample.tsv: rollback 후 100.0%/99.9% **baseline 복원**
- 새 dict (실패본)은 `data/training_run_1/` 보관 (학습 메커니즘 학습용)

## 변경 파일

- (코드 변경 없음 — Tokenizer 코드 그대로)
- `data/training_run_1/rust_dict/`: 실패한 학습 산출물 (보관)
- `docs/research/accuracy/2026-05-27_sprint167_crf_retrain_rollback.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Track B 전체 진척

| Sprint | Step | 상태 |
|--------|------|------|
| 164 | Step 1: 빌드 환경 | ✅ |
| 165 | Step 2: 학습 데이터 | ✅ |
| 166 | Step 3: 학습 + dict 변환 | ✅ |
| **167** | **Step 4: Rust 통합 + 검증** | **❌ 회귀 → rollback** |
| 168 (옵션) | Step 5: 튜닝 / corpus 확장 / 종료 | **사용자 결정 필요** |

## 사용자 결정 필요

Track B 1차 시도 실패. Sprint 168 옵션:
- **A**: Self-training (단, 효과 제한적)
- **B**: 학습 corpus 확장 (단, leakage)
- **C**: Sejong 코퍼스 입수 (단, 라이선스)
- **D**: Track B 종료

비가역 대규모 작업 → 사용자 confirm 필요.
