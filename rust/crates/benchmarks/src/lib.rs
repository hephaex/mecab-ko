//! MeCab-Ko Benchmarks
//!
//! This crate contains performance benchmarks for the mecab-ko project.
//! It is not published and is only used for internal testing.

#![allow(unused)]

use serde::{Deserialize, Serialize};

/// Korean text samples for benchmarking
pub mod samples {
    /// Short texts (5-10 characters) - social media style
    pub const SHORT: &[&str] = &[
        "안녕하세요",
        "좋아요",
        "감사합니다",
        "잘 부탁드립니다",
        "좋은 하루 되세요",
        "오늘 날씨 좋네요",
        "맛있게 먹었어요",
        "다음에 또 올게요",
    ];

    /// Medium texts (50-100 characters) - general conversation
    pub const MEDIUM: &[&str] = &[
        "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다",
        "아버지가 방에 들어가신다는 문장을 분석해보겠습니다",
        "오늘은 날씨가 맑고 화창하여 산책하기 좋은 날입니다",
        "형태소 분석 결과는 품사 태깅과 함께 제공됩니다",
        "자연어 처리는 인공지능의 중요한 응용 분야 중 하나입니다",
    ];

    /// Long texts (200+ characters) - news article style
    pub const LONG: &[&str] = &[
        "대한민국의 수도인 서울은 조선시대부터 600년이 넘는 역사를 가진 도시로서 \
         현대적인 빌딩과 전통 한옥이 조화를 이루며 독특한 도시 경관을 형성하고 있습니다",
        "인공지능 기술의 발전으로 자연어 처리 분야에서도 놀라운 성과들이 나타나고 있으며 \
         특히 대규모 언어 모델의 등장은 기계 번역과 텍스트 생성 분야에 혁신적인 변화를 가져왔습니다",
        "한국어는 교착어로 분류되며 조사와 어미가 발달한 언어로서 형태소 분석의 중요성이 \
         다른 언어들에 비해 더욱 강조되며 이에 따라 정확한 형태소 분석기의 개발이 필수적입니다",
    ];

    /// Technical texts with specialized vocabulary
    pub const TECHNICAL: &[&str] = &[
        "Rust 프로그래밍 언어는 메모리 안전성을 보장하면서도 고성능을 제공합니다",
        "Double-Array Trie는 효율적인 문자열 검색을 위한 자료구조입니다",
        "Viterbi 알고리즘은 HMM에서 최적 경로를 찾는 동적 프로그래밍 기법입니다",
        "형태소 분석은 토큰화, 품사 태깅, 구문 분석의 전처리 단계입니다",
    ];

    /// Mixed texts (Korean + English + numbers + symbols)
    pub const MIXED: &[&str] = &[
        "Apple의 iPhone 15 Pro는 A17 칩을 탑재했습니다",
        "2024년 1월 1일부터 새로운 정책이 시행됩니다",
        "AI(Artificial Intelligence)는 인공지능을 의미합니다",
        "서울시 강남구 역삼동 123-45번지",
        "https://example.com 웹사이트를 방문하세요",
    ];

    /// All samples combined
    pub fn all() -> Vec<&'static str> {
        SHORT
            .iter()
            .chain(MEDIUM.iter())
            .chain(LONG.iter())
            .chain(TECHNICAL.iter())
            .chain(MIXED.iter())
            .copied()
            .collect()
    }
}

/// Benchmark result data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Mean execution time in nanoseconds
    pub mean_ns: f64,
    /// Standard deviation
    pub std_dev_ns: f64,
    /// Throughput in elements per second
    pub throughput_per_sec: Option<f64>,
    /// Memory usage in bytes (if available)
    pub memory_bytes: Option<usize>,
}

/// Helper function to generate repeated text
pub fn generate_text(base: &str, repeat: usize) -> String {
    base.repeat(repeat)
}

/// Helper function to create batch of texts
pub fn create_batch(template: &str, count: usize) -> Vec<String> {
    (0..count).map(|_| template.to_string()).collect()
}

/// Helper function to create mixed-length batch
pub fn create_mixed_batch(count: usize) -> Vec<String> {
    let templates = samples::all();
    (0..count)
        .map(|i| templates[i % templates.len()].to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_samples_not_empty() {
        assert!(!samples::SHORT.is_empty());
        assert!(!samples::MEDIUM.is_empty());
        assert!(!samples::LONG.is_empty());
        assert!(!samples::all().is_empty());
    }

    #[test]
    fn test_generate_text() {
        let text = generate_text("가", 10);
        assert_eq!(text.len(), 30); // "가" is 3 bytes in UTF-8
    }

    #[test]
    fn test_create_batch() {
        let batch = create_batch("테스트", 5);
        assert_eq!(batch.len(), 5);
        assert!(batch.iter().all(|s| s == "테스트"));
    }

    #[test]
    fn test_create_mixed_batch() {
        let batch = create_mixed_batch(100);
        assert_eq!(batch.len(), 100);
    }
}
