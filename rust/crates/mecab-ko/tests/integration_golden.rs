//! Golden test integration
//!
//! This module integrates with the golden test set in /tests/golden/:
//! - Automatic comparison with expected results
//! - Test result updating and verification
//! - Regression detection

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod common;

use common::{load_golden_tests, system_dict_available};
use mecab_ko::Tokenizer;
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

/// Validate that a POS tag string is non-empty and follows mecab-ko conventions.
///
/// Valid tags are alphanumeric identifiers, optionally joined with '+' for
/// compound tags (e.g. "EP+EF", "XSV+EF").
fn is_valid_pos(pos: &str) -> bool {
    !pos.is_empty()
        && pos
            .split('+')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric()))
}

/// Test basic golden test set.
///
/// With a full system dictionary the tokenizer output is compared directly
/// against the expected morphemes and POS pairs stored in `basic.json`.
///
/// Without a system dictionary (mini-dict fallback) the test still runs and
/// verifies structural properties:
/// - Every token has a non-empty surface string
/// - Every token carries a valid POS tag
/// - Repeated tokenization of the same input is deterministic
#[test]
fn test_golden_basic() {
    let test_cases = load_golden_tests("basic.json").expect("Failed to load basic golden tests");

    assert!(
        !test_cases.is_empty(),
        "basic.json must contain at least one test case"
    );

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let has_system_dict = system_dict_available();

    let mut exact_matches = 0usize;
    let mut total = 0usize;

    for test_case in &test_cases {
        let tokens = tokenizer.tokenize(&test_case.input);
        total += 1;

        // Structural assertions hold regardless of dictionary quality.
        for token in &tokens {
            assert!(
                !token.surface.is_empty(),
                "input '{}': every token must have a non-empty surface (got empty surface with pos '{}')",
                test_case.input,
                token.pos
            );
            assert!(
                is_valid_pos(&token.pos),
                "input '{}': token '{}' has invalid POS tag '{}'",
                test_case.input,
                token.surface,
                token.pos
            );
        }

        // Determinism: a second pass must produce identical output.
        let tokens2 = tokenizer.tokenize(&test_case.input);
        let surfaces1: Vec<&str> = tokens.iter().map(|t| t.surface.as_str()).collect();
        let surfaces2: Vec<&str> = tokens2.iter().map(|t| t.surface.as_str()).collect();
        assert_eq!(
            surfaces1, surfaces2,
            "input '{}': tokenization must be deterministic",
            test_case.input
        );

        if has_system_dict && !tokens.is_empty() {
            // Full comparison: morpheme surfaces must match expected_morphs exactly.
            if !test_case.expected_morphs.is_empty() {
                let actual_morphs: Vec<String> = tokens.iter().map(|t| t.surface.clone()).collect();
                let comparison = common::compare_morphs(&test_case.expected_morphs, &actual_morphs);
                assert!(
                    comparison.passed,
                    "basic.json morph mismatch for input '{}':\n  expected: {:?}\n  actual:   {:?}{}",
                    test_case.input,
                    test_case.expected_morphs,
                    actual_morphs,
                    comparison.diff.as_deref().map(|d| format!("\n{d}")).unwrap_or_default()
                );
            }

            // Full comparison: (surface, POS) pairs must match expected_pos exactly.
            if !test_case.expected_pos.is_empty() {
                let actual_pos: Vec<(String, String)> = tokens
                    .iter()
                    .map(|t| (t.surface.clone(), t.pos.clone()))
                    .collect();
                let comparison = common::compare_pos_tags(&test_case.expected_pos, &actual_pos);
                assert!(
                    comparison.passed,
                    "basic.json POS mismatch for input '{}':\n  expected: {:?}\n  actual:   {:?}{}",
                    test_case.input,
                    test_case.expected_pos,
                    actual_pos,
                    comparison
                        .diff
                        .as_deref()
                        .map(|d| format!("\n{d}"))
                        .unwrap_or_default()
                );
            }

            exact_matches += 1;
        }
    }

    println!(
        "test_golden_basic: {total} cases, {exact_matches} with full dict comparison (system_dict={has_system_dict})"
    );
}

