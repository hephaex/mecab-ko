//! Dictionary integration tests
//!
//! This module tests dictionary loading, searching, and management:
//! - System dictionary loading
//! - Entry lookup and retrieval
//! - Connection cost matrix operations
//! - Trie-based search performance
//! - Memory-mapped dictionary access

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::uninlined_format_args,
    clippy::useless_vec
)]

mod common;

use mecab_ko_dict::Entry;

/// Test that dictionary entry struct is properly defined
#[test]
fn test_dict_entry_creation() {
    let entry = Entry {
        surface: "안녕".to_string(),
        left_id: 100,
        right_id: 200,
        cost: 500,
        feature: "NNG,*,F,안녕,*,*,*,*".to_string(),
    };

    assert_eq!(entry.surface, "안녕");
    assert_eq!(entry.left_id, 100);
    assert_eq!(entry.right_id, 200);
    assert_eq!(entry.cost, 500);
    assert!(entry.feature.contains("NNG"));
}

/// Test dictionary entry cloning
#[test]
fn test_dict_entry_clone() {
    let entry1 = Entry {
        surface: "하다".to_string(),
        left_id: 1,
        right_id: 2,
        cost: 100,
        feature: "VV".to_string(),
    };

    let entry2 = entry1.clone();
    assert_eq!(entry1, entry2);
}

/// Test system dictionary loading with mini test dictionary
#[test]
fn test_load_system_dictionary() {
    use mecab_ko_dict::loader::MmapDictionary;

    let dict_path = common::mini_dict::mini_dict_path();

    if !common::mini_dict::mini_dict_exists() {
        println!(
            "Skipping test: mini dictionary not found at {:?}",
            dict_path
        );
        println!("Run: cd rust/test-fixtures && cargo run --release");
        return;
    }

    let dict = MmapDictionary::load(&dict_path).expect("Failed to load mini test dictionary");

    // Verify dictionary is loaded and has entries
    assert!(!dict.entries().is_empty(), "Dictionary should have entries");
    println!(
        "Loaded {} entries from mini dictionary",
        dict.entries().len()
    );
}

/// Test dictionary lookup functionality with mini test dictionary
#[test]
fn test_dictionary_lookup() {
    use mecab_ko_dict::{loader::MmapDictionary, Dictionary};

    let dict_path = common::mini_dict::mini_dict_path();

    if !common::mini_dict::mini_dict_exists() {
        println!(
            "Skipping test: mini dictionary not found at {:?}",
            dict_path
        );
        println!("Run: cd rust/test-fixtures && cargo run --release");
        return;
    }

    let dict = MmapDictionary::load(&dict_path).expect("Failed to load mini test dictionary");

    // Test lookup for common Korean words in the mini dictionary
    let test_words = vec!["안녕", "감사", "한국어", "사람"];

    for word in test_words {
        let entries = dict.lookup(word);
        assert!(!entries.is_empty(), "Should find entries for '{}'", word);

        for entry in &entries {
            assert_eq!(entry.surface, word);
            assert!(!entry.feature.is_empty(), "Feature should not be empty");
            println!("Found: {} -> {}", word, entry.feature);
        }
    }
}

/// Test connection cost matrix loading from definition reader
#[test]
fn test_matrix_loading() {
    use mecab_ko_dict::matrix::{DenseMatrix, Matrix};
    use std::io::BufReader;

    // Minimal matrix.def format: "<lsize> <rsize>\n<right_id> <left_id> <cost>\n..."
    let def_content = "3 3\n0 0 100\n0 1 200\n1 0 300\n1 2 -50\n2 2 0\n";
    let reader = BufReader::new(def_content.as_bytes());
    let matrix = DenseMatrix::from_def_reader(reader).expect("Failed to parse matrix.def");

    assert!(matrix.left_size() > 0, "Matrix should have left entries");
    assert!(matrix.right_size() > 0, "Matrix should have right entries");
}

/// Test connection cost retrieval from a DenseMatrix
#[test]
fn test_connection_cost() {
    use mecab_ko_dict::matrix::{DenseMatrix, Matrix};

    let mut matrix = DenseMatrix::new(50, 50, 0);
    matrix.set(10, 20, 500);

    // Matrix::get takes (right_id, left_id)
    let cost = matrix.get(10, 20);
    assert_eq!(cost, 500, "Retrieved cost should match the stored value");
}

/// Test dense matrix creation, set, and get
#[test]
fn test_dense_matrix() {
    use mecab_ko_dict::matrix::{DenseMatrix, Matrix};

    let mut matrix = DenseMatrix::new(100, 100, 0);

    matrix.set(10, 20, 500);
    assert_eq!(matrix.get(10, 20), 500);

    matrix.set(0, 0, -100);
    assert_eq!(matrix.get(0, 0), -100);

    assert_eq!(matrix.left_size(), 100, "Left size should be 100");
    assert_eq!(matrix.right_size(), 100, "Right size should be 100");
}

