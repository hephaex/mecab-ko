# PROGRESS — mecab-ko Sprint 166 (Track B Step 3: 학습 + Dict 변환)

> 마지막 업데이트: 2026-05-27

## Sprint 166 — Track B Step 3: 1차 학습 + 새 Dict 생성

| Task | 상태 | 결과 |
|------|------|------|
| S166-B1: 학습 디렉토리 준비 | ✅ 완료 | data/training_run_1/seed (CSV 링크) |
| S166-B2: mecab-dict-index seed 빌드 | ✅ 완료 | seed_built/ (sys.dic, matrix.bin), 2.5초 |
| S166-B3: mecab-cost-train 학습 실행 | ✅ 완료 | **29 iter, 37.7초, F=0.99977** |
| S166-B4: mecab-dict-gen 새 dict 생성 | ✅ 완료 | new_dict/ (CSV + matrix.def), 20.5초 |
| S166-B5: 새 dict binary 빌드 | ✅ 완료 | new_dict_built/ (sys.dic 81MB), 1.9초 |
| S166-B6: Sanity check (mecab CLI) | ✅ 완료 | 정상 출력 |

## 핵심 성과

### 전체 파이프라인 시간

| 단계 | 시간 | 산출물 |
|------|------|--------|
| seed binary 빌드 (dict-index) | 2.5초 | sys.dic + matrix.bin |
| CRF 학습 (cost-train) | **37.7초** | model.def (11MB, 279,762 lines) |
| 새 dict 생성 (dict-gen) | 20.5초 | new_dict/ (CSV + matrix.def) |
| 새 dict binary 빌드 (dict-index) | 1.9초 | sys.dic (81MB) + matrix.bin (21MB) |
| **합계** | **62.6초** | 완전한 새 mecab dict |

학습은 빠름. 작은 corpus (3016 sentences) 효과.

### 학습 결과 (29 iterations)

```
iter=0  err=0.32X F=0.??? target=large
...
iter=15 err=0.00265 F=0.99976 target=266.34
...
iter=29 err=0.00232 F=0.99977 target=257.19
```

학습 데이터에 거의 완벽 적합 (F=0.99977). 실제 평가 (other datasets) 결과는 Sprint 167에서.

### Sanity check 결과

```bash
$ echo "안녕하세요 반갑습니다" | mecab -d new_dict_built
안녕하   VA,*,F,안녕하,*,*,*,*
세요    EP+EC+VCP,*,F,세요,Inflect,EP,VCP,시/EP/*+어요/EC/*+이/VCP/*
반갑    VA,*,T,반갑,*,*,*,*
습니다  EC,*,F,습니다,*,*,*,*
EOS
```

기존 mecab-ko-dic과 다른 분석 (예: "안녕"이 안녕하/VA로). 작은 corpus 학습 effect.

### 문제 해결

| 문제 | 해결 |
|------|------|
| "cannot open tokenizer" | seed에 binary 필요 (sys.dic) → mecab-dict-index로 빌드 |
| entries.csv "cannot find LEFT-ID" | entries.csv는 통합 파일, 개별 *.csv만 사용 |
| "no dictrc" | dicrc + .def 파일들을 seed_built에 복사 필요 |
| "no dictionary found" (dict-gen) | seed_built에 source CSV들도 link 필요 |

mecab 학습 파이프라인은 다음을 같은 디렉토리에 요구:
- binary: sys.dic, matrix.bin, char.bin, unk.dic
- source: *.csv files
- config: *.def files, dicrc

## 변경 파일

- `data/training_run_1/` (gitignore 대상, 큰 binary 산출물):
  - seed/ (CSV + def 링크)
  - seed_built/ (sys.dic + binary + def + CSV)
  - model.def (11MB CRF 학습 결과)
  - new_dict/ (CSV + matrix.def + def)
  - new_dict_built/ (binary, 81MB sys.dic)
- `PLAN.md`, `PROGRESS.md` 갱신

## 검증

- 학습 정상 종료 (29 iterations, convergence)
- 새 dict 모든 파일 생성 (sys.dic, matrix.bin, char.bin, unk.dic, model.bin)
- 단위 sanity check (mecab CLI 정상 출력)
- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass, 코드 변경 없음)

## Sprint 167 — Track B Step 4: Rust 통합 + 5-gate 검증

### 작업

1. 새 dict를 Rust 측 dict-builder 입력으로 사용
2. mecab-ko-dict-builder로 Rust binary 생성 (sys.dic.zst, matrix.bin.zst, entries.bin)
3. accuracy_eval test에서 새 dict 경로 사용
4. 5-gate 측정 (sample.tsv hard rule 핵심)
5. 회귀 분석 + 결정

### 예상 결과

- 학습 corpus가 작음 → KLUE/UD 평가 정확도 미미한 변동 가능
- sample.tsv 회귀 가능성 있음 → 즉시 rollback 준비
- 일부 패턴 (안녕하/VA) 의도하지 않은 분석 변화

### Track B 전체 진척

| Sprint | Step | 상태 |
|--------|------|------|
| 164 | Step 1: 빌드 환경 | ✅ |
| 165 | Step 2: 학습 데이터 | ✅ |
| **166** | **Step 3: 학습 + dict 변환** | **✅** |
| 167 | Step 4: Rust 통합 + 검증 | 다음 |
| 168 (옵션) | Step 5: 튜닝 | 대기 |
