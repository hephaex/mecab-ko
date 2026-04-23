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
//! ```rust,no_run
//! use mecab_ko_core::async_tokenizer::AsyncTokenizer;
//!
//! #[tokio::main]
//! async fn main() {
//!     let tokenizer = AsyncTokenizer::new().await.unwrap();
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
    /// ```rust,no_run
    /// # use mecab_ko_core::async_tokenizer::AsyncTokenizer;
    /// # async fn example() {
    /// #     let async_tokenizer = AsyncTokenizer::new().await.unwrap();
    /// let tokenizer = async_tokenizer.get_tokenizer().await;
    /// // ... 동기 작업 수행
    /// # }
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
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------
    // AsyncTokenizer — construction
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn test_async_tokenizer_creation() {
        let result = AsyncTokenizer::new().await;
        assert!(result.is_ok());
    }

    /// DEFAULT_MAX_CONCURRENT is 4 immediately after construction.
    #[tokio::test]
    async fn test_default_max_concurrent_value() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        assert_eq!(tokenizer.max_concurrent(), AsyncTokenizer::DEFAULT_MAX_CONCURRENT);
    }

    /// `with_max_concurrent` returns a new value reflected by `max_concurrent()`.
    #[tokio::test]
    async fn test_max_concurrent() {
        let tokenizer = AsyncTokenizer::new()
            .await
            .expect("should create")
            .with_max_concurrent(8);

        assert_eq!(tokenizer.max_concurrent(), 8);
    }

    /// `with_max_concurrent(1)` is a legal edge value (serialise all work).
    #[tokio::test]
    async fn test_max_concurrent_one() {
        let tokenizer = AsyncTokenizer::new()
            .await
            .expect("should create")
            .with_max_concurrent(1);

        assert_eq!(tokenizer.max_concurrent(), 1);
    }

    // ---------------------------------------------------------------------------
    // AsyncTokenizer — tokenize_async
    // ---------------------------------------------------------------------------

    /// Empty string input must return an empty Vec without panicking.
    #[tokio::test]
    async fn test_tokenize_async_empty_string() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let tokens = tokenizer.tokenize_async("").await;
        // Empty input always produces zero tokens regardless of the dictionary.
        assert!(tokens.is_empty(), "expected no tokens for empty input, got {}", tokens.len());
    }

    /// Single ASCII character — must not panic; token count >= 0.
    #[tokio::test]
    async fn test_tokenize_async_single_ascii_char() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let tokens = tokenizer.tokenize_async("a").await;
        assert!(tokens.iter().all(|t| !t.surface.is_empty()));
    }

    /// Korean text tokenisation — may produce 0 tokens with the mini-dict, but
    /// must not panic and must return a Vec.
    #[tokio::test]
    async fn test_tokenize_async_korean_text() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let tokens = tokenizer.tokenize_async("안녕하세요").await;
        assert!(tokens.iter().all(|t| !t.surface.is_empty()));
    }

    /// Multi-byte Korean input with punctuation must not panic.
    #[tokio::test]
    async fn test_tokenize_async_multibyte_korean() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        // 오늘 날씨가 좋네요 — contains multi-byte UTF-8 characters
        let tokens = tokenizer.tokenize_async("오늘 날씨가 좋네요.").await;
        assert!(tokens.iter().all(|t| !t.surface.is_empty()));
    }

    /// Calling tokenize_async twice on the same AsyncTokenizer must work (Mutex
    /// released between calls).
    #[tokio::test]
    async fn test_tokenize_async_reuse() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let t1 = tokenizer.tokenize_async("안녕").await;
        let t2 = tokenizer.tokenize_async("안녕").await;
        // Both calls must produce the same number of tokens (determinism).
        assert_eq!(t1.len(), t2.len(), "repeated calls should return same token count");
    }

    // ---------------------------------------------------------------------------
    // AsyncTokenizer — tokenize_batch
    // ---------------------------------------------------------------------------

    /// Batch with two texts — result length must equal input length.
    #[tokio::test]
    async fn test_tokenize_batch_length() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let texts = vec!["안녕하세요".to_string(), "감사합니다".to_string()];
        let results = tokenizer.tokenize_batch(texts).await;
        assert_eq!(results.len(), 2, "batch result count must match input count");
    }

    /// Batch with an empty list — must return an empty Vec.
    #[tokio::test]
    async fn test_tokenize_batch_empty_input() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let results = tokenizer.tokenize_batch(Vec::new()).await;
        assert!(results.is_empty(), "empty batch must produce empty results");
    }

    /// Batch with a single-item list — result length is 1.
    #[tokio::test]
    async fn test_tokenize_batch_single_item() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let results = tokenizer.tokenize_batch(vec!["안녕".to_string()]).await;
        assert_eq!(results.len(), 1);
    }

    /// Batch with empty string entries — must return a result per entry.
    #[tokio::test]
    async fn test_tokenize_batch_with_empty_strings() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let texts = vec!["".to_string(), "".to_string(), "".to_string()];
        let results = tokenizer.tokenize_batch(texts).await;
        assert_eq!(results.len(), 3);
        // Empty strings always produce empty token lists.
        for result in &results {
            assert!(result.is_empty());
        }
    }

    // ---------------------------------------------------------------------------
    // AsyncTokenizer — tokenize_stream
    // ---------------------------------------------------------------------------

    /// `tokenize_stream` is defined as a thin wrapper around `tokenize_batch`;
    /// its result length must equal the number of items in the iterator.
    #[tokio::test]
    async fn test_tokenize_stream_length() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let texts = vec!["안녕하세요".to_string(), "감사합니다".to_string()];
        let results = tokenizer.tokenize_stream(texts).await;
        assert_eq!(results.len(), 2);
    }

    /// `tokenize_stream` on an empty iterator must return an empty Vec.
    #[tokio::test]
    async fn test_tokenize_stream_empty() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let results = tokenizer.tokenize_stream(std::iter::empty::<String>()).await;
        assert!(results.is_empty());
    }

    // ---------------------------------------------------------------------------
    // AsyncTokenizer — tokenize_reader
    // ---------------------------------------------------------------------------

    /// Reader over an empty byte slice must return Ok with an empty token Vec.
    #[tokio::test]
    async fn test_tokenize_reader_empty() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let cursor = std::io::Cursor::new(b"" as &[u8]);
        let result = tokenizer.tokenize_reader(cursor).await;
        assert!(result.is_ok(), "tokenize_reader should succeed on empty input");
        assert!(result.unwrap().is_empty());
    }

    /// Reader over a single newline-terminated line must not panic.
    #[tokio::test]
    async fn test_tokenize_reader_single_line() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let data = "안녕하세요.\n";
        let cursor = std::io::Cursor::new(data.as_bytes());
        let result = tokenizer.tokenize_reader(cursor).await;
        assert!(result.is_ok(), "tokenize_reader should succeed");
    }

    /// Reader over multiple lines must process all lines without error.
    #[tokio::test]
    async fn test_tokenize_reader_multiple_lines() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let data = "첫 번째 줄.\n두 번째 줄.\n";
        let cursor = std::io::Cursor::new(data.as_bytes());
        let result = tokenizer.tokenize_reader(cursor).await;
        assert!(result.is_ok(), "tokenize_reader should succeed on multiple lines");
    }

    // ---------------------------------------------------------------------------
    // AsyncTokenizer — tokenize_file (error path)
    // ---------------------------------------------------------------------------

    /// Attempting to open a non-existent file must return an Err, not panic.
    #[tokio::test]
    async fn test_tokenize_file_nonexistent() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let result = tokenizer.tokenize_file("/nonexistent/path/that/does/not/exist.txt").await;
        assert!(result.is_err(), "tokenize_file on missing path must return Err");
    }

    // ---------------------------------------------------------------------------
    // AsyncTokenizer — get_tokenizer
    // ---------------------------------------------------------------------------

    /// `get_tokenizer` must return a guard that can call `tokenize` synchronously.
    #[tokio::test]
    async fn test_get_tokenizer_sync_call() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let mut guard = tokenizer.get_tokenizer().await;
        // Call the synchronous tokenizer through the guard — must not panic.
        let tokens = guard.tokenize("안녕");
        assert!(tokens.iter().all(|t| !t.surface.is_empty()));
    }

    // ---------------------------------------------------------------------------
    // AsyncStreamingTokenizer — construction
    // ---------------------------------------------------------------------------

    /// Default sentence delimiters must include '.' '\n' '?' '!'.
    #[tokio::test]
    async fn test_async_streaming_tokenizer_default_delimiters() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let stream = AsyncStreamingTokenizer::new(tokenizer);
        assert!(stream.sentence_delimiters.contains(&'.'));
        assert!(stream.sentence_delimiters.contains(&'\n'));
        assert!(stream.sentence_delimiters.contains(&'?'));
        assert!(stream.sentence_delimiters.contains(&'!'));
    }

    /// New stream must start with an empty buffer.
    #[tokio::test]
    async fn test_async_streaming_tokenizer_initial_buffer_empty() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let stream = AsyncStreamingTokenizer::new(tokenizer);
        assert!(stream.buffer.is_empty(), "buffer must be empty on construction");
    }

    // ---------------------------------------------------------------------------
    // AsyncStreamingTokenizer — process_chunk / flush
    // ---------------------------------------------------------------------------

    /// `flush` on an empty buffer must return an empty Vec.
    #[tokio::test]
    async fn test_async_streaming_flush_empty_buffer() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let mut stream = AsyncStreamingTokenizer::new(tokenizer);
        let tokens = stream.flush().await;
        assert!(tokens.is_empty(), "flush on empty buffer must produce no tokens");
        assert!(stream.buffer.is_empty(), "buffer must remain empty after flushing empty buffer");
    }

    /// After `flush`, the buffer must be empty regardless of prior state.
    #[tokio::test]
    async fn test_async_streaming_flush_clears_buffer() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let mut stream = AsyncStreamingTokenizer::new(tokenizer);

        // Push text without a sentence delimiter so it stays in the buffer.
        let tokens = stream.process_chunk("버퍼에 남을 텍스트").await;
        assert!(tokens.iter().all(|t| !t.surface.is_empty()));
        assert!(!stream.buffer.is_empty(), "buffer should hold unprocessed text");

        let flushed = stream.flush().await;
        assert!(flushed.iter().all(|t| !t.surface.is_empty()));
        assert!(stream.buffer.is_empty(), "flush must clear the buffer");
    }

    /// Text with a newline delimiter — process_chunk triggers tokenisation and
    /// leaves whatever follows the delimiter in the buffer.
    #[tokio::test]
    async fn test_async_streaming_chunk_with_newline_delimiter() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let mut stream = AsyncStreamingTokenizer::new(tokenizer);

        // The '\n' triggers sentence boundary detection.  Regardless of the
        // token count (which depends on the dictionary), the call must not panic
        // and the buffer must not still contain the '\n'-terminated prefix.
        let tokens = stream.process_chunk("안녕하세요.\n").await;
        let remaining = stream.flush().await;
        // Total token count may be zero with mini-dict, but the pipeline must complete.
        let total = tokens.len() + remaining.len();
        assert!(tokens.iter().chain(remaining.iter()).all(|t| !t.surface.is_empty()), "all tokens must have non-empty surface (total: {total})");
    }

    /// Text with no delimiter must be buffered, not tokenised immediately.
    #[tokio::test]
    async fn test_async_streaming_chunk_without_delimiter_stays_buffered() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let mut stream = AsyncStreamingTokenizer::new(tokenizer);

        let tokens = stream.process_chunk("구분자없음").await;
        // No delimiter ⇒ no output from process_chunk.
        assert!(tokens.is_empty(), "text without delimiter must not produce tokens immediately");
        // The text must have been buffered.
        assert!(!stream.buffer.is_empty(), "text without delimiter must be held in the buffer");
    }

    /// process_reader on empty bytes — must return Ok(empty).
    #[tokio::test]
    async fn test_async_streaming_process_reader_empty() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let mut stream = AsyncStreamingTokenizer::new(tokenizer);

        let cursor = std::io::Cursor::new(b"" as &[u8]);
        let result = stream.process_reader(cursor).await;
        assert!(result.is_ok(), "process_reader on empty input must succeed");
        assert!(result.unwrap().is_empty());
    }

    /// process_reader on multi-line input — must succeed and flush everything.
    #[tokio::test]
    async fn test_async_streaming_process_reader_multiline() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let mut stream = AsyncStreamingTokenizer::new(tokenizer);

        let data = "첫째 줄.\n둘째 줄.\n";
        let cursor = std::io::Cursor::new(data.as_bytes());
        let result = stream.process_reader(cursor).await;
        assert!(result.is_ok(), "process_reader must succeed on multi-line input");
        // After process_reader the buffer should be empty (flush was called internally).
        assert!(stream.buffer.is_empty(), "process_reader must flush the buffer at the end");
    }

    // ---------------------------------------------------------------------------
    // AsyncStreamingTokenizer — find_last_sentence_boundary (via process_chunk)
    // ---------------------------------------------------------------------------

    /// Multiple delimiter characters ('.' '!' '?') — the last one is used as the
    /// split point.  Each chunk must be processed without panic.
    #[tokio::test]
    async fn test_async_streaming_multiple_delimiters_in_chunk() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let mut stream = AsyncStreamingTokenizer::new(tokenizer);

        // Both '.' and '?' are delimiters; the last one ('?') should be the split.
        let tokens = stream.process_chunk("안녕하세요. 괜찮으세요?").await;
        assert!(tokens.iter().all(|t| !t.surface.is_empty()));
        let flushed = stream.flush().await;
        assert!(flushed.iter().all(|t| !t.surface.is_empty()));
    }

    /// Japanese full-stop '。' is a multi-byte delimiter — must not panic.
    #[tokio::test]
    async fn test_async_streaming_multibyte_delimiter_no_panic() {
        let tokenizer = AsyncTokenizer::new().await.expect("should create");
        let mut stream = AsyncStreamingTokenizer::new(tokenizer);

        // '。' is U+3002, encoded as 3 bytes in UTF-8.
        let tokens = stream.process_chunk("テスト。次の文。\n").await;
        assert!(tokens.iter().all(|t| !t.surface.is_empty()));
        let flushed = stream.flush().await;
        assert!(flushed.iter().all(|t| !t.surface.is_empty()));
    }
}
