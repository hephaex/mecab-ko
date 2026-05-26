# PROGRESS — mecab-ko Sprint 164 (Track B Step 1: CRF 빌드 환경)

> 마지막 업데이트: 2026-05-27

## Sprint 164 — Track B Step 1: CRF Retrain 환경 구축

| Task | 상태 | 결과 |
|------|------|------|
| S164-B1: NIKL Modu 다운로드 4번째 체크 | ⏸ 미다운로드 | Track B 진행 결정 |
| S164-B2: Sprint 136 인프라 보고서 검토 | ✅ 완료 | mecab-cost-train 경로 식별 |
| S164-B3: legacy/ 빌드 시도 (1st) | ❌ 실패 | Linux .o 파일과 macOS arm64 충돌 |
| S164-B4: ./configure 재실행 | ✅ 완료 | arm-apple-darwin25.5.0 |
| S164-B5: make clean + make -j4 | ✅ 완료 | libmecab.dylib + 5개 executable |
| S164-B6: mecab-cost-train 실행 검증 | ✅ 완료 | --help 정상 출력 |
| S164-B7: 학습 데이터 형식 파악 | ✅ 완료 | .tagged 형식 (entries.csv 호환) |
| S164-B8: Track B 계획 수립 | ✅ 완료 | Sprint 164~167 (~+168) |

## 핵심 성과

### macOS arm64 빌드 성공

```
src/.libs/libmecab.dylib       ← 핵심 라이브러리
src/.libs/mecab-cost-train     ← CRF 학습 도구 ✓
src/.libs/mecab-dict-gen       ← dict 생성 ✓
src/.libs/mecab-dict-index     ← 인덱싱 ✓
src/.libs/mecab-system-eval    ← 평가
src/.libs/mecab-test-gen       ← 테스트 생성
```

기존 Linux .o 파일 → `make clean` 후 macOS arm64로 재컴파일 필요.

### mecab-cost-train 정상 동작

```bash
DYLD_LIBRARY_PATH=src/.libs src/.libs/mecab-cost-train --help
```

주요 옵션:
- `-d, --dicdir` (dict 디렉토리)
- `-M, --old-model` (warm start)
- `-c, --cost` (regularization)
- `-p, --thread` (멀티스레딩)

### 학습 데이터 형식 (`.tagged`)

```
<surface>\t<feature1>,<feature2>,...,<reading>
...
EOS
```

mecab-ko-dic entries.csv와 호환. KLUE/UD train data를 이 형식으로 변환 필요.

## Track B 계획 (Sprint 164~167)

| Sprint | Step | 작업 |
|--------|------|------|
| **164** ✅ | Step 1 | 빌드 환경 구축 |
| 165 | Step 2 | 학습 데이터 준비 (KLUE/UD → .tagged) |
| 166 | Step 3 | 1차 학습 + 변환 |
| 167 | Step 4 | Rust 통합 + 검증 (5-gate) |
| 168 (옵션) | Step 5 | 파라미터 튜닝 |

## 위험 요소

| 위험 | 완화 |
|------|------|
| 학습 시간 (수십분~시간) | -p 4, 작은 코퍼스 시작 |
| 회귀 (sample.tsv) | hard rule, 즉시 rollback |
| binary 호환성 | 별도 dict 디렉토리 격리 |
| 코퍼스 라이선스 | KLUE/UD CC BY-SA 사용 가능 |

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass)
- 5-gate sample.tsv: 영향 없음 (코드 변경 없음)
- legacy 빌드: 성공 (macOS arm64)

## 변경 파일

- `docs/research/accuracy/2026-05-27_sprint164_crf_build_env.md` (신규)
- `legacy/`: macOS arm64 빌드 산출물 (gitignore 대상, 추적 안 함)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 165 — Track B Step 2

학습 데이터 준비:
1. `tools/to_mecab_tagged.py` 작성 (TSV → .tagged 변환)
2. KLUE DP train (12K) → `.tagged`
3. UD Kaist/GSD train → `.tagged`
4. 학습용 합본 corpus 생성
