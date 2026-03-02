//! Configuration for dictionary API clients.

use crate::error::{Result, SyncError};

/// Configuration for the `OpenDict` API client.
#[derive(Debug, Clone)]
pub struct OpenDictConfig {
    /// API authentication key from 공공데이터포털
    pub api_key: String,

    /// Base URL for the API
    pub base_url: String,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Maximum number of results per request
    pub max_results: u32,
}

impl OpenDictConfig {
    /// Default base URL for the `OpenDict` API.
    pub const DEFAULT_BASE_URL: &'static str = "https://opendict.korean.go.kr/api";

    /// Default timeout in seconds.
    pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

    /// Default maximum results per request.
    pub const DEFAULT_MAX_RESULTS: u32 = 100;

    /// Creates a new configuration with the given API key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - API authentication key from 공공데이터포털
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::config::OpenDictConfig;
    ///
    /// let config = OpenDictConfig::new("your-api-key");
    /// assert_eq!(config.api_key, "your-api-key");
    /// ```
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: Self::DEFAULT_BASE_URL.to_string(),
            timeout_secs: Self::DEFAULT_TIMEOUT_SECS,
            max_results: Self::DEFAULT_MAX_RESULTS,
        }
    }

    /// Sets the base URL for the API.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::config::OpenDictConfig;
    ///
    /// let config = OpenDictConfig::new("key")
    ///     .with_base_url("https://custom.api.url");
    /// assert_eq!(config.base_url, "https://custom.api.url");
    /// ```
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets the request timeout in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::config::OpenDictConfig;
    ///
    /// let config = OpenDictConfig::new("key")
    ///     .with_timeout_secs(60);
    /// assert_eq!(config.timeout_secs, 60);
    /// ```
    #[must_use]
    pub const fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Sets the maximum number of results per request.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::config::OpenDictConfig;
    ///
    /// let config = OpenDictConfig::new("key")
    ///     .with_max_results(50);
    /// assert_eq!(config.max_results, 50);
    /// ```
    #[must_use]
    pub const fn with_max_results(mut self, max: u32) -> Self {
        self.max_results = max;
        self
    }

    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - API key is empty
    /// - Base URL is invalid
    /// - Timeout is zero
    /// - Max results is zero
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(SyncError::InvalidConfig(
                "API key cannot be empty".to_string(),
            ));
        }

        if self.base_url.is_empty() {
            return Err(SyncError::InvalidConfig(
                "Base URL cannot be empty".to_string(),
            ));
        }

        // Validate URL format
        url::Url::parse(&self.base_url)
            .map_err(|e| SyncError::InvalidConfig(format!("Invalid base URL: {e}")))?;

        if self.timeout_secs == 0 {
            return Err(SyncError::InvalidConfig(
                "Timeout must be greater than zero".to_string(),
            ));
        }

        if self.max_results == 0 {
            return Err(SyncError::InvalidConfig(
                "Max results must be greater than zero".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for OpenDictConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: Self::DEFAULT_BASE_URL.to_string(),
            timeout_secs: Self::DEFAULT_TIMEOUT_SECS,
            max_results: Self::DEFAULT_MAX_RESULTS,
        }
    }
}

/// Configuration for the Korean Dictionary API (표준국어대사전/한국어기초사전).
#[derive(Debug, Clone)]
pub struct KrDictConfig {
    /// API authentication key from 공공데이터포털
    pub api_key: String,

    /// Base URL for the API
    pub base_url: String,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Maximum number of results per request
    pub max_results: u32,
}

impl KrDictConfig {
    /// Default base URL for the Korean Dictionary API.
    pub const DEFAULT_BASE_URL: &'static str = "https://krdict.korean.go.kr/api";

    /// Default timeout in seconds.
    pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

    /// Default maximum results per request.
    pub const DEFAULT_MAX_RESULTS: u32 = 100;

