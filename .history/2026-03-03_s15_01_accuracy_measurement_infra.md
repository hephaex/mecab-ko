# Sprint 15-01: 정확도 측정 인프라 구축

**날짜**: 2026-03-03
**작업자**: Claude Opus 4.5 & hephaex
**스프린트**: Sprint 15 (Phase 7 - 사전 품질 & 정확도)

## 개요

형태소 분석 정확도를 객관적으로 측정할 수 있는 평가 인프라를 구축했습니다. 이를 통해 시스템 개선 효과를 정량적으로 확인할 수 있게 되었습니다.

## 구현 내용

### 1. 평가 모듈 (`mecab-ko-core/src/evaluate.rs`)

#### 주요 구조체

```rust
// 정답 토큰
pub struct GoldToken {
    pub surface: String,
    pub pos: String,
}

// 정답 문장
pub struct GoldSentence {
    pub text: String,
    pub tokens: Vec<GoldToken>,
}

// 테스트 데이터셋
pub struct TestDataset {
    pub sentences: Vec<GoldSentence>,
}

// 평가 결과
pub struct EvaluationResult {
    pub total_sentences: usize,
    pub total_gold_tokens: usize,
    pub total_pred_tokens: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub exact_match_sentences: usize,
    pub token_accuracy: f64,
    pub sentence_accuracy: f64,
    pub pos_accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub pos_stats: HashMap<String, PosStats>,
}
```

#### 주요 기능

1. **TestDataset::from_tsv()**: TSV 파일에서 테스트 데이터 로드
   - 형식: `원문\t토큰1/품사1 토큰2/품사2 ...`
   - 주석(`#`), 빈 줄 무시
   - 에러 위치 정보 제공

2. **evaluate_dataset()**: 데이터셋 전체 평가
   - 토큰 단위 매칭
   - 품사별 통계 수집
   - 다중 메트릭 계산

3. **EvaluationResult::format_report()**: 읽기 쉬운 리포트 생성
   - 전체 정확도 지표
   - 토큰 통계
   - 품사별 정확도 (상위 15개)

### 2. CLI 서브커맨드 (`mecab evaluate`)

#### 옵션

```bash
mecab evaluate [OPTIONS]

Options:
  -i, --input <FILE>        테스트 데이터 파일 (TSV 형식)
  -d, --dicdir <PATH>       사전 경로
  -o, --output <FILE>       결과 저장 파일 (없으면 stdout)
  -v, --verbose             상세 출력 (틀린 문장 표시)
  --format <FORMAT>         입력 형식 [default: tsv] [possible: tsv, json]
```

#### 사용 예제

```bash
# 기본 평가
mecab evaluate -i test.tsv -d data/dict-output

# 결과 파일 저장
mecab evaluate -i test.tsv -o report.txt

# 상세 분석 (틀린 문장 최대 10개 표시)
mecab evaluate -i test.tsv -v
```

### 3. 샘플 테스트 데이터

**파일**: `data/eval/sample.tsv`
**문장 수**: 160개
**커버리지**:
- 기본 문장 (5개)
- 명사구 (5개)
- 동사 활용 (5개)
- 형용사 (5개)
- 조사 활용 (5개)
- 복합 문장 (5개)
- 의문문 (5개)
- 명령문과 청유문 (5개)
- 부정문 (5개)
- 수식어 (5개)
- 숫자와 단위 (5개)
- 시간 표현 (5개)
- 관계 표현 (5개)
- 접속 표현 (5개)
- 존댓말 (5개)
- 감탄사와 호칭 (5개)
- 피동 표현 (5개)
- 사동 표현 (5개)
- 보조 용언 (5개)
- 연결 어미 (5개)
- 종결 어미 (5개)
- 명사형 어미 (5개)
- 관형형 어미 (5개)
- 부사형 어미 (5개)
- 인용 표현 (5개)
- 의존 명사 (5개)
- 접두사 (5개)
- 접미사 (5개)
- 어근 (5개)
- 외래어 (5개)
- 한자어 (5개)
- 의성어 의태어 (5개)

## 평가 지표

### 1. Token Accuracy (토큰 정확도)
정의: 정확하게 예측한 토큰 수 / 전체 정답 토큰 수

```
Token Accuracy = true_positives / total_gold_tokens
```

### 2. Sentence Accuracy (문장 정확도)
정의: 완전히 일치한 문장 수 / 전체 문장 수

```
Sentence Accuracy = exact_match_sentences / total_sentences
```

### 3. POS Accuracy (품사 정확도)
정의: 표면형이 일치하고 품사도 일치한 토큰 수 / 전체 정답 토큰 수

```
POS Accuracy = pos_correct / total_gold_tokens
```

### 4. Precision (정밀도)
정의: 정확하게 예측한 토큰 수 / 전체 예측 토큰 수

```
Precision = true_positives / total_pred_tokens
```

### 5. Recall (재현율)
정의: 정확하게 예측한 토큰 수 / 전체 정답 토큰 수

```
Recall = true_positives / total_gold_tokens
```

### 6. F1 Score
정의: Precision과 Recall의 조화 평균

```
F1 = 2 * (Precision * Recall) / (Precision + Recall)
```

## 테스트 결과 (샘플 데이터)

