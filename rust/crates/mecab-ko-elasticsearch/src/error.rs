//! 에러 타입 정의

use thiserror::Error;

/// Elasticsearch 플러그인 에러
#[derive(Error, Debug)]
pub enum Error {
    /// `MeCab` 코어 엔진 에러
    #[error("MeCab core error: {0}")]
    MecabCore(#[from] mecab_ko_core::Error),

    /// 설정 에러
    #[error("Configuration error: {0}")]
    Config(String),

    /// 토크나이저 에러
    #[error("Tokenizer error: {0}")]
    Tokenizer(String),

    /// 필터 에러
    #[error("Filter error: {0}")]
    Filter(String),

    /// 사용자 사전 로드 에러
    #[error("Failed to load user dictionary from {path}: {source}")]
    UserDictionary {
        /// 사전 경로
        path: String,
        /// 원인
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// JNI 에러
    #[cfg(feature = "jni-bindings")]
    #[error("JNI error: {0}")]
    Jni(String),

    /// 직렬화/역직렬화 에러
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O 에러
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// 기타 에러
    #[error("{0}")]
    Other(String),
}

/// Result 타입 별칭
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// 설정 에러 생성
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// 토크나이저 에러 생성
    pub fn tokenizer(msg: impl Into<String>) -> Self {
        Self::Tokenizer(msg.into())
    }

    /// 필터 에러 생성
    pub fn filter(msg: impl Into<String>) -> Self {
        Self::Filter(msg.into())
    }

    /// 사용자 사전 에러 생성
    pub fn user_dictionary(
        path: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::UserDictionary {
            path: path.into(),
            source: Box::new(source),
        }
    }

    /// JNI 에러 생성
    #[cfg(feature = "jni-bindings")]
    pub fn jni(msg: impl Into<String>) -> Self {
        Self::Jni(msg.into())
    }
}
