# S15-02: 사전 품질 검증 도구 개선

**날짜**: 2026-03-03
**작업**: mecab-ko-dict-validator 크레이트 기능 강화
**상태**: ✅ 완료

## 작업 개요

mecab-ko-dict-validator 크레이트에 고급 통계 분석 및 품질 검증 기능을 추가하여 사전 품질을 체계적으로 분석하고 개선할 수 있도록 했습니다.

## 구현 내용

### 1. 새로운 모듈: `analyzer.rs`

고급 사전 분석 기능을 제공하는 새로운 모듈을 추가했습니다.

#### 주요 구조체

```rust
pub struct DictAnalyzer;

pub struct AnalysisReport {
    pub total_entries: usize,
    pub pos_distribution: PosDistribution,
    pub cost_distribution: CostDistribution,
    pub consistency_issues: ConsistencyIssues,
    pub recommendations: Vec<Recommendation>,
}

pub struct PosDistribution {
    pub tags: Vec<PosTagStat>,
}

pub struct CostDistribution {
    pub min: i32,
    pub max: i32,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub histogram: Vec<HistogramBin>,
    pub outliers: Vec<OutlierInfo>,
}
```

#### 핵심 기능

1. **품사 태그 분포 분석**
   - 각 품사별 엔트리 수 계산
   - 품사별 비율 계산 (백분율)
   - 품사 불균형 탐지

2. **비용 값 분포 분석**
   - 평균, 중앙값, 표준편차 계산
   - 히스토그램 생성 (20개 구간)
   - 이상치 탐지 (3σ 기준)

3. **일관성 검사**
   - 중복 엔트리 탐지 (완전 중복)
   - 품사 태그 유효성 검증 (세종 품사 태그 기준)
   - 읽기/쓰기 ID 범위 검증
   - 비정상 비용 값 탐지

4. **자동 권장 시스템**
   - 품사 분포 불균형 경고
   - 희귀 품사 탐지
   - 비용 이상치 경고
   - 일관성 문제 알림

### 2. CLI 개선

`dict-validate` 명령에 새로운 플래그 추가:

```bash
# 분석 모드 실행
dict-validate dict.csv --analyze

# 자동 수정 제안 포함
dict-validate dict.csv --analyze --fix

# JSON 형식 출력
dict-validate dict.csv --analyze --format json
```

#### 출력 예시 (텍스트)

```
═══ 사전 품질 리포트 ═══

총 엔트리: 10
중복 엔트리: 1
유효하지 않은 품사: 0

품사 분포:
  NNG      :        8 ( 80.0%)
  VV       :        2 ( 20.0%)

비용 분포:
  범위       : 300 ~ 600
  평균       : 460.0
  중앙값     : 490.0
  표준편차   : 94.8
  이상치     : 0개

비용 히스토그램 (상위 10개 구간):
  [   300 ~    314]:      2 █
  [   495 ~    509]:      2 █
  ...

권장 사항:
  ⚠️  [POS Distribution] 'NNG' 품사가 전체의 80.0%를 차지합니다.
  ❌ [Consistency] 1개의 중복 엔트리가 발견되었습니다.
```

### 3. 파일 변경 사항

- **신규 생성**:
  - `rust/crates/mecab-ko-dict-validator/src/analyzer.rs` (530 lines)

- **수정**:
  - `rust/crates/mecab-ko-dict-validator/src/lib.rs` - analyzer 모듈 export
  - `rust/crates/mecab-ko-dict-validator/src/validator.rs` - entries 저장
  - `rust/crates/mecab-ko-dict-validator/src/report.rs` - entries 필드 추가
  - `rust/crates/mecab-ko-dict-validator/src/bin/dict-validate.rs` - CLI 플래그 추가

## 기술적 세부사항

### 통계 알고리즘

1. **중앙값 계산**
   ```rust
   fn calculate_median(sorted_values: &[i32]) -> f64 {
       let len = sorted_values.len();
       if len % 2 == 0 {
           (sorted_values[len/2-1] + sorted_values[len/2]) / 2.0
       } else {
           sorted_values[len/2] as f64
       }
   }
   ```