    /// Creates a new configuration with the given API key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - API authentication key from 공공데이터포털
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::config::KrDictConfig;
    ///
    /// let config = KrDictConfig::new("your-api-key");
    /// assert_eq!(config.api_key, "your-api-key");
    /// ```
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: Self::DEFAULT_BASE_URL.to_string(),
            timeout_secs: Self::DEFAULT_TIMEOUT_SECS,
            max_results: Self::DEFAULT_MAX_RESULTS,
        }
    }

    /// Sets the base URL for the API.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::config::KrDictConfig;
    ///
    /// let config = KrDictConfig::new("key")
    ///     .with_base_url("https://custom.api.url");
    /// assert_eq!(config.base_url, "https://custom.api.url");
    /// ```
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets the request timeout in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::config::KrDictConfig;
    ///
    /// let config = KrDictConfig::new("key")
    ///     .with_timeout_secs(60);
    /// assert_eq!(config.timeout_secs, 60);
    /// ```
    #[must_use]
    pub const fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Sets the maximum number of results per request.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::config::KrDictConfig;
    ///
    /// let config = KrDictConfig::new("key")
    ///     .with_max_results(50);
    /// assert_eq!(config.max_results, 50);
    /// ```
    #[must_use]
    pub const fn with_max_results(mut self, max: u32) -> Self {
        self.max_results = max;
        self
    }

    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - API key is empty
    /// - Base URL is invalid
    /// - Timeout is zero
    /// - Max results is zero
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(SyncError::InvalidConfig(
                "API key cannot be empty".to_string(),
            ));
        }

        if self.base_url.is_empty() {
            return Err(SyncError::InvalidConfig(
                "Base URL cannot be empty".to_string(),
            ));
        }

        // Validate URL format
        url::Url::parse(&self.base_url)
            .map_err(|e| SyncError::InvalidConfig(format!("Invalid base URL: {e}")))?;

        if self.timeout_secs == 0 {
            return Err(SyncError::InvalidConfig(
                "Timeout must be greater than zero".to_string(),
            ));
        }

        if self.max_results == 0 {
            return Err(SyncError::InvalidConfig(
                "Max results must be greater than zero".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for KrDictConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: Self::DEFAULT_BASE_URL.to_string(),
            timeout_secs: Self::DEFAULT_TIMEOUT_SECS,
            max_results: Self::DEFAULT_MAX_RESULTS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config() {
        let config = OpenDictConfig::new("test-key");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.base_url, OpenDictConfig::DEFAULT_BASE_URL);
        assert_eq!(config.timeout_secs, OpenDictConfig::DEFAULT_TIMEOUT_SECS);
        assert_eq!(config.max_results, OpenDictConfig::DEFAULT_MAX_RESULTS);
    }

    #[test]
    fn test_builder_pattern() {
        let config = OpenDictConfig::new("key")
            .with_base_url("https://test.com")
            .with_timeout_secs(60)
            .with_max_results(50);

        assert_eq!(config.api_key, "key");
        assert_eq!(config.base_url, "https://test.com");
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.max_results, 50);
    }

    #[test]
    fn test_validate_empty_api_key() {
        let config = OpenDictConfig::new("");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_url() {
        let config = OpenDictConfig::new("key").with_base_url("not-a-url");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_zero_timeout() {
        let config = OpenDictConfig::new("key").with_timeout_secs(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_zero_max_results() {
        let config = OpenDictConfig::new("key").with_max_results(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_success() {
        let config = OpenDictConfig::new("valid-key");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_krdict_new_config() {
        let config = KrDictConfig::new("test-key");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.base_url, KrDictConfig::DEFAULT_BASE_URL);
        assert_eq!(config.timeout_secs, KrDictConfig::DEFAULT_TIMEOUT_SECS);
        assert_eq!(config.max_results, KrDictConfig::DEFAULT_MAX_RESULTS);
    }

    #[test]
    fn test_krdict_builder_pattern() {
        let config = KrDictConfig::new("key")
            .with_base_url("https://test.com")
            .with_timeout_secs(60)
            .with_max_results(50);

        assert_eq!(config.api_key, "key");
        assert_eq!(config.base_url, "https://test.com");
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.max_results, 50);
    }

    #[test]
    fn test_krdict_validate_empty_api_key() {
        let config = KrDictConfig::new("");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_krdict_validate_invalid_url() {
        let config = KrDictConfig::new("key").with_base_url("not-a-url");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_krdict_validate_zero_timeout() {
        let config = KrDictConfig::new("key").with_timeout_secs(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_krdict_validate_zero_max_results() {
        let config = KrDictConfig::new("key").with_max_results(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_krdict_validate_success() {
        let config = KrDictConfig::new("valid-key");
        assert!(config.validate().is_ok());
    }
}
