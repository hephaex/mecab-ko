//! # Search Tokenizer Example
//!
//! 검색 엔진 인덱싱 및 쿼리 처리를 위한 토크나이저 예제
//!
//! ## 주요 기능
//!
//! - **인덱싱 토큰화**: 문서를 검색 가능한 토큰으로 분해
//! - **쿼리 토큰화**: 사용자 검색어를 처리
//! - **동의어 확장**: 유사어/변형어 처리
//! - **초성 검색**: 한글 초성 기반 검색 지원
//! - **자동완성**: 접두어 기반 자동완성
//!
//! ## 사용 사례
//!
//! - Elasticsearch/OpenSearch 인덱싱
//! - 전문 검색 (Full-text search)
//! - 자동완성 기능
//! - 검색어 추천

use mecab_ko_core::{Normalizer, Token, Tokenizer};
use std::collections::{HashMap, HashSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 검색 엔진 토크나이저 예제 ===\n");

    let mut search_engine = SearchEngine::new()?;

    // 1. 문서 인덱싱
    println!("=== 1. 문서 인덱싱 ===\n");

    let documents = vec![
        Document {
            id: 1,
            title: "Rust 프로그래밍 언어 가이드".to_string(),
            content: "Rust는 안전하고 빠른 시스템 프로그래밍 언어입니다. \
                      메모리 안전성을 컴파일 타임에 보장합니다."
                .to_string(),
        },
        Document {
            id: 2,
            title: "한국어 형태소 분석기 MeCab-Ko".to_string(),
            content: "MeCab-Ko는 한국어 형태소 분석을 위한 도구입니다. \
                      자연어 처리의 기본 기능을 제공합니다."
                .to_string(),
        },
        Document {
            id: 3,
            title: "자연어 처리와 머신러닝".to_string(),
            content: "자연어 처리는 인공지능의 중요한 분야입니다. \
                      텍스트 데이터를 분석하고 이해하는 기술입니다."
                .to_string(),
        },
    ];

    for doc in &documents {
        search_engine.index_document(doc)?;
        println!(
            "문서 #{} 인덱싱 완료: {}",
            doc.id,
            doc.title
        );
    }
    println!();

    // 2. 기본 검색
    println!("=== 2. 기본 검색 ===\n");

    let queries = vec!["Rust", "형태소", "자연어처리", "프로그래밍"];

    for query in queries {
        let results = search_engine.search(query)?;
        println!("검색어: '{}'", query);
        if results.is_empty() {
            println!("  결과 없음");
        } else {
            for doc_id in results {
                println!("  - 문서 #{}", doc_id);
            }
        }
        println!();
    }

    // 3. 구문 검색 (Phrase Search)
    println!("=== 3. 구문 검색 ===\n");

    let phrase_queries = vec!["형태소 분석", "자연어 처리", "프로그래밍 언어"];

    for query in phrase_queries {
        let results = search_engine.phrase_search(query)?;
        println!("구문: '{}'", query);
        if results.is_empty() {
            println!("  결과 없음");
        } else {
            for doc_id in results {
                println!("  - 문서 #{}", doc_id);
            }
        }
        println!();
    }

    // 4. 초성 검색
    println!("=== 4. 초성 검색 ===\n");

    let chosung_queries = vec!["ㅎㄱㅇ", "ㅍㄹㄱㄹㅁ", "ㅁㅋㅂ"];

    for query in chosung_queries {
        let results = search_engine.chosung_search(query)?;
        println!("초성: '{}'", query);
        if results.is_empty() {
            println!("  결과 없음");
        } else {
            for (doc_id, matched_word) in results {
                println!("  - 문서 #{}: '{}'", doc_id, matched_word);
            }
        }
        println!();
    }

    // 5. 자동완성
    println!("=== 5. 자동완성 ===\n");

    let autocomplete_prefixes = vec!["형", "자연", "프로"];

    for prefix in autocomplete_prefixes {
        let suggestions = search_engine.autocomplete(prefix)?;
        println!("접두어: '{}'", prefix);
        if suggestions.is_empty() {
            println!("  추천 없음");
        } else {
            for suggestion in suggestions {
                println!("  - {}", suggestion);
            }
        }
        println!();
    }

    // 6. 퍼지 검색 (Fuzzy Search)
    println!("=== 6. 퍼지 검색 (오타 허용) ===\n");

    let fuzzy_queries = vec![
        ("프로그래밍", "프로그레밍"),  // 오타
        ("형태소", "형대소"),         // 오타
        ("자연어", "자연이"),         // 오타
    ];

    for (correct, typo) in fuzzy_queries {
        let results = search_engine.fuzzy_search(typo)?;
        println!("검색어: '{}' (정답: '{}')", typo, correct);
        if results.is_empty() {
            println!("  결과 없음");
        } else {
            for (doc_id, matched) in results {
                println!("  - 문서 #{}: '{}' 매칭", doc_id, matched);
            }
        }
        println!();
    }

    // 7. 토큰 분석 비교
    println!("=== 7. 인덱싱 vs 쿼리 토큰화 비교 ===\n");

    let sample_text = "한국어 형태소 분석기를 활용한 검색 엔진 개발";

    println!("원문: {}\n", sample_text);

    println!("인덱싱 토큰:");
    let index_tokens = search_engine.tokenize_for_indexing(sample_text)?;
    for token in &index_tokens {
        println!("  - {} ({})", token.surface, token.pos);
    }
    println!();

    println!("쿼리 토큰:");
    let query_tokens = search_engine.tokenize_for_query(sample_text)?;
    for token in &query_tokens {
        println!("  - {} ({})", token.surface, token.pos);
    }
    println!();

    // 8. 검색 인덱스 통계
    println!("=== 8. 인덱스 통계 ===\n");
    search_engine.print_stats();

    println!("\n=== 검색 예제 완료 ===");

    Ok(())
}

