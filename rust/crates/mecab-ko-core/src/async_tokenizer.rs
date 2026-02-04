//! # Async Tokenizer Module
//!
//! 비동기 형태소 분석 API (tokio 기반)
//!
//! ## 주요 기능
//!
//! - 비동기 파일 처리
//! - 비동기 스트림 처리
//! - 동시성 제어 (병렬 처리)
//!
//! ## Example
//!
//! ```rust,ignore
//! use mecab_ko_core::async_tokenizer::AsyncTokenizer;
//!
//! #[tokio::main]
//! async fn main() {
//!     let tokenizer = AsyncTokenizer::new().await?;
//!     let tokens = tokenizer.tokenize_async("안녕하세요").await;
//!
//!     for token in tokens {
//!         println!("{}: {}", token.surface, token.pos);
//!     }
//! }
//! ```

use std::path::Path;
use std::sync::Arc;

use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::{Mutex, Semaphore};

use crate::tokenizer::{Token, Tokenizer};
use crate::Result;

/// 비동기 토크나이저
///
/// tokio 런타임을 사용하여 비동기 형태소 분석을 수행합니다.
/// 내부적으로 동기 Tokenizer를 Mutex로 보호하여 사용합니다.
pub struct AsyncTokenizer {
    /// 동기 토크나이저 (Arc<Mutex>로 공유)
    tokenizer: Arc<Mutex<Tokenizer>>,

    /// 동시 실행 제한 (semaphore)
    semaphore: Arc<Semaphore>,

    /// 최대 동시 실행 수
    max_concurrent: usize,
}

impl AsyncTokenizer {
    /// 기본 동시 실행 수
    pub const DEFAULT_MAX_CONCURRENT: usize = 4;

    /// 새 비동기 토크나이저 생성
    ///
    /// # Errors
    ///
    /// 토크나이저 초기화 실패 시
    pub async fn new() -> Result<Self> {
        let tokenizer = tokio::task::spawn_blocking(Tokenizer::new)
            .await
            .map_err(|e| crate::Error::Init(format!("Failed to spawn task: {e}")))?
            .map_err(|e| crate::Error::Init(format!("Failed to create tokenizer: {e}")))?;

        Ok(Self {
            tokenizer: Arc::new(Mutex::new(tokenizer)),
            semaphore: Arc::new(Semaphore::new(Self::DEFAULT_MAX_CONCURRENT)),
            max_concurrent: Self::DEFAULT_MAX_CONCURRENT,
        })
    }

    /// 사전 경로 지정하여 생성
    ///
    /// # Arguments
    ///
    /// * `dict_path` - 사전 디렉토리 경로
    ///
    /// # Errors
    ///
    /// 토크나이저 초기화 실패 시
    pub async fn with_dict<P: AsRef<Path> + Send + 'static>(dict_path: P) -> Result<Self> {
        let tokenizer = tokio::task::spawn_blocking(move || Tokenizer::with_dict(dict_path))
            .await
            .map_err(|e| crate::Error::Init(format!("Failed to spawn task: {e}")))?
            .map_err(|e| crate::Error::Init(format!("Failed to create tokenizer: {e}")))?;

