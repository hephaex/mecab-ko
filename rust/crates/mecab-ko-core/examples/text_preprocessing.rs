//! # Text Preprocessing Pipeline Example
//!
//! 한국어 자연어 처리(NLP)를 위한 실전 텍스트 전처리 파이프라인
//!
//! ## 주요 기능
//!
//! - **텍스트 정규화**: 공백, 특수문자, 외래어 표기 정규화
//! - **문장 분리**: 한국어 문장 경계 감지
//! - **품사 필터링**: 특정 품사만 추출 (명사, 동사 등)
//! - **불용어 제거**: 분석에 불필요한 조사, 어미 등 제거
//! - **어간 추출**: 동사/형용사의 기본형 추출
//!
//! ## 사용 사례
//!
//! - 텍스트 마이닝
//! - 감성 분석 전처리
//! - 문서 분류
//! - 키워드 추출

#![allow(
    clippy::uninlined_format_args,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::missing_errors_doc,
    clippy::unused_self,
    clippy::single_char_pattern,
    clippy::no_effect_replace,
    clippy::unnecessary_wraps,
    missing_docs
)]

use mecab_ko_core::{Normalizer, Token, Tokenizer};
use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 한국어 텍스트 전처리 파이프라인 ===\n");

    // 1. 샘플 문서
    let raw_text = "
        안녕하세요! MeCab-Ko는 한국어 형태소 분석기입니다.
        자연어처리(NLP)에서 가장 기본적인 도구죠.
        이 라이브러이는 Rust로 재작성되었습니다!!!
        성능이 매우 빠르고, 메모리 안전성도 보장됩니다.

        여러분도 한번 사용해보세요~
    ";

    println!("원본 텍스트:\n{}\n", raw_text);

    // 2. 전처리 파이프라인 초기화
    let mut pipeline = PreprocessingPipeline::new()?;

    // 3. 전체 전처리 실행
    let processed = pipeline.process(raw_text)?;

    println!("전처리 결과:\n{}\n", processed.summary());

    // 4. 개별 단계별 처리 예제
    println!("=== 단계별 처리 예제 ===\n");

    // 4-1. 텍스트 정규화
    println!("1. 텍스트 정규화:");
    let normalized = pipeline.normalize_text(raw_text);
    println!("   {}\n", normalized);

    // 4-2. 문장 분리
    println!("2. 문장 분리:");
    let sentences = pipeline.split_sentences(&normalized);
    for (i, sentence) in sentences.iter().enumerate() {
        println!("   [{}] {}", i + 1, sentence);
    }
    println!();

    // 4-3. 토큰화 및 품사 태깅
    println!("3. 토큰화 및 품사 태깅:");
    let tokens = pipeline.tokenize(&normalized)?;
    for token in tokens.iter().take(15) {
        println!("   {} / {}", token.surface, token.pos);
    }
    println!("   ... (총 {} 토큰)\n", tokens.len());

    // 4-4. 명사 추출
    println!("4. 명사만 추출:");
    let nouns = pipeline.extract_nouns(&normalized)?;
    println!("   {:?}\n", nouns);

    // 4-5. 불용어 제거
    println!("5. 불용어 제거:");
    let filtered = pipeline.remove_stopwords(&normalized)?;
    println!("   {:?}\n", filtered);

    // 4-6. 어간 추출 (기본형)
    println!("6. 어간 추출:");
    let lemmas = pipeline.extract_lemmas(&normalized)?;
    println!("   {:?}\n", lemmas);

    // 5. 실전 활용 예제
    println!("=== 실전 활용 예제 ===\n");

    // 5-1. 문서 분류를 위한 특징 추출
    println!("1. 문서 분류용 특징 벡터:");
    let features = pipeline.extract_features_for_classification(raw_text)?;
    println!("   명사: {:?}", features.nouns);
    println!("   동사: {:?}", features.verbs);
    println!("   형용사: {:?}", features.adjectives);
    println!();

    // 5-2. 검색 인덱싱을 위한 토큰 추출
    println!("2. 검색 인덱싱용 토큰:");
    let search_tokens = pipeline.extract_search_tokens(raw_text)?;
    println!("   {:?}\n", search_tokens);

    // 5-3. 감성 분석을 위한 전처리
    println!("3. 감성 분석용 전처리:");
    let sentiment_tokens = pipeline.preprocess_for_sentiment(raw_text)?;
    println!("   {:?}\n", sentiment_tokens);

    println!("=== 전처리 완료 ===");

    Ok(())
}

