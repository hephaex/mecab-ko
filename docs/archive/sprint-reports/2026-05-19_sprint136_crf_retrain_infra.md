# Sprint 136 P1 — CRF Retrain 인프라 조사

> **조사 목적**: dict 추가 효율 포화(+0.007pp/entry) 이후 +1pp 이상 lift는 connection cost 또는 CRF retrain만 가능. 실제 가능 범위와 비용을 사전 평가.

---

## 1. mecab-ko-dic CRF 구조 분석

### 1.1 파일 인벤토리 (data/mecab-ko-dic-2.1.1-20180720/)

| 파일 | 역할 | 크기/항목 |
|------|------|----------|
| `feature.def` | CRF 피처 템플릿 (Unigram/Bigram) | 33줄 |
| `matrix.def` | 연접 비용 행렬 (left_id × right_id) | 10,292,647줄 (~10M) |
| `left-id.def` | 좌문맥 ID → 자질열 매핑 | 2,693 IDs |
| `right-id.def` | 우문맥 ID → 자질열 매핑 | 3,822 IDs |
| `model.def` | CRF 학습 결과 (weight per feature) | training output |
| `char.def` | 문자 카테고리 정의 | (UNKNOWN 처리) |
| `unk.def` | 미등록어 처리 규칙 | (CharCategory별 POS) |
| `*.csv` (NNG/NNP/EC/EF/...) | 시스템 사전 엔트리 (각 품사별) | per-tag |
| `tools/` | `add-userdic.sh`, `convert_for_using_store.sh`, `mecab-bestn.sh` | shell scripts |

### 1.2 feature.def 템플릿 구조

```
# UNIGRAM 템플릿 (단일 형태소)
UNIGRAM U00:%F[0]              # POS만
UNIGRAM U01:%F[0],%F?[1]       # POS,의미분류
UNIGRAM R00:%F[3]              # 종성 여부
UNIGRAM R01:%F[0],%F[3]        # POS, 종성

# BIGRAM 템플릿 (연접)
BIGRAM B00:%L[0]/%R[0]                    # 좌POS/우POS
BIGRAM B01:%L[0],%L?[1]/%R[0]             # 좌POS,의미/우POS
BIGRAM B10:%L[0],%L?[2]/%R[0],%R?[3]      # 좌POS,읽기/우POS,종성
```

- `%F[n]`: UNIGRAM의 n번째 소성(feature)
- `%L[n]/%R[n]`: BIGRAM의 좌(L)/우(R) 형태소 소성
- `?` suffix: optional feature (`*`이면 무시)

### 1.3 model.def 형식

```
header (eta/freq/C/eval-size/unk-eval-size/charset)
<weight>\t<feature_name>

예:
-3.3284256933931973	B00:BOS/EOS/EC
-1.9746213666970398	B00:BOS/EOS/EF
1.4164787314252922	B00:BOS/EOS/NNG
```

각 feature(템플릿 인스턴스화 결과)에 학습된 weight 부여.
matrix.def는 model.def로부터 derived (BIGRAM features의 weight 합).

### 1.4 matrix.def 빌드 흐름

```
학습 코퍼스(.tagged) → mecab-cost-train → model.def
model.def → mecab-dict-gen → matrix.def + sys.dic 등 binary
```

legacy/src에 `mecab-cost-train.cpp`, `mecab-dict-gen.cpp` C++ 소스 존재.
build 산출물: `legacy/src/mecab-cost-train`, `legacy/src/mecab-dict-gen` (executables).

---

## 2. Rust 바인딩 측 로드 경로

### 2.1 matrix.def 로딩 (mecab-ko-dict-builder)

```rust
// rust/crates/mecab-ko-dict-builder/src/lib.rs:489
fn build_matrix(&self) -> Result<DenseMatrix> {
    let matrix_path = Path::new(&self.config.input_dir).join("matrix.def");
    DenseMatrix::from_def_file(&matrix_path).map_err(BuildError::Dict)
}
```

- `DenseMatrix::from_def_file`: 텍스트 matrix.def 파서 (Rust 자체 구현)
- 출력: `DenseMatrix` (메모리), `MmapMatrix` (mmap), `SparseMatrix` (희소)
- 빌드 시 `entries.csv` + `matrix.def` + `char.def` + `unk.def` → binary artifacts

### 2.2 Tokenizer runtime 로드

`mecab-ko-dict::lib.rs:61` — `pub use matrix::{ConnectionMatrix, DenseMatrix, MatrixLoader, MmapMatrix, SparseMatrix};`

런타임에 `matrix.def` 텍스트를 다시 읽지 않음 — 빌드 시 binary로 변환.
custom matrix.def 로드 시 dict-builder를 재실행해야 함.

---

## 3. CRF Retrain 가능성 평가

### 3.1 Full Retrain (mecab-cost-train 사용)

