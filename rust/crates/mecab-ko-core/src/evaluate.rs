//! # Evaluation Module
//!
//! 형태소 분석 정확도 측정 인프라
//!
//! ## 주요 기능
//!
//! - Token Accuracy: 토큰 단위 정확도
//! - Sentence Accuracy: 문장 단위 완전 일치율
//! - POS Accuracy: 품사 태그 정확도
//! - Precision/Recall/F1: 토큰 기준
//! - 품사별 정확도 리포트
//!
//! ## 예제
//!
//! ```rust,no_run
//! use mecab_ko_core::evaluate::{evaluate_dataset, TestDataset};
//! use mecab_ko_core::tokenizer::Tokenizer;
//!
//! let mut tokenizer = Tokenizer::new().unwrap();
//! let dataset = TestDataset::from_tsv("data/eval/sample.tsv").unwrap();
//! let result = evaluate_dataset(&mut tokenizer, &dataset);
//!
//! println!("Token Accuracy: {:.2}%", result.token_accuracy * 100.0);
//! println!("F1 Score: {:.3}", result.f1_score);
//! ```

use crate::sejong::SejongConverter;
use crate::tokenizer::{Token, Tokenizer};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

/// 평가 에러 타입
#[derive(Error, Debug)]
pub enum EvaluateError {
    /// 입출력 에러
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// 파싱 에러
    #[error("Parse error: {0}")]
    Parse(String),

    /// 데이터 에러
    #[error("Data error: {0}")]
    Data(String),
}

/// 평가 결과 타입
pub type Result<T> = std::result::Result<T, EvaluateError>;

/// 정답 토큰
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldToken {
    /// 표면형
    pub surface: String,
    /// 품사 태그
    pub pos: String,
}

impl GoldToken {
    /// 새로운 정답 토큰 생성
    ///
    /// # Arguments
    ///
    /// * `surface` - 표면형
    /// * `pos` - 품사 태그
    #[must_use]
    pub const fn new(surface: String, pos: String) -> Self {
        Self { surface, pos }
    }

    /// 문자열에서 파싱 (surface/pos 형식)
    ///
    /// # Arguments
    ///
    /// * `s` - 파싱할 문자열
    ///
    /// # Errors
    ///
    /// 형식이 잘못된 경우 에러 반환
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(EvaluateError::Parse(format!(
                "Invalid token format: {s} (expected surface/pos)"
            )));
        }

        Ok(Self {
            surface: SejongConverter::normalize_jamo(parts[0]),
            pos: parts[1].to_string(),
        })
    }
}

/// 정답 문장
#[derive(Debug, Clone)]
pub struct GoldSentence {
    /// 원문
    pub text: String,
    /// 정답 토큰 리스트
    pub tokens: Vec<GoldToken>,
    /// 어절별 형태소 개수 (선택). 어절 단위 평가에 사용.
    /// `None`이면 어절 정보가 TSV에 없음을 의미 (legacy format).
    pub eojeol_counts: Option<Vec<usize>>,
}

impl GoldSentence {
    /// 새로운 정답 문장 생성
    ///
    /// # Arguments
    ///
    /// * `text` - 원문
    /// * `tokens` - 정답 토큰 리스트
    #[must_use]
    pub const fn new(text: String, tokens: Vec<GoldToken>) -> Self {
        Self {
            text,
            tokens,
            eojeol_counts: None,
        }
    }

    /// TSV 라인에서 파싱
    ///
    /// 형식:
    /// - `text\ttokens` (legacy 2-column)
    /// - `text\ttokens\teojeol_counts` (3-column with eojeol info)
    ///   - `eojeol_counts`: comma-separated, e.g. "5,2,2,2,2,4"
    ///   - 합이 tokens 개수와 일치해야 함
    ///
    /// 각 토큰: surface/pos
    ///
    /// # Arguments
    ///
    /// * `line` - TSV 라인
    ///
    /// # Errors
    ///
    /// 파싱 실패 시 에러 반환
    pub fn parse_tsv_line(line: &str) -> Result<Self> {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(EvaluateError::Parse(format!(
                "Invalid TSV line: {line} (expected 2 or 3 tab-separated columns)"
            )));
        }

        let text = parts[0].trim().to_string();
        let tokens_str = parts[1].trim();

        let tokens = tokens_str
            .split_whitespace()
            .map(GoldToken::parse)
            .collect::<Result<Vec<_>>>()?;

        if tokens.is_empty() {
            return Err(EvaluateError::Data(format!(
                "Empty gold tokens for text: {text}"
            )));
        }