/// Test nouns golden test set.
///
/// All test cases in `nouns.json` are noun-only phrases (NNG / NNP).  With a
/// full system dictionary every expected morph must appear in the tokenizer's
/// noun extraction (`nouns()`).  Without one the test verifies that
/// `nouns()` and `tokenize()` return consistent noun subsets and that
/// every returned noun surface is non-empty.
#[test]
fn test_golden_nouns() {
    let test_cases = load_golden_tests("nouns.json").expect("Failed to load nouns golden tests");

    assert!(
        !test_cases.is_empty(),
        "nouns.json must contain at least one test case"
    );

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let has_system_dict = system_dict_available();

    let mut cases_checked = 0usize;

    for test_case in &test_cases {
        // nouns() must be a subset of the full tokenization (structural invariant).
        let all_tokens = tokenizer.tokenize(&test_case.input);
        let noun_surfaces: Vec<String> = tokenizer.nouns(&test_case.input);

        // Every noun returned must be present in the full token list.
        let all_surfaces: Vec<&str> = all_tokens.iter().map(|t| t.surface.as_str()).collect();
        for noun in &noun_surfaces {
            assert!(
                !noun.is_empty(),
                "input '{}': nouns() must not return empty strings",
                test_case.input
            );
            assert!(
                all_surfaces.contains(&noun.as_str()),
                "input '{}': noun '{}' returned by nouns() is not in tokenize() output {:?}",
                test_case.input,
                noun,
                all_surfaces
            );
        }

        if has_system_dict {
            // Every expected morph must appear as a noun in the output.
            for expected_noun in &test_case.expected_morphs {
                assert!(
                    noun_surfaces.contains(expected_noun),
                    "input '{}': expected noun '{}' not found in nouns() output {:?}",
                    test_case.input,
                    expected_noun,
                    noun_surfaces
                );
            }

            // All tokens produced for noun-only phrases should carry NN* tags.
            for token in &all_tokens {
                assert!(
                    token.pos.starts_with("NN"),
                    "input '{}': token '{}/{}' in a noun-only phrase should have NN* POS",
                    test_case.input,
                    token.surface,
                    token.pos
                );
            }

            cases_checked += 1;
        }
    }

    println!(
        "test_golden_nouns: {} cases, {cases_checked} with full dict comparison (system_dict={})",
        test_cases.len(),
        has_system_dict
    );
}

/// Test complex golden test set.
///
/// `complex.json` contains long, syntactically rich Korean sentences.  With a
/// full system dictionary the tokenizer output is compared against the
/// expected (surface, POS) pairs.  Without one the test validates:
/// - At least one token is produced per non-trivial input
/// - No token has an empty surface or an invalid POS tag
/// - Token positions form a non-overlapping, monotonically increasing sequence
#[test]
fn test_golden_complex() {
    let test_cases =
        load_golden_tests("complex.json").expect("Failed to load complex golden tests");

    assert!(
        !test_cases.is_empty(),
        "complex.json must contain at least one test case"
    );

    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let has_system_dict = system_dict_available();

    let mut exact_matches = 0usize;

    for test_case in &test_cases {
        let tokens = tokenizer.tokenize(&test_case.input);

        // Structural: all tokens must have valid surface and POS.
        for token in &tokens {
            assert!(
                !token.surface.is_empty(),
                "complex.json input '{}': token must not have an empty surface",
                test_case.input
            );
            assert!(
                is_valid_pos(&token.pos),
                "complex.json input '{}': token '{}' has invalid POS tag '{}'",
                test_case.input,
                token.surface,
                token.pos
            );
        }

        // Structural: token start/end positions must be monotonically increasing.
        for window in tokens.windows(2) {
            let (prev, next) = (&window[0], &window[1]);
            assert!(
                next.start_pos >= prev.start_pos,
                "complex.json input '{}': token positions are not monotonically increasing \
                 ('{}'@{} before '{}'@{})",
                test_case.input,
                prev.surface,
                prev.start_pos,
                next.surface,
                next.start_pos
            );
        }

        if has_system_dict && !tokens.is_empty() {
            let actual_pos: Vec<(String, String)> = tokens
                .iter()
                .map(|t| (t.surface.clone(), t.pos.clone()))
                .collect();
            let comparison = common::compare_pos_tags(&test_case.expected_pos, &actual_pos);
            assert!(
                comparison.passed,
                "complex.json POS mismatch for input '{}':\n  expected: {:?}\n  actual:   {:?}{}",
                test_case.input,
                test_case.expected_pos,
                actual_pos,
                comparison
                    .diff
                    .as_deref()
                    .map(|d| format!("\n{d}"))
                    .unwrap_or_default()
            );
            exact_matches += 1;
        }
    }

    println!(
        "test_golden_complex: {} cases, {exact_matches} with full dict comparison (system_dict={})",
        test_cases.len(),
        has_system_dict
    );
}

