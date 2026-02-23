//! Configuration file handling for validation rules.
//!
//! This module provides functionality to load and save validation configurations
//! from/to TOML files.

use crate::rules::ValidationConfig;
use std::fs;
use std::path::Path;

/// Error type for configuration operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// TOML parse error
    #[error("TOML parse error: {0}")]
    ParseError(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("TOML serialization error: {0}")]
    SerializeError(#[from] toml::ser::Error),
}

/// Loads validation configuration from a TOML file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ValidationConfig, ConfigError> {
    let contents = fs::read_to_string(path)?;
    let config: ValidationConfig = toml::from_str(&contents)?;
    Ok(config)
}

/// Saves validation configuration to a TOML file.
///
/// # Errors
///
/// Returns an error if the file cannot be written or the config cannot be serialized.
pub fn save_config<P: AsRef<Path>>(path: P, config: &ValidationConfig) -> Result<(), ConfigError> {
    let contents = toml::to_string_pretty(config)?;
    fs::write(path, contents)?;
    Ok(())
}

/// Generates a default configuration file template.
#[must_use]
pub fn generate_default_config() -> String {
    let config = ValidationConfig::default();
    toml::to_string_pretty(&config).unwrap_or_else(|_| String::from("# Failed to generate config"))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_save_and_load_config() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config = ValidationConfig::default();

        save_config(temp_file.path(), &config).expect("Failed to save config");

        let loaded_config = load_config(temp_file.path()).expect("Failed to load config");

        assert_eq!(
            loaded_config.csv_rules.expected_field_count,
            config.csv_rules.expected_field_count
        );
        assert_eq!(
            loaded_config.pos_rules.max_tag_depth,
            config.pos_rules.max_tag_depth
        );
    }

    #[test]
    fn test_generate_default_config() {
        let config_str = generate_default_config();
        assert!(config_str.contains("[csv_rules]"));
        assert!(config_str.contains("[pos_rules]"));
        assert!(config_str.contains("[cost_rules]"));
    }

    #[test]
    fn test_load_invalid_config() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(temp_file.path(), "invalid toml {{{").expect("Failed to write");

        let result = load_config(temp_file.path());
        assert!(result.is_err());
    }
}
