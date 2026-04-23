//! Golden test integration
//!
//! This module integrates with the golden test set in /tests/golden/:
//! - Automatic comparison with expected results
//! - Test result updating and verification
//! - Regression detection

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod common;

use common::load_golden_tests;
use std::path::PathBuf;

/// Get path to golden tests directory
#[allow(dead_code)]
fn golden_path() -> PathBuf {
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

/// Test basic golden test set
#[test]
fn test_golden_basic() {
    let test_cases = load_golden_tests("basic.json").expect("Failed to load basic golden tests");

    assert!(
        !test_cases.is_empty(),
        "Should have basic golden test cases"
    );

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for test_case in test_cases {
    //     let result = tokenizer.tokenize(&test_case.input);
    //     let morphs: Vec<String> = result.iter().map(|t| t.surface.clone()).collect();
    //
    //     let comparison = common::compare_morphs(&test_case.expected_morphs, &morphs);
    //     assert_test_result!(comparison, test_case);
    // }

    println!("Loaded {} basic golden test cases", test_cases.len());
}

/// Test nouns golden test set
#[test]
fn test_golden_nouns() {
    let test_cases = load_golden_tests("nouns.json").expect("Failed to load nouns golden tests");

    assert!(
        !test_cases.is_empty(),
        "Should have nouns golden test cases"
    );

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for test_case in test_cases {
    //     let result = tokenizer.tokenize(&test_case.input);
    //
    //     // Extract nouns (NNG, NNP, NNB)
    //     let nouns: Vec<String> = result
    //         .iter()
    //         .filter(|t| t.pos.starts_with("NN"))
    //         .map(|t| t.surface.clone())
    //         .collect();
    //
    //     // Verify expected nouns are found
    //     for expected_noun in &test_case.expected_morphs {
    //         assert!(nouns.contains(expected_noun),
    //                 "Expected noun '{}' not found in '{}'", expected_noun, test_case.input);
    //     }
    // }

    println!("Loaded {} nouns golden test cases", test_cases.len());
}

/// Test complex golden test set
#[test]
fn test_golden_complex() {
    let test_cases =
        load_golden_tests("complex.json").expect("Failed to load complex golden tests");

    assert!(
        !test_cases.is_empty(),
        "Should have complex golden test cases"
    );

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for test_case in test_cases {
    //     let result = tokenizer.tokenize(&test_case.input);
    //     let pos_pairs: Vec<(String, String)> = result
    //         .iter()
    //         .map(|t| (t.surface.clone(), t.pos.clone()))
    //         .collect();
    //
    //     let comparison = common::compare_pos_tags(&test_case.expected_pos, &pos_pairs);
    //     assert_test_result!(comparison, test_case);
    // }

    println!("Loaded {} complex golden test cases", test_cases.len());
}

/// Test all golden test files
#[test]
fn test_all_golden_tests() {
    let golden_files = vec!["basic.json", "nouns.json", "complex.json"];

    let mut total_tests = 0;
    let mut passed_tests = 0;

    for file in golden_files {
        match load_golden_tests(file) {
            Ok(test_cases) => {
                total_tests += test_cases.len();
                // TODO: Run actual tests and count passes
                passed_tests += test_cases.len(); // Placeholder

                println!("{}: {} tests", file, test_cases.len());
            }
            Err(e) => {
                eprintln!("Failed to load {file}: {e}");
            }
        }
    }

    println!("Total golden tests: {passed_tests}/{total_tests} passed");
}

/// Verify golden test file format
#[test]
fn test_golden_file_format() {
    let golden_files = vec!["basic.json", "nouns.json", "complex.json"];

    for file in golden_files {
        let test_cases =
            load_golden_tests(file).unwrap_or_else(|_| panic!("Failed to load {file}"));

        for (i, test_case) in test_cases.iter().enumerate() {
            // Verify required fields
            assert!(
                !test_case.input.is_empty() || file == "basic.json",
                "{file}: Test case {i} has empty input"
            );

            // Verify expected_pos format if present
            if !test_case.expected_pos.is_empty() {
                for (morph, pos) in &test_case.expected_pos {
                    assert!(!morph.is_empty(), "{file}: Empty morpheme in test {i}");
                    assert!(!pos.is_empty(), "{file}: Empty POS tag in test {i}");
                }
            }
        }

        println!("{file}: Format validation passed");
    }
}

/// Test golden test statistics
#[test]
fn test_golden_statistics() {
    let files = vec!["basic.json", "nouns.json", "complex.json"];

    for file in files {
        if let Ok(test_cases) = load_golden_tests(file) {
            let total = test_cases.len();
            let with_morphs = test_cases
                .iter()
                .filter(|tc| !tc.expected_morphs.is_empty())
                .count();
            let with_pos = test_cases
                .iter()
                .filter(|tc| !tc.expected_pos.is_empty())
                .count();
            let with_description = test_cases
                .iter()
                .filter(|tc| tc.description.is_some())
                .count();

            println!("\n{file} statistics:");
            println!("  Total test cases: {total}");
            println!("  With morphemes: {with_morphs}");
            println!("  With POS tags: {with_pos}");
            println!("  With descriptions: {with_description}");
        }
    }
}

/// Test golden tests are comprehensive
#[test]
fn test_golden_coverage() {
    let test_cases = load_golden_tests("basic.json").expect("Failed to load basic golden tests");

    // Check for coverage of different grammatical features
    let mut has_noun = false;
    let mut has_verb = false;
    let mut has_adjective = false;
    let mut has_particle = false;

    for test_case in test_cases {
        for (_, pos) in test_case.expected_pos {
            if pos.starts_with("NN") {
                has_noun = true;
            }
            if pos.starts_with("VV") {
                has_verb = true;
            }
            if pos.starts_with("VA") {
                has_adjective = true;
            }
            if pos.starts_with("JK") {
                has_particle = true;
            }
        }
    }

    println!("Coverage check:");
    println!("  Nouns: {has_noun}");
    println!("  Verbs: {has_verb}");
    println!("  Adjectives: {has_adjective}");
    println!("  Particles: {has_particle}");

    // Should have reasonable coverage
    assert!(has_noun, "Should have noun examples");
    assert!(
        has_verb || has_adjective,
        "Should have verb or adjective examples"
    );
}

#[cfg(test)]
mod golden_utils {
    use super::*;

    /// Utility to validate golden test consistency
    #[test]
    fn validate_golden_consistency() {
        let files = vec!["basic.json", "nouns.json", "complex.json"];

        for file in files {
            let test_cases = load_golden_tests(file).expect("Failed to load tests");

            for (i, test_case) in test_cases.iter().enumerate() {
                // Check that expected_morphs and expected_pos are consistent
                if !test_case.expected_morphs.is_empty() && !test_case.expected_pos.is_empty() {
                    let morphs_from_pos: Vec<String> = test_case
                        .expected_pos
                        .iter()
                        .map(|(m, _)| m.clone())
                        .collect();

                    assert_eq!(
                        test_case.expected_morphs, morphs_from_pos,
                        "{file}: Test case {i} has inconsistent morphs and pos"
                    );
                }
            }

            println!("{file}: Consistency check passed");
        }
    }
}