/// 전처리 파이프라인
struct PreprocessingPipeline {
    tokenizer: Tokenizer,
    #[allow(dead_code)]
    normalizer: Normalizer,
    stopwords: HashSet<String>,
}

impl PreprocessingPipeline {
    /// 새 파이프라인 생성
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tokenizer = Tokenizer::new()?;
        let normalizer = Normalizer::default()?;
        let stopwords = Self::load_stopwords();

        Ok(Self {
            tokenizer,
            normalizer,
            stopwords,
        })
    }

    /// 한국어 불용어 목록
    fn load_stopwords() -> HashSet<String> {
        let stopwords = vec![
            // 조사
            "은",
            "는",
            "이",
            "가",
            "을",
            "를",
            "의",
            "에",
            "에서",
            "로",
            "으로",
            "와",
            "과",
            "도",
            "만",
            "까지",
            "부터",
            "하고",
            "및",
            "그리고",
            "또는",
            // 어미
            "ㄴ다",
            "ㅂ니다",
            "습니다",
            "입니다",
            "였습니다",
            "했습니다",
            // 보조용언
            "있",
            "없",
            "하",
            "되",
            "수",
            "것",
            // 지시사
            "이",
            "그",
            "저",
            "이것",
            "그것",
            "저것",
            // 기타
            "등",
            "및",
            "또",
            "또한",
            "때문",
            "위해",
        ];

        stopwords.into_iter().map(String::from).collect()
    }

    /// 전체 전처리 실행
    fn process(&mut self, text: &str) -> Result<ProcessedDocument, Box<dyn std::error::Error>> {
        let normalized = self.normalize_text(text);
        let sentences = self.split_sentences(&normalized);
        let tokens = self.tokenize(&normalized)?;
        let nouns = self.extract_nouns(&normalized)?;
        let filtered_tokens = self.remove_stopwords(&normalized)?;

        Ok(ProcessedDocument {
            original: text.to_string(),
            normalized,
            sentences,
            tokens,
            nouns,
            filtered_tokens,
        })
    }

    /// 텍스트 정규화
    fn normalize_text(&self, text: &str) -> String {
        // 1. 공백 정규화
        let mut normalized = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        // 2. 연속된 공백 제거
        while normalized.contains("  ") {
            normalized = normalized.replace("  ", " ");
        }

        // 3. 특수문자 정리 (반복)
        normalized = normalized.replace("!!!", "!");
        normalized = normalized.replace("~~~", "~");
        normalized = normalized.replace("...", ".");

        // 4. 외래어 정규화
        // 실제로는 normalizer를 사용하지만, 여기서는 간단한 규칙 적용
        normalized = normalized.replace("라이브러이", "라이브러리");
        normalized = normalized.replace("라이브러리", "라이브러리");

        normalized.trim().to_string()
    }

    /// 문장 분리
    fn split_sentences(&self, text: &str) -> Vec<String> {
        // 한국어 문장 종결 부호: ., !, ?, ~
        let mut sentences = Vec::new();
        let mut current = String::new();

        for ch in text.chars() {
            current.push(ch);

            if matches!(ch, '.' | '!' | '?' | '~') {
                let sentence = current.trim().to_string();
                if !sentence.is_empty() {
                    sentences.push(sentence);
                }
                current.clear();
            }
        }

        // 남은 텍스트 추가
        let sentence = current.trim().to_string();
        if !sentence.is_empty() {
            sentences.push(sentence);
        }

        sentences
    }

    /// 토큰화
    fn tokenize(&mut self, text: &str) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
        Ok(self.tokenizer.tokenize(text))
    }

    /// 명사 추출
    fn extract_nouns(&mut self, text: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(text);

        Ok(tokens
            .into_iter()
            .filter(|token| {
                token.pos.starts_with("NN")  // NNG, NNP, NNB 등
                    || token.pos == "SL"      // 외래어
                    || token.pos == "SN" // 숫자
            })
            .map(|token| token.surface)
            .collect())
    }

    /// 불용어 제거
    fn remove_stopwords(&mut self, text: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(text);

        Ok(tokens
            .into_iter()
            .filter(|token| {
                // 불용어가 아니고, 내용어인 것만
                !self.stopwords.contains(&token.surface)
                    && !token.pos.starts_with('J')   // 조사 제외
                    && !token.pos.starts_with('E')   // 어미 제외
                    && !token.pos.starts_with('S')   // 부호 제외 (일부)
                    && token.surface.chars().all(|c| !c.is_whitespace())
            })
            .map(|token| token.surface)
            .collect())
    }

    /// 어간 추출 (기본형)
    fn extract_lemmas(&mut self, text: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(text);

        Ok(tokens
            .into_iter()
            .filter_map(|token| {
                // lemma가 있으면 사용, 없으면 surface 사용
                if let Some(lemma) = token.lemma {
                    Some(lemma)
                } else if token.pos.starts_with('V') || token.pos.starts_with('A') {
                    // 동사/형용사는 표면형 사용
                    Some(token.surface)
                } else {
                    None
                }
            })
            .collect())
    }

    /// 문서 분류용 특징 추출
    fn extract_features_for_classification(
        &mut self,
        text: &str,
    ) -> Result<DocumentFeatures, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(&self.normalize_text(text));

        let mut nouns = Vec::new();
        let mut verbs = Vec::new();
        let mut adjectives = Vec::new();

        for token in tokens {
            if token.pos.starts_with("NN") {
                nouns.push(token.surface);
            } else if token.pos.starts_with('V') {
                verbs.push(token.lemma.unwrap_or(token.surface));
            } else if token.pos.starts_with('A') {
                adjectives.push(token.lemma.unwrap_or(token.surface));
            }
        }

        Ok(DocumentFeatures {
            nouns,
            verbs,
            adjectives,
        })
    }

    /// 검색 인덱싱용 토큰 추출
    fn extract_search_tokens(
        &mut self,
        text: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let normalized = self.normalize_text(text);
        let tokens = self.tokenizer.tokenize(&normalized);

        Ok(tokens
            .into_iter()
            .filter(|token| {
                // 명사, 동사, 형용사, 외래어만 인덱싱
                token.pos.starts_with("NN")
                    || token.pos.starts_with('V')
                    || token.pos.starts_with('A')
                    || token.pos == "SL"
            })
            .map(|token| {
                // 동사/형용사는 기본형 사용
                if token.pos.starts_with('V') || token.pos.starts_with('A') {
                    token.lemma.unwrap_or(token.surface)
                } else {
                    token.surface
                }
            })
            .collect())
    }

    /// 감성 분석용 전처리
    fn preprocess_for_sentiment(
        &mut self,
        text: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let normalized = self.normalize_text(text);
        let tokens = self.tokenizer.tokenize(&normalized);

        Ok(tokens
            .into_iter()
            .filter(|token| {
                // 감성과 관련된 품사: 명사, 동사, 형용사, 부사
                token.pos.starts_with("NN")
                    || token.pos.starts_with('V')
                    || token.pos.starts_with('A')
                    || token.pos.starts_with("MA") // 부사
            })
            .map(|token| token.lemma.unwrap_or(token.surface))
            .collect())
    }
}

/// 전처리된 문서
#[derive(Debug)]
struct ProcessedDocument {
    original: String,
    normalized: String,
    sentences: Vec<String>,
    tokens: Vec<Token>,
    nouns: Vec<String>,
    filtered_tokens: Vec<String>,
}

impl ProcessedDocument {
    fn summary(&self) -> String {
        format!(
            "원본 길이: {} 문자\n\
             정규화 길이: {} 문자\n\
             문장 수: {}\n\
             전체 토큰 수: {}\n\
             명사 수: {}\n\
             필터링 후 토큰 수: {}",
            self.original.len(),
            self.normalized.len(),
            self.sentences.len(),
            self.tokens.len(),
            self.nouns.len(),
            self.filtered_tokens.len()
        )
    }
}

/// 문서 분류용 특징
#[derive(Debug)]
struct DocumentFeatures {
    nouns: Vec<String>,
    verbs: Vec<String>,
    adjectives: Vec<String>,
}