**필요 자원**:
- 학습 코퍼스 (Sejong tagged + 추가)
- C++ 빌드 환경 (legacy/src/mecab-cost-train)
- 컴퓨팅: full Sejong 학습 시간 수십 분 ~ 수 시간 (CRF 학습)

**가능성**: 가능. legacy 디렉토리에 학습 도구 소스 + 빌드 산출물 존재.

**리스크**:
- 학습 코퍼스 라이선스 (Sejong 코퍼스 재배포 제약)
- 학습 데이터 변경 시 기존 정확도 회귀 가능 (현재 KLUE DP 66.8%가 baseline)
- model.def → matrix.def 변환 시 left/right-id.def도 함께 변경되면 binary 호환성 깨짐

### 3.2 Partial Connection Cost 수동 조정 (CRF 전 단계)

**아이디어**: 특정 (left_id, right_id) 쌍의 cost만 수동 조정.

예: MAG → EF (부사 → 종결어미) 연접 비용을 낮추면 "굉장히/MAG" 같은 부사가 종결 위치에 올 때 split 회피.

**구현 경로**:
1. matrix.def에서 특정 쌍 cost 변경
2. dict-builder로 재빌드 → 새 binary matrix
3. 단일 케이스 테스트 + KLUE DP 회귀 검증

**리스크**:
- left-id/right-id가 ~2.7K × 3.8K = 10M 조합 — 어떤 쌍을 조정할지 식별 어려움
- 한 쌍 조정이 다른 케이스에 어떻게 영향 줄지 예측 어려움 (CRF는 전역 학습)
- 빈도 기반 분석 후 5-10개 핵심 쌍만 조정하는 접근 추천

### 3.3 Inflect.csv 기반 활용 보강

**관찰**: Inflect.csv가 별도 존재 — mecab-ko-dic의 활용형 expansion 메커니즘.
ㄹ불규칙(따라/몰라/달라)이 Inflect.csv에 명시되어 있는지 확인 필요.

dict-builder의 `inflect_gen.rs` 모듈이 Rust 측 활용형 처리.
정적 활용형 추가로 일부 SURFACE_MISMATCH 해소 가능 (Sprint 136 P3a normalize_endings의 상위 대체).

---

## 4. 권고 — Sprint 137 후보

### Track A: 보수적 — Connection Cost 부분 조정 실험

1. **분석**: KLUE DP per-eojeol 오류 중 SPLIT_DIFFERENT 카테고리(10.2%)를 (left_id, right_id) 쌍 분포로 매핑
2. **타깃 식별**: 상위 5-10개 problematic 쌍 (예: MAG↔NNG, VV↔NNG)
3. **수동 조정**: matrix.def에서 해당 쌍 cost만 변경 → 재빌드
4. **검증**: KLUE DP morph + sample.tsv 양쪽 무회귀 확인

**예상 효과**: +0.3 ~ +1.0pp (보수적)
**리스크**: 낮음 (개별 쌍 격리)
**비용**: 분석 1-2 sprint + 실험 1 sprint

### Track B: 적극적 — Full CRF Retrain

1. **학습 데이터 준비**: Sejong + KLUE DP train + (선택) 추가 도메인
2. **mecab-cost-train 빌드 + 실행** (legacy/src)
3. **새 model.def → matrix.def + left/right-id.def 재생성**
4. **dict-builder 재실행 → Rust binary**
5. **전체 정확도 회귀 검증**

**예상 효과**: +1.0 ~ +5.0pp (정확치 미정)
**리스크**: 높음 (binary 호환성, 코퍼스 라이선스, 학습 데이터 split)
**비용**: 3-5 sprint

### Track C: Inflect.csv 보강 (가벼운 첫걸음)

1. ㄹ불규칙 동사 활용형을 Inflect.csv에 정적 추가
2. dict-builder 재실행 → mecab이 활용형을 직접 인식
3. normalize_endings rule 단순화 가능

**예상 효과**: SURFACE_MISMATCH 흡수 +0.1-0.3pp
**리스크**: 낮음
**비용**: 1 sprint

---

## 5. 결론

| 항목 | 결과 |
|------|------|
| **mecab-ko-dic CRF 구조** | feature.def(33줄) + matrix.def(10M) + model.def 명확. left-id 2,693 / right-id 3,822 |
| **Rust 측 customization 가능성** | matrix.def 텍스트 수정 → dict-builder 재실행 경로 명확. binary 호환성 유지됨 |
| **Full retrain 도구** | legacy/src/mecab-cost-train 빌드 산출물 존재 |
| **Sprint 137 권고 진입점** | **Track A (Connection cost 분석 + 5-10쌍 수동 조정)** 부터 시작. Track C 병행 가능. Track B는 결과 부족 시 escalation |

**핵심 발견**: CRF retrain은 가능하나 진입 비용이 큼. 더 가벼운 첫걸음으로 (1) connection cost 분석으로 problematic pairs 식별, (2) Inflect.csv 정적 보강 두 가지가 안전한 ramp-up 경로.

---

*작성: 2026-05-19 (Sprint 136 P1)*
