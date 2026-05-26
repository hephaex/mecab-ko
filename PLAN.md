# PLAN — mecab-ko Sprint 165 (Track B Step 2: 학습 데이터 준비)

> 마지막 업데이트: 2026-05-27

## 완료: Sprint 164 — Track B Step 1: CRF 빌드 환경

### 결과

- legacy/ macOS arm64 빌드 성공
- `mecab-cost-train`, `mecab-dict-gen`, `mecab-dict-index` executable 생성
- 학습 데이터 형식 (`.tagged`) 파악
- Track B 4 sprint 계획 수립

## Sprint 165 — Track B Step 2

### 목표: 학습 데이터 준비

#### S165-B1: TSV → .tagged 변환 스크립트
- `tools/to_mecab_tagged.py` 작성
- Input: data/eval/klue_dp_val.tsv 형식 (surface/POS surface/POS ...)
- Output: `.tagged` 형식 (surface<TAB>feature\n EOS)
- entries.csv 형식 호환 (POS, semantic, ..., reading)

#### S165-B2: KLUE DP train 변환
- data/raw/klue/ 에서 train split 활용
- ~10K sentences 변환
- 변환 검증 (sample 확인)

#### S165-B3: UD train 변환
- UD Kaist train (~1.6K)
- UD GSD train (~5K)
- 동일 형식으로 변환

#### S165-B4: 학습 corpus 합본
- 통합 .tagged 파일 (KLUE + UD)
- 학습/평가 split (eval은 기존 5-gate dataset 그대로 사용)

#### S165-B5: 학습 input 디렉토리 준비
- seed/ 생성 (mecab-ko-dic 2.1.1 복사 + 학습용)
- feature.def, matrix.def, char.def 등 배치

## Track B 전체 계획

| Sprint | Step | 상태 |
|--------|------|------|
| 164 | Step 1: 빌드 환경 | ✅ 완료 |
| **165** | **Step 2: 학습 데이터** | **다음** |
| 166 | Step 3: 1차 학습 + 변환 | 대기 |
| 167 | Step 4: Rust 통합 + 검증 | 대기 |
| 168 (옵션) | Step 5: 파라미터 튜닝 | 대기 |

## 누적 진척 (Sprint 122 → 164)

| Metric | Baseline | 현재 |
|--------|---------|------|
| sample.tsv | 100%/99.9% | 100%/99.9% (보존) |
| KLUE morph practical | ~65.8% | 72.1% (+6.3pp) |
| KLUE surface canonical_lenient | ~89% | 95.6% (+6pp) |
| UD Kaist morph practical | — | 68.6% |
| UD GSD morph practical | — | 71.8% |

Track B 잠재 lift: **+1~5pp KLUE morph** (Sprint 167 후 측정).

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv hard rule)
- 새 dict는 별도 디렉토리로 격리 (점진적 교체)