        Ok(Self {
            tokenizer: Arc::new(Mutex::new(tokenizer)),
            semaphore: Arc::new(Semaphore::new(Self::DEFAULT_MAX_CONCURRENT)),
            max_concurrent: Self::DEFAULT_MAX_CONCURRENT,
        })
    }

    /// 최대 동시 실행 수 설정
    ///
    /// # Arguments
    ///
    /// * `max` - 최대 동시 실행 수
    #[must_use]
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self.semaphore = Arc::new(Semaphore::new(max));
        self
    }

    /// 비동기 형태소 분석
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트
    ///
    /// # Returns
    ///
    /// 토큰 목록
    pub async fn tokenize_async(&self, text: &str) -> Vec<Token> {
        // Semaphore로 동시 실행 제어
        let _permit = match self.semaphore.acquire().await {
            Ok(permit) => permit,
            Err(_) => return Vec::new(), // Semaphore closed, return empty result
        };

        let text_owned = text.to_string();
        let tokenizer = Arc::clone(&self.tokenizer);

        // 블로킹 작업을 별도 스레드에서 실행
        tokio::task::spawn_blocking(move || {
            let mut tok = tokenizer.blocking_lock();
            tok.tokenize(&text_owned)
        })
        .await
        .unwrap_or_default()
    }

    /// 비동기 파일 처리
    ///
    /// # Arguments
    ///
    /// * `path` - 파일 경로
    ///
    /// # Returns
    ///
    /// 모든 토큰 목록
    ///
    /// # Errors
    ///
    /// 파일 읽기 실패 시
    pub async fn tokenize_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Token>> {
        let file = File::open(path)
            .await
            .map_err(|e| crate::Error::Analysis(format!("Failed to open file: {e}")))?;

        self.tokenize_reader(file).await
    }

    /// 비동기 Reader 처리
    ///
    /// # Arguments
    ///
    /// * `reader` - 비동기 Reader
    ///
    /// # Returns
    ///
    /// 모든 토큰 목록
    ///
    /// # Errors
    ///
    /// 읽기 실패 시
    pub async fn tokenize_reader<R: AsyncRead + Unpin>(&self, reader: R) -> Result<Vec<Token>> {
        let mut buf_reader = BufReader::new(reader);
        let mut all_tokens = Vec::new();

        loop {
            let mut line = String::new();
            let bytes_read = buf_reader
                .read_line(&mut line)
                .await
                .map_err(|e| crate::Error::Analysis(format!("Failed to read line: {e}")))?;

            if bytes_read == 0 {
                break; // EOF
            }

            let tokens = self.tokenize_async(&line).await;
            all_tokens.extend(tokens);
        }

        Ok(all_tokens)
    }

    /// 배치 비동기 처리
    ///
    /// 여러 텍스트를 동시에 처리합니다.
    ///
    /// # Arguments
    ///
    /// * `texts` - 텍스트 목록
    ///
    /// # Returns
    ///
    /// 각 텍스트의 토큰 목록
    pub async fn tokenize_batch(&self, texts: Vec<String>) -> Vec<Vec<Token>> {
        let mut handles = Vec::new();

        for text in texts {
            let tokenizer = Arc::clone(&self.tokenizer);
            let semaphore = Arc::clone(&self.semaphore);

            let handle = tokio::spawn(async move {
                let _permit = match semaphore.acquire().await {
                    Ok(permit) => permit,
                    Err(_) => return Vec::new(), // Semaphore closed, return empty result
                };

                tokio::task::spawn_blocking(move || {
                    let mut tok = tokenizer.blocking_lock();
                    tok.tokenize(&text)
                })
                .await
                .unwrap_or_default()
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(tokens) = handle.await {
                results.push(tokens);
            } else {
                results.push(Vec::new());
            }
        }

        results
    }

    /// 스트림 처리
    ///
    /// # Arguments
    ///
    /// * `texts` - 텍스트 스트림
    ///
    /// # Returns
    ///
    /// 토큰 스트림
    pub async fn tokenize_stream<I>(&self, texts: I) -> Vec<Vec<Token>>
    where
        I: IntoIterator<Item = String>,
    {
        let texts_vec: Vec<_> = texts.into_iter().collect();
        self.tokenize_batch(texts_vec).await
    }

    /// 동기 토크나이저 참조 (async context에서 접근)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let tokenizer = async_tokenizer.get_tokenizer().await;
    /// // ... 동기 작업 수행
    /// ```
    pub async fn get_tokenizer(&self) -> tokio::sync::MutexGuard<'_, Tokenizer> {
        self.tokenizer.lock().await
    }

    /// 최대 동시 실행 수 조회
    #[must_use]
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }
}

/// 비동기 스트리밍 토크나이저
///
/// AsyncRead를 받아 비동기적으로 토큰을 생성합니다.
pub struct AsyncStreamingTokenizer {
    /// 비동기 토크나이저
    tokenizer: AsyncTokenizer,

    /// 버퍼
    buffer: String,

    /// 문장 구분자
    sentence_delimiters: Vec<char>,
}

