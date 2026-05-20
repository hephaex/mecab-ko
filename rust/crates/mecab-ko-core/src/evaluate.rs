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
/// `surface_eq_strict` 위임 — surface 일치는 항상 `==`.
/// surface 비교 함수 주입은 `evaluate_tokens_aligned_with_match` 사용.
#[must_use]
pub fn evaluate_tokens_aligned_with_pos_match(
    gold_tokens: &[GoldToken],
    pred_tokens: &[Token],
    pos_eq: PosMatchFn,
) -> (usize, usize, usize, usize) {
    evaluate_tokens_aligned_with_match(gold_tokens, pred_tokens, pos_eq, surface_eq_strict)
}

/// Greedy alignment 기반 토큰 평가 (POS + surface 비교 함수 주입, Sprint 128 P2).
///
/// 순서를 고려하되, 토큰 갯수 차이가 있어도 최선의 매칭을 시도합니다.
/// `pos_eq`로 strict/lenient POS, `surface_eq`로 strict/canonical surface 모드 선택.
///
/// # Returns
///
/// (`true_positives`, `false_positives`, `false_negatives`, `pos_match`)
#[must_use]
pub fn evaluate_tokens_aligned_with_match(
    gold_tokens: &[GoldToken],
    pred_tokens: &[Token],
    pos_eq: PosMatchFn,
    surface_eq: SurfaceMatchFn,
) -> (usize, usize, usize, usize) {
    let mut true_positives = 0;
    let mut pos_match = 0;

    let mut gold_idx = 0;
    let mut pred_idx = 0;

    while gold_idx < gold_tokens.len() && pred_idx < pred_tokens.len() {
        let gold = &gold_tokens[gold_idx];
        let pred = &pred_tokens[pred_idx];

        if surface_eq(&gold.surface, &pred.surface) {
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
                    && surface_eq(&gold.surface, &pred_tokens[pred_idx + look_ahead].surface)
                {
                    pred_idx += look_ahead;
                    found = true;
                    break;
                }
            }

            if !found {
                for look_ahead in 1..=3 {
                    if gold_idx + look_ahead < gold_tokens.len()
                        && surface_eq(&gold_tokens[gold_idx + look_ahead].surface, &pred.surface)
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
/// `surface_eq_strict` 위임. surface 비교까지 주입하려면 `_with_match` 사용.
pub fn evaluate_dataset_sejong_with_pos_match(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
    pos_eq: PosMatchFn,
) -> EvaluationResult {
    evaluate_dataset_sejong_with_match(tokenizer, dataset, pos_eq, surface_eq_strict)
}

/// 세종 호환 모드 평가 (POS + surface 비교 함수 주입, Sprint 128 P2).
///
/// `pos_eq`와 `surface_eq` 양쪽 모두 주입 가능.
/// strict / lenient (POS) × strict / canonical (surface) 조합 모두 지원.
#[allow(clippy::cast_precision_loss)]
pub fn evaluate_dataset_sejong_with_match(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
    pos_eq: PosMatchFn,
    surface_eq: SurfaceMatchFn,
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

        let (tp, fp, fn_, _pos_match) = evaluate_tokens_aligned_with_match(
            &gold_sentence.tokens,
            &converted_pred,
            pos_eq,
            surface_eq,
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
                if surface_eq(&gold_token.surface, &pred_token.surface) {
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

/// POS 태그 동치 그룹 — Practical (Sprint 126 P1, Sprint 136 P3 extension).
///
/// Conservative 그룹에 **counter words 관용 차이**를 추가합니다.
/// 언어학적으로는 NNB(의존명사)와 NNG(일반명사)가 다른 범주이지만, 실제
/// 데이터에서는 KLUE-vs-mecab의 NNB↔NNG 차이의 절대 다수가 counter words
/// (씨, 명, 회, 일, 달러 등)에 대한 convention 차이임이 Sprint 126 P1 분석으로
/// 입증됨 (158건 / NNB→NNG 케이스).
///
/// **Sprint 136 P3**: VA↔VV 동치 추가. "있다"의 VA(KLUE) vs VV(mecab)
/// convention 차이가 KLUE DP 41건. 한국어 문법에서 "있다"의 형용사적/동사적
/// 존재 분류는 진행 중인 논쟁이며 두 코퍼스의 convention 차이를 흡수.
///
/// **Sprint 147 A**: VV↔XSV 동치 추가. "했/됐"의 mecab(VV+EP) vs KLUE/UD gold
/// (하/XSV + 였/EP, 되/XSV + 었/EP) POS scheme 차이 흡수. mecab은 "하" 단독
/// stem을 VV로 분류, gold는 XSV(동사파생접사)로 분류 — 분류 convention 차이.
///
/// **Trade-off**: 진짜 NNB/NNG, VA/VV, VV/XSV 의미적 분류 오류도 함께 흡수됨.
/// 검색/색인 등 downstream 사용에는 이 구분이 중요하지 않은 경우가
/// 많아 practical mode가 유용. 정밀한 형태소 분석 평가에는 conservative 권장.
pub const TAG_EQUIVALENCE_GROUPS_PRACTICAL: &[&[&str]] = &[
    &["SP", "SC"],
    &["SS", "SY", "SSO", "SSC"],
    &["MM", "MMD", "MMN", "MMA"],
    &["SL", "NNP"],
    &["NNB", "NNG"],
    &["VA", "VV", "XSV"],
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

/// Surface 일치 판단 함수 타입 (Sprint 128 P2).
///
/// 평가 함수에 주입하여 strict(`==`) 또는 canonical(자모 통일) 모드를 선택.
/// KLUE DP처럼 morpheme surface가 jamo decomposition convention으로
/// 통일되지 않은 외부 코퍼스의 표기 차이를 흡수합니다.
pub type SurfaceMatchFn = fn(&str, &str) -> bool;

/// 엄격(strict) surface 비교 — 기본 동작.
#[must_use]
pub fn surface_eq_strict(a: &str, b: &str) -> bool {
    a == b
}

/// Canonical surface 비교 (Sprint 128 P2).
///
/// 양쪽 문자열을 fully decompose 후 다시 compose하여 자모/음절 표기 차이를 흡수.
/// 예: "한" (U+D55C) vs "하ㄴ" (U+D558 + U+3134) → 둘 다 "한"으로 정규화 후 비교.
///
/// Sprint 127 P1 분석에서 KLUE의 morpheme surface가 음절 보존(예: "한")인 반면
/// mecab은 어미 분해로 음절+자모 혼합("하"+"ㄴ")이 자주 발생함을 확인. 본 함수는
/// 이 표기 차이를 의미 손실 없이 흡수.
#[must_use]
pub fn surface_eq_canonical(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    canonical_form(a) == canonical_form(b)
}

/// Canonical + inflectional ending normalization (Sprint 128 P2 + Sprint 134 P3).
///
/// `surface_eq_canonical` + 어미 변환 동치:
/// - 하았 ↔ 하였 (Sprint 128: KLUE는 "였" 보존, mecab은 "았"으로 분해)
/// - 하어 ↔ 하여 (Sprint 128: KLUE는 "여" 보존, mecab은 "어"로 분해)
/// - 하아 ↔ 하여 (Sprint 134: 편하아요 vs 편하어요 — gold도 아 분해 케이스)
/// - 이습니다 → 입니다 (Sprint 134: mecab의 "이/VCP + 습니다/EF" 분해를 KLUE의
///   composed "X입니다."와 일치시킴; 본 normalization에서 가장 많은 흡수 패턴)
///
/// Sprint 128: `SURFACE_MISMATCH`의 22.6% 흡수.
/// Sprint 134: 추가 ~4-5% 흡수 (이습니다 패턴 ~80 cases + 하아 ~12 cases).
#[must_use]
pub fn surface_eq_canonical_lenient(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let a_can = canonical_form(a);
    let b_can = canonical_form(b);
    if a_can == b_can {
        return true;
    }
    normalize_endings(&a_can) == normalize_endings(&b_can)
}

/// 자모 ↔ 음절 표기를 canonical form으로 통일.
fn canonical_form(s: &str) -> String {
    use mecab_ko_hangul::{compose_str, decompose_str};
    compose_str(&decompose_str(s))
}

/// 어미 변환 동치 (Sprint 128 P2 + Sprint 134 P3 + Sprint 136 P3a).
///
/// 변환 규칙:
/// - 하았 → 하였 (Sprint 128)
/// - 하어 → 하여 (Sprint 128)
/// - 하아 → 하여 (Sprint 134: 편하아요 vs 편하어요 정규화)
/// - 이습니다 → 입니다 (Sprint 134: mecab "이/VCP+습니다/EF" 분해 흡수)
/// - ㄹ불규칙 활용 (Sprint 136 P3a): 따르아→따라, 모르아→몰라 등.
///   mecab은 "따르/VV + 아/EC" → "따르아"로 분해하나 KLUE는 활용된
///   "따라"를 보존. 단방향 정규화 (mecab → KLUE).
fn normalize_endings(s: &str) -> String {
    // Step 1: char-pair 변환 (하았/하어/하아)
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    for (i, &c) in chars.iter().enumerate() {
        let prev = if i > 0 { chars[i - 1] } else { '\0' };
        if prev == '하' && (c == '았' || c == '아') {
            // 하았 → 하였, 하아 → 하여 (둘 다 하여로 통일)
            out.push(if c == '았' { '였' } else { '여' });
        } else if c == '어' && prev == '하' {
            out.push('여');
        } else {
            out.push(c);
        }
    }

    // Step 2: 다중-char 패턴 (이습니다 → 입니다)
    // mecab의 "이/VCP + 습니다/EF" 분해를 KLUE의 "입니다"와 일치시킴.
    // 종결어미 위치(문장 끝)에서만 의미 있지만 전역 치환 — "이습니다"가
    // 다른 형태소 조합으로 자연 발생할 가능성은 매우 낮음.
    if out.contains("이습니다") {
        out = out.replace("이습니다", "입니다");
    }

    // Step 3: ㄹ불규칙 활용 (Sprint 136 P3a)
    // 어간 + 아/어 결합 시 르 → ㄹ + 라/러 활용. mecab은 어간 분해, KLUE는 활용형 보존.
    // 보수적으로 명시 목록만 처리 (자동 음절 분해 시 false positive 위험).
    // 모음조화: ㅏ/ㅗ → 아 → 라, 그 외 → 어 → 러.
    for (from, to) in R_IRREGULAR_PATTERNS {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }

    out
}

/// ㄹ불규칙 동사 활용 단방향 정규화 패턴 (Sprint 136 P3a).
///
/// mecab의 어간 분해 표기 → KLUE의 활용형 표기.
/// 명시 목록만 사용하여 false positive 방지 (예: 일반 음절 sequence "X르Y"가
/// 우연히 매칭되는 것을 피함).
const R_IRREGULAR_PATTERNS: &[(&str, &str)] = &[
    ("따르아", "따라"),  // 따르다 + 아
    ("모르아", "몰라"),  // 모르다 + 아
    ("다르아", "달라"),  // 다르다 + 아
    ("부르어", "불러"),  // 부르다 + 어
    ("흐르어", "흘러"),  // 흐르다 + 어
    ("오르아", "올라"),  // 오르다 + 아
    ("자르아", "잘라"),  // 자르다 + 아
    ("누르어", "눌러"),  // 누르다 + 어
    ("고르아", "골라"),  // 고르다 + 아
];

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

/// 이중 메트릭 평가 (POS 비교 함수 주입, Sprint 125).
///
/// `surface_eq_strict` 위임. surface까지 주입은 `_with_match` 사용.
#[must_use]
pub fn evaluate_dataset_dual_with_pos_match(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
    pos_eq: PosMatchFn,
) -> DualMetricResult {
    evaluate_dataset_dual_with_match(tokenizer, dataset, pos_eq, surface_eq_strict)
}

/// 이중 메트릭 평가 — **per-eojeol** 알고리즘 (Sprint 128 P1+P2).
///
/// 어절 정확도 측정에 **어절별 독립 토크나이즈** 알고리즘을 사용. Sprint 127 P1
/// 분석에서 sequence-based eojeol metric은 cascade로 33pp 이상 underestimate함이
/// 입증됨 (KLUE DP eojeol 19.2% sequence vs 52.4% per-eojeol).
///
/// Algorithm:
/// 1. `text.split_whitespace()` → 어절 리스트 (`eojeol_counts.len()`와 같아야 함)
/// 2. 각 어절: gold morphs = `tokens[gold_idx..+count_g]`,
///    pred morphs = `tokenize(eojeol).convert`
/// 3. surface concat 일치 후 morpheme별 (`surface_eq`, `pos_eq`) 비교
/// 4. cascade 없음 — 한 어절 mismatch가 다음 어절 카운팅에 영향 안 줌
///
/// Trade-off vs sequence: mecab의 cross-eojeol Viterbi context를 잃음. 한국어
/// 형태소 분석에서 어절 경계 너머 영향은 작아 분석 비교에 적합.
///
/// `morpheme` 부분은 기존 sequence-based `evaluate_dataset_sejong_with_match` 사용.
#[allow(clippy::cast_precision_loss)]
pub fn evaluate_dataset_dual_per_eojeol_with_match(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
    pos_eq: PosMatchFn,
    surface_eq: SurfaceMatchFn,
) -> DualMetricResult {
    let morpheme = evaluate_dataset_sejong_with_match(tokenizer, dataset, pos_eq, surface_eq);

    let converter = SejongConverter::new();
    let mut eojeol_correct: usize = 0;
    let mut eojeol_total: usize = 0;

    for gold_sentence in &dataset.sentences {
        let Some(eojeol_counts) = &gold_sentence.eojeol_counts else {
            continue;
        };
        let eojeols: Vec<&str> = gold_sentence.text.split_whitespace().collect();
        if eojeols.len() != eojeol_counts.len() {
            continue;
        }

        let mut gold_idx: usize = 0;
        for (eo_i, &count_g) in eojeol_counts.iter().enumerate() {
            eojeol_total += 1;
            if gold_idx + count_g > gold_sentence.tokens.len() {
                gold_idx = gold_sentence.tokens.len();
                continue;
            }
            let gold_slice = &gold_sentence.tokens[gold_idx..gold_idx + count_g];
            gold_idx += count_g;

            let pred_raw = tokenizer.tokenize(eojeols[eo_i]);
            let pred_sejong = converter.convert_tokens(&pred_raw);
            let pred_morphs: Vec<(String, String)> = pred_sejong
                .iter()
                .map(|t| (SejongConverter::normalize_jamo(&t.surface), t.pos.clone()))
                .collect();

            // Surface concat lenient: surface_eq with concat
            let gold_concat: String = gold_slice.iter().map(|t| t.surface.as_str()).collect();
            let pred_concat: String = pred_morphs.iter().map(|(s, _)| s.as_str()).collect();
            if !surface_eq(&gold_concat, &pred_concat) {
                continue;
            }

            // Same surface (under surface_eq). Now require same split + per-morph match.
            if gold_slice.len() != pred_morphs.len() {
                continue;
            }
            let all_match = gold_slice
                .iter()
                .zip(pred_morphs.iter())
                .all(|(g, (ps, pp))| surface_eq(&g.surface, ps) && pos_eq(&g.pos, pp));
            if all_match {
                eojeol_correct += 1;
            }
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

/// 이중 메트릭 평가 — per-eojeol 어절 + strict POS/surface (편의 함수).
#[must_use]
pub fn evaluate_dataset_dual_per_eojeol(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
) -> DualMetricResult {
    evaluate_dataset_dual_per_eojeol_with_match(
        tokenizer,
        dataset,
        pos_eq_strict,
        surface_eq_strict,
    )
}

/// 어절 surface-only 메트릭 결과 (Sprint 133 P2).
///
/// **Use case**: 검색/인덱싱 use case 전용. POS와 형태소 split을 무시하고
/// surface 문자열 일치만으로 정답 판정. 색인 빌드 또는 부분 일치 검색
/// 시스템의 정확도 추정에 사용.
///
/// **의미 손실**: 형태소 분석 품질은 측정하지 않음. 빈도/품사/동의어
/// 처리를 다운스트림에서 사용한다면 본 메트릭은 부적합.
#[derive(Debug, Clone)]
pub struct EojeolSurfaceResult {
    /// 어절 surface 일치 개수
    pub correct: usize,
    /// 어절 전체 개수
    pub total: usize,
    /// 어절 surface 정확도 (0.0 ~ 1.0)
    pub accuracy: f64,
}

impl EojeolSurfaceResult {
    /// 포맷된 보고서 생성
    #[must_use]
    pub fn format_report(&self) -> String {
        if self.total > 0 {
            format!(
                "Eojeol Surface-only Accuracy: {:.1}% ({} / {})",
                self.accuracy * 100.0,
                self.correct,
                self.total
            )
        } else {
            "어절 정보 없음 (legacy 2-column TSV)".to_string()
        }
    }
}

/// 어절 surface-only 평가 (Sprint 133 P2, 검색/인덱싱 use case).
///
/// 어절의 모든 형태소 surface를 concat한 결과가 `surface_eq`로 비교 시 일치하면
/// 정답. POS 태그와 inner split boundary는 무시.
///
/// **Trade-off**: 형태소 분석 품질 손실. 다음 용도에만 사용:
/// - 검색 색인 빌드 (어절 surface 보존이 중요, POS 무관)
/// - 부분 일치 검색 baseline
/// - Sprint 127 P1의 87.7% ceiling 같은 천장 추정
///
/// 형태소 분석 정확도가 필요하면 `evaluate_dataset_dual_per_eojeol_with_match`
/// 또는 `evaluate_dataset_sejong_with_match` 사용.
///
/// Algorithm (per-eojeol, no cascade):
/// 1. `text.split_whitespace` → 어절 리스트 (`eojeol_counts.len()`과 일치 필요)
/// 2. 각 어절: gold morphs surface concat + 어절별 토크나이즈 후 pred surface concat
/// 3. `surface_eq`로 비교 (strict / canonical / `canonical_lenient` 주입 가능)
#[allow(clippy::cast_precision_loss)]
pub fn evaluate_dataset_eojeol_surface_only_with_match(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
    surface_eq: SurfaceMatchFn,
) -> EojeolSurfaceResult {
    let converter = SejongConverter::new();
    let mut correct: usize = 0;
    let mut total: usize = 0;

    for gold_sentence in &dataset.sentences {
        let Some(eojeol_counts) = &gold_sentence.eojeol_counts else {
            continue;
        };
        let eojeols: Vec<&str> = gold_sentence.text.split_whitespace().collect();
        if eojeols.len() != eojeol_counts.len() {
            continue;
        }

        let mut gold_idx: usize = 0;
        for (eo_i, &count_g) in eojeol_counts.iter().enumerate() {
            total += 1;
            if gold_idx + count_g > gold_sentence.tokens.len() {
                gold_idx = gold_sentence.tokens.len();
                continue;
            }
            let gold_slice = &gold_sentence.tokens[gold_idx..gold_idx + count_g];
            gold_idx += count_g;

            let gold_concat: String = gold_slice.iter().map(|t| t.surface.as_str()).collect();

            let pred_raw = tokenizer.tokenize(eojeols[eo_i]);
            let pred_sejong = converter.convert_tokens(&pred_raw);
            let pred_concat: String = pred_sejong
                .iter()
                .map(|t| SejongConverter::normalize_jamo(&t.surface))
                .collect();

            if surface_eq(&gold_concat, &pred_concat) {
                correct += 1;
            }
        }
    }

    let accuracy = if total > 0 {
        correct as f64 / total as f64
    } else {
        0.0
    };

    EojeolSurfaceResult {
        correct,
        total,
        accuracy,
    }
}

/// 어절 surface-only 평가 (strict, 편의 함수).
///
/// `surface_eq_strict` 위임. canonical / lenient는 `_with_match` 사용.
#[must_use]
pub fn evaluate_dataset_eojeol_surface_only(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
) -> EojeolSurfaceResult {
    evaluate_dataset_eojeol_surface_only_with_match(tokenizer, dataset, surface_eq_strict)
}

/// 이중 메트릭 평가 (POS + surface 비교 함수 주입, Sprint 128 P2).
///
/// 형태소 레벨(morpheme) + 어절 레벨(eojeol) 두 메트릭을 함께 측정합니다.
/// `pos_eq`와 `surface_eq` 함수로 strict/lenient/canonical 모드를 선택.
/// **양쪽 메트릭 모두에 동일한 `pos_eq`/`surface_eq`** 적용.
///
/// 어절 레벨 평가:
/// - 정답 데이터셋에 `eojeol_counts`가 있어야 측정 가능
/// - 예측 토큰을 정답 어절 경계 기준 슬라이스로 분할 (정답과 같은 형태소 수)
/// - 어절 내 모든 형태소가 `surface_eq` + `pos_eq` 일치 시 어절 정답
///
/// 어절 정보가 없는 데이터셋에서는 `eojeol_total = 0`으로 보고.
#[allow(clippy::cast_precision_loss)]
pub fn evaluate_dataset_dual_with_match(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
    pos_eq: PosMatchFn,
    surface_eq: SurfaceMatchFn,
) -> DualMetricResult {
    // 형태소 레벨도 동일 pos_eq + surface_eq로 측정
    let morpheme = evaluate_dataset_sejong_with_match(tokenizer, dataset, pos_eq, surface_eq);

    // 어절 레벨 별도 측정 (pos_eq + surface_eq 적용)
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
                .all(|(g, (p_surf, p_pos))| {
                    surface_eq(&g.surface, p_surf) && pos_eq(&g.pos, p_pos)
                });

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
    fn test_pos_tags_equivalent_practical_includes_va_vv() {
        // Sprint 136 P3: "있다" VA(KLUE) vs VV(mecab) convention 흡수
        assert!(pos_tags_equivalent_practical("VA", "VV"));
        assert!(pos_tags_equivalent_practical("VV", "VA"));
        // Conservative는 여전히 VA/VV 구분 (진짜 동사/형용사 분류)
        assert!(!pos_tags_equivalent("VA", "VV"));
    }

    #[test]
    fn test_pos_tags_equivalent_practical_includes_xsv() {
        // Sprint 147 A: "했/됐" mecab(VV+EP) vs gold(XSV+EP) convention 흡수
        assert!(pos_tags_equivalent_practical("VV", "XSV"));
        assert!(pos_tags_equivalent_practical("XSV", "VV"));
        assert!(pos_tags_equivalent_practical("VA", "XSV"));
        assert!(pos_tags_equivalent_practical("XSV", "VA"));
        // Conservative는 XSV 구분 유지
        assert!(!pos_tags_equivalent("VV", "XSV"));
        assert!(!pos_tags_equivalent("XSV", "VV"));
    }

    #[test]
    fn test_surface_eq_strict_basic() {
        assert!(surface_eq_strict("한", "한"));
        assert!(!surface_eq_strict("한", "하ㄴ"));
    }

    #[test]
    fn test_surface_eq_canonical_jamo_syllable_mix() {
        // 음절 + 자모 혼합 → canonical 비교에서 동일
        assert!(surface_eq_canonical("한", "하ㄴ"));
        assert!(surface_eq_canonical("함께", "하ㅁ께"));
        assert!(surface_eq_canonical("역할", "역하ㄹ"));
    }

    #[test]
    fn test_surface_eq_canonical_pure_strict_match() {
        // 동일 string은 canonical도 true
        assert!(surface_eq_canonical("한", "한"));
        assert!(surface_eq_canonical("ㄱㅏ", "ㄱㅏ"));
    }

    #[test]
    fn test_surface_eq_canonical_distinct_words() {
        // 진짜 다른 단어는 canonical도 false
        assert!(!surface_eq_canonical("한", "둘"));
        assert!(!surface_eq_canonical("것이", "게"));
    }

    #[test]
    fn test_surface_eq_canonical_lenient_endings() {
        // Sprint 128: 하았 ↔ 하였
        assert!(surface_eq_canonical_lenient("인정하였다", "인정하았다"));
        // Sprint 128: 하어 ↔ 하여
        assert!(surface_eq_canonical_lenient("등장하여", "등장하어"));
        assert!(surface_eq_canonical_lenient("통하여", "통하어"));
        // canonical 단계도 함께 적용
        assert!(surface_eq_canonical_lenient("함께", "하ㅁ께"));
    }

    #[test]
    fn test_surface_eq_canonical_lenient_does_not_overcorrect() {
        // "았"이 "하" 직후가 아니면 그대로 (false negative 방지)
        assert!(!surface_eq_canonical_lenient("먹었다", "먹였다"));
    }

    #[test]
    fn test_surface_eq_canonical_lenient_haa_to_haye() {
        // Sprint 134: 하아 ↔ 하여 (편하아요 vs 편하어요 → 둘 다 편하여요로 정규화)
        assert!(surface_eq_canonical_lenient("편하아요", "편하어요"));
        assert!(surface_eq_canonical_lenient("가능하아요", "가능하어요"));
        // 양방향 매칭 (하어 → 하여, 하아 → 하여 모두 같은 형태로 통일)
        assert!(surface_eq_canonical_lenient("말하아", "말하어"));
    }

    #[test]
    fn test_surface_eq_canonical_lenient_imnida() {
        // Sprint 134: 이습니다 → 입니다
        assert!(surface_eq_canonical_lenient("것입니다", "것이습니다"));
        assert!(surface_eq_canonical_lenient("숙소입니다", "숙소이습니다"));
        assert!(surface_eq_canonical_lenient("입니다", "이습니다"));
        // composed jamo (이ㅂ니다) → canonical → 입니다
        assert!(surface_eq_canonical_lenient("것이ㅂ니다", "것이습니다"));
    }

    #[test]
    fn test_surface_eq_canonical_lenient_r_irregular() {
        // Sprint 136 P3a: ㄹ불규칙 활용 (mecab 어간 분해 → KLUE 활용형)
        assert!(surface_eq_canonical_lenient("따라", "따르아"));
        assert!(surface_eq_canonical_lenient("따라서", "따르아서"));
        assert!(surface_eq_canonical_lenient("몰라요", "모르아요"));
        assert!(surface_eq_canonical_lenient("달라", "다르아"));
        assert!(surface_eq_canonical_lenient("불러", "부르어"));
        assert!(surface_eq_canonical_lenient("흘러", "흐르어"));
        assert!(surface_eq_canonical_lenient("올라", "오르아"));
        assert!(surface_eq_canonical_lenient("잘라", "자르아"));
        assert!(surface_eq_canonical_lenient("눌러", "누르어"));
        assert!(surface_eq_canonical_lenient("골라", "고르아"));
    }

    #[test]
    fn test_surface_eq_canonical_lenient_r_irregular_does_not_overcorrect() {
        // Sprint 136 P3a: 명시 목록 외 ㄹ-패턴은 false positive 방지
        // "푸르다"는 러불규칙(이르다와 함께) — 패턴이 다르므로 제외
        assert!(!surface_eq_canonical_lenient("푸르러", "푸르어"));
        // 명시 목록에 없는 "기르아"는 normalize 대상 아님
        assert!(!surface_eq_canonical_lenient("길러", "기르어"));
    }

    #[test]
    fn test_surface_eq_canonical_lenient_imnida_overcorrect() {
        // "이습니다"가 분리된 의미일 때는 잘 매칭되지 않아야 함
        // 그러나 분리된 단어 "이"가 "습니다" 앞에 우연히 오는 경우는 거의 없음
        // — Korean morphology에서 "이/VCP + 습니다/EF"는 사실상 표준 패턴
        // 본 테스트는 다른 음소가 그대로 유지됨을 확인
        assert!(!surface_eq_canonical_lenient("이것입니다", "그것입니다"));
        // 입니다와 이ㅁ니다(잘못된 분해)는 매칭되지 않음 (안전 boundary)
        assert!(!surface_eq_canonical_lenient("입니다", "다닙니다"));
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
    fn test_eojeol_surface_result_format_empty() {
        let result = EojeolSurfaceResult {
            correct: 0,
            total: 0,
            accuracy: 0.0,
        };
        assert!(result.format_report().contains("legacy"));
    }

    #[test]
    fn test_eojeol_surface_result_format_populated() {
        let result = EojeolSurfaceResult {
            correct: 875,
            total: 1000,
            accuracy: 0.875,
        };
        let report = result.format_report();
        assert!(report.contains("87.5%"));
        assert!(report.contains("875"));
        assert!(report.contains("1000"));
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