2. **표준편차 계산**
   ```rust
   fn calculate_std_dev(values: &[i32], mean: f64) -> f64 {
       let variance = values.iter()
           .map(|&v| (v as f64 - mean).powi(2))
           .sum::<f64>() / values.len() as f64;
       variance.sqrt()
   }
   ```

3. **이상치 탐지 (3σ 방식)**
   ```rust
   let lower_bound = 3.0f64.mul_add(-std_dev, mean);
   let upper_bound = 3.0f64.mul_add(std_dev, mean);
   ```

4. **히스토그램 생성**
   - 고정 20개 구간으로 비용 범위 분할
   - 동적 bin width 계산
   - 빈 구간 필터링

### 최적화

- `rayon`을 사용한 병렬 처리 (기존 validator.rs)
- 메모리 효율적인 히스토그램 생성
- 이상치 제한 (최대 50개)

## 테스트

### 단위 테스트 (analyzer.rs)

```rust
#[test]
fn test_analyze_basic()
fn test_pos_distribution()
fn test_cost_distribution()
fn test_median_calculation()
fn test_std_dev_calculation()
fn test_histogram_generation()
fn test_outlier_detection()
fn test_consistency_check()
fn test_invalid_pos_tags()
```

### 테스트 결과

```
running 28 tests
test analyzer::tests::test_analyze_basic ... ok
test analyzer::tests::test_cost_distribution ... ok
test analyzer::tests::test_consistency_check ... ok
test analyzer::tests::test_histogram_generation ... ok
test analyzer::tests::test_invalid_pos_tags ... ok
test analyzer::tests::test_median_calculation ... ok
test analyzer::tests::test_outlier_detection ... ok
test analyzer::tests::test_pos_distribution ... ok
test analyzer::tests::test_std_dev_calculation ... ok
... (19 more tests)

test result: ok. 28 passed; 0 failed; 0 ignored
```

### 통합 테스트

```
running 14 tests
test test_validate_valid_dictionary ... ok
test test_detect_exact_duplicates ... ok
test test_text_output ... ok
test test_json_output ... ok
... (10 more tests)

test result: ok. 14 passed; 0 failed; 0 ignored
```

### Clippy 검증

```bash
cargo clippy --package mecab-ko-dict-validator -- -D warnings
# 경고 없음, 모든 검사 통과
```

## 코드 품질

- ✅ Rust API Guidelines 준수
- ✅ Clippy strict mode 통과 (-D warnings)
- ✅ 모든 public API에 rustdoc 추가
- ✅ 에러 처리 완전 (unwrap 사용 최소화)
- ✅ 타입 안전성 (newtype pattern)

## 사용 예시

### 기본 검증
```bash
dict-validate dictionary.csv
```

### 상세 분석
```bash
dict-validate dictionary.csv --analyze
```

### JSON 출력으로 분석
```bash
dict-validate dictionary.csv --analyze --format json > report.json
```

### 수정 제안 포함
```bash
dict-validate dictionary.csv --analyze --fix
```

### 설정 파일 생성
```bash
dict-validate --generate-config validator.toml
```

## 학습 포인트

1. **통계 알고리즘 구현**: 중앙값, 표준편차, 히스토그램, 이상치 탐지를 Rust로 효율적으로 구현
2. **CLI 설계 패턴**: clap을 사용한 서브모드 구현 (--analyze)
3. **Clippy strict 모드**: cast_lossless, suboptimal_flops 등 고급 lint 규칙 적용

## 다음 단계

- [ ] S15-05: Unknown 단어 처리 개선
- [ ] S15-06: 복합명사 분해 개선
- [ ] 실제 mecab-ko-dic (816K entries)로 성능 테스트

## 참고 자료

- [Statistics for Software Engineers](https://www.statsoft.pl/textbook/stathome_stat.html?https://www2.statsoft.com/)
- [Outlier Detection Algorithms](https://towardsdatascience.com/outlier-detection-theory-visualizations-and-code-a4fd39de540c)
- [Clippy Lints Reference](https://rust-lang.github.io/rust-clippy/rust-1.92.0/)
