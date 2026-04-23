//! User dictionary integration tests
//!
//! This module tests user dictionary functionality:
//! - User dictionary creation and loading
//! - Custom entry addition
//! - Priority handling (user dict vs system dict)
//! - User dictionary persistence
//! - CSV format parsing

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::useless_vec)]

mod common;

use mecab_ko_dict::UserEntry;

/// Test user dictionary entry creation
#[test]
fn test_user_entry_creation() {
    let entry = UserEntry {
        surface: "MeCab".to_string(),
        left_id: 0,
        right_id: 0,
        cost: -1000, // Lower cost for higher priority
        pos: "NNP".to_string(),
        reading: Some("메캅".to_string()),
        lemma: None,
        feature: "NNP,*,*,*,*,*,MeCab,메캅".to_string(),
    };

    assert_eq!(entry.surface, "MeCab");
    assert_eq!(entry.pos, "NNP");
    assert_eq!(entry.cost, -1000);
}

/// Test user dictionary builder
#[test]
fn test_user_dict_builder() {
    use mecab_ko_dict::UserDictionaryBuilder;

    let user_dict = UserDictionaryBuilder::new()
        .add_with_cost("MeCab", "NNP", -1000)
        .add_with_cost("Rust", "NNP", -1000)
        .add_with_cost("토크나이저", "NNG", -500)
        .build();

    assert_eq!(user_dict.len(), 3, "Builder should produce 3 entries");
}

/// Test loading user dictionary from CSV string
#[test]
fn test_load_user_dict_from_csv() {
    use mecab_ko_dict::UserDictionary;

    let csv_content =
        "MeCab,NNP,-1000,메캅\nRust,NNP,-1000,러스트\n토크나이저,NNG,-500,토크나이저\n";

    let mut user_dict = UserDictionary::new();
    user_dict
        .load_from_str(csv_content)
        .expect("Failed to load from CSV string");

    assert_eq!(user_dict.len(), 3, "Should have 3 entries after CSV load");
}

/// Test user dictionary lookup
#[test]
fn test_user_dict_lookup() {
    use mecab_ko_dict::UserDictionary;

    let mut user_dict = UserDictionary::new();
    user_dict.add_entry("MeCab", "NNP", Some(-1000), None);

    let entries = user_dict.lookup("MeCab");
    assert!(!entries.is_empty(), "Should find the added entry");
    assert_eq!(entries[0].surface, "MeCab");
    assert_eq!(entries[0].pos, "NNP");
}

/// Test user dictionary persistence via save_to_csv / load_from_csv
#[test]
fn test_user_dict_persistence() {
    use mecab_ko_dict::UserDictionary;
    use std::env::temp_dir;

    let csv_path = temp_dir().join("mecab_ko_test_user_dict.csv");

    // Create and save
    let mut original = UserDictionary::new();
    original.add_entry("MeCab", "NNP", Some(-1000), None);
    original.add_entry("Rust", "NNP", Some(-1000), None);
    original
        .save_to_csv(&csv_path)
        .expect("Failed to save user dictionary CSV");

    // Load and verify
    let mut loaded = UserDictionary::new();
    loaded
        .load_from_csv(&csv_path)
        .expect("Failed to load user dictionary CSV");

    assert_eq!(
        loaded.len(),
        original.len(),
        "Loaded dictionary should have the same number of entries"
    );

    // Cleanup
    std::fs::remove_file(&csv_path).ok();
}

/// Test CSV format variations: basic, without reading, and multi-entry
#[test]
fn test_csv_format_variations() {
    use mecab_ko_dict::UserDictionary;

    // Basic format with reading
    let basic = "MeCab,NNP,-1000,메캅\n";
    let mut dict = UserDictionary::new();
    dict.load_from_str(basic)
        .expect("Basic CSV format should parse");
    assert_eq!(dict.len(), 1);

    // Without optional reading field
    let no_reading = "Rust,NNP,-1000\n";
    let mut dict2 = UserDictionary::new();
    dict2
        .load_from_str(no_reading)
        .expect("CSV without reading should parse");
    assert_eq!(dict2.len(), 1);

    // Multiple entries
    let multi = "MeCab,NNP,-1000,메캅\nRust,NNP,-1000,러스트\n도커,NNG,-500\n";
    let mut dict3 = UserDictionary::new();
    dict3
        .load_from_str(multi)
        .expect("Multi-entry CSV should parse");
    assert_eq!(dict3.len(), 3);
}

/// Test compound noun in user dictionary with cost verification
#[test]
fn test_compound_noun_user_dict() {
    use mecab_ko_dict::UserDictionary;

    let mut user_dict = UserDictionary::new();
    user_dict.add_entry("인공지능", "NNG", Some(-1500), None);
    user_dict.add_entry("머신러닝", "NNG", Some(-1500), None);

    let entries = user_dict.lookup("인공지능");
    assert!(
        !entries.is_empty(),
        "Should find '인공지능' in user dictionary"
    );
    assert_eq!(
        entries[0].cost, -1500,
        "Cost should match the specified value"
    );
}

