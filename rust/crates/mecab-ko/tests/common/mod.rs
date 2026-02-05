//! Common test utilities and fixtures
//!
//! This module provides shared functionality for integration tests.

pub mod fixtures;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Test fixture for morphological analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MorphTestCase {
    /// Input text to analyze
    pub input: String,
    /// Expected morphemes
    #[serde(default)]
    pub expected_morphs: Vec<String>,
    /// Expected morpheme-POS pairs
    #[serde(default)]
    pub expected_pos: Vec<(String, String)>,
    /// Test description
    #[serde(default)]
    pub description: Option<String>,
    /// Test category
    #[serde(default)]
    pub category: Option<String>,
}

/// Test result comparison
#[derive(Debug, Clone, PartialEq)]
pub struct TestResult {
    /// Whether the test passed
    pub passed: bool,
    /// Expected output
    pub expected: String,
    /// Actual output
    pub actual: String,
    /// Difference description
    pub diff: Option<String>,
}

/// Load test fixtures from JSON file
///
/// # Arguments
///
/// * `filename` - Name of the fixture file (relative to fixtures directory)
///
/// # Returns
///
/// Vector of test cases
///
/// # Errors
///
/// Returns error if file cannot be read or parsed
pub fn load_fixtures(filename: &str) -> Result<Vec<MorphTestCase>, Box<dyn std::error::Error>> {
    let fixtures_path = get_fixtures_path();
    let file_path = fixtures_path.join(filename);

    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read fixture file {file_path:?}: {e}"))?;

    let cases: Vec<MorphTestCase> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse fixture file {file_path:?}: {e}"))?;

    Ok(cases)
}

/// Load golden test cases
///
/// # Arguments
///
/// * `filename` - Name of the golden test file
///
/// # Returns
///
/// Vector of test cases
///
/// # Errors
///
/// Returns error if file cannot be read or parsed
pub fn load_golden_tests(filename: &str) -> Result<Vec<MorphTestCase>, Box<dyn std::error::Error>> {
    let golden_path = get_golden_path();
    let file_path = golden_path.join(filename);

    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read golden test file {file_path:?}: {e}"))?;

    let cases: Vec<MorphTestCase> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse golden test file {file_path:?}: {e}"))?;

    Ok(cases)
}

/// Get path to fixtures directory
pub fn get_fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Get path to golden tests directory
pub fn get_golden_path() -> PathBuf {
    // CARGO_MANIFEST_DIR = /home/mare/mecab-ko/rust/crates/mecab-ko
    // Golden tests are in /home/mare/mecab-ko/tests/golden
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates
        .and_then(|p| p.parent()) // rust
        .and_then(|p| p.parent()) // mecab-ko
        .expect("Failed to get project root directory")
        .join("tests")
        .join("golden")
}

/// Compare expected and actual morphemes
///
/// # Arguments
///
/// * `expected` - Expected morphemes
/// * `actual` - Actual morphemes
///
/// # Returns
///
/// Test result with comparison details
pub fn compare_morphs(expected: &[String], actual: &[String]) -> TestResult {
    let passed = expected == actual;
    let expected_str = format!("{expected:?}");
    let actual_str = format!("{actual:?}");

    let diff = if !passed {
        Some(generate_diff(expected, actual))
    } else {
        None
    };

    TestResult {
        passed,
        expected: expected_str,
        actual: actual_str,
        diff,
    }
}

/// Compare expected and actual POS tags
///
/// # Arguments
///
/// * `expected` - Expected morpheme-POS pairs
/// * `actual` - Actual morpheme-POS pairs
///
/// # Returns
///
/// Test result with comparison details
pub fn compare_pos_tags(expected: &[(String, String)], actual: &[(String, String)]) -> TestResult {
    let passed = expected == actual;
    let expected_str = format!("{expected:?}");
    let actual_str = format!("{actual:?}");

    let diff = if !passed {
        Some(generate_pos_diff(expected, actual))
    } else {
        None
    };

    TestResult {
        passed,
        expected: expected_str,
        actual: actual_str,
        diff,
    }
}

/// Generate human-readable diff for morphemes
fn generate_diff(expected: &[String], actual: &[String]) -> String {
    let mut diff = String::new();
    diff.push_str("Morpheme differences:\n");

    let max_len = expected.len().max(actual.len());
    for i in 0..max_len {
        let exp = expected.get(i).map(|s| s.as_str()).unwrap_or("<missing>");
        let act = actual.get(i).map(|s| s.as_str()).unwrap_or("<missing>");

        if exp != act {
            diff.push_str(&format!("  Position {i}: expected '{exp}', got '{act}'\n"));
        }
    }

    diff
}