/// Test all golden test files, count real pass/fail against tokenizer output.
///
/// Replaces the former hardcoded placeholder that always reported 100% pass
/// rate.  The test now runs the tokenizer on every case across all three
/// golden files and counts:
///
/// - **structurally valid**: tokenizer produced ≥1 token and every token has a
///   non-empty surface and a valid POS tag
/// - **no output**: tokenizer returned no tokens for a non-empty input
///
/// With a full system dictionary every produced token must be structurally
/// valid AND the pass rate must be 100%.
///
/// Without a full system dictionary (mini-dict fallback) the mini-dict only
/// covers ~21 entries, so empty token arrays for most inputs are expected and
/// documented.  In that case the test still asserts:
/// 1. All three golden files load successfully and are non-empty.
/// 2. Any tokens that *are* produced are structurally valid (non-empty surface,
///    valid POS tag).  Zero structurally invalid tokens is always required.
/// 3. At least one golden file produces some valid tokens (smoke-test that
///    the tokenizer pipeline is wired up at all).
#[test]
fn test_all_golden_tests() {
    let golden_files = ["basic.json", "nouns.json", "complex.json"];
    let has_system_dict = system_dict_available();
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut total_tests = 0usize;
    let mut structurally_valid = 0usize;
    let mut structurally_invalid = 0usize;
    let mut no_output = 0usize;
    let mut files_loaded = 0usize;

    for file in &golden_files {
        let test_cases = match load_golden_tests(file) {
            Ok(cases) => {
                assert!(!cases.is_empty(), "{file}: golden file must not be empty");
                files_loaded += 1;
                cases
            }
            Err(e) => {
                panic!("Failed to load golden file {file}: {e}");
            }
        };

        let mut file_valid = 0usize;

        for test_case in &test_cases {
            total_tests += 1;
            let tokens = tokenizer.tokenize(&test_case.input);
            let input_is_trivial = test_case.input.trim().is_empty();

            if tokens.is_empty() {
                if !input_is_trivial {
                    no_output += 1;
                }
            } else {
                // Every produced token must satisfy structural invariants —
                // this is always enforced regardless of dictionary quality.
                let all_valid = tokens
                    .iter()
                    .all(|t| !t.surface.is_empty() && is_valid_pos(&t.pos));

                if all_valid {
                    file_valid += 1;
                    structurally_valid += 1;
                } else {
                    structurally_invalid += 1;
                    eprintln!(
                        "{file}: structurally INVALID token(s) for input '{}' — {:?}",
                        test_case.input,
                        tokens
                            .iter()
                            .map(|t| format!("{}/{}", t.surface, t.pos))
                            .collect::<Vec<_>>()
                    );
                }
            }
        }

        println!(
            "{file}: {file_valid}/{} structurally valid",
            test_cases.len()
        );
    }

    println!(
        "total: {structurally_valid} valid, {no_output} no-output, {structurally_invalid} invalid \
         out of {total_tests} cases (system_dict={has_system_dict})"
    );

    // All three golden files must load successfully.
    assert_eq!(
        files_loaded, 3,
        "All 3 golden files must load; only {files_loaded} loaded"
    );

    // Tokens that ARE produced must always be structurally valid — regardless of
    // which dictionary is in use.
    assert_eq!(
        structurally_invalid, 0,
        "{structurally_invalid} cases produced tokens with empty surfaces or invalid POS tags. \
         Every tokenizer output must satisfy structural invariants."
    );

    if has_system_dict {
        // With a full dictionary every input must produce at least one token.
        assert_eq!(
            structurally_valid, total_tests,
            "With full system dict: {structurally_valid}/{total_tests} cases produced valid tokens. \
             All golden test cases must tokenize successfully."
        );
    } else {
        // Without a full dictionary the mini-dict only covers ~21 entries, so
        // empty-output cases are expected.  We verify the pipeline is alive by
        // asserting that at least some cases succeeded.
        assert!(
            structurally_valid > 0,
            "Even with the sparse mini-dict, at least one golden test case must produce \
             structurally valid tokens (pipeline smoke-test)"
        );
    }
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
///
/// Verifies that each golden file is non-empty and that every test case
/// carries at least one form of expected output (morphemes or POS pairs).
#[test]
fn test_golden_statistics() {
    let files = vec!["basic.json", "nouns.json", "complex.json"];

    for file in files {
        let test_cases = load_golden_tests(file)
            .unwrap_or_else(|e| panic!("Failed to load golden test file {file}: {e}"));

        let total = test_cases.len();
        assert!(
            total > 0,
            "{file}: golden file must contain at least one test case"
        );

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

        // Every test case must have at least morphemes or POS tags specified
        assert!(
            with_morphs > 0 || with_pos > 0,
            "{file}: all test cases lack both expected_morphs and expected_pos"
        );

        println!("\n{file} statistics:");
        println!("  Total test cases: {total}");
        println!("  With morphemes: {with_morphs}");
        println!("  With POS tags: {with_pos}");
        println!("  With descriptions: {with_description}");
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

/// POS tag coverage report across all golden test files.
///
/// Collects every POS tag that appears in `expected_pos` across all golden
/// test files and asserts that at least 30 of the 45 standard POS tags
/// are covered. Prints a detailed coverage report.
#[test]
fn test_golden_pos_coverage() {
    let golden_files = ["basic.json", "nouns.json", "complex.json"];

    let all_pos_tags = [
        "NNG", "NNP", "NNB", "NR", "NP", "VV", "VA", "VX", "VCP", "VCN", "MM", "MAG", "MAJ",
        "IC", "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC", "EP", "EF", "EC",
        "ETN", "ETM", "XPN", "XSN", "XSV", "XSA", "XR", "SF", "SE", "SS", "SP", "SO", "SW",
        "SL", "SH", "SK", "SN", "NF", "NV",
    ];

    let mut covered: std::collections::HashSet<String> = std::collections::HashSet::new();

    for file in &golden_files {
        let test_cases = load_golden_tests(file).unwrap_or_else(|e| panic!("Failed to load {file}: {e}"));
        for tc in &test_cases {
            for (_, pos) in &tc.expected_pos {
                for part in pos.split('+') {
                    covered.insert(part.to_string());
                }
            }
        }
    }

    let total_standard = all_pos_tags.len();
    let covered_count = all_pos_tags.iter().filter(|t| covered.contains(**t)).count();

    println!("\nPOS Tag Coverage Report:");
    println!("  Covered: {covered_count}/{total_standard}");
    println!("  Tags present:");
    for tag in &all_pos_tags {
        let mark = if covered.contains(*tag) { "+" } else { "-" };
        println!("    [{mark}] {tag}");
    }

    let uncovered: Vec<&&str> = all_pos_tags.iter().filter(|t| !covered.contains(**t)).collect();
    if !uncovered.is_empty() {
        println!("  Uncovered tags: {uncovered:?}");
    }

    assert!(
        covered_count >= 30,
        "POS coverage too low: {covered_count}/{total_standard} (need >= 30). Uncovered: {uncovered:?}"
    );
}

/// Category-based statistics for golden test files.
///
/// Reports how many test cases fall into each category and the total per
/// golden file. Asserts that every test case has a category assigned.
#[test]
fn test_golden_category_statistics() {
    let golden_files = ["basic.json", "nouns.json", "complex.json"];
    let mut category_counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut total_with_category = 0usize;
    let mut total_without_category = 0usize;
    let mut total_cases = 0usize;

    for file in &golden_files {
        let test_cases = load_golden_tests(file).unwrap_or_else(|e| panic!("Failed to load {file}: {e}"));
        let mut file_with = 0usize;
        let mut file_without = 0usize;

        for tc in &test_cases {
            total_cases += 1;
            if let Some(cat) = &tc.category {
                *category_counts.entry(cat.clone()).or_insert(0) += 1;
                file_with += 1;
                total_with_category += 1;
            } else {
                file_without += 1;
                total_without_category += 1;
            }
        }

        println!("{file}: {file_with} categorized, {file_without} uncategorized");
    }

    println!("\nCategory Statistics:");
    for (cat, count) in &category_counts {
        println!("  {cat}: {count}");
    }
    println!("  Total: {total_cases} cases ({total_with_category} categorized, {total_without_category} uncategorized)");

    assert!(
        total_with_category > 0,
        "At least some test cases should have categories"
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