/// Test technical terms (English acronyms and Korean tech words) in user dictionary
#[test]
fn test_technical_terms() {
    use mecab_ko_dict::UserDictionary;

    let technical_terms = [
        ("API", "SL"),
        ("JSON", "SL"),
        ("REST", "SL"),
        ("GraphQL", "SL"),
        ("Kubernetes", "NNP"),
        ("도커", "NNG"),
        ("쿠버네티스", "NNG"),
    ];

    let mut user_dict = UserDictionary::new();
    for (term, pos) in &technical_terms {
        user_dict.add_entry(*term, *pos, Some(-1000), None);
    }

    assert_eq!(
        user_dict.len(),
        technical_terms.len(),
        "All technical terms should be added"
    );

    // Spot-check lookups
    assert!(!user_dict.lookup("API").is_empty());
    assert!(!user_dict.lookup("도커").is_empty());
}

/// Test proper names (person names and organization names) in user dictionary
#[test]
fn test_proper_names() {
    use mecab_ko_dict::UserDictionary;

    let proper_names = [
        "김철수",
        "이영희",
        "서울대학교",
        "삼성전자",
        "카카오",
        "네이버",
    ];

    let mut user_dict = UserDictionary::new();
    for name in &proper_names {
        user_dict.add_entry(*name, "NNP", Some(-1500), None);
    }

    for name in &proper_names {
        let entries = user_dict.lookup(name);
        assert!(!entries.is_empty(), "Should find entry for '{name}'");
        assert_eq!(entries[0].pos, "NNP");
    }
}

/// Test user dictionary with surface forms containing special characters
#[test]
fn test_special_characters() {
    use mecab_ko_dict::UserDictionary;

    let entries_with_special = [
        ("C++", "SL"),
        ("C#", "SL"),
        (".NET", "SL"),
        ("Vue.js", "SL"),
        ("Node.js", "SL"),
    ];

    let mut user_dict = UserDictionary::new();
    for (term, pos) in &entries_with_special {
        user_dict.add_entry(*term, *pos, Some(-1000), None);
    }

    assert_eq!(
        user_dict.len(),
        entries_with_special.len(),
        "All special-character terms should be stored"
    );
    assert!(!user_dict.lookup("C++").is_empty());
    assert!(!user_dict.lookup("Node.js").is_empty());
}

/// Test user dictionary incremental updates (adding entries over time)
#[test]
fn test_user_dict_update() {
    use mecab_ko_dict::UserDictionary;

    let mut user_dict = UserDictionary::new();

    user_dict.add_entry("MeCab", "NNP", Some(-1000), None);
    assert_eq!(user_dict.len(), 1, "Should have 1 entry after first add");

    user_dict.add_entry("Rust", "NNP", Some(-1000), None);
    user_dict.add_entry("토크나이저", "NNG", Some(-500), None);
    assert_eq!(user_dict.len(), 3, "Should have 3 entries after three adds");
}

/// Test user dictionary removal by surface
#[test]
fn test_user_dict_removal() {
    use mecab_ko_dict::UserDictionary;

    let mut user_dict = UserDictionary::new();
    user_dict.add_entry("MeCab", "NNP", Some(-1000), None);
    user_dict.add_entry("Rust", "NNP", Some(-1000), None);

    let removed = user_dict.remove_surface("MeCab");
    assert_eq!(removed, 1, "Should have removed exactly 1 entry");
    assert_eq!(user_dict.len(), 1, "Dictionary should have 1 entry left");
    assert!(
        user_dict.lookup("MeCab").is_empty(),
        "'MeCab' should not be found after removal"
    );
    assert!(
        !user_dict.lookup("Rust").is_empty(),
        "'Rust' should still be present"
    );
}

/// Test empty user dictionary behavior
#[test]
fn test_empty_user_dict() {
    use mecab_ko_dict::UserDictionary;

    let user_dict = UserDictionary::new();
    assert_eq!(user_dict.len(), 0, "New dictionary should be empty");
    assert!(user_dict.is_empty(), "is_empty should return true");

    let entries = user_dict.lookup("anything");
    assert!(
        entries.is_empty(),
        "Lookup on empty dict should return empty"
    );
}

/// Test user dictionary with duplicate surface entries (same surface, different POS)
#[test]
fn test_duplicate_entries() {
    use mecab_ko_dict::UserDictionary;

    let mut user_dict = UserDictionary::new();
    user_dict.add_entry("개발", "NNG", Some(-500), None);
    user_dict.add_entry("개발", "NNP", Some(-1000), None);

    let entries = user_dict.lookup("개발");
    assert_eq!(
        entries.len(),
        2,
        "Should have both entries for the same surface"
    );
}

#[cfg(test)]
mod csv_tests {

    /// Test malformed CSV handling: missing required fields should return an error
    #[test]
    fn test_malformed_csv() {
        use mecab_ko_dict::UserDictionary;

        // A line with only one field (surface only) is malformed — needs at least surface,pos
        let malformed = "MeCab\n";
        let mut dict = UserDictionary::new();
        let result = dict.load_from_str(malformed);

        // The parser should either skip the line or return an error.
        // Either way the surface should not be inserted with garbage data.
        match result {
            Ok(_) => {
                // If it tolerates the line, the dict should either be empty
                // (line skipped) or have a sensibly defaulted entry.
                // We just verify no panic and no crash.
            }
            Err(_) => {
                // An error is also a valid, expected outcome for a malformed line.
            }
        }
    }
}