/// Generate human-readable diff for POS tags
fn generate_pos_diff(expected: &[(String, String)], actual: &[(String, String)]) -> String {
    let mut diff = String::new();
    diff.push_str("POS tag differences:\n");

    let max_len = expected.len().max(actual.len());
    for i in 0..max_len {
        let exp = expected
            .get(i)
            .map(|(m, p)| format!("{m}/{p}"))
            .unwrap_or_else(|| "<missing>".to_string());
        let act = actual
            .get(i)
            .map(|(m, p)| format!("{m}/{p}"))
            .unwrap_or_else(|| "<missing>".to_string());

        if exp != act {
            diff.push_str(&format!("  Position {i}: expected '{exp}', got '{act}'\n"));
        }
    }

    diff
}

/// Assert that test result passed, with detailed error message
#[macro_export]
macro_rules! assert_test_result {
    ($result:expr, $test_case:expr) => {
        if !$result.passed {
            let mut msg = format!("Test failed for input: '{}'", $test_case.input);
            if let Some(desc) = &$test_case.description {
                msg.push_str(&format!("\nDescription: {desc}"));
            }
            msg.push_str(&format!("\nExpected: {}", $result.expected));
            msg.push_str(&format!("\nActual:   {}", $result.actual));
            if let Some(diff) = &$result.diff {
                msg.push_str(&format!("\n{diff}"));
            }
            panic!("{msg}");
        }
    };
}

/// Create a sample dictionary for testing
///
/// This is a stub that will be replaced with actual dictionary creation
/// once the dictionary builder is implemented.
pub fn create_test_dict() -> PathBuf {
    // For now, return a path that would contain a test dictionary
    // This will be implemented when DIC-001 is complete
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test_dict")
}

/// Performance benchmarking utilities
pub mod perf {
    use std::time::{Duration, Instant};

    /// Performance measurement result
    #[derive(Debug, Clone)]
    pub struct PerfResult {
        /// Operation name
        pub name: String,
        /// Duration
        pub duration: Duration,
        /// Number of iterations
        pub iterations: usize,
        /// Average time per iteration
        pub avg_per_iter: Duration,
    }

    impl PerfResult {
        /// Format as human-readable string
        pub fn format(&self) -> String {
            format!(
                "{}: {:.2}ms total, {:.2}μs per iteration ({} iterations)",
                self.name,
                self.duration.as_secs_f64() * 1000.0,
                self.avg_per_iter.as_secs_f64() * 1_000_000.0,
                self.iterations
            )
        }
    }

    /// Measure performance of a function
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the operation
    /// * `iterations` - Number of times to run
    /// * `f` - Function to benchmark
    ///
    /// # Returns
    ///
    /// Performance measurement result
    pub fn measure<F>(name: &str, iterations: usize, mut f: F) -> PerfResult
    where
        F: FnMut(),
    {
        let start = Instant::now();
        for _ in 0..iterations {
            f();
        }
        let duration = start.elapsed();

        PerfResult {
            name: name.to_string(),
            duration,
            iterations,
            avg_per_iter: duration / iterations as u32,
        }
    }

    /// Assert that performance is within acceptable range
    ///
    /// # Arguments
    ///
    /// * `result` - Performance result to check
    /// * `max_avg_micros` - Maximum acceptable average time in microseconds
    ///
    /// # Panics
    ///
    /// Panics if performance is worse than threshold
    pub fn assert_performance(result: &PerfResult, max_avg_micros: f64) {
        let actual_micros = result.avg_per_iter.as_secs_f64() * 1_000_000.0;
        assert!(
            actual_micros <= max_avg_micros,
            "Performance regression detected for '{}': expected ≤{:.2}μs, got {:.2}μs",
            result.name,
            max_avg_micros,
            actual_micros
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_morphs_equal() {
        let expected = vec!["안녕".to_string(), "하".to_string(), "세요".to_string()];
        let actual = vec!["안녕".to_string(), "하".to_string(), "세요".to_string()];

        let result = compare_morphs(&expected, &actual);
        assert!(result.passed);
        assert!(result.diff.is_none());
    }

    #[test]
    fn test_compare_morphs_different() {
        let expected = vec!["안녕".to_string(), "하".to_string(), "세요".to_string()];
        let actual = vec!["안녕".to_string(), "하세요".to_string()];

        let result = compare_morphs(&expected, &actual);
        assert!(!result.passed);
        assert!(result.diff.is_some());
    }

    #[test]
    fn test_compare_pos_tags_equal() {
        let expected = vec![
            ("안녕".to_string(), "NNG".to_string()),
            ("하".to_string(), "XSV".to_string()),
        ];
        let actual = vec![
            ("안녕".to_string(), "NNG".to_string()),
            ("하".to_string(), "XSV".to_string()),
        ];

        let result = compare_pos_tags(&expected, &actual);
        assert!(result.passed);
    }

    #[test]
    fn test_fixtures_path() {
        let path = get_fixtures_path();
        assert!(path.ends_with("tests/fixtures"));
    }

    #[test]
    fn test_golden_path() {
        let path = get_golden_path();
        assert!(path.to_string_lossy().contains("tests/golden"));
    }
}
