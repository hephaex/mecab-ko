# PLAN — mecab-ko Sprint 167 (Track B Step 4: Rust 통합 + 5-gate 검증)

> 마지막 업데이트: 2026-05-27

## 완료: Sprint 166 — Track B Step 3: 학습 + Dict 변환

### 결과 (전체 파이프라인 62.6초)

- seed 빌드 (2.5초) → 학습 (37.7초, 29 iter, F=0.99977) → dict 생성 (20.5초) → binary 빌드 (1.9초)
- `data/training_run_1/new_dict_built/`: 새 mecab dict (sys.dic 81MB, matrix.bin 21MB)
- Sanity check: mecab CLI 정상 동작 (기존과 다른 분석 패턴 — 작은 corpus 효과)

## Sprint 167 — Track B Step 4

### 목표

새 dict를 Rust 측에 통합하고 5-gate로 정확도 측정. sample.tsv 무회귀 hard rule.

### 작업

#### S167-B1: Rust dict-builder 입력 준비
- 새 dict (`data/training_run_1/new_dict/`)의 CSV + def 파일들 사용
- mecab-ko-dict-builder의 build_dict 함수 호출

#### S167-B2: Rust binary 재생성
```bash
cd rust
cargo run --bin dict-builder -- --input ../data/training_run_1/new_dict --output ../data/training_run_1/rust_dict
```
- 산출: sys.dic.zst, matrix.bin.zst, entries.bin (Rust 측 형식)

#### S167-B3: accuracy_eval 테스트 실행 (MECAB_DIC_PATH=새 dict)
```bash
MECAB_DIC_PATH=data/training_run_1/rust_dict \
  cargo test --package mecab-ko-core --test accuracy_eval \
  -- test_accuracy_gate --nocapture --ignored
```

#### S167-B4: 5-gate 전체 측정
- sample.tsv (hard rule)
- KLUE morph (dual)
- KLUE surface_only
- UD Kaist (dual)
- UD GSD (dual)

#### S167-B5: 결과 분석 + 결정

| 시나리오 | 액션 |
|---------|------|
| sample.tsv 회귀 | 즉시 rollback, 학습 데이터 분석 |
| KLUE +0.5pp 이상 lift | Track B 성공, Sprint 168 튜닝 |
| KLUE 변화 미미 (±0.2pp) | 학습 corpus 작은 효과, Sprint 168에서 corpus 확장 |
| KLUE 회귀 | rollback, 원인 분석 (작은 corpus가 over-specialized?) |

### 위험 & 격리

- 새 dict는 별도 디렉토리 (`data/training_run_1/`)
- 기존 mecab-ko-dic-2.1.1 그대로 보존
- Rust 코드 변경 없이 환경 변수로 dict 경로 교체
- 회귀 시 환경 변수만 unset

## Track B 전체 진척

| Sprint | Step | 상태 |
|--------|------|------|
| 164 | Step 1: 빌드 환경 | ✅ |
| 165 | Step 2: 학습 데이터 | ✅ |
| 166 | Step 3: 학습 + dict 변환 | ✅ |
| **167** | **Step 4: Rust 통합 + 검증** | **다음** |
| 168 (옵션) | Step 5: 튜닝 / corpus 확장 | 결과 따라 |

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass (코드 변경 시)
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **sample.tsv hard rule**: 100%/99.9% 무회귀
- 5-gate 측정 결과 기록 (lift 또는 회귀)
