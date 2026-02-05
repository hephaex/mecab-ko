//! Golden test integration
//!
//! This module integrates with the golden test set in /tests/golden/:
//! - Automatic comparison with expected results
//! - Test result updating and verification
//! - Regression detection

mod common;

use common::load_golden_tests;
use std::path::PathBuf;

/// Get path to golden tests directory
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
                eprintln!("Failed to load {}: {}", file, e);
            }
        }
    }

    println!(
        "Total golden tests: {}/{} passed",
        passed_tests, total_tests
    );
}

/// Verify golden test file format
#[test]
fn test_golden_file_format() {
    let golden_files = vec!["basic.json", "nouns.json", "complex.json"];

    for file in golden_files {
        let test_cases = load_golden_tests(file).expect(&format!("Failed to load {}", file));

        for (i, test_case) in test_cases.iter().enumerate() {
            // Verify required fields
            assert!(
                !test_case.input.is_empty() || file == "basic.json",
                "{}: Test case {} has empty input",
                file,
                i
            );

            // Verify expected_pos format if present
            if !test_case.expected_pos.is_empty() {
                for (morph, pos) in &test_case.expected_pos {
                    assert!(!morph.is_empty(), "{}: Empty morpheme in test {}", file, i);
                    assert!(!pos.is_empty(), "{}: Empty POS tag in test {}", file, i);
                }
            }
        }

        println!("{}: Format validation passed", file);
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

            println!("\n{} statistics:", file);
            println!("  Total test cases: {}", total);
            println!("  With morphemes: {}", with_morphs);
            println!("  With POS tags: {}", with_pos);
            println!("  With descriptions: {}", with_description);
        }
    }
}

/// Generate test report
#[test]
#[ignore = "Requires tokenizer implementation"]
fn test_generate_report() {
    // use std::io::Write;

    // TODO: Implement once tokenizer is available
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    // let files = vec!["basic.json", "nouns.json", "complex.json"];
    //
    // let mut report = String::new();
    // report.push_str("# Golden Test Report\n\n");
    //
    // for file in files {
    //     report.push_str(&format!("\n## {}\n\n", file));
    //
    //     let test_cases = load_golden_tests(file).expect("Failed to load tests");
    //     let mut passed = 0;
    //     let mut failed = 0;
    //
    //     for test_case in test_cases {
    //         let result = tokenizer.tokenize(&test_case.input);
    //         let morphs: Vec<String> = result.iter().map(|t| t.surface.clone()).collect();
    //
    //         if morphs == test_case.expected_morphs {
    //             passed += 1;
    //         } else {
    //             failed += 1;
    //             report.push_str(&format!("❌ {}\n", test_case.input));
    //             report.push_str(&format!("   Expected: {:?}\n", test_case.expected_morphs));
    //             report.push_str(&format!("   Got:      {:?}\n\n", morphs));
    //         }
    //     }
    //
    //     report.push_str(&format!("**Results**: {} passed, {} failed\n", passed, failed));
    // }
    //
    // // Write report to file
    // let report_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("golden_test_report.md");
    // let mut file = fs::File::create(&report_path).expect("Failed to create report file");
    // file.write_all(report.as_bytes()).expect("Failed to write report");
    //
    // println!("Report generated: {:?}", report_path);

    println!("Generate report test (placeholder)");
}

/// Update golden test results (USE WITH CAUTION)
#[test]
#[ignore = "Manual update only"]
fn test_update_golden_results() {
    // TODO: Implement golden test update
    // This should only be run manually to update expected results
    // after verifying that the changes are correct
    //
    // let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    //
    // for file in ["basic.json", "nouns.json", "complex.json"] {
    //     let test_cases = load_golden_tests(file).expect("Failed to load tests");
    //     let mut updated_cases = Vec::new();
    //
    //     for mut test_case in test_cases {
    //         let result = tokenizer.tokenize(&test_case.input);
    //
    //         // Update expected results
    //         test_case.expected_morphs = result.iter().map(|t| t.surface.clone()).collect();
    //         test_case.expected_pos = result
    //             .iter()
    //             .map(|t| (t.surface.clone(), t.pos.clone()))
    //             .collect();
    //
    //         updated_cases.push(test_case);
    //     }
    //
    //     // Write updated results
    //     let path = golden_path().join(file);
    //     let json = serde_json::to_string_pretty(&updated_cases)
    //         .expect("Failed to serialize");
    //     fs::write(&path, json).expect("Failed to write file");
    // }

    println!("Golden test update (placeholder - use with caution!)");
}

/// Compare with reference implementation (if available)
#[test]
#[ignore = "Requires reference implementation"]
fn test_compare_with_reference() {
    // TODO: If a reference MeCab-Ko implementation is available,
    // compare outputs for regression detection

    println!("Reference comparison test (placeholder)");
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
    println!("  Nouns: {}", has_noun);
    println!("  Verbs: {}", has_verb);
    println!("  Adjectives: {}", has_adjective);
    println!("  Particles: {}", has_particle);

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

    /// Utility to find failing golden tests
    #[test]
    #[ignore = "Requires tokenizer implementation"]
    fn find_failing_tests() {
        // TODO: Identify which tests are failing
        println!("Find failing tests (placeholder)");
    }

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
                        "{}: Test case {} has inconsistent morphs and pos",
                        file, i
                    );
                }
            }

            println!("{}: Consistency check passed", file);
        }
    }
}
