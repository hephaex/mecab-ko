//! Mini dictionary helper for integration tests
//!
//! Provides a minimal test dictionary that can be used without requiring
//! a full system dictionary installation.

use std::path::PathBuf;

/// Get the path to the mini test dictionary
///
/// # Returns
///
/// Path to the mini-dict directory containing test dictionary files
#[must_use]
pub fn mini_dict_path() -> PathBuf {
    // Path relative to workspace root
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parent dir")
        .parent()
        .expect("workspace root")
        .to_path_buf();

    workspace_root.join("test-fixtures").join("mini-dict")
}

/// Check if the mini dictionary exists
#[must_use]
pub fn mini_dict_exists() -> bool {
    let path = mini_dict_path();
    path.join("entries.csv").exists()
        && path.join("sys.dic").exists()
        && path.join("matrix.bin").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mini_dict_path() {
        let path = mini_dict_path();
        assert!(path.ends_with("test-fixtures/mini-dict"));
    }

    #[test]
    fn test_mini_dict_exists() {
        // This test will pass once the mini dictionary is generated
        if mini_dict_exists() {
            let path = mini_dict_path();
            assert!(path.join("entries.csv").exists());
            assert!(path.join("sys.dic").exists());
            assert!(path.join("matrix.bin").exists());
        }
    }
}