        let eojeol_counts = if parts.len() == 3 {
            let counts: Vec<usize> = parts[2]
                .trim()
                .split(',')
                .map(|s| {
                    s.trim().parse::<usize>().map_err(|e| {
                        EvaluateError::Parse(format!("Invalid eojeol count '{s}': {e}"))
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            let sum: usize = counts.iter().sum();
            if sum != tokens.len() {
                return Err(EvaluateError::Data(format!(
                    "eojeol_counts sum ({sum}) does not match tokens len ({}) for text: {text}",
                    tokens.len()
                )));
            }
            Some(counts)
        } else {
            None
        };

        Ok(Self {
            text,
            tokens,
            eojeol_counts,
        })
    }
}

/// 테스트 데이터셋
#[derive(Debug, Clone)]
pub struct TestDataset {
    /// 정답 문장 리스트
    pub sentences: Vec<GoldSentence>,
}

impl TestDataset {
    /// 새로운 빈 데이터셋 생성
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sentences: Vec::new(),
        }
    }

    /// TSV 파일에서 로드
    ///
    /// 형식:
    /// - 각 라인: 원문\t정답토큰1 정답토큰2 ...
    /// - 각 토큰: surface/pos
    /// - # 주석 라인 무시
    /// - 빈 라인 무시
    ///
    /// # Arguments
    ///
    /// * `path` - TSV 파일 경로
    ///
    /// # Errors
    ///
    /// 파일 읽기 실패 또는 파싱 에러 시 에러 반환
    pub fn from_tsv<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut sentences = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let trimmed = line.trim();

            // 주석과 빈 라인 무시
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let sentence = GoldSentence::parse_tsv_line(trimmed)
                .map_err(|e| EvaluateError::Parse(format!("Line {}: {}", line_num + 1, e)))?;

            sentences.push(sentence);
        }

        if sentences.is_empty() {
            return Err(EvaluateError::Data("Empty dataset".to_string()));
        }

        Ok(Self { sentences })
    }

    /// 문장 추가
    ///
    /// # Arguments
    ///
    /// * `sentence` - 추가할 정답 문장
    pub fn add_sentence(&mut self, sentence: GoldSentence) {
        self.sentences.push(sentence);
    }

    /// 데이터셋 크기 반환
    #[must_use]
    pub fn len(&self) -> usize {
        self.sentences.len()
    }

    /// 데이터셋이 비어있는지 확인
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sentences.is_empty()
    }
}

impl Default for TestDataset {
    fn default() -> Self {
        Self::new()
    }
}

/// 평가 결과
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// 총 테스트 문장 수
    pub total_sentences: usize,
    /// 총 정답 토큰 수
    pub total_gold_tokens: usize,
    /// 총 예측 토큰 수
    pub total_pred_tokens: usize,

    /// True Positive: 정확하게 예측한 토큰 수
    pub true_positives: usize,
    /// False Positive: 잘못 예측한 토큰 수
    pub false_positives: usize,
    /// False Negative: 누락한 토큰 수
    pub false_negatives: usize,

    /// 완전히 일치한 문장 수
    pub exact_match_sentences: usize,

    /// 토큰 정확도 (0.0 ~ 1.0)
    pub token_accuracy: f64,
    /// 문장 정확도 (0.0 ~ 1.0)
    pub sentence_accuracy: f64,
    /// 품사 정확도 (0.0 ~ 1.0)
    pub pos_accuracy: f64,
    /// Precision (0.0 ~ 1.0)
    pub precision: f64,
    /// Recall (0.0 ~ 1.0)
    pub recall: f64,
    /// F1 Score (0.0 ~ 1.0)
    pub f1_score: f64,

    /// 품사별 통계
    pub pos_stats: HashMap<String, PosStats>,
}

/// 품사별 통계
#[derive(Debug, Clone, Default)]
pub struct PosStats {
    /// 정답 토큰 수
    pub gold_count: usize,
    /// 예측 토큰 수
    pub pred_count: usize,
    /// 정확하게 예측한 수
    pub correct: usize,
    /// 정확도
    pub accuracy: f64,
}

impl EvaluationResult {
    /// 빈 결과 생성
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_sentences: 0,
            total_gold_tokens: 0,
            total_pred_tokens: 0,
            true_positives: 0,
            false_positives: 0,
            false_negatives: 0,
            exact_match_sentences: 0,
            token_accuracy: 0.0,
            sentence_accuracy: 0.0,
            pos_accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            pos_stats: HashMap::new(),
        }
    }

    /// 포맷된 리포트 생성
    ///
    /// # Returns
    ///
    /// 사람이 읽기 쉬운 형태의 평가 리포트 문자열
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::unwrap_used)]
    pub fn format_report(&self) -> String {
        use std::fmt::Write;

        let mut report = String::new();

        report.push_str("=== 정확도 평가 결과 ===\n");
        writeln!(report, "테스트 문장: {}", self.total_sentences).unwrap();
        writeln!(
            report,
            "Token Accuracy: {:.1}%",
            self.token_accuracy * 100.0
        )
        .unwrap();
        writeln!(
            report,
            "Sentence Accuracy: {:.1}%",
            self.sentence_accuracy * 100.0
        )
        .unwrap();
        writeln!(report, "POS Accuracy: {:.1}%", self.pos_accuracy * 100.0).unwrap();
        writeln!(report, "Precision: {:.3}", self.precision).unwrap();
        writeln!(report, "Recall: {:.3}", self.recall).unwrap();
        writeln!(report, "F1 Score: {:.3}", self.f1_score).unwrap();
        report.push('\n');

        report.push_str("토큰 통계:\n");
        writeln!(report, "  정답 토큰: {}", self.total_gold_tokens).unwrap();
        writeln!(report, "  예측 토큰: {}", self.total_pred_tokens).unwrap();
        writeln!(
            report,
            "  완전 일치 문장: {} / {} ({:.1}%)",
            self.exact_match_sentences,
            self.total_sentences,
            (self.exact_match_sentences as f64 / self.total_sentences as f64) * 100.0
        )
        .unwrap();
        report.push('\n');

        // 품사별 정확도 (상위 15개)
        let mut pos_sorted: Vec<_> = self.pos_stats.iter().collect();
        pos_sorted.sort_by_key(|b| std::cmp::Reverse(b.1.gold_count));

        if !pos_sorted.is_empty() {
            report.push_str("품사별 정확도:\n");
            for (pos, stats) in pos_sorted.iter().take(15) {
                writeln!(
                    report,
                    "  {pos:<6} ({}개): {:.1}%",
                    stats.gold_count,
                    stats.accuracy * 100.0
                )
                .unwrap();
            }

            if pos_sorted.len() > 15 {
                writeln!(report, "  ... 외 {}개 품사", pos_sorted.len() - 15).unwrap();
            }
        }

        report
    }
}