```
=== 정확도 평가 결과 ===
테스트 문장: 160
Token Accuracy: 15.2%
Sentence Accuracy: 8.1%
POS Accuracy: 15.2%
Precision: 0.181
Recall: 0.152
F1 Score: 0.165

토큰 통계:
  정답 토큰: 594
  예측 토큰: 498
  완전 일치 문장: 13 / 160 (8.1%)

품사별 정확도:
  VV     (134개): 1.5%
  NNG    (102개): 48.0%
  EF     (84개): 21.4%
  EC     (46개): 0.0%
  VA     (30개): 23.3%
  ETN    (19개): 10.5%
  MAG    (19개): 0.0%
  NNB    (18개): 5.6%
  NP     (17개): 5.9%
  EP     (17개): 0.0%
  VX     (14개): 0.0%
  ETM    (12개): 0.0%
  JKB    (11개): 18.2%
  JKO    (11개): 0.0%
  XPN    (10개): 0.0%
  ... 외 11개 품사
```

### 분석

1. **명사(NNG)는 상대적으로 높은 정확도 (48.0%)**
   - 사전에 잘 등록되어 있는 단어들
   - 복합명사 분해는 여전히 과제

2. **동사(VV)와 어미 정확도가 낮음 (1.5%, 0.0%)**
   - 활용형 처리의 복잡성
   - Unknown handler 개선 필요
   - 어미 분리 로직 개선 필요

3. **조사(JKB, JKO) 정확도 문제**
   - 격조사 인식 오류
   - 관형형 어미와 혼동

4. **전체적으로 낮은 정확도 (15.2%)**
   - 현재 사전의 한계 (2018년 기준)
   - Unknown 단어 처리 미흡
   - 복합명사 분해 로직 미흡

## 향후 개선 방향

### 1. 사전 품질 개선 (S15-02)
- 사전 일관성 검사
- 품사 태그 분포 분석
- 비용 값 최적화

### 2. Unknown 단어 처리 개선 (S15-05)
- Unknown 패턴 분석
- 추측 규칙 개선
- 외래어/고유명사 처리 강화

### 3. 복합명사 분해 개선 (S15-06)
- DecompoundMode 로직 개선
- 복합명사 사전 확장
- 분해 정확도 테스트

### 4. 테스트 데이터 확장
- 세종 코퍼스 데이터 통합
- 도메인별 테스트 셋 구축
- 신조어 테스트 셋 추가

## 기술적 세부사항

### Clippy 이슈 해결

1. **const fn 추천**
   - `GoldToken::new()`, `GoldSentence::new()`, `TestDataset::new()` → const fn으로 변경

2. **format_push_string**
   - `format!()` + `push_str()` → `writeln!()` 사용

3. **cast_precision_loss**
   - `#[allow(clippy::cast_precision_loss)]` 추가 (의도적 변환)

4. **unwrap_used**
   - String에 write는 실패하지 않으므로 `#[allow]` 추가

5. **doc_markdown**
   - 문서 주석의 변수명에 백틱 추가

### 테스트 구조 개선

1. **Token 구조체 필드 추가**
   - start_pos, end_pos, cost, features, normalized 필드 추가
   - 테스트 코드에서 모든 필드 초기화

2. **tempfile 의존성**
   - `test-utils` feature로 조건부 컴파일
   - 실제 임시 파일 생성 테스트

## 파일 변경 내역

### 새로 생성
- `rust/crates/mecab-ko-core/src/evaluate.rs` (695 lines)
- `data/eval/sample.tsv` (332 lines)

### 수정
- `rust/crates/mecab-ko-core/src/lib.rs` (+3 exports)
- `rust/crates/mecab-ko-cli/src/main.rs` (+200 lines)
  - Evaluate 서브커맨드 추가
  - EvalFormat enum 추가
  - run_evaluate() 함수 추가
- `PROGRESS.md` (S15-01 완료 표시)

## 커밋

```
feat(evaluate): Implement accuracy measurement infrastructure (S15-01)

Core Features:
- Token Accuracy, Sentence Accuracy, POS Accuracy
- Precision/Recall/F1 metrics
- POS-wise accuracy report

Components:
1. mecab-ko-core/src/evaluate.rs: Evaluation module
2. mecab evaluate subcommand: CLI interface
3. data/eval/sample.tsv: 160 sample sentences

Tests: 5 unit tests, all pass
Clippy: Clean with -D warnings
```

## 성과

1. **정량적 평가 가능**
   - 시스템 개선 효과를 수치로 확인
   - 버전 간 성능 비교 가능

2. **품사별 분석 가능**
   - 취약한 품사 태그 식별
   - 집중 개선 영역 파악

3. **재현 가능한 테스트**
   - 샘플 데이터 160문장 제공
   - CI/CD 통합 가능

4. **확장 가능한 구조**
   - JSON 형식 지원 준비
   - 다양한 메트릭 추가 가능

## 다음 단계

- **S15-02**: 사전 품질 검증 도구 개선
- **S15-05**: Unknown 단어 처리 개선
- **S15-06**: 복합명사 분해 개선
- 세종 코퍼스 데이터 통합 검토

## 참고 자료

- [Keep a Changelog](https://keepachangelog.com/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [세종 코퍼스](https://ithub.korean.go.kr/)
