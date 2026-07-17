# mecab CRF Retrain 파이프라인 (macOS arm64)

> mecab-ko-dic 같은 mecab dictionary를 재학습하는 완전한 절차 + 실패 사례.

---

## 문제

mecab의 정확도를 향상시키기 위해 connection cost (matrix.def)를 재학습해야 할 때:
- 수동 cost 조정은 cascade 회귀 위험 (Sprint 138)
- Full CRF Retrain은 학습 데이터 quality가 절대적 (Sprint 167)

## 해결 방법 (4단계 파이프라인, 62.6초)

### 사전: legacy/ macOS arm64 빌드

```bash
cd legacy/
./configure --prefix=/tmp/mecab-build
make clean       # Linux .o 파일 충돌 제거 필수!
make -j4
```

산출물:
- `src/.libs/libmecab.dylib`
- `src/.libs/mecab-cost-train`
- `src/.libs/mecab-dict-gen`
- `src/.libs/mecab-dict-index`

### 1단계: Seed dict 빌드 (mecab-dict-index)

```bash
# seed/ 디렉토리 = mecab-ko-dic + 학습용 (entries.csv 제외)
mkdir -p data/training/seed
for f in data/mecab-ko-dic-2.1.1-20180720/*.csv; do
  [ "$(basename "$f")" = "entries.csv" ] && continue  # 통합 파일 제외 (LEFT-ID 충돌)
  ln -sf "$(realpath "$f")" "data/training/seed/$(basename "$f")"
done
for f in data/mecab-ko-dic-2.1.1-20180720/*.def data/mecab-ko-dic-2.1.1-20180720/dicrc; do
  ln -sf "$(realpath "$f")" "data/training/seed/$(basename "$f")"
done

# Binary 빌드
DYLD_LIBRARY_PATH=legacy/src/.libs legacy/src/.libs/mecab-dict-index \
  -d data/training/seed \
  -o data/training/seed_built \
  -c UTF-8 -f UTF-8
# (2.5초)

# seed_built에 def + dicrc + source CSV link 필요!
for f in data/training/seed/*.def data/training/seed/dicrc; do
  cp "$f" "data/training/seed_built/"
done
for f in data/training/seed/*.csv; do
  ln -sf "$(realpath "$f")" "data/training/seed_built/$(basename "$f")"
done
```

### 2단계: CRF 학습 (mecab-cost-train)

```bash
DYLD_LIBRARY_PATH=legacy/src/.libs legacy/src/.libs/mecab-cost-train \
  -d data/training/seed_built \
  -p 4 -f 1 \
  data/train/corpus.tagged \
  data/training/model.def
# (37.7초, 4 threads, 29 iterations)
```

학습 데이터 형식 (`.tagged`):
```
<surface>\t<POS>,<semantic1>,...,<reading>
...
EOS
<surface>\t...
```

### 3단계: 새 Dict 생성 (mecab-dict-gen)

```bash
DYLD_LIBRARY_PATH=legacy/src/.libs legacy/src/.libs/mecab-dict-gen \
  -d data/training/seed_built \
  -o data/training/new_dict \
  -m data/training/model.def
# (20.5초)
```

### 4단계: 새 Binary 빌드 (mecab-dict-index 재실행)

```bash
DYLD_LIBRARY_PATH=legacy/src/.libs legacy/src/.libs/mecab-dict-index \
  -d data/training/new_dict \
  -o data/training/new_dict_built \
  -c UTF-8 -f UTF-8
# (1.9초)
```

### Rust 통합 (mecab-ko-dict-builder)

```bash
rust/target/release/mecab-ko-dict-builder \
  -i data/training/new_dict \
  -o data/training/rust_dict \
  -e utf8 -c 0  # uncompressed (Tokenizer가 .zst 미인식 시)
```

평가:
```bash
MECAB_DIC_PATH=$(pwd)/data/training/rust_dict \
  cargo test --package mecab-ko-core --test accuracy_eval \
  -- test_accuracy_gate --ignored --nocapture
```

## 주의사항

### 1. 학습 데이터 features 절대적

❌ **POS only 학습 데이터**:
```
surface\tPOS,*,*,*,*,*,*,*,*
```
→ catastrophic regression (Sprint 167: -37.8pp sample.tsv).

✅ **Full features**:
```
surface\tPOS,semantic1,semantic2,...,reading
```
→ mecab feature.def의 UNIGRAM U01 (POS+semantic), BIGRAM B01 (좌POS,의미/우POS) 활용 가능.

### 2. F=0.999 학습 적합도는 false signal

학습 데이터 overfit일 수 있음. 항상 hold-out evaluation 필수.

### 3. 격리 메커니즘 — zero-cost rollback

- 새 dict는 별도 디렉토리
- MECAB_DIC_PATH 환경 변수로 활성/비활성
- 회귀 시 `unset MECAB_DIC_PATH` 만으로 즉시 원복

### 4. 자주 발생하는 문제

| 에러 | 해결 |
|------|------|
| "cannot open tokenizer" | seed에 binary 필요 → mecab-dict-index 먼저 |
| "cannot find LEFT-ID for 788,SC,..." | entries.csv 제외, 개별 *.csv만 사용 |
| "no such file: dicrc/feature.def" | def + dicrc를 seed_built에도 복사 |
| "no dictionary found" (dict-gen) | source CSV를 seed_built에도 link |
| "Trie file not found: sys.dic" | compression 0 사용 (`-c 0`), 또는 절대 경로 |
| "unknown file type" (ld) | `make clean` 후 재빌드 (다른 architecture 충돌) |

### 5. viterbi/CRF 변경은 cascade 회귀 위험 (mecab-ko 4회 확인)

- Sprint 138: matrix.def cost 수동 → -0.9pp
- Sprint 145: multi-syllable VV+ETM splitter → -1 sentence
- Sprint 155: dict cost=-5000 NNP → -0.2pp
- Sprint 167: CRF retrain (POS only) → -37.8pp

→ 항상 sample.tsv hard rule + 즉시 rollback 정책.

## 관련 문서

- `docs/research/accuracy/2026-05-27_sprint164_crf_build_env.md` (S164 빌드 환경)
- `docs/research/accuracy/2026-05-27_sprint167_crf_retrain_rollback.md` (S167 회귀 분석)
- `docs/research/accuracy/2026-05-27_sprint168_track_b_termination.md` (Track B 종료)
- `docs/research/accuracy/2026-05-27_sprint174_cycle_termination.md` (cycle 총결산)
- `docs/archive/sprint-reports/2026-05-19_sprint136_crf_retrain_infra.md` (인프라 조사 원본)
- `tools/to_mecab_tagged.py` (CoNLL-U → .tagged 변환)

---

*작성: 2026-05-27 (mecab-ko Sprint 174, Track B 종료 후)*
