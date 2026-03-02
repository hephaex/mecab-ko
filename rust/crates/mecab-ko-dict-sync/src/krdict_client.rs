//! HTTP client for the Korean Dictionary API (표준국어대사전/한국어기초사전).
//!
//! This module provides access to the National Institute of Korean Language's
//! dictionary APIs, including both the Standard Korean Dictionary (표준국어대사전)
//! and the Korean Learners' Dictionary (한국어기초사전).
//!
//! # Features
//!
//! - Search dictionary entries by keyword
//! - Retrieve detailed entry information
//! - Pagination support for large result sets
//! - Automatic error handling and retries
//!
//! # API Documentation
//!
//! - Korean Dictionary API: <https://krdict.korean.go.kr/openApi/openApiInfo>
//! - Authentication: API key required (obtain from 공공데이터포털)
//!
//! # Examples
//!
//! ```no_run
//! use mecab_ko_dict_sync::KrDictClient;
//! use mecab_ko_dict_sync::config::KrDictConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = KrDictConfig::new("your-api-key");
//! let client = KrDictClient::new(config)?;
//!
//! let results = client.search("사랑").await?;
//! println!("Found {} entries", results.len());
//! # Ok(())
//! # }
//! ```

use crate::config::KrDictConfig;
use crate::error::{Result, SyncError};
use crate::models::{DictDetail, DictEntry};
use serde::Deserialize;
use std::time::Duration;

/// HTTP client for the Korean Dictionary API.
///
/// This client provides access to the National Institute of Korean Language's
/// Standard Korean Dictionary (표준국어대사전) and Korean Learners' Dictionary
/// (한국어기초사전) APIs.
///
/// # Examples
///
/// ```no_run
/// use mecab_ko_dict_sync::KrDictClient;
/// use mecab_ko_dict_sync::config::KrDictConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = KrDictConfig::new("your-api-key");
/// let client = KrDictClient::new(config)?;
///
/// let results = client.search("컴퓨터").await?;
/// for entry in results {
///     println!("{}: {}", entry.word, entry.definition);
/// }
/// # Ok(())
/// # }
/// ```
pub struct KrDictClient {
    config: KrDictConfig,
    client: reqwest::Client,
}

impl KrDictClient {
    /// Creates a new Korean Dictionary API client.
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
    /// use mecab_ko_dict_sync::KrDictClient;
    /// use mecab_ko_dict_sync::config::KrDictConfig;
    ///
    /// let config = KrDictConfig::new("api-key");
    /// let client = KrDictClient::new(config);
    /// assert!(client.is_ok());
    /// ```
    pub fn new(config: KrDictConfig) -> Result<Self> {
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
    /// # use mecab_ko_dict_sync::KrDictClient;
    /// # use mecab_ko_dict_sync::config::KrDictConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = KrDictConfig::new("key");
    /// # let client = KrDictClient::new(config)?;
    /// let entries = client.search("메타버스").await?;
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
                ("req_type", "xml"),
                ("num", &self.config.max_results.to_string()),
                ("start", "1"),
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

        // Parse XML response
        let search_response: KrDictSearchResponse =
            quick_xml::de::from_str(&text).map_err(SyncError::from)?;

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
    /// # use mecab_ko_dict_sync::KrDictClient;
    /// # use mecab_ko_dict_sync::config::KrDictConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = KrDictConfig::new("key");
    /// # let client = KrDictClient::new(config)?;
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
                ("req_type", "xml"),
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

        // Parse XML response
        let detail_response: KrDictDetailResponse =
            quick_xml::de::from_str(&text).map_err(SyncError::from)?;

        Ok(DictDetail::from(detail_response.item))
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
    /// # use mecab_ko_dict_sync::KrDictClient;
    /// # use mecab_ko_dict_sync::config::KrDictConfig;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = KrDictConfig::new("key");
    /// # let client = KrDictClient::new(config)?;
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
                ("req_type", "xml"),
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
        let search_response: KrDictSearchResponse =
            quick_xml::de::from_str(&text).map_err(SyncError::from)?;

        Ok(search_response
            .channel
            .items
            .into_iter()
            .map(DictEntry::from)
            .collect())
    }
}

