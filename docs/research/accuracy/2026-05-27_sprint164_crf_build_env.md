# Sprint 164 — Track B Step 1: CRF Retrain 빌드 환경 구축

> **결과**: legacy/ C++ 도구를 macOS arm64에서 빌드 성공. mecab-cost-train 실행 검증. 학습 데이터 형식 파악. Track B 진행 가능 확인.

---

## 1. 빌드 환경 구축

### 1.1 macOS arm64 호환 빌드

기존 `config.status`는 Linux x86_64 환경:
```
host_triplet = x86_64-unknown-linux-gnu
```

`./configure` 재실행 후 macOS 환경:
```
host_triplet = arm-apple-darwin25.5.0
CXX = g++ (clang++ 호환)
```

### 1.2 빌드 단계

```bash
cd legacy/
./configure --prefix=/tmp/mecab-build
make clean  # Linux .o 파일 제거 필수
make -j4
```

결과:
- `src/.libs/libmecab.dylib` — 핵심 라이브러리
- `src/.libs/mecab-cost-train` — CRF 학습 도구 ✓
- `src/.libs/mecab-dict-gen` — dict 생성 ✓
- `src/.libs/mecab-dict-index` — 인덱싱 ✓
- `src/.libs/mecab-system-eval` — 평가
- `src/.libs/mecab-test-gen` — 테스트 생성

### 1.3 실행 검증

```bash
DYLD_LIBRARY_PATH=src/.libs src/.libs/mecab-cost-train --help
```

옵션:
| Option | 설명 |
|--------|------|
| `-d, --dicdir` | dict 디렉토리 |
| `-M, --old-model` | 기존 CRF 모델 (warm start) |
| `-c, --cost` | regularization C |
| `-f, --freq` | frequency cutoff (default 1) |
| `-e, --eta` | tolerance |
| `-p, --thread` | thread count |

---

## 2. 학습 데이터 형식

### 2.1 `.tagged` 형식 (legacy/tests/cost-train/ipa.train 참조)

```
<surface>\t<feature1>,<feature2>,...,<reading>
<surface>\t<feature1>,<feature2>,...,<reading>
...
EOS
<surface>\t...
```

각 라인이 하나의 morpheme (정답 분석). 문장 끝에 `EOS` 마커.
Feature 컬럼은 mecab-ko-dic의 entries.csv 형식과 동일.

### 2.2 학습 입력 디렉토리 (seed/)

```
char.def     — 문자 카테고리 (UNKNOWN 처리)
dic.csv      — 시드 dict
dicrc        — runtime config
feature.def  — CRF feature templates (UNIGRAM/BIGRAM)
matrix.def   — 초기 connection cost (재학습 대상)
unk.def      — 미등록어 규칙
```

mecab-ko-dic 2.1.1이 이 모든 파일을 제공 (`data/mecab-ko-dic-2.1.1-20180720/`).

---

## 3. Track B 작업 계획 (3-5 sprint)

### Sprint 164 ✅ (Step 1: 환경 구축)
- legacy/ 빌드 (mecab-cost-train, mecab-dict-gen)
- 학습 도구 실행 검증
- 학습 데이터 형식 파악

### Sprint 165 — Step 2: 학습 데이터 준비
- KLUE DP train (12K) → `.tagged` 형식 변환
- UD Kaist/GSD train → `.tagged` 형식 변환
- 변환 스크립트 작성 (`tools/to_mecab_tagged.py`)
- 학습용 합본 corpus 생성

### Sprint 166 — Step 3: 1차 학습 + 변환
- `mecab-cost-train -d <seed> -p 4 <tagged_files>` 실행
- 새 `model.def` 생성
- `mecab-dict-gen`으로 `matrix.def` + `sys.dic` 재생성
- 학습 시간 / 결과 크기 측정

### Sprint 167 — Step 4: Rust 통합 + 검증
- `mecab-ko-dict-builder`로 새 binary 재빌드
- 새 dict로 5-gate 측정
- 회귀 검증 (sample.tsv hard rule)
- 결과: success → commit, fail → analysis + iterate

### Sprint 168 (옵션) — Step 5: 튜닝
- 학습 파라미터 (-c, -e, -f) 조정
- 학습 코퍼스 비율 조정
- 최종 성능 확정

---

## 4. 위험 요소

| 위험 | 완화 |
|------|------|
| 학습 시간 (수십 분~시간) | -p 4 (멀티스레딩), 작은 코퍼스로 시작 |
| 새 model이 기존 정확도 회귀 | sample.tsv hard rule, rollback 즉시 |
| binary 호환성 (left/right-id.def 변경) | 별도 dict 디렉토리로 격리, 점진적 교체 |
| 코퍼스 라이선스 | KLUE (CC BY-SA), UD (CC BY-SA) 모두 학습 가능 |
| 학습 데이터 부족 | sample.tsv + KLUE train + UD train 통합 |

---

## 5. 변경 파일

- (코드 변경 없음 — 환경 구축 + 분석만)
- `legacy/`: macOS arm64 빌드 산출물 생성 (untracked, build cache)
- `docs/research/accuracy/2026-05-27_sprint164_crf_build_env.md` (본 문서)

---

## 6. 결론

Track B 진행 가능. 빌드 환경 구축 완료. 다음 단계 (학습 데이터 준비)부터는 코드 작업.

---

*작성: 2026-05-27 (Sprint 164, Track B Step 1)*
