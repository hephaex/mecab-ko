# Sprint 167 — Track B Step 4: Rust 통합 + Catastrophic 회귀 → Rollback

> **결과**: 새 CRF로 학습된 dict 사용 시 sample.tsv Token 100% → **62.2%** (-37.8pp) catastrophic 회귀. Sprint 138 정책에 따라 즉시 rollback. 원인: 학습 데이터 features 부족 (POS만 정확, semantic/reading은 `*`).

---

## 1. Track B Step 4 실행

### 1.1 Rust dict 변환

```bash
rust/target/release/mecab-ko-dict-builder \
  -i data/training_run_1/new_dict \
  -o data/training_run_1/rust_dict \
  -e utf8 -c 0
```

- 시간: 25.66초
- 산출물: sys.dic (uncompressed), matrix.bin, entries.bin, entries.csv, unk.bin
- 816,286 entries
- compression 0 (uncompressed) 사용 — Tokenizer가 sys.dic.zst 인식 못 함

### 1.2 5-gate sample.tsv 측정

```bash
cd rust
MECAB_DIC_PATH=/Users/mare/Simon/mecab-ko/data/training_run_1/rust_dict \
  cargo test --package mecab-ko-core --test accuracy_eval \
  -- test_accuracy_gate --nocapture --ignored
```

**결과**: Token accuracy **62.2%** (기존 100%), F1=0.574 (기존 1.000)

→ **Catastrophic 회귀 -37.8pp.** sample.tsv hard rule 위반.

---

## 2. 원인 분석

### 2.1 학습 데이터 features 부족

`tools/to_mecab_tagged.py`가 생성한 .tagged 형식:

```
surface\tPOS,*,*,*,*,*,*,*,*
```

- POS만 정확 (UD에서 추출)
- semantic1~5, reading 등 나머지 8 fields는 `*` (wildcard)

mecab-ko-dic의 `feature.def`는 다음 features를 활용:
```
UNIGRAM U01:%F[0],%F?[1]     # POS + semantic
UNIGRAM R00:%F[3]            # 종성 여부
UNIGRAM R01:%F[0],%F[3]      # POS + 종성
BIGRAM B01:%L[0],%L?[1]/%R[0]  # 좌POS,의미/우POS
```

대부분 features가 `*`이므로:
- U01, R00, R01 등 features 인스턴스화 시 `*` value → 학습 정보 손실
- BIGRAM features도 의미 features 활용 못함
- CRF가 effective patterns 학습 못함

### 2.2 작은 corpus + sparse features → over-specialization

- 3016 sentences × 8 features 무력화 → 실질 학습 data 매우 sparse
- F=0.99977은 학습 데이터에만 적합 (overfit)
- 실제 sample.tsv (general Korean) 평가에서 catastrophic regression

### 2.3 Sanity check 신호

Sprint 166의 sanity check:
```
안녕하세요 → 안녕하/VA + 세요/EP+EC+VCP  ← 비정상 분석
```

기존 mecab은:
```
안녕하세요 → 안녕/NNG + 하/XSV + 세요/EP+EF
```

이미 학습된 model이 일반 한국어 패턴을 잘못 분석. 이는 catastrophic 회귀의 전조였음.

---

## 3. Rollback

학습된 dict 사용 중단. 환경 변수만 unset하면 기존 dict로 복귀:

```bash
unset MECAB_DIC_PATH  # 기존 mecab-ko-dic-2.1.1-20180720 사용
```

코드 변경 없음. 별도 디렉토리 격리 효과 — 기존 dict는 그대로 보존.

Sample.tsv 5-gate 검증 (rollback 후):

```bash
cd rust
cargo test --package mecab-ko-core --test accuracy_eval -- test_accuracy_gate --nocapture --ignored
# → Token 100.0%, Sentence 99.9% (baseline 복원)
```

---

## 4. 핵심 학습 포인트

### 4.1 CRF features의 중요성

mecab-cost-train의 효과는 학습 데이터의 features 풍부도에 비례. POS만으로는 부족.

| 학습 데이터 features | 효과 |
|---------------------|------|
| POS only (Sprint 167) | Catastrophic regression |
| POS + semantic + reading (full) | 학습 효과 (예상) |

### 4.2 Sprint 145/155/167 viterbi cascade 패턴 재확인

| Sprint | 시도 | 결과 |
|--------|------|------|
| 138 | matrix.def 수동 cost | -0.9pp sample.tsv |
| 145 D | multi-syllable VV+ETM | -1 sentence |
| 155 A | dict cost=-5000 NNP | -0.2pp KLUE |
| **167** | **CRF retrain** | **-37.8pp sample.tsv** |

CRF retrain은 가장 큰 회귀 — 학습 데이터 quality가 절대적.

### 4.3 작은 corpus + sparse features의 함정

- F=0.99977 학습 적합도 = false signal (overfit)
- 실제 평가 결과만이 의미 있음
- "F1 학습 결과" ≠ "generalization 성능"

### 4.4 격리의 가치

별도 디렉토리 + 환경 변수 = 코드 변경 없이 격리. 회귀 시 즉시 rollback 가능. Sprint 138/155와 동일 패턴.

---

## 5. Track B 1차 시도 결론

### 실패 원인 (단일)

학습 데이터 features 부족 (POS only). UD CoNLL-U에서 추출 가능한 정보는 surface + lemma + POS만. mecab의 풍부한 features (semantic, reading) 활용 불가.

### Track B Step 5 (Sprint 168) 옵션

#### Option A: Self-training (학습 데이터 features 보강)
- 기존 mecab-ko-dic으로 KLUE/UD train tokenize → 그 features를 학습 데이터로
- 단, 기존 mecab의 오류를 학습할 위험 (self-amplification)
- 효과: 제한적 (기존 mecab 이상 못 함)

#### Option B: 학습 corpus 확장
- KLUE val (1995) + UD test (1638+971) 모두 학습에 사용
- 단, **evaluation leakage** 문제
- 별도 hold-out test set 필요

#### Option C: Sejong 코퍼스 입수
- 원본 mecab-ko-dic 학습 코퍼스 (Sejong tagged corpus)
- 라이선스 제약 (학술 전용 또는 비공개)
- 입수 가능 시 가장 정확한 학습

#### Option D: Track B 종료
- 학습 데이터 features 부족이 근본 원인
- 추가 sprint로 해결 어려움
- Track B 종료 + 정확도 sprint 종료 또는 다른 방향

---

## 6. 변경 파일

- (코드 변경 없음 — Tokenizer/dict 코드 그대로)
- `data/training_run_1/`: 학습 산출물 (gitignore, 보관)
- `docs/research/accuracy/2026-05-27_sprint167_crf_retrain_rollback.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

## 7. 5-gate 측정 (rollback 후 확인)

| Metric | 새 dict (실패) | 원래 dict (rollback) | 회귀? |
|--------|--------------|-------------------|------|
| sample.tsv Token | **62.2%** | 100.0% | ✓ rollback OK |
| sample.tsv Sentence | (측정 불요) | 99.9% | ✓ |
| KLUE, UD | (측정 불요) | 기존 baseline | ✓ |

기존 dict는 그대로 보존 (격리 효과). 환경 변수만 unset.

---

*작성: 2026-05-27 (Sprint 167, Track B Step 4 실패)*