impl Default for EvaluationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 토큰 리스트 평가
///
/// # Arguments
///
/// * `gold_tokens` - 정답 토큰 리스트
/// * `pred_tokens` - 예측 토큰 리스트
///
/// # Returns
///
/// (`true_positives`, `false_positives`, `false_negatives`, `pos_match`)
#[must_use]
pub fn evaluate_tokens(
    gold_tokens: &[GoldToken],
    pred_tokens: &[Token],
) -> (usize, usize, usize, usize) {
    let min_len = gold_tokens.len().min(pred_tokens.len());

    let mut true_positives = 0;
    let mut pos_match = 0;

    // 위치 기반 매칭 (순서대로 비교)
    for i in 0..min_len {
        let gold = &gold_tokens[i];
        let pred = &pred_tokens[i];

        if gold.surface == pred.surface && gold.pos == pred.pos {
            true_positives += 1;
            pos_match += 1;
        } else if gold.surface == pred.surface {
            pos_match += 1;
        }
    }

    let false_positives = pred_tokens.len().saturating_sub(true_positives);
    let false_negatives = gold_tokens.len().saturating_sub(true_positives);

    (true_positives, false_positives, false_negatives, pos_match)
}

/// Greedy alignment 기반 토큰 평가 (strict 기본).
///
/// `evaluate_tokens_aligned_with_pos_match`를 `pos_eq_strict`로 호출.
#[must_use]
pub fn evaluate_tokens_aligned(
    gold_tokens: &[GoldToken],
    pred_tokens: &[Token],
) -> (usize, usize, usize, usize) {
    evaluate_tokens_aligned_with_pos_match(gold_tokens, pred_tokens, pos_eq_strict)
}

