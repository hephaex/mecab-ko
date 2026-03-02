//! Error types for dictionary synchronization and conversion.

/// Errors that can occur during dictionary synchronization and conversion.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error: {message}")]
    ApiError {
        /// Error message from the API
        message: String,
    },

    /// Failed to parse API response.
    #[error("Failed to parse response: {source}")]
    ParseError {
        /// Source of the parsing error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Invalid API key.
    #[error("Invalid API key")]
    InvalidApiKey,

    /// API rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// XML parsing error.
    #[error("XML parsing error: {0}")]
    XmlError(#[from] quick_xml::DeError),

    /// URL parsing error.
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),

    /// Unknown POS tag encountered during mapping.
    #[error("Unknown POS tag: {0}")]
    UnknownPosTag(String),

    /// Invalid entry data.
    #[error("Invalid entry: {0}")]
    InvalidEntry(String),

    /// CSV processing error.
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl SyncError {
    /// Creates a new parse error from any error type.
    pub fn parse_error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::ParseError {
            source: Box::new(error),
        }
    }

    /// Creates a new API error with a message.
    pub fn api_error(message: impl Into<String>) -> Self {
        Self::ApiError {
            message: message.into(),
        }
    }
}

/// Result type for dictionary synchronization operations.
pub type Result<T> = std::result::Result<T, SyncError>;
