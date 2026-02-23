//! # Keyword Extraction Example
//!
//! 한국어 텍스트에서 키워드를 추출하는 다양한 방법 예제
//!
//! ## 주요 알고리즘
//!
//! - **빈도 기반 추출**: TF (Term Frequency)
//! - **TF-IDF**: 문서 내 중요도와 희소성 고려
//! - **품사 패턴 기반**: 명사구, 복합명사 추출
//! - **공기어 분석**: 함께 자주 등장하는 단어 쌍
//!
//! ## 사용 사례
//!
//! - 문서 요약
//! - 태그 생성

#![allow(
    clippy::uninlined_format_args,
    clippy::unnecessary_wraps,
    clippy::if_not_else,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::option_if_let_else,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::type_complexity,
    missing_docs
)]
//! - 검색어 추천
//! - 토픽 모델링 전처리

use mecab_ko_core::{Token, Tokenizer};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 한국어 키워드 추출 예제 ===\n");

    // 샘플 문서
    let document = "
        인공지능 기술이 빠르게 발전하고 있습니다.
        특히 자연어 처리 분야에서 대형 언어 모델의 성능이 놀라울 정도입니다.
        GPT와 같은 생성형 인공지능은 다양한 분야에서 활용되고 있습니다.
        한국어 자연어 처리도 점점 발전하고 있으며, 형태소 분석기의 정확도도 높아지고 있습니다.
        MeCab-Ko는 대표적인 한국어 형태소 분석기로 많은 프로젝트에서 사용됩니다.
        Rust로 재작성된 MeCab-Ko는 성능과 안정성이 더욱 향상되었습니다.
        자연어 처리 연구자와 개발자들은 이러한 도구를 활용하여 혁신적인 애플리케이션을 만듭니다.
    ";

    println!("원본 문서:\n{}\n", document);

    // 키워드 추출기 초기화
    let mut extractor = KeywordExtractor::new()?;

    // 1. 빈도 기반 키워드 추출
    println!("=== 1. 빈도 기반 키워드 추출 (Top 10) ===");
    let freq_keywords = extractor.extract_by_frequency(document, 10)?;
    for (i, (keyword, count)) in freq_keywords.iter().enumerate() {
        println!("  {}. {} ({}회)", i + 1, keyword, count);
    }
    println!();

    // 2. 품사 필터링 키워드 추출 (명사만)
    println!("=== 2. 명사 키워드 추출 (Top 10) ===");
    let noun_keywords = extractor.extract_nouns(document, 10)?;
    for (i, (keyword, count)) in noun_keywords.iter().enumerate() {
        println!("  {}. {} ({}회)", i + 1, keyword, count);
    }
    println!();

    // 3. 복합명사 추출
    println!("=== 3. 복합명사 추출 ===");
    let compound_nouns = extractor.extract_compound_nouns(document)?;
    for (i, noun) in compound_nouns.iter().enumerate().take(10) {
        println!("  {}. {}", i + 1, noun);
    }
    println!();

    // 4. 명사구 패턴 추출 (명사 + 명사, 관형사 + 명사 등)
    println!("=== 4. 명사구 패턴 추출 ===");
    let noun_phrases = extractor.extract_noun_phrases(document)?;
    for (i, phrase) in noun_phrases.iter().enumerate().take(10) {
        println!("  {}. {}", i + 1, phrase);
    }
    println!();

    // 5. TF-IDF 기반 키워드 추출 (다중 문서)
    println!("=== 5. TF-IDF 기반 키워드 추출 ===");
    let documents = vec![
        document,
        "머신러닝과 딥러닝은 인공지능의 핵심 기술입니다. 데이터 과학자들은 이를 활용합니다.",
        "자연어 처리는 텍스트 데이터를 분석하는 기술입니다. 형태소 분석이 기본입니다.",
    ];

    let tfidf_keywords = extractor.extract_by_tfidf(&documents, 0, 10)?;
    println!("  첫 번째 문서의 키워드:");
    for (i, (keyword, score)) in tfidf_keywords.iter().enumerate() {
        println!("    {}. {} (TF-IDF: {:.4})", i + 1, keyword, score);
    }
    println!();

    // 6. 공기어 분석 (Collocation)
    println!("=== 6. 공기어 분석 (자주 같이 등장하는 단어 쌍) ===");
    let collocations = extractor.extract_collocations(document, 5)?;
    for (i, (word1, word2, count)) in collocations.iter().enumerate() {
        println!("  {}. {} + {} ({}회)", i + 1, word1, word2, count);
    }
    println!();

    // 7. 가중치 기반 키워드 추출 (품사별 가중치 적용)
    println!("=== 7. 가중치 기반 키워드 추출 ===");
    let weighted_keywords = extractor.extract_weighted_keywords(document, 10)?;
    for (i, (keyword, score)) in weighted_keywords.iter().enumerate() {
        println!("  {}. {} (점수: {:.2})", i + 1, keyword, score);
    }
    println!();

    // 8. 실전 활용: 문서 태그 생성
    println!("=== 8. 문서 태그 자동 생성 ===");
    let tags = extractor.generate_tags(document, 5)?;
    println!("  추천 태그: {}", tags.join(", "));
    println!();

    println!("=== 키워드 추출 완료 ===");

    Ok(())
}

