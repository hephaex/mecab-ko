//! Fixture loading and management utilities

use super::{get_fixtures_path, get_golden_path, MorphTestCase};
use std::collections::HashMap;
use std::path::PathBuf;

/// Fixture manager for cached loading
pub struct FixtureManager {
    cache: HashMap<String, Vec<MorphTestCase>>,
    #[allow(dead_code)]
    fixtures_dir: PathBuf,
    #[allow(dead_code)]
    golden_dir: PathBuf,
}

impl FixtureManager {
    /// Create a new fixture manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            fixtures_dir: get_fixtures_path(),
            golden_dir: get_golden_path(),
        }
    }

    /// Load or retrieve cached fixture
    ///
    /// # Arguments
    ///
    /// * `filename` - Name of the fixture file
    ///
    /// # Returns
    ///
    /// Reference to cached test cases
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be loaded
    #[allow(dead_code)]
    pub fn get_fixture(&mut self, filename: &str) -> Result<&[MorphTestCase], String> {
        if !self.cache.contains_key(filename) {
            let path = self.fixtures_dir.join(filename);
            let cases = Self::load_from_path(&path)?;
            self.cache.insert(filename.to_string(), cases);
        }

        Ok(self
            .cache
            .get(filename)
            .ok_or_else(|| format!("Failed to get fixture {filename}"))?)
    }

    /// Load golden test cases
    ///
    /// # Arguments
    ///
    /// * `filename` - Name of the golden test file
    ///
    /// # Returns
    ///
    /// Reference to cached test cases
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be loaded
    #[allow(dead_code)]
    pub fn get_golden(&mut self, filename: &str) -> Result<&[MorphTestCase], String> {
        let key = format!("golden:{filename}");
        if !self.cache.contains_key(&key) {
            let path = self.golden_dir.join(filename);
            let cases = Self::load_from_path(&path)?;
            self.cache.insert(key.clone(), cases);
        }

        Ok(self
            .cache
            .get(&key)
            .ok_or_else(|| format!("Failed to get golden test {filename}"))?)
    }

    /// Load test cases from a path
    #[allow(dead_code)]
    fn load_from_path(path: &PathBuf) -> Result<Vec<MorphTestCase>, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read {path:?}: {e}"))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse {path:?}: {e}"))
    }

    /// Clear the cache
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for FixtureManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Sample text generator for testing
pub struct SampleTextGenerator;

impl SampleTextGenerator {
    /// Generate basic Korean sentences
    #[must_use]
    pub fn basic_sentences() -> Vec<String> {
        vec![
            "안녕하세요".to_string(),
            "감사합니다".to_string(),
            "좋은 아침입니다".to_string(),
            "오늘 날씨가 좋네요".to_string(),
            "밥 먹었어요?".to_string(),
        ]
    }

    /// Generate complex sentences with various grammatical structures
    #[must_use]
    pub fn complex_sentences() -> Vec<String> {
        vec![
            "저는 한국어를 공부하고 있습니다".to_string(),
            "서울은 대한민국의 수도입니다".to_string(),
            "내일 친구를 만나러 갈 예정이에요".to_string(),
            "그 영화는 정말 재미있었어요".to_string(),
            "커피를 마시면서 책을 읽었습니다".to_string(),
        ]
    }

    /// Generate sentences with technical terms
    #[must_use]
    pub fn technical_sentences() -> Vec<String> {
        vec![
            "인공지능은 현대 기술의 핵심입니다".to_string(),
            "데이터베이스 최적화가 필요합니다".to_string(),
            "클라우드 컴퓨팅이 급속도로 발전하고 있습니다".to_string(),
            "머신러닝 알고리즘을 학습시켰습니다".to_string(),
            "API 서버를 구축했습니다".to_string(),
        ]
    }

    /// Generate edge case sentences
    #[must_use]
    pub fn edge_cases() -> Vec<String> {
        vec![
            String::new(),            // Empty string
            " ".to_string(),          // Single space
            "ㅋㅋㅋ".to_string(),     // Consonants only
            "123".to_string(),        // Numbers only
            "Hello 안녕".to_string(), // Mixed Korean/English
            "가나다@#$".to_string(),  // With symbols
            "ㄱ".to_string(),         // Single consonant
            "ㅏ".to_string(),         // Single vowel
            "ABC".to_string(),        // English only
            "😀🎉".to_string(),       // Emojis
        ]
    }

    /// Generate sentences for noun extraction testing
    #[must_use]
    pub fn noun_sentences() -> Vec<String> {
        vec![
            "사과와 바나나를 샀어요".to_string(),
            "서울대학교는 유명합니다".to_string(),
            "김철수 씨가 왔습니다".to_string(),
            "컴퓨터 프로그래밍을 배웁니다".to_string(),
            "대한민국 축구 국가대표팀".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_manager_new() {
        let manager = FixtureManager::new();
        assert!(manager.cache.is_empty());
    }

    #[test]
    fn test_sample_text_generator() {
        let basic = SampleTextGenerator::basic_sentences();
        assert!(!basic.is_empty());
        assert_eq!(basic[0], "안녕하세요");

        let complex = SampleTextGenerator::complex_sentences();
        assert!(!complex.is_empty());

        let technical = SampleTextGenerator::technical_sentences();
        assert!(!technical.is_empty());

        let edge = SampleTextGenerator::edge_cases();
        assert!(!edge.is_empty());
        assert_eq!(edge[0], "");

        let nouns = SampleTextGenerator::noun_sentences();
        assert!(!nouns.is_empty());
    }
}
