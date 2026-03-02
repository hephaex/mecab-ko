//! HTTP client for the `OpenDict` API.

use crate::config::OpenDictConfig;
use crate::error::{Result, SyncError};
use crate::models::{DictDetail, DictEntry, SearchResponse};
use std::time::Duration;

/// HTTP client for the `OpenDict` (우리말샘) API.
///
/// This client provides access to the Korean National Institute's
/// `OpenDict` API for searching and retrieving dictionary entries.
///
/// # Examples
///
/// ```no_run
/// use mecab_ko_dict_sync::client::OpenDictClient;
/// use mecab_ko_dict_sync::config::OpenDictConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = OpenDictConfig::new("your-api-key");
/// let client = OpenDictClient::new(config)?;
///
/// let results = client.search("사랑").await?;
/// println!("Found {} entries", results.len());
/// # Ok(())
/// # }
/// ```
pub struct OpenDictClient {
    config: OpenDictConfig,
    client: reqwest::Client,
}

impl OpenDictClient {
    /// Creates a new `OpenDict` API client.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the API client
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration validation fails
    /// - HTTP client creation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::client::OpenDictClient;
    /// use mecab_ko_dict_sync::config::OpenDictConfig;
    ///
    /// let config = OpenDictConfig::new("api-key");
    /// let client = OpenDictClient::new(config);
    /// assert!(client.is_ok());
    /// ```
    pub fn new(config: OpenDictConfig) -> Result<Self> {
        config.validate()?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(SyncError::from)?;

        Ok(Self { config, client })
    }

    /// Searches for dictionary entries matching the query.
    ///
    /// # Arguments
    ///
    /// * `query` - Search term
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - HTTP request fails
    /// - API returns an error
    /// - Response parsing fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mecab_ko_dict_sync::client::OpenDictClient;
    /// # use mecab_ko_dict_sync::config::OpenDictConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = OpenDictConfig::new("key");
    /// # let client = OpenDictClient::new(config)?;
    /// let entries = client.search("컴퓨터").await?;
    /// for entry in entries {
    ///     println!("{}: {}", entry.word, entry.definition);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, query: &str) -> Result<Vec<DictEntry>> {
        let url = format!("{}/search", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("key", self.config.api_key.as_str()),
                ("q", query),
                ("req_type", "json"),
                ("num", &self.config.max_results.to_string()),
            ])
            .send()
            .await?;

        // Check for HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 | 403 => Err(SyncError::InvalidApiKey),
                429 => Err(SyncError::RateLimitExceeded),
                _ => Err(SyncError::api_error(format!(
                    "HTTP {status}: {body}"
                ))),
            };
        }

        let text = response.text().await?;

        // Parse JSON response
        let search_response: SearchResponse = serde_json::from_str(&text)
            .map_err(SyncError::parse_error)?;

        Ok(search_response
            .channel
            .items
            .into_iter()
            .map(DictEntry::from)
            .collect())
    }

    /// Retrieves detailed information for a specific dictionary entry.
    ///
    /// # Arguments
    ///
    /// * `target_code` - Unique identifier for the entry
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - HTTP request fails
    /// - API returns an error
    /// - Response parsing fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mecab_ko_dict_sync::client::OpenDictClient;
    /// # use mecab_ko_dict_sync::config::OpenDictConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = OpenDictConfig::new("key");
    /// # let client = OpenDictClient::new(config)?;
    /// let detail = client.get_detail("12345").await?;
    /// println!("Word: {}", detail.word);
    /// println!("Examples: {:?}", detail.examples);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_detail(&self, target_code: &str) -> Result<DictDetail> {
        let url = format!("{}/view", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("key", self.config.api_key.as_str()),
                ("target_code", target_code),
                ("req_type", "json"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 | 403 => Err(SyncError::InvalidApiKey),
                429 => Err(SyncError::RateLimitExceeded),
                404 => Err(SyncError::api_error(format!(
                    "Entry not found: {target_code}"
                ))),
                _ => Err(SyncError::api_error(format!(
                    "HTTP {status}: {body}"
                ))),
            };
        }

        let text = response.text().await?;

        // For now, parse as basic detail (simplified version)
        // Full parsing would require detailed XML/JSON schema
        let detail: DictDetail = serde_json::from_str(&text)
            .map_err(SyncError::parse_error)?;

        Ok(detail)
    }

    /// Searches with pagination support.
    ///
    /// # Arguments
    ///
    /// * `query` - Search term
    /// * `start` - Starting position (1-based)
    /// * `num` - Number of results to fetch
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mecab_ko_dict_sync::client::OpenDictClient;
    /// # use mecab_ko_dict_sync::config::OpenDictConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = OpenDictConfig::new("key");
    /// # let client = OpenDictClient::new(config)?;
    /// // Get results 11-20
    /// let entries = client.search_paginated("사랑", 11, 10).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_paginated(
        &self,
        query: &str,
        start: u32,
        num: u32,
    ) -> Result<Vec<DictEntry>> {
        let url = format!("{}/search", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("key", self.config.api_key.as_str()),
                ("q", query),
                ("req_type", "json"),
                ("start", &start.to_string()),
                ("num", &num.to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            return match status.as_u16() {
                401 | 403 => Err(SyncError::InvalidApiKey),
                429 => Err(SyncError::RateLimitExceeded),
                _ => Err(SyncError::api_error(format!("HTTP {status}"))),
            };
        }

        let text = response.text().await?;
        let search_response: SearchResponse = serde_json::from_str(&text)
            .map_err(SyncError::parse_error)?;

        Ok(search_response
            .channel
            .items
            .into_iter()
            .map(DictEntry::from)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client_valid_config() {
        let config = OpenDictConfig::new("valid-key");
        let result = OpenDictClient::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_client_invalid_config() {
        let config = OpenDictConfig::new(""); // Empty API key
        let result = OpenDictClient::new(config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_invalid_api_key() {
        let config = OpenDictConfig::new("invalid-key");
        let client = OpenDictClient::new(config).expect("Failed to create client");

        // This will fail with a real API request
        // In production, you'd use a mock server for testing
        let result = client.search("테스트").await;

        // We expect either InvalidApiKey or Http error
        assert!(result.is_err());
    }
}