/// 키워드 추출기
struct KeywordExtractor {
    tokenizer: Tokenizer,
}

impl KeywordExtractor {
    /// 새 추출기 생성
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            tokenizer: Tokenizer::new()?,
        })
    }

    /// 빈도 기반 키워드 추출
    fn extract_by_frequency(
        &mut self,
        text: &str,
        top_n: usize,
    ) -> Result<Vec<(String, usize)>, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(text);

        // 빈도 계산
        let mut freq_map: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            // 내용어만 카운트 (명사, 동사, 형용사)
            if self.is_content_word(&token) {
                *freq_map.entry(token.surface).or_insert(0) += 1;
            }
        }

        // 빈도순 정렬
        let mut freq_vec: Vec<_> = freq_map.into_iter().collect();
        freq_vec.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        freq_vec.truncate(top_n);

        Ok(freq_vec)
    }

    /// 명사 키워드 추출
    fn extract_nouns(
        &mut self,
        text: &str,
        top_n: usize,
    ) -> Result<Vec<(String, usize)>, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(text);

        let mut freq_map: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            if token.pos.starts_with("NN") && token.surface.chars().count() >= 2 {
                *freq_map.entry(token.surface).or_insert(0) += 1;
            }
        }

        let mut freq_vec: Vec<_> = freq_map.into_iter().collect();
        freq_vec.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        freq_vec.truncate(top_n);

        Ok(freq_vec)
    }

    /// 복합명사 추출 (연속된 명사)
    fn extract_compound_nouns(
        &mut self,
        text: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(text);

        let mut compounds = Vec::new();
        let mut current_compound = String::new();

        for token in tokens {
            if token.pos.starts_with("NN") {
                if !current_compound.is_empty() {
                    current_compound.push_str(&token.surface);
                } else {
                    current_compound = token.surface;
                }
            } else {
                if current_compound.chars().count() >= 4 {
                    // 4글자 이상 복합명사만
                    compounds.push(current_compound.clone());
                }
                current_compound.clear();
            }
        }

        // 마지막 복합명사 처리
        if current_compound.chars().count() >= 4 {
            compounds.push(current_compound);
        }

        // 중복 제거
        compounds.sort();
        compounds.dedup();

        Ok(compounds)
    }

    /// 명사구 패턴 추출
    fn extract_noun_phrases(
        &mut self,
        text: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(text);

        let mut phrases = Vec::new();
        let mut current_phrase = Vec::new();

        for token in tokens {
            // 명사구 패턴: (관형사|명사)+ 명사
            if token.pos.starts_with("NN") || token.pos.starts_with("MM") {
                current_phrase.push(token.surface);
            } else {
                if current_phrase.len() >= 2 {
                    phrases.push(current_phrase.join(" "));
                }
                current_phrase.clear();
            }
        }

        // 마지막 명사구 처리
        if current_phrase.len() >= 2 {
            phrases.push(current_phrase.join(" "));
        }

        // 중복 제거
        phrases.sort();
        phrases.dedup();

        Ok(phrases)
    }

    /// TF-IDF 기반 키워드 추출
    fn extract_by_tfidf(
        &mut self,
        documents: &[&str],
        doc_index: usize,
        top_n: usize,
    ) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
        // 1. 각 문서별 TF 계산
        let mut doc_tf_maps = Vec::new();
        let mut doc_lengths = Vec::new();

        for doc in documents {
            let tokens = self.tokenizer.tokenize(doc);
            let length = tokens.len();

            let mut tf_map: HashMap<String, usize> = HashMap::new();
            for token in tokens {
                if self.is_content_word(&token) {
                    *tf_map.entry(token.surface).or_insert(0) += 1;
                }
            }

            doc_tf_maps.push(tf_map);
            doc_lengths.push(length);
        }

        // 2. IDF 계산 (전체 문서에서)
        let mut df_map: HashMap<String, usize> = HashMap::new();
        for tf_map in &doc_tf_maps {
            for term in tf_map.keys() {
                *df_map.entry(term.clone()).or_insert(0) += 1;
            }
        }

        let n_docs = documents.len() as f64;

        // 3. 지정된 문서의 TF-IDF 계산
        let tf_map = &doc_tf_maps[doc_index];
        let doc_len = doc_lengths[doc_index] as f64;

        let mut tfidf_vec: Vec<(String, f64)> = tf_map
            .iter()
            .map(|(term, tf)| {
                let tf_normalized = *tf as f64 / doc_len;
                let df = *df_map.get(term).unwrap() as f64;
                let idf = (n_docs / df).ln();
                let tfidf = tf_normalized * idf;
                (term.clone(), tfidf)
            })
            .collect();

        tfidf_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        tfidf_vec.truncate(top_n);

        Ok(tfidf_vec)
    }

    /// 공기어 분석 (Collocation)
    fn extract_collocations(
        &mut self,
        text: &str,
        top_n: usize,
    ) -> Result<Vec<(String, String, usize)>, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(text);

        // 내용어만 추출
        let content_words: Vec<String> = tokens
            .into_iter()
            .filter(|t| self.is_content_word(t))
            .map(|t| t.surface)
            .collect();

        // 연속된 단어 쌍 빈도 계산
        let mut pair_freq: HashMap<(String, String), usize> = HashMap::new();
        for window in content_words.windows(2) {
            let pair = (window[0].clone(), window[1].clone());
            *pair_freq.entry(pair).or_insert(0) += 1;
        }

        // 빈도순 정렬
        let mut collocations: Vec<_> = pair_freq
            .into_iter()
            .map(|((w1, w2), count)| (w1, w2, count))
            .collect();
        collocations.sort_by(|a, b| b.2.cmp(&a.2));
        collocations.truncate(top_n);

        Ok(collocations)
    }

    /// 가중치 기반 키워드 추출
    fn extract_weighted_keywords(
        &mut self,
        text: &str,
        top_n: usize,
    ) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(text);

        // 품사별 가중치
        let weights = HashMap::from([
            ("NN", 1.0), // 명사
            ("VV", 0.8), // 동사
            ("VA", 0.8), // 형용사
            ("SL", 0.9), // 외래어
            ("SN", 0.6), // 숫자
        ]);

        let mut score_map: HashMap<String, f64> = HashMap::new();

        for token in tokens {
            let pos_prefix = &token.pos[..2.min(token.pos.len())];
            if let Some(&weight) = weights.get(pos_prefix) {
                let score = weight * (token.surface.chars().count() as f64).mul_add(0.1, 1.0);
                *score_map.entry(token.surface).or_insert(0.0) += score;
            }
        }

        let mut score_vec: Vec<_> = score_map.into_iter().collect();
        score_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        score_vec.truncate(top_n);

        Ok(score_vec)
    }

    /// 문서 태그 자동 생성
    fn generate_tags(
        &mut self,
        text: &str,
        max_tags: usize,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // 1. 명사 키워드 추출
        let noun_keywords = self.extract_nouns(text, max_tags * 2)?;

        // 2. 길이 필터링 (2-6글자)
        let tags: Vec<String> = noun_keywords
            .into_iter()
            .map(|(word, _)| word)
            .filter(|word| {
                let len = word.chars().count();
                (2..=6).contains(&len)
            })
            .take(max_tags)
            .collect();

        Ok(tags)
    }

    /// 내용어 여부 판단
    #[allow(clippy::unused_self)]
    fn is_content_word(&self, token: &Token) -> bool {
        // 명사, 동사, 형용사, 외래어
        token.pos.starts_with("NN")
            || token.pos.starts_with("VV")
            || token.pos.starts_with("VA")
            || token.pos == "SL"
    }
}