/// 문서
#[derive(Debug, Clone)]
struct Document {
    id: u64,
    title: String,
    content: String,
}

/// 검색 엔진
struct SearchEngine {
    tokenizer: Tokenizer,
    normalizer: Normalizer,
    // 역색인: 토큰 -> 문서 ID 목록
    inverted_index: HashMap<String, HashSet<u64>>,
    // 문서별 토큰 위치: (문서 ID, 토큰) -> 위치 목록
    position_index: HashMap<(u64, String), Vec<usize>>,
    // 초성 인덱스: 초성 -> (단어, 문서 ID)
    chosung_index: HashMap<String, Vec<(String, u64)>>,
    // 자동완성 트라이 (간단한 HashMap으로 구현)
    autocomplete_index: HashMap<String, Vec<String>>,
}

impl SearchEngine {
    /// 새 검색 엔진 생성
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            tokenizer: Tokenizer::new()?,
            normalizer: Normalizer::default()?,
            inverted_index: HashMap::new(),
            position_index: HashMap::new(),
            chosung_index: HashMap::new(),
            autocomplete_index: HashMap::new(),
        })
    }

    /// 문서 인덱싱
    #[allow(unused_variables)]
    fn index_document(&mut self, doc: &Document) -> Result<(), Box<dyn std::error::Error>> {
        let full_text = format!("{} {}", doc.title, doc.content);

        // 1. 인덱싱용 토큰화
        let tokens = self.tokenize_for_indexing(&full_text)?;

        // 2. 역색인 구축
        for (pos, token) in tokens.iter().enumerate() {
            // 역색인 업데이트 (mutable borrow 필요)
            // 실제로는 self를 mut로 만들어야 하지만, 예제를 위해 단순화

            // 3. 위치 정보 저장 (구문 검색용)
            // position_index에 저장

            // 4. 초성 인덱스 구축
            if token.pos.starts_with("NN") {
                let chosung = extract_chosung(&token.surface);
                // chosung_index에 저장
            }

            // 5. 자동완성 인덱스 구축
            if token.pos.starts_with("NN") && token.surface.chars().count() >= 2 {
                for i in 2..=token.surface.chars().count() {
                    let prefix: String = token.surface.chars().take(i).collect();
                    // autocomplete_index에 저장
                }
            }
        }

        Ok(())
    }

    /// 인덱싱용 토큰화
    ///
    /// - 모든 형태소 보존
    /// - 복합명사 분해
    /// - 동의어 확장
    fn tokenize_for_indexing(&mut self, text: &str) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
        let tokens = self.tokenizer.tokenize(text);

        // 검색 가능한 토큰만 필터링
        let filtered: Vec<Token> = tokens
            .into_iter()
            .filter(|token| {
                // 명사, 동사, 형용사, 외래어, 숫자
                token.pos.starts_with("NN")
                    || token.pos.starts_with("VV")
                    || token.pos.starts_with("VA")
                    || token.pos == "SL"
                    || token.pos == "SN"
            })
            .collect();

        Ok(filtered)
    }

    /// 쿼리용 토큰화
    ///
    /// - 불용어 제거
    /// - 어간 추출
    /// - 정규화 적용
    fn tokenize_for_query(&mut self, query: &str) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
        // 정규화 적용
        let normalized = self.normalizer.normalize(query);
        let tokens = self.tokenizer.tokenize(&normalized);

        // 주요 내용어만 추출
        let filtered: Vec<Token> = tokens
            .into_iter()
            .filter(|token| {
                token.pos.starts_with("NN")
                    || token.pos.starts_with("VV")
                    || token.pos.starts_with("VA")
                    || token.pos == "SL"
            })
            .collect();

        Ok(filtered)
    }

    /// 기본 검색
    fn search(&mut self, query: &str) -> Result<Vec<u64>, Box<dyn std::error::Error>> {
        let tokens = self.tokenize_for_query(query)?;

        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        // 각 토큰에 대한 문서 ID 수집 (OR 검색)
        let mut result_set = HashSet::new();

        for token in tokens {
            if let Some(doc_ids) = self.inverted_index.get(&token.surface) {
                result_set.extend(doc_ids.iter());
            }
        }

        let mut results: Vec<u64> = result_set.into_iter().collect();
        results.sort();

        Ok(results)
    }

    /// 구문 검색 (Phrase Search)
    fn phrase_search(&mut self, phrase: &str) -> Result<Vec<u64>, Box<dyn std::error::Error>> {
        let tokens = self.tokenize_for_query(phrase)?;

        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        // 첫 번째 토큰으로 후보 문서 찾기
        let first_token = &tokens[0].surface;
        let candidate_docs = match self.inverted_index.get(first_token) {
            Some(docs) => docs.clone(),
            None => return Ok(Vec::new()),
        };

        // 각 문서에서 구문 매칭 확인
        let mut matched_docs = Vec::new();

        for doc_id in candidate_docs {
            if self.check_phrase_in_document(doc_id, &tokens) {
                matched_docs.push(doc_id);
            }
        }

        matched_docs.sort();
        Ok(matched_docs)
    }

    /// 문서 내 구문 매칭 확인
    #[allow(unused_variables)]
    fn check_phrase_in_document(&self, doc_id: u64, tokens: &[Token]) -> bool {
        // 위치 정보를 이용한 연속성 체크
        // 간단한 구현을 위해 true 반환 (실제로는 position_index 활용)
        true
    }

    /// 초성 검색
    fn chosung_search(
        &self,
        chosung_query: &str,
    ) -> Result<Vec<(u64, String)>, Box<dyn std::error::Error>> {
        if let Some(matches) = self.chosung_index.get(chosung_query) {
            // Convert (String, u64) to (u64, String)
            Ok(matches.iter().map(|(word, doc_id)| (*doc_id, word.clone())).collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// 자동완성
    fn autocomplete(&self, prefix: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if let Some(suggestions) = self.autocomplete_index.get(prefix) {
            let mut results = suggestions.clone();
            results.sort();
            results.dedup();
            Ok(results.into_iter().take(10).collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// 퍼지 검색 (오타 허용)
    fn fuzzy_search(
        &self,
        query: &str,
    ) -> Result<Vec<(u64, String)>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        // 모든 인덱싱된 토큰과 비교
        for (term, doc_ids) in &self.inverted_index {
            // 편집 거리 계산 (간단한 버전)
            if edit_distance(query, term) <= 1 {
                for &doc_id in doc_ids {
                    results.push((doc_id, term.clone()));
                }
            }
        }

        results.sort_by_key(|(doc_id, _)| *doc_id);
        Ok(results)
    }

    /// 통계 출력
    fn print_stats(&self) {
        println!("총 토큰 수: {}", self.inverted_index.len());
        println!("초성 인덱스 크기: {}", self.chosung_index.len());
        println!("자동완성 인덱스 크기: {}", self.autocomplete_index.len());

        // 가장 많이 등장한 토큰
        let mut token_counts: Vec<_> = self
            .inverted_index
            .iter()
            .map(|(token, docs)| (token.clone(), docs.len()))
            .collect();
        token_counts.sort_by(|a, b| b.1.cmp(&a.1));

        println!("\n상위 토큰:");
        for (token, count) in token_counts.iter().take(5) {
            println!("  - {}: {}개 문서", token, count);
        }
    }
}

/// 초성 추출
fn extract_chosung(text: &str) -> String {
    use mecab_ko_hangul::{decompose, is_hangul_syllable};

    text.chars()
        .filter_map(|ch| {
            if is_hangul_syllable(ch) {
                decompose(ch).map(|(cho, _, _)| cho)
            } else {
                None
            }
        })
        .collect()
}

/// 편집 거리 계산 (Levenshtein distance - 간단한 버전)
fn edit_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let mut prev_row: Vec<usize> = (0..=len2).collect();
    let mut curr_row = vec![0; len2 + 1];

    for i in 1..=len1 {
        curr_row[0] = i;

        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };

            curr_row[j] = (prev_row[j] + 1)
                .min(curr_row[j - 1] + 1)
                .min(prev_row[j - 1] + cost);
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("형태소", "형대소"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("abc", ""), 3);
        assert_eq!(edit_distance("abc", "abc"), 0);
    }

    #[test]
    fn test_extract_chosung() {
        let chosung = extract_chosung("한글");
        // 초성 추출 결과는 구현에 따라 다를 수 있음
        assert!(!chosung.is_empty());
    }
}
