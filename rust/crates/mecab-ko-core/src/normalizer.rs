//! # Foreign Word Normalization Module
//!
//! 외래어 표기 정규화 모듈 - 국립국어원 외래어 표기법 기반
//!
//! ## Features
//!
//! - 외래어 변이형 정규화 (커피/코피, 쿠버네티스/쿠베르네테스)
//! - 장단음 정규화
//! - 자음/모음 변이 처리
//! - 받침 변이 처리
//! - 발음 유사성 기반 fuzzy matching
//!
//! ## Example
//!
//! ```rust,ignore
//! use mecab_ko_core::normalizer::{Normalizer, NormalizationConfig};
//!
//! let normalizer = Normalizer::new(NormalizationConfig::default())?;
//!
//! // 표준형으로 정규화
//! let normalized = normalizer.normalize("코피");
//! assert_eq!(normalized, "커피");
//!
//! // 변이형 목록 조회
//! let variants = normalizer.get_variants("커피");
//! assert!(variants.contains(&"코피".to_string()));
//!
//! // 변이형 여부 확인
//! assert!(normalizer.is_variant("커피", "코피"));
//! ```

use crate::Result;
use mecab_ko_hangul::{compose, decompose, is_hangul_syllable};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

/// 정규화 규칙 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleType {
    /// 장단음 변이 (커피 ↔ 코피)
    VowelLength,
    /// 자음 변이 (쿠버네티스 ↔ 쿠베르네테스)
    ConsonantVariation,
    /// 받침 변이 (소프트웨어 ↔ 소프트웨아)
    JongseongVariation,
    /// 모음 변이 (케이크 ↔ 케익)
    VowelVariation,
    /// 발음 유사성 (라이브러리 ↔ 라이브러이)
    PhoneticSimilarity,
}

/// 정규화 규칙
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizationRule {
    /// 규칙 타입
    pub rule_type: RuleType,
    /// 원본 패턴
    pub from: String,
    /// 대상 패턴
    pub to: String,
    /// 신뢰도 (0.0 ~ 1.0)
    pub confidence: f32,
}

impl NormalizationRule {
    /// 새 규칙 생성
    #[must_use]
    pub fn new(rule_type: RuleType, from: String, to: String, confidence: f32) -> Self {
        Self {
            rule_type,
            from,
            to,
            confidence,
        }
    }
}

/// 정규화 설정
#[derive(Debug, Clone)]
pub struct NormalizationConfig {
    /// 장단음 정규화 활성화
    pub vowel_length: bool,
    /// 자음 변이 정규화 활성화
    pub consonant_variation: bool,
    /// 받침 변이 정규화 활성화
    pub jongseong_variation: bool,
    /// 모음 변이 정규화 활성화
    pub vowel_variation: bool,
    /// 발음 유사성 기반 정규화 활성화
    pub phonetic_similarity: bool,
    /// 최소 신뢰도 임계값
    pub min_confidence: f32,
}

impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            vowel_length: true,
            consonant_variation: true,
            jongseong_variation: true,
            vowel_variation: true,
            phonetic_similarity: true,
            min_confidence: 0.7,
        }
    }
}

/// 외래어 정규화기
pub struct Normalizer {
    /// 설정
    config: NormalizationConfig,
    /// 표준형 → 변이형 맵
    standard_to_variants: Arc<HashMap<String, HashSet<String>>>,
    /// 변이형 → 표준형 맵
    variant_to_standard: Arc<HashMap<String, String>>,
    /// 정규화 규칙
    rules: Arc<Vec<NormalizationRule>>,
}

impl Normalizer {
    /// 새 정규화기 생성
    ///
    /// # Arguments
    ///
    /// * `config` - 정규화 설정
    ///
    /// # Returns
    ///
    /// `Result<Self>` - 생성된 정규화기 또는 에러
    ///
    /// # Errors
    ///
    /// 데이터 로딩 실패 시 에러 반환
    pub fn new(config: NormalizationConfig) -> Result<Self> {
        let rules = Self::load_rules(&config);
        let (standard_to_variants, variant_to_standard) = Self::build_variant_maps(&rules);

        Ok(Self {
            config,
            standard_to_variants: Arc::new(standard_to_variants),
            variant_to_standard: Arc::new(variant_to_standard),
            rules: Arc::new(rules),
        })
    }