/// Greedy alignment 기반 토큰 평가 (POS 비교 함수 주입, Sprint 125).
///
/// 순서를 고려하되, 토큰 갯수 차이가 있어도 최선의 매칭을 시도합니다.
/// `pos_eq`로 strict(`pos_eq_strict`) 또는 lenient(`pos_tags_equivalent`)
/// 모드 선택.
///
/// # Returns
///
/// (`true_positives`, `false_positives`, `false_negatives`, `pos_match`)
#[must_use]
pub fn evaluate_tokens_aligned_with_pos_match(
    gold_tokens: &[GoldToken],
    pred_tokens: &[Token],
    pos_eq: PosMatchFn,
) -> (usize, usize, usize, usize) {
    let mut true_positives = 0;
    let mut pos_match = 0;

    let mut gold_idx = 0;
    let mut pred_idx = 0;

    while gold_idx < gold_tokens.len() && pred_idx < pred_tokens.len() {
        let gold = &gold_tokens[gold_idx];
        let pred = &pred_tokens[pred_idx];

        if gold.surface == pred.surface {
            pos_match += 1;
            if pos_eq(&gold.pos, &pred.pos) {
                true_positives += 1;
            }
            gold_idx += 1;
            pred_idx += 1;
        } else {
            let mut found = false;
            for look_ahead in 1..=3 {
                if pred_idx + look_ahead < pred_tokens.len()
                    && pred_tokens[pred_idx + look_ahead].surface == gold.surface
                {
                    pred_idx += look_ahead;
                    found = true;
                    break;
                }
            }

            if !found {
                for look_ahead in 1..=3 {
                    if gold_idx + look_ahead < gold_tokens.len()
                        && gold_tokens[gold_idx + look_ahead].surface == pred.surface
                    {
                        gold_idx += look_ahead;
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                gold_idx += 1;
                pred_idx += 1;
            }
        }
    }

    let false_positives = pred_tokens.len().saturating_sub(true_positives);
    let false_negatives = gold_tokens.len().saturating_sub(true_positives);

    (true_positives, false_positives, false_negatives, pos_match)
}

/// 데이터셋 평가
///
/// # Arguments
///
/// * `tokenizer` - 형태소 분석기
/// * `dataset` - 테스트 데이터셋
///
/// # Returns
///
/// 평가 결과
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn evaluate_dataset(tokenizer: &mut Tokenizer, dataset: &TestDataset) -> EvaluationResult {
    let mut result = EvaluationResult::new();
    result.total_sentences = dataset.len();

    for gold_sentence in &dataset.sentences {
        let pred_tokens = tokenizer.tokenize(&gold_sentence.text);

        result.total_gold_tokens += gold_sentence.tokens.len();
        result.total_pred_tokens += pred_tokens.len();

        let (tp, fp, fn_, _pos_match) = evaluate_tokens(&gold_sentence.tokens, &pred_tokens);

        result.true_positives += tp;
        result.false_positives += fp;
        result.false_negatives += fn_;

        // 문장 완전 일치 확인
        if gold_sentence.tokens.len() == pred_tokens.len() && tp == gold_sentence.tokens.len() {
            result.exact_match_sentences += 1;
        }

        // 품사별 통계 업데이트
        for (i, gold_token) in gold_sentence.tokens.iter().enumerate() {
            let pos_stat = result.pos_stats.entry(gold_token.pos.clone()).or_default();

            pos_stat.gold_count += 1;

            if i < pred_tokens.len() {
                let pred_token = &pred_tokens[i];
                if gold_token.surface == pred_token.surface {
                    pos_stat.pred_count += 1;
                    if gold_token.pos == pred_token.pos {
                        pos_stat.correct += 1;
                    }
                }
            }
        }
    }

    // 메트릭 계산
    let total_tokens = result.total_gold_tokens;
    if total_tokens > 0 {
        result.token_accuracy = result.true_positives as f64 / total_tokens as f64;
    }

    if result.total_sentences > 0 {
        result.sentence_accuracy =
            result.exact_match_sentences as f64 / result.total_sentences as f64;
    }

    let total_pred = result.total_pred_tokens;
    if total_pred > 0 {
        result.precision = result.true_positives as f64 / total_pred as f64;
    }

    if total_tokens > 0 {
        result.recall = result.true_positives as f64 / total_tokens as f64;
    }

    if result.precision + result.recall > 0.0 {
        result.f1_score =
            2.0 * (result.precision * result.recall) / (result.precision + result.recall);
    }

    // 품사 정확도
    let mut total_pos_correct = 0;
    let mut total_pos_gold = 0;

    for pos_stat in result.pos_stats.values_mut() {
        if pos_stat.gold_count > 0 {
            pos_stat.accuracy = pos_stat.correct as f64 / pos_stat.gold_count as f64;
        }
        total_pos_correct += pos_stat.correct;
        total_pos_gold += pos_stat.gold_count;
    }

    if total_pos_gold > 0 {
        result.pos_accuracy = total_pos_correct as f64 / total_pos_gold as f64;
    }

    result
}

/// 세종 코퍼스 호환 모드로 데이터셋 평가 (strict 기본).
///
/// `MeCab-Ko`의 복합 태그(VV+EF 등)를 세종 코퍼스 형식으로 변환하여 평가합니다.
/// 이를 통해 토큰화 기준 차이를 보정하고 더 공정한 정확도를 측정합니다.
#[must_use]
pub fn evaluate_dataset_sejong(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
) -> EvaluationResult {
    evaluate_dataset_sejong_with_pos_match(tokenizer, dataset, pos_eq_strict)
}

/// 세종 호환 모드 평가 (lenient, Sprint 125).
///
/// `pos_tags_equivalent`을 사용하여 동치 태그 그룹을 동일하게 취급.
#[must_use]
pub fn evaluate_dataset_sejong_lenient(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
) -> EvaluationResult {
    evaluate_dataset_sejong_with_pos_match(tokenizer, dataset, pos_tags_equivalent)
}

/// 세종 호환 모드 평가 (POS 비교 함수 주입, Sprint 125).
///
/// strict / lenient 둘 다 지원. 자세한 내용은 `evaluate_dataset_sejong`,
/// `evaluate_dataset_sejong_lenient` 참고.
#[allow(clippy::cast_precision_loss)]
pub fn evaluate_dataset_sejong_with_pos_match(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
    pos_eq: PosMatchFn,
) -> EvaluationResult {
    let converter = SejongConverter::new();
    let mut result = EvaluationResult::new();
    result.total_sentences = dataset.len();

    for gold_sentence in &dataset.sentences {
        let pred_tokens = tokenizer.tokenize(&gold_sentence.text);
        let sejong_tokens = converter.convert_tokens(&pred_tokens);

        let converted_pred: Vec<Token> = sejong_tokens
            .iter()
            .map(|st| Token {
                surface: SejongConverter::normalize_jamo(&st.surface),
                pos: st.pos.clone(),
                start_pos: st.start_pos,
                end_pos: st.end_pos,
                start_byte: 0,
                end_byte: 0,
                reading: None,
                lemma: None,
                cost: 0,
                features: String::new(),
                normalized: None,
            })
            .collect();

        result.total_gold_tokens += gold_sentence.tokens.len();
        result.total_pred_tokens += converted_pred.len();

        let (tp, fp, fn_, _pos_match) = evaluate_tokens_aligned_with_pos_match(
            &gold_sentence.tokens,
            &converted_pred,
            pos_eq,
        );

        result.true_positives += tp;
        result.false_positives += fp;
        result.false_negatives += fn_;

        if gold_sentence.tokens.len() == converted_pred.len() && tp == gold_sentence.tokens.len() {
            result.exact_match_sentences += 1;
        }

        for (i, gold_token) in gold_sentence.tokens.iter().enumerate() {
            let pos_stat = result
                .pos_stats
                .entry(gold_token.pos.clone())
                .or_insert_with(|| PosStats {
                    gold_count: 0,
                    pred_count: 0,
                    correct: 0,
                    accuracy: 0.0,
                });
            pos_stat.gold_count += 1;

            if i < converted_pred.len() {
                let pred_token = &converted_pred[i];
                if gold_token.surface == pred_token.surface {
                    pos_stat.pred_count += 1;
                    if pos_eq(&gold_token.pos, &pred_token.pos) {
                        pos_stat.correct += 1;
                    }
                }
            }
        }
    }

    let total_tokens = result.total_gold_tokens;
    if total_tokens > 0 {
        result.token_accuracy = result.true_positives as f64 / total_tokens as f64;
    }

    if result.total_sentences > 0 {
        result.sentence_accuracy =
            result.exact_match_sentences as f64 / result.total_sentences as f64;
    }

    let total_pred = result.total_pred_tokens;
    if total_pred > 0 {
        result.precision = result.true_positives as f64 / total_pred as f64;
    }

    if total_tokens > 0 {
        result.recall = result.true_positives as f64 / total_tokens as f64;
    }

    if result.precision + result.recall > 0.0 {
        result.f1_score =
            2.0 * (result.precision * result.recall) / (result.precision + result.recall);
    }

    let mut total_pos_correct = 0;
    let mut total_pos_gold = 0;

    for pos_stat in result.pos_stats.values_mut() {
        if pos_stat.gold_count > 0 {
            pos_stat.accuracy = pos_stat.correct as f64 / pos_stat.gold_count as f64;
        }
        total_pos_correct += pos_stat.correct;
        total_pos_gold += pos_stat.gold_count;
    }

    if total_pos_gold > 0 {
        result.pos_accuracy = total_pos_correct as f64 / total_pos_gold as f64;
    }

    result
}

/// POS 태그 동치 그룹 — Conservative (Sprint 125 + 126).
///
/// 같은 그룹 내 태그는 lenient 평가 시 동일한 것으로 간주됩니다.
/// **언어학적으로 명백한 표기/관용 차이만 포함** — 의미적으로 다른 태그는 제외.
///
/// 출처:
/// - Sprint 124 Phase 1: 구두점/괄호/관형사 (KLUE 세분 vs mecab 통합)
/// - Sprint 126 P1: SL↔NNP (영문 약어 convention, KLUE는 SL, mecab은 NNP)
///
/// **포함하지 않는 그룹** (의미적으로 다른 태그):
/// - NNG/NNP — 일반/고유명사 분류는 mecab의 real error로 분류됨 (NNG↔NNP 242건)
/// - VV/VA — 동사/형용사 진짜 구분
/// - EC/EF — 연결/종결어미 진짜 구분
/// - NNB/NNG — Counter words(씨/명/회/일) convention. `_PRACTICAL` 그룹 참조
pub const TAG_EQUIVALENCE_GROUPS: &[&[&str]] = &[
    &["SP", "SC"],
    &["SS", "SY", "SSO", "SSC"],
    &["MM", "MMD", "MMN", "MMA"],
    &["SL", "NNP"],
];

/// POS 태그 동치 그룹 — Practical (Sprint 126 P1).
///
/// Conservative 그룹에 **counter words 관용 차이**를 추가합니다.
/// 언어학적으로는 NNB(의존명사)와 NNG(일반명사)가 다른 범주이지만, 실제
/// 데이터에서는 KLUE-vs-mecab의 NNB↔NNG 차이의 절대 다수가 counter words
/// (씨, 명, 회, 일, 달러 등)에 대한 convention 차이임이 Sprint 126 P1 분석으로
/// 입증됨 (158건 / NNB→NNG 케이스).
///
/// **Trade-off**: 진짜 NNB/NNG 의미적 분류 오류도 함께 흡수됨.
/// 검색/검색 인덱싱 등 downstream 사용에는 NNB/NNG 구분이 중요하지 않은 경우가
/// 많아 practical mode가 유용. 정밀한 형태소 분석 평가에는 conservative 권장.
pub const TAG_EQUIVALENCE_GROUPS_PRACTICAL: &[&[&str]] = &[
    &["SP", "SC"],
    &["SS", "SY", "SSO", "SSC"],
    &["MM", "MMD", "MMN", "MMA"],
    &["SL", "NNP"],
    &["NNB", "NNG"],
];

/// 두 POS 태그가 conservative 동치 그룹 기준 동일한지 확인 (Sprint 125+126).
///
/// `a == b`이거나 같은 `TAG_EQUIVALENCE_GROUPS` 그룹에 속하면 true.
#[must_use]
pub fn pos_tags_equivalent(a: &str, b: &str) -> bool {
    pos_tags_equivalent_in(a, b, TAG_EQUIVALENCE_GROUPS)
}

/// 두 POS 태그가 practical 동치 그룹 기준 동일한지 확인 (Sprint 126 P1).
///
/// Conservative + counter words(NNB/NNG) convention 흡수.
#[must_use]
pub fn pos_tags_equivalent_practical(a: &str, b: &str) -> bool {
    pos_tags_equivalent_in(a, b, TAG_EQUIVALENCE_GROUPS_PRACTICAL)
}

/// 주어진 그룹 집합에서 두 태그의 동치 여부 확인 (내부 헬퍼).
#[must_use]
fn pos_tags_equivalent_in(a: &str, b: &str, groups: &[&[&str]]) -> bool {
    if a == b {
        return true;
    }
    groups
        .iter()
        .any(|group| group.contains(&a) && group.contains(&b))
}

/// POS 일치 판단 함수 타입 (Sprint 125).
///
/// 평가 함수에 주입하여 strict(`==`) 또는 lenient(`pos_tags_equivalent`)
/// 모드를 선택할 수 있게 합니다.
pub type PosMatchFn = fn(&str, &str) -> bool;

/// 엄격(strict) POS 비교 — 기본 동작.
#[must_use]
pub fn pos_eq_strict(a: &str, b: &str) -> bool {
    a == b
}

/// 이중 메트릭 평가 결과 (Sprint 124)
///
/// 형태소 레벨과 어절 레벨 정확도를 함께 보고합니다.
/// - **Morpheme-level**: 개별 형태소의 surface + POS 일치
/// - **Eojeol-level**: 한 어절 내 모든 형태소가 정확해야 어절 정답
///
/// 어절 단위 평가는 KLUE DP처럼 어절 정보가 포함된 데이터셋에서만 의미 있음.
#[derive(Debug, Clone)]
pub struct DualMetricResult {
    /// 형태소 레벨 평가 결과 (기존 `evaluate_dataset_sejong`와 동일)
    pub morpheme: EvaluationResult,
    /// 어절 단위 정답 개수
    pub eojeol_correct: usize,
    /// 어절 단위 전체 개수
    pub eojeol_total: usize,
    /// 어절 정확도 (0.0 ~ 1.0). 어절 정보 없는 데이터셋에서는 0.0.
    pub eojeol_accuracy: f64,
}

impl DualMetricResult {
    /// 포맷된 보고서 생성
    #[must_use]
    pub fn format_report(&self) -> String {
        use std::fmt::Write;
        let mut report = self.morpheme.format_report();
        report.push('\n');
        report.push_str("=== 어절 레벨 (Eojeol-level) ===\n");
        if self.eojeol_total > 0 {
            writeln!(
                report,
                "Eojeol Accuracy: {:.1}% ({} / {})",
                self.eojeol_accuracy * 100.0,
                self.eojeol_correct,
                self.eojeol_total
            )
            .unwrap();
        } else {
            report.push_str("어절 정보 없음 (legacy 2-column TSV)\n");
        }
        report
    }
}

/// 이중 메트릭 평가 (strict 모드, 기본).
///
/// `evaluate_dataset_dual_with_pos_match`를 `pos_eq_strict`으로 호출.
#[must_use]
pub fn evaluate_dataset_dual(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
) -> DualMetricResult {
    evaluate_dataset_dual_with_pos_match(tokenizer, dataset, pos_eq_strict)
}

/// 이중 메트릭 평가 (lenient 모드, Sprint 125).
///
/// `pos_tags_equivalent`을 사용하여 동치 태그 그룹(SP/SC, SS/SY/SSO/SSC,
/// MM/MMD/MMN/MMA)을 동일하게 취급합니다. KLUE DP 같은 외부 코퍼스의
/// tag scheme 차이를 흡수하여 진짜 분석 정확도를 측정.
#[must_use]
pub fn evaluate_dataset_dual_lenient(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
) -> DualMetricResult {
    evaluate_dataset_dual_with_pos_match(tokenizer, dataset, pos_tags_equivalent)
}

/// 이중 메트릭 평가 (POS 비교 함수 주입).
///
/// 형태소 레벨(morpheme) + 어절 레벨(eojeol) 두 메트릭을 함께 측정합니다.
/// `pos_eq` 함수로 strict/lenient 모드를 선택. **양쪽 메트릭 모두에 동일 `pos_eq`** 적용.
///
/// 어절 레벨 평가:
/// - 정답 데이터셋에 `eojeol_counts`가 있어야 측정 가능
/// - 예측 토큰을 정답 어절 경계 기준 슬라이스로 분할 (정답과 같은 형태소 수)
/// - 어절 내 모든 형태소가 surface + POS 일치(주입된 `pos_eq` 기준) 시 어절 정답
///
/// 어절 정보가 없는 데이터셋에서는 `eojeol_total = 0`으로 보고.
#[allow(clippy::cast_precision_loss)]
pub fn evaluate_dataset_dual_with_pos_match(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
    pos_eq: PosMatchFn,
) -> DualMetricResult {
    // 형태소 레벨도 동일 pos_eq로 측정 (Sprint 125: lenient 지원)
    let morpheme = evaluate_dataset_sejong_with_pos_match(tokenizer, dataset, pos_eq);

    // 어절 레벨 별도 측정 (pos_eq 적용)
    let converter = SejongConverter::new();
    let mut eojeol_correct: usize = 0;
    let mut eojeol_total: usize = 0;

    for gold_sentence in &dataset.sentences {
        let Some(counts) = &gold_sentence.eojeol_counts else {
            continue;
        };

        let pred_raw = tokenizer.tokenize(&gold_sentence.text);
        let pred_sejong = converter.convert_tokens(&pred_raw);

        let pred_morphs: Vec<(String, String)> = pred_sejong
            .iter()
            .map(|t| {
                (
                    SejongConverter::normalize_jamo(&t.surface),
                    t.pos.clone(),
                )
            })
            .collect();

        let mut gold_idx = 0;
        let mut pred_idx = 0;

        for &count in counts {
            eojeol_total += 1;

            let gold_end = gold_idx + count;
            let pred_end = pred_idx + count;

            if gold_end > gold_sentence.tokens.len() || pred_end > pred_morphs.len() {
                gold_idx = gold_end.min(gold_sentence.tokens.len());
                pred_idx = pred_end.min(pred_morphs.len());
                continue;
            }

            let gold_slice = &gold_sentence.tokens[gold_idx..gold_end];
            let pred_slice = &pred_morphs[pred_idx..pred_end];

            let matches = gold_slice
                .iter()
                .zip(pred_slice.iter())
                .all(|(g, (p_surf, p_pos))| g.surface == *p_surf && pos_eq(&g.pos, p_pos));

            if matches {
                eojeol_correct += 1;
            }

            gold_idx = gold_end;
            pred_idx = pred_end;
        }
    }

    let eojeol_accuracy = if eojeol_total > 0 {
        eojeol_correct as f64 / eojeol_total as f64
    } else {
        0.0
    };

    DualMetricResult {
        morpheme,
        eojeol_correct,
        eojeol_total,
        eojeol_accuracy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_tags_equivalent_strict_match() {
        assert!(pos_tags_equivalent("NNG", "NNG"));
        assert!(pos_tags_equivalent("VV", "VV"));
    }

    #[test]
    fn test_pos_tags_equivalent_groups() {
        // 구두점/공백 그룹
        assert!(pos_tags_equivalent("SP", "SC"));
        assert!(pos_tags_equivalent("SC", "SP"));
        // 괄호/기호 그룹
        assert!(pos_tags_equivalent("SS", "SY"));
        assert!(pos_tags_equivalent("SS", "SSO"));
        assert!(pos_tags_equivalent("SSC", "SY"));
        // 관형사 그룹
        assert!(pos_tags_equivalent("MM", "MMD"));
        assert!(pos_tags_equivalent("MMA", "MMN"));
    }

    #[test]
    fn test_pos_tags_equivalent_distinct() {
        // 의미적으로 다른 태그는 conservative lenient에서도 다름
        assert!(!pos_tags_equivalent("NNG", "NNP")); // real classification error
        assert!(!pos_tags_equivalent("NNG", "NNB")); // 의존명사 — practical만 동치
        assert!(!pos_tags_equivalent("VV", "VA"));
        assert!(!pos_tags_equivalent("EC", "EF"));
        // 다른 그룹 간 비동치
        assert!(!pos_tags_equivalent("SP", "SS"));
        assert!(!pos_tags_equivalent("MM", "SP"));
    }

    #[test]
    fn test_pos_tags_equivalent_sl_nnp_added_in_sprint126() {
        // Sprint 126 P1: 영문 약어 convention 흡수
        assert!(pos_tags_equivalent("SL", "NNP"));
        assert!(pos_tags_equivalent("NNP", "SL"));
    }

    #[test]
    fn test_pos_tags_equivalent_practical_includes_nnb_nng() {
        // Practical: counter words convention 흡수
        assert!(pos_tags_equivalent_practical("NNB", "NNG"));
        assert!(pos_tags_equivalent_practical("NNG", "NNB"));
        // Conservative는 여전히 NNB/NNG 구분
        assert!(!pos_tags_equivalent("NNB", "NNG"));
    }

    #[test]
    fn test_pos_tags_equivalent_practical_inherits_conservative() {
        // Practical은 conservative를 포함
        assert!(pos_tags_equivalent_practical("SP", "SC"));
        assert!(pos_tags_equivalent_practical("SS", "SSO"));
        assert!(pos_tags_equivalent_practical("MM", "MMD"));
        assert!(pos_tags_equivalent_practical("SL", "NNP"));
        // Practical도 NNG/NNP 진짜 오류는 동치 안 함
        assert!(!pos_tags_equivalent_practical("NNG", "NNP"));
    }

    #[test]
    fn test_pos_eq_strict() {
        assert!(pos_eq_strict("NNG", "NNG"));
        assert!(!pos_eq_strict("NNG", "NNP"));
        assert!(!pos_eq_strict("SP", "SC")); // strict는 동치 그룹 무시
    }

    #[test]
    fn test_gold_token_parse() {
        let token = GoldToken::parse("나/NP").unwrap();
        assert_eq!(token.surface, "나");
        assert_eq!(token.pos, "NP");

        assert!(GoldToken::parse("invalid").is_err());
        assert!(GoldToken::parse("too/many/parts").is_err());
    }

    #[test]
    fn test_gold_sentence_parse() {
        let sentence =
            GoldSentence::parse_tsv_line("나는 학생이다\t나/NP 는/JX 학생/NNG 이/VCP 다/EF")
                .unwrap();
        assert_eq!(sentence.text, "나는 학생이다");
        assert_eq!(sentence.tokens.len(), 5);
        assert_eq!(sentence.tokens[0].surface, "나");
        assert_eq!(sentence.tokens[0].pos, "NP");
    }

    #[test]
    fn test_evaluate_tokens_perfect_match() {
        let gold = vec![
            GoldToken::new("나".to_string(), "NP".to_string()),
            GoldToken::new("는".to_string(), "JX".to_string()),
        ];

        let pred = vec![
            Token {
                surface: "나".to_string(),
                pos: "NP".to_string(),
                start_pos: 0,
                end_pos: 1,
                start_byte: 0,
                end_byte: 3,
                reading: None,
                lemma: None,
                cost: 0,
                features: String::new(),
                normalized: None,
            },
            Token {
                surface: "는".to_string(),
                pos: "JX".to_string(),
                start_pos: 1,
                end_pos: 2,
                start_byte: 3,
                end_byte: 6,
                reading: None,
                lemma: None,
                cost: 0,
                features: String::new(),
                normalized: None,
            },
        ];

        let (tp, fp, fn_, _) = evaluate_tokens(&gold, &pred);
        assert_eq!(tp, 2);
        assert_eq!(fp, 0);
        assert_eq!(fn_, 0);
    }

    #[test]
    fn test_evaluate_tokens_mismatch() {
        let gold = vec![
            GoldToken::new("나".to_string(), "NP".to_string()),
            GoldToken::new("는".to_string(), "JX".to_string()),
        ];

        let pred = vec![Token {
            surface: "나".to_string(),
            pos: "NP".to_string(),
            start_pos: 0,
            end_pos: 1,
            start_byte: 0,
            end_byte: 3,
            reading: None,
            lemma: None,
            cost: 0,
            features: String::new(),
            normalized: None,
        }];

        let (tp, fp, fn_, _) = evaluate_tokens(&gold, &pred);
        assert_eq!(tp, 1);
        assert_eq!(fp, 0);
        assert_eq!(fn_, 1);
    }

    #[test]
    fn test_evaluation_result_format() {
        let mut result = EvaluationResult::new();
        result.total_sentences = 10;
        result.total_gold_tokens = 50;
        result.total_pred_tokens = 48;
        result.true_positives = 45;
        result.false_positives = 3;
        result.false_negatives = 5;
        result.exact_match_sentences = 7;
        result.token_accuracy = 0.9;
        result.sentence_accuracy = 0.7;
        result.pos_accuracy = 0.92;
        result.precision = 0.9375;
        result.recall = 0.9;
        result.f1_score = 0.9184;

        let report = result.format_report();
        assert!(report.contains("테스트 문장: 10"));
        assert!(report.contains("Token Accuracy: 90.0%"));
        assert!(report.contains("F1 Score: 0.918"));
    }

    #[test]
    #[cfg(feature = "test-utils")]
    fn test_dataset_from_tsv() {
        use std::io::Write;

        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "# 주석").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "나는 학생\t나/NP 는/JX 학생/NNG").unwrap();
        writeln!(file, "오늘 날씨\t오늘/NNG 날씨/NNG").unwrap();
        file.flush().unwrap();

        let dataset = TestDataset::from_tsv(file.path()).unwrap();
        assert_eq!(dataset.len(), 2);
        assert_eq!(dataset.sentences[0].text, "나는 학생");
        assert_eq!(dataset.sentences[0].tokens.len(), 3);
        assert_eq!(dataset.sentences[1].text, "오늘 날씨");
        assert_eq!(dataset.sentences[1].tokens.len(), 2);
    }
}