/// Test sparse matrix construction from a dense matrix
#[test]
fn test_sparse_matrix() {
    use mecab_ko_dict::matrix::{DenseMatrix, Matrix, SparseMatrix};

    let mut dense = DenseMatrix::new(10, 10, 0);
    dense.set(2, 3, 500);
    dense.set(4, 5, 600);

    let sparse = SparseMatrix::from_dense(&dense, 0);

    // Stored entries should be accessible via the dense roundtrip
    let roundtripped = sparse.to_dense();
    assert_eq!(roundtripped.get(2, 3), 500);
    assert_eq!(roundtripped.get(4, 5), 600);
    assert_eq!(roundtripped.get(0, 0), 0, "Default value should be 0");
}

/// Test trie building and exact/prefix searching
#[test]
fn test_trie_build_and_search() {
    use mecab_ko_dict::trie::{Trie, TrieBuilder};

    let entries = [("안녕", 0u32), ("안녕하세요", 1u32), ("감사", 2u32)];
    let bytes = TrieBuilder::build(&entries).expect("Failed to build trie");
    let trie = Trie::from_vec(bytes);

    // Exact matches
    assert_eq!(trie.exact_match("안녕"), Some(0));
    assert_eq!(trie.exact_match("감사"), Some(2));
    assert_eq!(trie.exact_match("없는단어"), None);

    // Prefix search on "안녕하세요" should find "안녕" and "안녕하세요"
    let matches: Vec<_> = trie.common_prefix_search("안녕하세요").collect();
    assert!(matches.len() >= 2, "Should find at least 2 prefix matches");
}

/// Test feature string parsing
#[test]
fn test_feature_parsing() {
    let feature = "NNG,*,F,안녕,Compound,*,*,안녕/NNG/*";

    // Expected format: POS,POS1,POS2,Reading,Type,Start,End,Expression
    let parts: Vec<&str> = feature.split(',').collect();
    assert_eq!(parts[0], "NNG"); // POS
    assert_eq!(parts[3], "안녕"); // Reading

    // Test various feature formats
    let features = vec![
        "NNG,*,F,안녕,*,*,*,*",
        "VV,*,F,하다,*,*,*,*",
        "NNP,*,T,서울,Compound,*,*,서울/NNP/*",
    ];

    for f in features {
        let parts: Vec<&str> = f.split(',').collect();
        assert!(parts.len() >= 4, "Feature should have at least 4 fields");
        assert!(!parts[0].is_empty(), "POS tag should not be empty: '{f}'");
    }
}

#[cfg(test)]
mod matrix_tests {
    use mecab_ko_dict::matrix::{DenseMatrix, Matrix, SparseMatrix};

    /// Test matrix bounds: valid access at corners and interior
    #[test]
    fn test_matrix_bounds() {
        let mut matrix = DenseMatrix::new(10, 10, 0);

        // Corners
        matrix.set(0, 0, 1);
        matrix.set(9, 9, 2);
        assert_eq!(matrix.get(0, 0), 1);
        assert_eq!(matrix.get(9, 9), 2);

        // Interior
        matrix.set(5, 7, 999);
        assert_eq!(matrix.get(5, 7), 999);
    }

    /// Test matrix memory usage reporting
    #[test]
    fn test_matrix_memory_usage() {
        let dense = DenseMatrix::new(100, 100, 0);
        let sparse = SparseMatrix::new(100, 100, 0);

        // Dense matrix memory is predictable: at least lsize * rsize * sizeof(i16) bytes
        let dense_mem = dense.memory_size();
        assert!(
            dense_mem >= 100 * 100 * 2,
            "Dense matrix should use at least lsize*rsize*2 bytes"
        );

        // Sparse with no entries should use less than dense
        let sparse_mem = sparse.memory_size();
        assert!(
            sparse_mem < dense_mem,
            "Empty sparse matrix should be smaller than dense"
        );
    }
}

#[cfg(test)]
mod trie_tests {
    use mecab_ko_dict::trie::{Trie, TrieBuilder};

    /// Test trie exact match with Korean single-syllable entries
    #[test]
    fn test_trie_korean() {
        let entries = [("가", 0u32), ("각", 1u32), ("간", 2u32), ("갈", 3u32)];
        let bytes = TrieBuilder::build(&entries).expect("Failed to build trie");
        let trie = Trie::from_vec(bytes);

        assert_eq!(trie.exact_match("가"), Some(0));
        assert_eq!(trie.exact_match("갈"), Some(3));
        assert_eq!(
            trie.exact_match("감"),
            None,
            "Non-existent key should return None"
        );
    }

    /// Test trie common prefix search over a longer Korean string
    #[test]
    fn test_trie_common_prefix() {
        let entries = [("안녕", 0u32), ("안녕하", 1u32), ("안녕하세요", 2u32)];
        let bytes = TrieBuilder::build(&entries).expect("Failed to build trie");
        let trie = Trie::from_vec(bytes);

        let text = "안녕하세요반갑습니다";
        let prefixes: Vec<_> = trie.common_prefix_search(text).collect();

        assert!(!prefixes.is_empty(), "Should find prefix matches");
        assert!(
            prefixes.len() >= 2,
            "Should find at least '안녕' and '안녕하'"
        );
    }
}