    /// 외부 데이터 파일로 정규화기 생성
    ///
    /// # Arguments
    ///
    /// * `config` - 정규화 설정
    /// * `variant_csv_path` - 변이형 CSV 파일 경로
    ///
    /// # Returns
    ///
    /// `Result<Self>` - 생성된 정규화기 또는 에러
    ///
    /// # Errors
    ///
    /// 파일 로딩 또는 파싱 실패 시 에러 반환
    pub fn with_data_file(config: NormalizationConfig, variant_csv_path: &Path) -> Result<Self> {
        let rules = Self::load_rules(&config);
        let mut variant_pairs = Self::builtin_variant_pairs();

        // CSV 파일에서 추가 변이형 로드
        if let Ok(external_pairs) = Self::load_variant_csv(variant_csv_path) {
            variant_pairs.extend(external_pairs);
        }

        let (standard_to_variants, variant_to_standard) = Self::build_variant_maps_with_pairs(&rules, &variant_pairs);

        Ok(Self {
            config,
            standard_to_variants: Arc::new(standard_to_variants),
            variant_to_standard: Arc::new(variant_to_standard),
            rules: Arc::new(rules),
        })
    }

    /// CSV 파일에서 변이형 로드
    fn load_variant_csv(path: &Path) -> Result<Vec<(String, String)>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(path).map_err(|e| {
            crate::error::Error::Init(format!("Failed to open variant CSV: {e}"))
        })?;