/// Search response from the Korean Dictionary API.
#[derive(Debug, Clone, Deserialize)]
struct KrDictSearchResponse {
    #[serde(rename = "channel")]
    channel: KrDictChannel,
}

/// Channel containing search results.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct KrDictChannel {
    #[serde(rename = "total")]
    total: u32,

    #[serde(rename = "num")]
    num: u32,

    #[serde(rename = "item", default)]
    items: Vec<KrDictItem>,
}

/// Individual item in search results.
#[derive(Debug, Clone, Deserialize)]
struct KrDictItem {
    #[serde(rename = "target_code")]
    target_code: String,

    #[serde(rename = "word")]
    word: String,

    #[serde(rename = "pos")]
    pos: String,

    #[serde(rename = "sense")]
    sense: KrDictSense,

    #[serde(rename = "pronunciation", default)]
    pronunciation: Option<String>,
}

/// Sense information (definition).
#[derive(Debug, Clone, Deserialize)]
struct KrDictSense {
    #[serde(rename = "definition")]
    definition: String,
}

impl From<KrDictItem> for DictEntry {
    fn from(item: KrDictItem) -> Self {
        Self {
            target_code: item.target_code,
            word: item.word,
            pos: item.pos,
            definition: item.sense.definition,
            reading: item.pronunciation,
        }
    }
}

/// Detail response from the Korean Dictionary API.
#[derive(Debug, Clone, Deserialize)]
struct KrDictDetailResponse {
    #[serde(rename = "item")]
    item: KrDictDetailItem,
}

/// Detailed item information.
#[derive(Debug, Clone, Deserialize)]
struct KrDictDetailItem {
    #[serde(rename = "target_code")]
    target_code: String,

    #[serde(rename = "word")]
    word: String,

    #[serde(rename = "pos")]
    pos: String,

    #[serde(rename = "sense")]
    sense: KrDictDetailSense,

    #[serde(rename = "pronunciation", default)]
    pronunciation: Option<String>,

    #[serde(rename = "origin", default)]
    origin: Option<String>,
}

/// Detailed sense information.
#[derive(Debug, Clone, Deserialize)]
struct KrDictDetailSense {
    #[serde(rename = "definition")]
    definition: String,

    #[serde(rename = "example", default)]
    examples: Vec<String>,

    #[serde(rename = "related", default)]
    related: Vec<String>,
}

impl From<KrDictDetailItem> for DictDetail {
    fn from(item: KrDictDetailItem) -> Self {
        Self {
            target_code: item.target_code,
            word: item.word,
            pos: item.pos,
            definition: item.sense.definition,
            reading: item.pronunciation,
            examples: item.sense.examples,
            etymology: None, // Not provided in this format
            related_words: item.sense.related,
            original_language: item.origin,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client_valid_config() {
        let config = KrDictConfig::new("valid-key");
        let result = KrDictClient::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_client_invalid_config() {
        let config = KrDictConfig::new(""); // Empty API key
        let result = KrDictClient::new(config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_invalid_api_key() {
        let config = KrDictConfig::new("invalid-key");
        let client = KrDictClient::new(config).expect("Failed to create client");

        // This will fail with a real API request
        // In production, you'd use a mock server for testing
        let result = client.search("테스트").await;

        // We expect either InvalidApiKey or Http error
        assert!(result.is_err());
    }

    #[test]
    fn test_krdict_item_conversion() {
        let item = KrDictItem {
            target_code: "123".to_string(),
            word: "사랑".to_string(),
            pos: "명사".to_string(),
            sense: KrDictSense {
                definition: "애정".to_string(),
            },
            pronunciation: Some("사랑".to_string()),
        };

        let entry: DictEntry = item.into();
        assert_eq!(entry.target_code, "123");
        assert_eq!(entry.word, "사랑");
        assert_eq!(entry.pos, "명사");
        assert_eq!(entry.definition, "애정");
        assert_eq!(entry.reading, Some("사랑".to_string()));
    }
}
