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
        Self { text, tokens }
    }

    /// TSV 라인에서 파싱
    ///
    /// 형식: 원문\t정답토큰1 정답토큰2 ...
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
        if parts.len() != 2 {
            return Err(EvaluateError::Parse(format!(
                "Invalid TSV line: {line} (expected text\\ttokens)"
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

        Ok(Self { text, tokens })
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

/// Greedy alignment 기반 토큰 평가
///
/// 순서를 고려하되, 토큰 갯수 차이가 있어도 최선의 매칭을 시도합니다.
/// 예를 들어 gold와 pred의 토큰 갯수가 다르면, 건너뛰면서 매칭을 시도합니다.
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
pub fn evaluate_tokens_aligned(
    gold_tokens: &[GoldToken],
    pred_tokens: &[Token],
) -> (usize, usize, usize, usize) {
    let mut true_positives = 0;
    let mut pos_match = 0;

    let mut gold_idx = 0;
    let mut pred_idx = 0;

    while gold_idx < gold_tokens.len() && pred_idx < pred_tokens.len() {
        let gold = &gold_tokens[gold_idx];
        let pred = &pred_tokens[pred_idx];

        if gold.surface == pred.surface {
            // Surface가 일치하면 매칭
            pos_match += 1;
            if gold.pos == pred.pos {
                true_positives += 1;
            }
            gold_idx += 1;
            pred_idx += 1;
        } else {
            // Surface가 불일치하면 다음 pred에서 현재 gold를 찾아봄
            let mut found = false;

            // pred에서 최대 3개 앞까지 탐색
            for look_ahead in 1..=3 {
                if pred_idx + look_ahead < pred_tokens.len()
                    && pred_tokens[pred_idx + look_ahead].surface == gold.surface
                {
                    // 중간 pred 토큰들은 false positive로 처리
                    pred_idx += look_ahead;
                    found = true;
                    break;
                }
            }

            if !found {
                // gold에서 최대 3개 앞까지 탐색
                for look_ahead in 1..=3 {
                    if gold_idx + look_ahead < gold_tokens.len()
                        && gold_tokens[gold_idx + look_ahead].surface == pred.surface
                    {
                        // 중간 gold 토큰들은 false negative로 처리
                        gold_idx += look_ahead;
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                // 매칭 실패 - 둘 다 한 칸씩 전진
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

/// 세종 코퍼스 호환 모드로 데이터셋 평가
///
/// `MeCab-Ko`의 복합 태그(VV+EF 등)를 세종 코퍼스 형식으로 변환하여 평가합니다.
/// 이를 통해 토큰화 기준 차이를 보정하고 더 공정한 정확도를 측정합니다.
///
/// # Arguments
///
/// * `tokenizer` - `MeCab-Ko` 토크나이저
/// * `dataset` - 테스트 데이터셋
///
/// # Returns
///
/// 세종 호환 모드로 평가된 결과
#[allow(clippy::cast_precision_loss)]
pub fn evaluate_dataset_sejong(
    tokenizer: &mut Tokenizer,
    dataset: &TestDataset,
) -> EvaluationResult {
    let converter = SejongConverter::new();
    let mut result = EvaluationResult::new();
    result.total_sentences = dataset.len();

    for gold_sentence in &dataset.sentences {
        let pred_tokens = tokenizer.tokenize(&gold_sentence.text);

        // 세종 형식으로 변환
        let sejong_tokens = converter.convert_tokens(&pred_tokens);

        // 변환된 토큰을 GoldToken 형식으로 변환하여 비교
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

        let (tp, fp, fn_, _pos_match) =
            evaluate_tokens_aligned(&gold_sentence.tokens, &converted_pred);

        result.true_positives += tp;
        result.false_positives += fp;
        result.false_negatives += fn_;

        // 문장 완전 일치 확인
        if gold_sentence.tokens.len() == converted_pred.len() && tp == gold_sentence.tokens.len() {
            result.exact_match_sentences += 1;
        }

        // 품사별 통계 수집
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
                    if gold_token.pos == pred_token.pos {
                        pos_stat.correct += 1;
                    }
                }
            }
        }
    }

    // 메트릭 계산 (기존과 동일)
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

#[cfg(test)]
mod tests {
    use super::*;

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