impl AsyncStreamingTokenizer {
    /// 새 비동기 스트리밍 토크나이저 생성
    ///
    /// # Arguments
    ///
    /// * `tokenizer` - 비동기 토크나이저
    #[must_use]
    pub fn new(tokenizer: AsyncTokenizer) -> Self {
        Self {
            tokenizer,
            buffer: String::new(),
            sentence_delimiters: vec!['.', '!', '?', '。', '．', '\n'],
        }
    }

    /// 청크 처리 (비동기)
    ///
    /// # Arguments
    ///
    /// * `chunk` - 입력 청크
    ///
    /// # Returns
    ///
    /// 토큰 목록
    pub async fn process_chunk(&mut self, chunk: &str) -> Vec<Token> {
        self.buffer.push_str(chunk);

        // 마지막 문장 구분자 찾기
        let split_pos = self.find_last_sentence_boundary();

        if let Some(pos) = split_pos {
            let to_process = self.buffer[..=pos].to_string();
            let remaining = self.buffer[pos + 1..].to_string();

            let tokens = self.tokenizer.tokenize_async(&to_process).await;

            self.buffer = remaining;
            tokens
        } else {
            Vec::new()
        }
    }

    /// 마지막 문장 경계 찾기
    fn find_last_sentence_boundary(&self) -> Option<usize> {
        let mut last_pos = None;

        for (i, ch) in self.buffer.char_indices() {
            if self.sentence_delimiters.contains(&ch) {
                last_pos = Some(i);
            }
        }

        last_pos
    }

    /// 남은 버퍼 처리 (비동기)
    pub async fn flush(&mut self) -> Vec<Token> {
        if self.buffer.is_empty() {
            return Vec::new();
        }

        let to_process = std::mem::take(&mut self.buffer);
        self.tokenizer.tokenize_async(&to_process).await
    }

    /// Reader에서 스트리밍 처리 (비동기)
    ///
    /// # Arguments
    ///
    /// * `reader` - 비동기 Reader
    ///
    /// # Returns
    ///
    /// 모든 토큰 목록
    ///
    /// # Errors
    ///
    /// 읽기 실패 시
    pub async fn process_reader<R: AsyncRead + Unpin>(&mut self, reader: R) -> Result<Vec<Token>> {
        let mut buf_reader = BufReader::new(reader);
        let mut all_tokens = Vec::new();

        loop {
            let mut line = String::new();
            let bytes_read = buf_reader
                .read_line(&mut line)
                .await
                .map_err(|e| crate::Error::Analysis(format!("Failed to read line: {e}")))?;

            if bytes_read == 0 {
                break; // EOF
            }

            let tokens = self.process_chunk(&line).await;
            all_tokens.extend(tokens);
        }

        // Flush
        let remaining = self.flush().await;
        all_tokens.extend(remaining);

        Ok(all_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_tokenizer_creation() {
        let result = AsyncTokenizer::new().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tokenize_async() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let tokens = tokenizer.tokenize_async("안녕하세요").await;

        assert!(!tokens.is_empty());
    }

    #[tokio::test]
    async fn test_tokenize_batch() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let texts = vec!["안녕하세요".to_string(), "감사합니다".to_string()];

        let results = tokenizer.tokenize_batch(texts).await;

        assert_eq!(results.len(), 2);
        assert!(!results[0].is_empty());
        assert!(!results[1].is_empty());
    }

    #[tokio::test]
    async fn test_max_concurrent() {
        let tokenizer = AsyncTokenizer::new()
            .await
            .expect("should create")
            .with_max_concurrent(8);

        assert_eq!(tokenizer.max_concurrent(), 8);
    }

    #[tokio::test]
    async fn test_async_streaming_tokenizer() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let mut stream = AsyncStreamingTokenizer::new(tokenizer);

        let tokens = stream.process_chunk("안녕하세요.\n").await;
        assert!(!tokens.is_empty() || !stream.buffer.is_empty());

        let remaining = stream.flush().await;
        let total_tokens = tokens.len() + remaining.len();
        assert!(total_tokens > 0);
    }

    #[tokio::test]
    async fn test_tokenize_stream() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let texts = vec!["안녕하세요".to_string(), "감사합니다".to_string()];

        let results = tokenizer.tokenize_stream(texts).await;

        assert_eq!(results.len(), 2);
    }
}