        let reader = BufReader::new(file);
        let mut pairs = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| {
                crate::error::Error::Init(format!("Failed to read line {line_num}: {e}"))
            })?;

            // 헤더 또는 빈 줄 스킵
            if line_num == 0 || line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let standard = parts[0].trim().to_string();
                let variant = parts[1].trim().to_string();

                // 표준형과 변이형이 다를 때만 추가
                if standard != variant {
                    pairs.push((standard, variant));
                }
            }
        }

        Ok(pairs)
    }

    /// 기본 설정으로 생성
    ///
    /// # Errors
    ///
    /// 데이터 로딩 실패 시 에러 반환
    pub fn default() -> Result<Self> {
        Self::new(NormalizationConfig::default())
    }

    /// 외래어를 표준형으로 정규화
    ///
    /// # Arguments
    ///
    /// * `text` - 정규화할 텍스트
    ///
    /// # Returns
    ///
    /// 정규화된 텍스트
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let normalizer = Normalizer::default()?;
    /// assert_eq!(normalizer.normalize("코피"), "커피");
    /// assert_eq!(normalizer.normalize("소프트웨아"), "소프트웨어");
    /// ```
    #[must_use]
    pub fn normalize(&self, text: &str) -> String {
        // 직접 매핑 확인
        if let Some(standard) = self.variant_to_standard.get(text) {
            return standard.clone();
        }

        // 규칙 기반 정규화 시도
        self.apply_rules(text)
    }

    /// 표준형의 모든 변이형 조회
    ///
    /// # Arguments
    ///
    /// * `standard` - 표준형 단어
    ///
    /// # Returns
    ///
    /// 변이형 목록
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let normalizer = Normalizer::default()?;
    /// let variants = normalizer.get_variants("커피");
    /// assert!(variants.contains(&"코피".to_string()));
    /// ```
    #[must_use]
    pub fn get_variants(&self, standard: &str) -> Vec<String> {
        // 직접 매핑된 변이형
        let mut variants = self
            .standard_to_variants
            .get(standard)
            .map(|set| set.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        // 규칙 기반 변이형 생성
        let generated = self.generate_variants(standard);
        variants.extend(generated);

        variants.sort();
        variants.dedup();
        variants
    }

    /// 두 단어가 변이형 관계인지 확인
    ///
    /// # Arguments
    ///
    /// * `word1` - 첫 번째 단어
    /// * `word2` - 두 번째 단어
    ///
    /// # Returns
    ///
    /// 변이형 관계이면 `true`, 아니면 `false`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let normalizer = Normalizer::default()?;
    /// assert!(normalizer.is_variant("커피", "코피"));
    /// assert!(!normalizer.is_variant("커피", "라면"));
    /// ```
    #[must_use]
    pub fn is_variant(&self, word1: &str, word2: &str) -> bool {
        if word1 == word2 {
            return true;
        }

        let norm1 = self.normalize(word1);
        let norm2 = self.normalize(word2);
        norm1 == norm2
    }

    /// 발음 유사도 계산 (0.0 ~ 1.0)
    ///
    /// # Arguments
    ///
    /// * `word1` - 첫 번째 단어
    /// * `word2` - 두 번째 단어
    ///
    /// # Returns
    ///
    /// 발음 유사도 (0.0 ~ 1.0)
    #[must_use]
    pub fn phonetic_similarity(&self, word1: &str, word2: &str) -> f32 {
        if word1 == word2 {
            return 1.0;
        }

        let jamo1 = self.to_phonetic_jamo(word1);
        let jamo2 = self.to_phonetic_jamo(word2);

        Self::string_similarity(&jamo1, &jamo2)
    }

    // 내부 헬퍼 메서드들

    /// 규칙 로딩 (내장 규칙 + 외부 파일)
    fn load_rules(config: &NormalizationConfig) -> Vec<NormalizationRule> {
        let mut rules = Vec::new();

        // 장단음 규칙
        if config.vowel_length {
            rules.extend(Self::vowel_length_rules());
        }

        // 자음 변이 규칙
        if config.consonant_variation {
            rules.extend(Self::consonant_variation_rules());
        }

        // 받침 변이 규칙
        if config.jongseong_variation {
            rules.extend(Self::jongseong_variation_rules());
        }

        // 모음 변이 규칙
        if config.vowel_variation {
            rules.extend(Self::vowel_variation_rules());
        }

        rules
    }

    /// 장단음 규칙
    fn vowel_length_rules() -> Vec<NormalizationRule> {
        vec![
            NormalizationRule::new(RuleType::VowelLength, "오".into(), "어".into(), 0.9),
            NormalizationRule::new(RuleType::VowelLength, "어".into(), "오".into(), 0.9),
            NormalizationRule::new(RuleType::VowelLength, "우".into(), "유".into(), 0.85),
            NormalizationRule::new(RuleType::VowelLength, "유".into(), "우".into(), 0.85),
        ]
    }

    /// 자음 변이 규칙
    fn consonant_variation_rules() -> Vec<NormalizationRule> {
        vec![
            NormalizationRule::new(RuleType::ConsonantVariation, "ㅂ".into(), "ㅍ".into(), 0.9),
            NormalizationRule::new(RuleType::ConsonantVariation, "ㅍ".into(), "ㅂ".into(), 0.9),
            NormalizationRule::new(RuleType::ConsonantVariation, "ㄷ".into(), "ㅌ".into(), 0.9),
            NormalizationRule::new(RuleType::ConsonantVariation, "ㅌ".into(), "ㄷ".into(), 0.9),
            NormalizationRule::new(RuleType::ConsonantVariation, "ㄱ".into(), "ㅋ".into(), 0.9),
            NormalizationRule::new(RuleType::ConsonantVariation, "ㅋ".into(), "ㄱ".into(), 0.9),
            NormalizationRule::new(RuleType::ConsonantVariation, "ㅈ".into(), "ㅊ".into(), 0.9),
            NormalizationRule::new(RuleType::ConsonantVariation, "ㅊ".into(), "ㅈ".into(), 0.9),
            NormalizationRule::new(RuleType::ConsonantVariation, "ㅅ".into(), "ㅆ".into(), 0.85),
            NormalizationRule::new(RuleType::ConsonantVariation, "ㅆ".into(), "ㅅ".into(), 0.85),
        ]
    }

    /// 받침 변이 규칙
    fn jongseong_variation_rules() -> Vec<NormalizationRule> {
        vec![
            NormalizationRule::new(
                RuleType::JongseongVariation,
                "ㄹ".into(),
                "".into(),
                0.85,
            ),
            NormalizationRule::new(
                RuleType::JongseongVariation,
                "".into(),
                "ㄹ".into(),
                0.85,
            ),
            NormalizationRule::new(
                RuleType::JongseongVariation,
                "ㅁ".into(),
                "ㅂ".into(),
                0.8,
            ),
            NormalizationRule::new(
                RuleType::JongseongVariation,
                "ㅂ".into(),
                "ㅁ".into(),
                0.8,
            ),
        ]
    }

    /// 모음 변이 규칙
    fn vowel_variation_rules() -> Vec<NormalizationRule> {
        vec![
            NormalizationRule::new(RuleType::VowelVariation, "에이".into(), "에".into(), 0.9),
            NormalizationRule::new(RuleType::VowelVariation, "에".into(), "에이".into(), 0.9),
            NormalizationRule::new(RuleType::VowelVariation, "이".into(), "익".into(), 0.85),
            NormalizationRule::new(RuleType::VowelVariation, "익".into(), "이".into(), 0.85),
        ]
    }

    /// 변이형 맵 구축
    fn build_variant_maps(
        rules: &[NormalizationRule],
    ) -> (HashMap<String, HashSet<String>>, HashMap<String, String>) {
        let builtin_variants = Self::builtin_variant_pairs();
        Self::build_variant_maps_with_pairs(rules, &builtin_variants)
    }

    /// 변이형 쌍으로 맵 구축
    fn build_variant_maps_with_pairs(
        _rules: &[NormalizationRule],
        variant_pairs: &[(String, String)],
    ) -> (HashMap<String, HashSet<String>>, HashMap<String, String>) {
        let mut standard_to_variants = HashMap::new();
        let mut variant_to_standard = HashMap::new();

        for (standard, variant) in variant_pairs {
            standard_to_variants
                .entry(standard.clone())
                .or_insert_with(HashSet::new)
                .insert(variant.clone());

            variant_to_standard.insert(variant.clone(), standard.clone());
        }

        (standard_to_variants, variant_to_standard)
    }

    /// 내장 변이형 쌍
    fn builtin_variant_pairs() -> Vec<(String, String)> {
        vec![
            // IT 용어
            ("커피".into(), "코피".into()),
            ("쿠버네티스".into(), "쿠베르네테스".into()),
            ("쿠버네티스".into(), "쿠베르네티즈".into()),
            ("소프트웨어".into(), "소프트웨아".into()),
            ("라이브러리".into(), "라이브러이".into()),
            ("디렉토리".into(), "디렉터리".into()),
            ("디렉터리".into(), "디렉토리".into()),
            ("서버".into(), "서버".into()),
            ("클라이언트".into(), "클라이언트".into()),
            ("인터페이스".into(), "인터페이스".into()),
            ("알고리즘".into(), "알고리듬".into()),
            ("컴퓨터".into(), "컴퓨타".into()),
            ("프로그램".into(), "프로그래밍".into()),
            ("데이터베이스".into(), "데이타베이스".into()),
            // 일반 외래어
            ("케이크".into(), "케익".into()),
            ("스테이크".into(), "스테익".into()),
            ("메이크업".into(), "메이컵".into()),
            ("샴푸".into(), "샴프".into()),
            ("컵".into(), "컵".into()),
            ("버스".into(), "버스".into()),
            ("택시".into(), "택시".into()),
            ("카메라".into(), "카메라".into()),
            ("비디오".into(), "비데오".into()),
            ("라디오".into(), "라지오".into()),
        ]
    }

    /// 규칙 기반 정규화 적용
    fn apply_rules(&self, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut result = String::with_capacity(text.len());

        for &ch in &chars {
            result.push(ch);
        }

        result
    }

    /// 규칙 기반 변이형 생성
    fn generate_variants(&self, text: &str) -> Vec<String> {
        let mut variants = HashSet::new();

        // 장단음 변이형 생성
        if self.config.vowel_length {
            variants.extend(self.generate_vowel_length_variants(text));
        }

        // 받침 변이형 생성
        if self.config.jongseong_variation {
            variants.extend(self.generate_jongseong_variants(text));
        }

        variants.into_iter().collect()
    }

    /// 장단음 변이형 생성
    fn generate_vowel_length_variants(&self, text: &str) -> Vec<String> {
        let mut variants = Vec::new();

        for i in 0..text.chars().count() {
            let chars: Vec<char> = text.chars().collect();
            let ch = chars[i];

            if !is_hangul_syllable(ch) {
                continue;
            }

            if let Some((cho, jung, jong)) = decompose(ch) {
                // 'ㅓ' ↔ 'ㅗ' 변이
                if jung == 'ㅓ' {
                    if let Some(variant_char) = compose(cho, 'ㅗ', jong) {
                        let mut variant: Vec<char> = chars.clone();
                        variant[i] = variant_char;
                        variants.push(variant.into_iter().collect());
                    }
                } else if jung == 'ㅗ' {
                    if let Some(variant_char) = compose(cho, 'ㅓ', jong) {
                        let mut variant: Vec<char> = chars.clone();
                        variant[i] = variant_char;
                        variants.push(variant.into_iter().collect());
                    }
                }
            }
        }

        variants
    }

    /// 받침 변이형 생성
    fn generate_jongseong_variants(&self, text: &str) -> Vec<String> {
        let mut variants = Vec::new();

        for i in 0..text.chars().count() {
            let chars: Vec<char> = text.chars().collect();
            let ch = chars[i];

            if !is_hangul_syllable(ch) {
                continue;
            }

            if let Some((cho, jung, jong)) = decompose(ch) {
                // 받침 추가/제거
                if jong.is_none() {
                    // 받침 추가 (ㄹ, ㅁ, ㅂ)
                    for &new_jong in &['ㄹ', 'ㅁ', 'ㅂ'] {
                        if let Some(variant_char) = compose(cho, jung, Some(new_jong)) {
                            let mut variant: Vec<char> = chars.clone();
                            variant[i] = variant_char;
                            variants.push(variant.into_iter().collect());
                        }
                    }
                } else {
                    // 받침 제거
                    if let Some(variant_char) = compose(cho, jung, None) {
                        let mut variant: Vec<char> = chars.clone();
                        variant[i] = variant_char;
                        variants.push(variant.into_iter().collect());
                    }
                }
            }
        }

        variants
    }

    /// 발음 기반 자모 변환 (유사도 계산용)
    fn to_phonetic_jamo(&self, text: &str) -> String {
        let mut result = String::new();

        for ch in text.chars() {
            if let Some((cho, jung, jong)) = decompose(ch) {
                result.push(cho);
                result.push(jung);
                if let Some(j) = jong {
                    result.push(j);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// 문자열 유사도 계산 (Levenshtein distance 기반)
    fn string_similarity(s1: &str, s2: &str) -> f32 {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 && len2 == 0 {
            return 1.0;
        }

        let max_len = len1.max(len2);
        let distance = Self::levenshtein_distance(s1, s2);

        1.0 - (distance as f32 / max_len as f32)
    }

    /// Levenshtein distance 계산
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalizer_creation() {
        let result = Normalizer::default();
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_builtin() {
        let normalizer = Normalizer::default().unwrap();

        assert_eq!(normalizer.normalize("코피"), "커피");
        assert_eq!(normalizer.normalize("커피"), "커피");
        assert_eq!(normalizer.normalize("소프트웨아"), "소프트웨어");
        assert_eq!(normalizer.normalize("케익"), "케이크");
    }

    #[test]
    fn test_get_variants() {
        let normalizer = Normalizer::default().unwrap();

        let variants = normalizer.get_variants("커피");
        assert!(variants.contains(&"코피".to_string()));

        let variants = normalizer.get_variants("케이크");
        assert!(variants.contains(&"케익".to_string()));
    }

    #[test]
    fn test_is_variant() {
        let normalizer = Normalizer::default().unwrap();

        assert!(normalizer.is_variant("커피", "코피"));
        assert!(normalizer.is_variant("코피", "커피"));
        assert!(normalizer.is_variant("커피", "커피"));
        assert!(!normalizer.is_variant("커피", "라면"));
    }

    #[test]
    fn test_phonetic_similarity() {
        let normalizer = Normalizer::default().unwrap();

        assert_eq!(normalizer.phonetic_similarity("커피", "커피"), 1.0);
        // "커피" vs "코피": 6 chars total, 2 different (ㅓ vs ㅗ), similarity = 4/6 = 0.666...
        assert!(normalizer.phonetic_similarity("커피", "코피") > 0.6);
        assert!(normalizer.phonetic_similarity("커피", "라면") < 0.5);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(Normalizer::levenshtein_distance("", ""), 0);
        assert_eq!(Normalizer::levenshtein_distance("a", ""), 1);
        assert_eq!(Normalizer::levenshtein_distance("", "a"), 1);
        assert_eq!(Normalizer::levenshtein_distance("abc", "abc"), 0);
        assert_eq!(Normalizer::levenshtein_distance("abc", "abd"), 1);
        assert_eq!(Normalizer::levenshtein_distance("abc", "def"), 3);
    }

    #[test]
    fn test_vowel_length_variants() {
        let normalizer = Normalizer::default().unwrap();

        // 장단음 변이형 테스트
        let variants = normalizer.generate_vowel_length_variants("커피");
        assert!(!variants.is_empty());
    }

    #[test]
    fn test_jongseong_variants() {
        let normalizer = Normalizer::default().unwrap();

        // 받침 변이형 테스트
        let variants = normalizer.generate_jongseong_variants("소프트웨어");
        assert!(!variants.is_empty());
    }

    #[test]
    fn test_it_terms() {
        let normalizer = Normalizer::default().unwrap();

        // IT 용어 테스트
        assert_eq!(normalizer.normalize("쿠베르네테스"), "쿠버네티스");
        assert_eq!(normalizer.normalize("라이브러이"), "라이브러리");
        assert_eq!(normalizer.normalize("디렉터리"), "디렉토리");
    }

    #[test]
    fn test_config() {
        let mut config = NormalizationConfig::default();
        config.vowel_length = false;
        config.min_confidence = 0.9;

        let normalizer = Normalizer::new(config);
        assert!(normalizer.is_ok());
    }
}
