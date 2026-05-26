# PLAN — mecab-ko Sprint 166 (Track B Step 3: 1차 학습 + 변환)

> 마지막 업데이트: 2026-05-27

## 완료: Sprint 165 — Track B Step 2: 학습 데이터 준비

### 결과

- `tools/to_mecab_tagged.py` 작성 (CoNLL-U → .tagged)
- UD Kaist dev (2066) + UD GSD dev (950) → 통합 corpus
- `data/train/corpus_dev.tagged` 3016 sentences, ~73K morphemes
- seed dictdir = mecab-ko-dic 2.1.1 (변경 없음)

### 학습 데이터 제약

train split 부재. dev splits만 사용 (3016 sentences). 작지만 첫 시도로 충분.
KLUE val + UD test는 평가용 그대로 보존 (leakage 방지).

## Sprint 166 — Track B Step 3: 1차 학습 + Dict 변환

### 목표

mecab-cost-train으로 학습 → model.def → mecab-dict-gen → 새 binary dict.

### 작업

#### S166-B1: 학습 디렉토리 준비
- `data/training_run_1/` 생성
- seed dict (mecab-ko-dic 2.1.1) 복사 or 심볼릭 링크
- 학습 corpus 배치

#### S166-B2: mecab-cost-train 실행
```bash
cd legacy
DYLD_LIBRARY_PATH=src/.libs src/.libs/mecab-cost-train \
  -d ../data/training_run_1/seed \
  -p 4 -f 1 \
  ../data/train/corpus_dev.tagged \
  ../data/training_run_1/model.def
```
- 학습 시간 측정
- 결과 model.def 검증 (size, 형식)

#### S166-B3: mecab-dict-gen 실행
```bash
DYLD_LIBRARY_PATH=src/.libs src/.libs/mecab-dict-gen \
  -o ../data/training_run_1/new_dict \
  -m ../data/training_run_1/model.def \
  -d ../data/training_run_1/seed
```
- 새 matrix.def, sys.dic 생성

#### S166-B4: Sanity check
- 새 dict로 mecab CLI 실행 (단순 문장)
- 출력 형식 정상 확인

### 위험 & Mitigations

- **학습 시간**: 73K morphemes는 작음, 수 분 이내 예상
- **작은 corpus → 회귀 위험**: Sprint 167에서 5-gate 검증
- **메모리/디스크**: matrix.def ~10M lines 재생성 (수십 MB)
- **별도 디렉토리 격리**: 기존 dict 영향 없음

## Track B 전체 진척

| Sprint | Step | 상태 |
|--------|------|------|
| 164 | Step 1: 빌드 환경 | ✅ 완료 |
| 165 | Step 2: 학습 데이터 | ✅ 완료 |
| **166** | **Step 3: 학습 + 변환** | **다음** |
| 167 | Step 4: Rust 통합 + 5-gate 검증 | 대기 |
| 168 (옵션) | Step 5: 파라미터 튜닝 | 대기 |

## 검증 기준

- 학습 정상 종료
- 새 model.def + matrix.def + sys.dic 생성 확인
- `cargo test --workspace` 변경 없음 (코드 변경 없음, Sprint 167까지)
- 5-gate sample.tsv hard rule (Sprint 167 통합 시)
