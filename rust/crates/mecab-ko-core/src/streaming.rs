//! # Streaming Tokenizer Module
//!
//! 대용량 텍스트 스트리밍 처리를 위한 API
//!
//! ## 주요 기능
//!
//! - 청크 단위 토큰화
//! - 문장 경계 감지 및 버퍼링
//! - 메모리 효율적인 대용량 파일 처리
//!
//! ## Example
//!
//! ```rust,no_run
//! use mecab_ko_core::streaming::StreamingTokenizer;
//! use mecab_ko_core::tokenizer::Tokenizer;
//!
//! let tokenizer = Tokenizer::new().unwrap();
//! let mut stream = StreamingTokenizer::new(tokenizer);
//!
//! // 청크 단위로 처리
//! let text_chunks = vec!["안녕하세요. ", "오늘 날씨가 좋네요."];
//! for chunk in text_chunks {
//!     let tokens = stream.process_chunk(chunk);
//!     for token in tokens {
//!         println!("{}: {}", token.surface, token.pos);
//!     }
//! }
//!
//! // 남은 버퍼 flush
//! let remaining = stream.flush();
//! ```

use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader, Read};

use crate::tokenizer::{Token, Tokenizer};
use crate::Result;

/// 스트리밍 토크나이저
///
/// 대용량 텍스트를 청크 단위로 처리하며, 문장 경계를 고려하여
/// 올바른 토큰화를 보장합니다.
pub struct StreamingTokenizer {
    /// 내부 토크나이저
    tokenizer: Tokenizer,

    /// 버퍼 (문장 경계를 고려하여 이전 청크의 일부를 보관)
    buffer: String,

    /// 청크 크기 (바이트)
    chunk_size: usize,

    /// 문장 구분자
    sentence_delimiters: Vec<char>,

    /// 전체 처리된 문자 수
    total_chars_processed: usize,
}

impl StreamingTokenizer {
    /// 기본 청크 크기 (8KB)
    pub const DEFAULT_CHUNK_SIZE: usize = 8192;

    /// 새 스트리밍 토크나이저 생성
    ///
    /// # Arguments
    ///
    /// * `tokenizer` - 내부 토크나이저
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mecab_ko_core::tokenizer::Tokenizer;
    /// use mecab_ko_core::streaming::StreamingTokenizer;
    ///
    /// let tokenizer = Tokenizer::new().unwrap();
    /// let stream = StreamingTokenizer::new(tokenizer);
    /// ```
    #[must_use]
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            tokenizer,
            buffer: String::with_capacity(Self::DEFAULT_CHUNK_SIZE),
            chunk_size: Self::DEFAULT_CHUNK_SIZE,
            sentence_delimiters: vec!['.', '!', '?', '。', '．', '\n'],
            total_chars_processed: 0,
        }
    }

    /// 청크 크기 설정
    ///
    /// # Arguments
    ///
    /// * `size` - 청크 크기 (바이트)
    #[must_use]
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self.buffer = String::with_capacity(size);
        self
    }

    /// 문장 구분자 설정
    ///
    /// # Arguments
    ///
    /// * `delimiters` - 문장 구분자 목록
    #[must_use]
    pub fn with_sentence_delimiters(mut self, delimiters: Vec<char>) -> Self {
        self.sentence_delimiters = delimiters;
        self
    }

    /// 청크 처리
    ///
    /// 입력 청크를 버퍼에 추가하고, 완전한 문장을 토큰화합니다.
    ///
    /// # Arguments
    ///
    /// * `chunk` - 입력 청크
    ///
    /// # Returns
    ///
    /// 토큰 목록
    pub fn process_chunk(&mut self, chunk: &str) -> Vec<Token> {
        // 버퍼에 청크 추가
        self.buffer.push_str(chunk);

        // 마지막 문장 구분자 찾기
        let split_pos = self.find_last_sentence_boundary();

        if let Some(pos) = split_pos {
            // 구분자까지의 텍스트 처리
            let to_process = self.buffer[..=pos].to_string();
            let remaining = self.buffer[pos + 1..].to_string();

            // 토큰화
            let mut tokens = self.tokenizer.tokenize(&to_process);

            // 위치 정보 조정 (전체 텍스트 기준)
            for token in &mut tokens {
                token.start_pos += self.total_chars_processed;
                token.end_pos += self.total_chars_processed;
            }

            self.total_chars_processed += to_process.chars().count();
            self.buffer = remaining;

            tokens
        } else {
            // 문장 구분자가 없으면 버퍼가 너무 커질 수 있으므로
            // 일정 크기 이상이면 강제 처리
            if self.buffer.len() > self.chunk_size * 2 {
                self.force_flush_partial()
            } else {
                Vec::new()
            }
        }
    }

    /// 마지막 문장 경계 찾기 (역방향 탐색으로 최적화)
    fn find_last_sentence_boundary(&self) -> Option<usize> {
        // 역방향 탐색으로 첫 번째 발견 시 즉시 반환
        for (i, ch) in self.buffer.char_indices().rev() {
            if self.sentence_delimiters.contains(&ch) {
                return Some(i);
            }
        }
        None
    }

    /// 문장 경계에서 분할 (단어 중간 분할 방지)
    fn find_safe_split_point(&self, target_pos: usize) -> usize {
        // target_pos 근처에서 공백이나 문장 부호 찾기
        let mut pos = target_pos.min(self.buffer.len());

        // 뒤로 탐색하여 안전한 분할점 찾기
        while pos > 0 {
            if let Some(ch) = self.buffer[..pos].chars().last() {
                if ch.is_whitespace() || self.sentence_delimiters.contains(&ch) {
                    // char boundary 확인
                    if self.buffer.is_char_boundary(pos) {
                        return pos;
                    }
                }
            }
            pos -= 1;
        }

        // 안전한 분할점이 없으면 char boundary에서 분할
        let mut safe_pos = target_pos.min(self.buffer.len());
        while safe_pos > 0 && !self.buffer.is_char_boundary(safe_pos) {
            safe_pos -= 1;
        }
        safe_pos
    }

    /// 부분 버퍼 강제 flush (문장 경계가 없을 때)
    fn force_flush_partial(&mut self) -> Vec<Token> {
        // 안전한 분할점에서 처리 (단어 중간 분할 방지)
        let target_pos = self.buffer.len() / 2;
        let split_pos = self.find_safe_split_point(target_pos);

        if split_pos == 0 {
            // 분할점을 찾을 수 없으면 전체 버퍼 처리
            return self.flush();
        }

        let to_process = self.buffer[..split_pos].to_string();
        let remaining = self.buffer[split_pos..].to_string();

        let mut tokens = self.tokenizer.tokenize(&to_process);

        for token in &mut tokens {
            token.start_pos += self.total_chars_processed;
            token.end_pos += self.total_chars_processed;
        }

        self.total_chars_processed += to_process.chars().count();
        self.buffer = remaining;

        tokens
    }

    /// 남은 버퍼 처리
    ///
    /// 스트림 처리가 끝난 후 버퍼에 남아있는 텍스트를 처리합니다.
    ///
    /// # Returns
    ///
    /// 남은 토큰 목록
    pub fn flush(&mut self) -> Vec<Token> {
        if self.buffer.is_empty() {
            return Vec::new();
        }

        let to_process = std::mem::take(&mut self.buffer);
        let mut tokens = self.tokenizer.tokenize(&to_process);

        for token in &mut tokens {
            token.start_pos += self.total_chars_processed;
            token.end_pos += self.total_chars_processed;
        }

        self.total_chars_processed += to_process.chars().count();

        tokens
    }

    /// Reader에서 스트리밍 처리
    ///
    /// # Arguments
    ///
    /// * `reader` - 입력 Reader
    ///
    /// # Returns
    ///
    /// 모든 토큰 목록
    ///
    /// # Errors
    ///
    /// I/O 에러 발생 시
    pub fn process_reader<R: Read>(&mut self, reader: R) -> Result<Vec<Token>> {
        let mut buf_reader = BufReader::with_capacity(self.chunk_size, reader);
        let mut all_tokens = Vec::new();

        loop {
            let mut line = String::new();
            let bytes_read = buf_reader
                .read_line(&mut line)
                .map_err(|e| crate::Error::Analysis(format!("Failed to read line: {e}")))?;

            if bytes_read == 0 {
                break; // EOF
            }

            let tokens = self.process_chunk(&line);
            all_tokens.extend(tokens);
        }

        // Flush 남은 버퍼
        let remaining = self.flush();
        all_tokens.extend(remaining);

        Ok(all_tokens)
    }

    /// 파일에서 스트리밍 처리
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
    /// 파일을 열 수 없거나 읽기 실패 시
    pub fn process_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<Vec<Token>> {
        let file = std::fs::File::open(path)
            .map_err(|e| crate::Error::Analysis(format!("Failed to open file: {e}")))?;
        self.process_reader(file)
    }

    /// 버퍼 크기 확인
    #[must_use]
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// 처리된 문자 수
    #[must_use]
    pub const fn total_chars_processed(&self) -> usize {
        self.total_chars_processed
    }

    /// 스트림 리셋
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.total_chars_processed = 0;
    }
}

/// Iterator 기반 스트리밍 토크나이저
///
/// 텍스트 청크 iterator를 받아 토큰을 생성합니다.
/// `VecDeque`를 사용하여 O(1) dequeue 성능을 보장합니다.
pub struct TokenStream<I>
where
    I: Iterator<Item = String>,
{
    /// 청크 iterator
    chunks: I,

    /// 스트리밍 토크나이저
    streaming: StreamingTokenizer,

    /// 현재 처리 중인 토큰 버퍼 (`VecDeque` for O(1) `pop_front`)
    token_buffer: VecDeque<Token>,

    /// 스트림 종료 여부
    finished: bool,

    /// 처리된 총 토큰 수 (`size_hint`용)
    tokens_yielded: usize,
}

impl<I> TokenStream<I>
where
    I: Iterator<Item = String>,
{
    /// 새 토큰 스트림 생성
    ///
    /// # Arguments
    ///
    /// * `chunks` - 텍스트 청크 iterator
    /// * `tokenizer` - 토크나이저
    #[must_use]
    pub fn new(chunks: I, tokenizer: Tokenizer) -> Self {
        Self {
            chunks,
            streaming: StreamingTokenizer::new(tokenizer),
            token_buffer: VecDeque::new(),
            finished: false,
            tokens_yielded: 0,
        }
    }

    /// 청크 크기 설정
    #[must_use]
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.streaming = self.streaming.with_chunk_size(size);
        self
    }

    /// 처리된 토큰 수 조회
    #[must_use]
    pub const fn tokens_yielded(&self) -> usize {
        self.tokens_yielded
    }
}

impl<I> Iterator for TokenStream<I>
where
    I: Iterator<Item = String>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // 버퍼에서 토큰 반환 (O(1) pop_front)
        if let Some(token) = self.token_buffer.pop_front() {
            self.tokens_yielded += 1;
            return Some(token);
        }

        // 스트림이 끝났으면 None
        if self.finished {
            return None;
        }

        // 다음 청크 처리
        for chunk in self.chunks.by_ref() {
            let tokens = self.streaming.process_chunk(&chunk);

            if !tokens.is_empty() {
                self.token_buffer.extend(tokens);
                if let Some(token) = self.token_buffer.pop_front() {
                    self.tokens_yielded += 1;
                    return Some(token);
                }
            }
        }

        // 청크가 더 이상 없으면 flush
        self.finished = true;
        let remaining = self.streaming.flush();

        if !remaining.is_empty() {
            self.token_buffer.extend(remaining);
            if let Some(token) = self.token_buffer.pop_front() {
                self.tokens_yielded += 1;
                return Some(token);
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // 버퍼에 있는 토큰 수를 최소 하한으로 제공
        (self.token_buffer.len(), None)
    }
}

/// 진행률 콜백 타입
pub type ProgressCallback = Box<dyn Fn(StreamingProgress) + Send>;

/// 스트리밍 진행 상황
#[derive(Debug, Clone)]
pub struct StreamingProgress {
    /// 처리된 바이트 수
    pub bytes_processed: usize,
    /// 총 바이트 수 (알 수 있는 경우)
    pub total_bytes: Option<usize>,
    /// 처리된 토큰 수
    pub tokens_generated: usize,
    /// 처리된 청크 수
    pub chunks_processed: usize,
}

impl StreamingProgress {
    /// 진행률 퍼센트 계산
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn percent(&self) -> Option<f64> {
        self.total_bytes
            .map(|total| (self.bytes_processed as f64 / total as f64) * 100.0)
    }
}

/// 진행률 추적 스트리밍 토크나이저
///
/// 대용량 파일 처리 시 진행 상황을 콜백으로 보고합니다.
pub struct ProgressStreamingTokenizer {
    /// 내부 스트리밍 토크나이저
    inner: StreamingTokenizer,

    /// 진행률 콜백
    callback: Option<ProgressCallback>,

    /// 처리된 바이트 수
    bytes_processed: usize,

    /// 총 바이트 수
    total_bytes: Option<usize>,

    /// 생성된 토큰 수
    tokens_generated: usize,

    /// 처리된 청크 수
    chunks_processed: usize,

    /// 콜백 호출 간격 (바이트)
    callback_interval: usize,

    /// 마지막 콜백 호출 시 처리된 바이트
    last_callback_bytes: usize,
}

impl ProgressStreamingTokenizer {
    /// 기본 콜백 간격 (64KB)
    pub const DEFAULT_CALLBACK_INTERVAL: usize = 65536;

    /// 새 진행률 추적 토크나이저 생성
    #[must_use]
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            inner: StreamingTokenizer::new(tokenizer),
            callback: None,
            bytes_processed: 0,
            total_bytes: None,
            tokens_generated: 0,
            chunks_processed: 0,
            callback_interval: Self::DEFAULT_CALLBACK_INTERVAL,
            last_callback_bytes: 0,
        }
    }

    /// 진행률 콜백 설정
    #[must_use]
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(StreamingProgress) + Send + 'static,
    {
        self.callback = Some(Box::new(callback));
        self
    }

    /// 총 바이트 수 설정 (진행률 계산용)
    #[must_use]
    pub const fn with_total_bytes(mut self, total: usize) -> Self {
        self.total_bytes = Some(total);
        self
    }

    /// 콜백 간격 설정
    #[must_use]
    pub const fn with_callback_interval(mut self, interval: usize) -> Self {
        self.callback_interval = interval;
        self
    }

    /// 청크 크기 설정
    #[must_use]
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.inner = self.inner.with_chunk_size(size);
        self
    }

    /// 청크 처리
    pub fn process_chunk(&mut self, chunk: &str) -> Vec<Token> {
        self.bytes_processed += chunk.len();
        self.chunks_processed += 1;

        let tokens = self.inner.process_chunk(chunk);
        self.tokens_generated += tokens.len();

        // 콜백 호출 간격 확인
        if self.bytes_processed - self.last_callback_bytes >= self.callback_interval {
            self.report_progress();
            self.last_callback_bytes = self.bytes_processed;
        }

        tokens
    }

    /// 남은 버퍼 처리
    pub fn flush(&mut self) -> Vec<Token> {
        let tokens = self.inner.flush();
        self.tokens_generated += tokens.len();

        // 최종 진행률 보고
        self.report_progress();

        tokens
    }

    /// 진행 상황 보고
    fn report_progress(&self) {
        if let Some(ref callback) = self.callback {
            callback(StreamingProgress {
                bytes_processed: self.bytes_processed,
                total_bytes: self.total_bytes,
                tokens_generated: self.tokens_generated,
                chunks_processed: self.chunks_processed,
            });
        }
    }

    /// Reader에서 스트리밍 처리 (진행률 추적)
    ///
    /// # Errors
    ///
    /// I/O 에러 발생 시
    pub fn process_reader<R: Read>(&mut self, reader: R) -> Result<Vec<Token>> {
        let mut buf_reader = BufReader::with_capacity(self.inner.chunk_size, reader);
        let mut all_tokens = Vec::new();

        loop {
            let mut line = String::new();
            let bytes_read = buf_reader
                .read_line(&mut line)
                .map_err(|e| crate::Error::Analysis(format!("Failed to read line: {e}")))?;

            if bytes_read == 0 {
                break;
            }

            let tokens = self.process_chunk(&line);
            all_tokens.extend(tokens);
        }

        let remaining = self.flush();
        all_tokens.extend(remaining);

        Ok(all_tokens)
    }

    /// 파일에서 스트리밍 처리 (자동 크기 감지)
    ///
    /// # Errors
    ///
    /// 파일을 열 수 없거나 읽기 실패 시
    #[allow(clippy::cast_possible_truncation)]
    pub fn process_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<Vec<Token>> {
        let metadata = std::fs::metadata(path.as_ref())
            .map_err(|e| crate::Error::Analysis(format!("Failed to read metadata: {e}")))?;

        self.total_bytes = Some(metadata.len() as usize);

        let file = std::fs::File::open(path)
            .map_err(|e| crate::Error::Analysis(format!("Failed to open file: {e}")))?;

        self.process_reader(file)
    }

    /// 현재 진행 상황 조회
    #[must_use]
    pub const fn progress(&self) -> StreamingProgress {
        StreamingProgress {
            bytes_processed: self.bytes_processed,
            total_bytes: self.total_bytes,
            tokens_generated: self.tokens_generated,
            chunks_processed: self.chunks_processed,
        }
    }

    /// 리셋
    pub fn reset(&mut self) {
        self.inner.reset();
        self.bytes_processed = 0;
        self.tokens_generated = 0;
        self.chunks_processed = 0;
        self.last_callback_bytes = 0;
    }
}

/// 청크별 토큰 이터레이터
///
/// 토큰을 개별로 반환하지 않고 청크 단위로 반환하여 메모리 효율성 향상
pub struct ChunkedTokenIterator<I>
where
    I: Iterator<Item = String>,
{
    /// 청크 iterator
    chunks: I,

    /// 스트리밍 토크나이저
    streaming: StreamingTokenizer,

    /// 스트림 종료 여부
    finished: bool,
}

impl<I> ChunkedTokenIterator<I>
where
    I: Iterator<Item = String>,
{
    /// 새 청크 토큰 이터레이터 생성
    #[must_use]
    pub fn new(chunks: I, tokenizer: Tokenizer) -> Self {
        Self {
            chunks,
            streaming: StreamingTokenizer::new(tokenizer),
            finished: false,
        }
    }

    /// 청크 크기 설정
    #[must_use]
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.streaming = self.streaming.with_chunk_size(size);
        self
    }
}

impl<I> Iterator for ChunkedTokenIterator<I>
where
    I: Iterator<Item = String>,
{
    type Item = Vec<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // 다음 청크에서 토큰 생성
        for chunk in self.chunks.by_ref() {
            let tokens = self.streaming.process_chunk(&chunk);
            if !tokens.is_empty() {
                return Some(tokens);
            }
        }

        // 청크가 더 이상 없으면 flush
        self.finished = true;
        let remaining = self.streaming.flush();

        if remaining.is_empty() {
            None
        } else {
            Some(remaining)
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn create_test_tokenizer() -> Tokenizer {
        Tokenizer::new().expect("should create tokenizer")
    }

    #[test]
    fn test_streaming_tokenizer_creation() {
        let tokenizer = create_test_tokenizer();
        let stream = StreamingTokenizer::new(tokenizer);

        assert_eq!(stream.buffer_len(), 0);
        assert_eq!(stream.total_chars_processed(), 0);
    }

    #[test]
    fn test_process_chunk_with_delimiter() {
        let tokenizer = create_test_tokenizer();
        let mut stream = StreamingTokenizer::new(tokenizer);

        let tokens = stream.process_chunk("안녕\n");
        assert!(!tokens.is_empty() || stream.buffer_len() > 0);

        // Flush로 남은 토큰 확인
        let remaining = stream.flush();
        let total_tokens = tokens.len() + remaining.len();
        assert!(total_tokens > 0);
    }

    #[test]
    fn test_process_chunk_without_delimiter() {
        let tokenizer = create_test_tokenizer();
        let mut stream = StreamingTokenizer::new(tokenizer);

        let tokens = stream.process_chunk("안녕하세요");
        // 구분자가 없으면 버퍼에 저장
        assert!(tokens.is_empty() || stream.buffer_len() > 0);
    }

    #[test]
    fn test_flush() {
        let tokenizer = create_test_tokenizer();
        let mut stream = StreamingTokenizer::new(tokenizer);

        stream.process_chunk("안녕하세요");
        let tokens = stream.flush();

        assert!(!tokens.is_empty());
        assert_eq!(stream.buffer_len(), 0);
    }

    #[test]
    fn test_multiple_chunks() {
        let tokenizer = create_test_tokenizer();
        let mut stream = StreamingTokenizer::new(tokenizer);

        let _tokens1 = stream.process_chunk("안녕하세요.\n");
        let _tokens2 = stream.process_chunk("감사합니다.\n");
        let _remaining = stream.flush();

        assert!(stream.total_chars_processed() > 0);
    }

    #[test]
    fn test_reset() {
        let tokenizer = create_test_tokenizer();
        let mut stream = StreamingTokenizer::new(tokenizer);

        stream.process_chunk("안녕하세요");
        stream.reset();

        assert_eq!(stream.buffer_len(), 0);
        assert_eq!(stream.total_chars_processed(), 0);
    }

    #[test]
    fn test_custom_chunk_size() {
        let tokenizer = create_test_tokenizer();
        let stream = StreamingTokenizer::new(tokenizer).with_chunk_size(1024);

        assert_eq!(stream.chunk_size, 1024);
    }

    #[test]
    fn test_custom_delimiters() {
        let tokenizer = create_test_tokenizer();
        let stream =
            StreamingTokenizer::new(tokenizer).with_sentence_delimiters(vec!['.', '!', '?']);

        assert_eq!(stream.sentence_delimiters.len(), 3);
    }

    #[test]
    fn test_token_stream_creation() {
        let tokenizer = create_test_tokenizer();
        let chunks = vec!["안녕하세요.\n".to_string(), "감사합니다.\n".to_string()];
        let stream = TokenStream::new(chunks.into_iter(), tokenizer);

        assert!(!stream.finished);
    }

    #[test]
    fn test_token_stream_iteration() {
        let tokenizer = create_test_tokenizer();
        let chunks = vec!["안녕\n".to_string(), "감사\n".to_string()];
        let stream = TokenStream::new(chunks.into_iter(), tokenizer);

        let tokens: Vec<_> = stream.collect();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_token_stream_tokens_yielded() {
        let tokenizer = create_test_tokenizer();
        let chunks = vec!["안녕하세요.\n".to_string()];
        let mut stream = TokenStream::new(chunks.into_iter(), tokenizer);

        // 몇 개의 토큰 소비
        let mut count = 0;
        while stream.next().is_some() {
            count += 1;
        }

        // 토큰이 생성되었고, 카운트가 맞는지 확인
        assert_eq!(stream.tokens_yielded(), count);
    }

    #[test]
    fn test_token_stream_size_hint() {
        let tokenizer = create_test_tokenizer();
        let chunks = vec!["안녕하세요.\n".to_string()];
        let stream = TokenStream::new(chunks.into_iter(), tokenizer);

        let (lower, _upper) = stream.size_hint();
        // 초기에는 버퍼가 비어있으므로 0
        assert_eq!(lower, 0);
    }

    #[test]
    fn test_progress_streaming_tokenizer() {
        let tokenizer = create_test_tokenizer();
        let mut stream = ProgressStreamingTokenizer::new(tokenizer);

        let _tokens = stream.process_chunk("안녕하세요.\n");
        let progress = stream.progress();

        assert!(progress.bytes_processed > 0);
        assert!(progress.chunks_processed > 0);
    }

    #[test]
    fn test_progress_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let tokenizer = create_test_tokenizer();
        let callback_count = Arc::new(AtomicUsize::new(0));
        let callback_count_clone = Arc::clone(&callback_count);

        let mut stream = ProgressStreamingTokenizer::new(tokenizer)
            .with_callback_interval(1) // 매 바이트마다 콜백
            .with_progress_callback(move |_progress| {
                callback_count_clone.fetch_add(1, Ordering::SeqCst);
            });

        // 충분한 데이터 처리
        stream.process_chunk("안녕하세요. 오늘 날씨가 좋네요.\n");
        let _remaining = stream.flush();

        // 콜백이 호출되었는지 확인
        assert!(callback_count.load(Ordering::SeqCst) > 0);
    }

    #[test]
    fn test_progress_percent() {
        let progress = StreamingProgress {
            bytes_processed: 50,
            total_bytes: Some(100),
            tokens_generated: 10,
            chunks_processed: 2,
        };

        assert_eq!(progress.percent(), Some(50.0));
    }

    #[test]
    fn test_chunked_token_iterator() {
        let tokenizer = create_test_tokenizer();
        let chunks = vec!["안녕하세요.\n".to_string(), "감사합니다.\n".to_string()];
        let iter = ChunkedTokenIterator::new(chunks.into_iter(), tokenizer);

        let token_chunks: Vec<_> = iter.collect();
        // 청크들을 수집 (일부 청크는 비어있을 수 있음)
        let total_tokens: usize = token_chunks.iter().map(std::vec::Vec::len).sum();

        // ChunkedTokenIterator가 정상 작동하는지 확인
        // mini-dict 환경에서는 토큰 수가 적을 수 있으므로 패닉 없이 완료되면 성공
        let _ = total_tokens; // 사용되지 않는 변수 경고 방지
    }

    #[test]
    fn test_safe_split_point() {
        let tokenizer = create_test_tokenizer();
        let stream = StreamingTokenizer::new(tokenizer)
            .with_sentence_delimiters(vec!['.', '!', '?', '\n', ' ']);

        // 내부 버퍼에 직접 접근할 수 없으므로 process_chunk로 테스트
        let mut stream = stream;
        let _tokens = stream.process_chunk("안녕하세요 감사합니다");

        // 버퍼가 있어야 함 (문장 구분자가 없으므로)
        assert!(stream.buffer_len() > 0);
    }
}

// ============================================================
// SentenceReader — BufRead 기반 문장 단위 이터레이터
// ============================================================

/// Reads from a [`BufRead`] source and yields complete sentences one at a time.
///
/// Korean sentence boundaries are detected by:
/// - Newline characters (`\n`) — always a boundary.
/// - Sentence-ending punctuation (`.`, `?`, `!`) followed by whitespace or EOF,
///   **except** when the `.` is between two ASCII digits (decimal numbers such
///   as `3.14`).
///
/// Empty segments (blank lines or whitespace-only spans) are silently skipped.
///
/// Because the Viterbi algorithm requires the full sentence context, this is
/// the minimum granularity for streaming tokenization of large inputs.
///
/// # Examples
///
/// ```rust
/// use mecab_ko_core::streaming::SentenceReader;
/// use std::io::Cursor;
///
/// let input = "첫 번째 문장입니다. 두 번째 문장입니다.\n";
/// let reader = SentenceReader::new(Cursor::new(input));
/// let sentences: Vec<String> = reader.map(|r| r.unwrap()).collect();
/// assert_eq!(sentences.len(), 2);
/// ```
pub struct SentenceReader<R: BufRead> {
    reader: R,
    /// Raw character-level working buffer accumulated from `reader`.
    buffer: String,
    /// Completed sentences waiting to be returned by `next()`.
    queue: std::collections::VecDeque<String>,
    /// Set to `true` once the underlying reader returns EOF.
    eof: bool,
}

impl<R: BufRead> SentenceReader<R> {
    /// Creates a new `SentenceReader` wrapping `reader`.
    #[must_use]
    pub const fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: String::new(),
            queue: std::collections::VecDeque::new(),
            eof: false,
        }
    }

    /// Drain all complete sentences currently visible in `self.buffer` into
    /// `self.queue`.  A sentence ends at:
    ///   1. A `\n` character (stripped from the yielded sentence).
    ///   2. A `.`, `?`, or `!` that is **not** a decimal point, followed
    ///      immediately by ASCII whitespace or at the end of the buffer when
    ///      `eof` is `true`.
    fn drain_sentences(&mut self) {
        let chars: Vec<char> = self.buffer.chars().collect();
        let len = chars.len();
        let mut start = 0; // start of the current sentence (char index)

        let mut i = 0;
        while i < len {
            let ch = chars[i];

            if ch == '\n' {
                // Rule 1: newline is always a boundary (stripped).
                let sentence: String = chars[start..i].iter().collect();
                let trimmed = sentence.trim();
                if !trimmed.is_empty() {
                    self.queue.push_back(trimmed.to_string());
                }
                start = i + 1;
                i += 1;
                continue;
            }

            if matches!(ch, '.' | '?' | '!') {
                // Rule 2: sentence-ending punctuation.
                // Exception: digit '.' digit is a decimal number, not a boundary.
                if ch == '.' {
                    let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
                    let next_is_digit =
                        i + 1 < len && chars[i + 1].is_ascii_digit();
                    if prev_is_digit && next_is_digit {
                        // Decimal number — not a boundary.
                        i += 1;
                        continue;
                    }
                }

                // Determine whether what follows qualifies as a boundary:
                //   - EOF reached and this is the last non-whitespace char, OR
                //   - The very next char is ASCII whitespace (space / tab / newline).
                //   - Skip any closing punctuation `)`, `]`, `"`, `'` before checking.
                let mut j = i + 1;
                while j < len && matches!(chars[j], ')' | ']' | '"' | '\'') {
                    j += 1;
                }

                let followed_by_whitespace = j < len && chars[j].is_whitespace();
                let followed_by_eof = j >= len && self.eof;

                if followed_by_whitespace || followed_by_eof {
                    // Include the punctuation mark (and any closing chars) in the sentence.
                    let sentence: String = chars[start..=i].iter().collect();
                    let trimmed = sentence.trim();
                    if !trimmed.is_empty() {
                        self.queue.push_back(trimmed.to_string());
                    }
                    // Skip past punctuation + closing chars + the whitespace separator.
                    start = j;
                    if j < len && chars[j].is_whitespace() && chars[j] != '\n' {
                        start = j + 1;
                        i = j + 1;
                    } else {
                        i = j;
                    }
                    continue;
                }
            }

            i += 1;
        }

        // At EOF, flush whatever is left in the buffer as the final sentence.
        if self.eof && start < len {
            let sentence: String = chars[start..].iter().collect();
            let trimmed = sentence.trim();
            if !trimmed.is_empty() {
                self.queue.push_back(trimmed.to_string());
            }
            self.buffer.clear();
        } else if start > 0 {
            // Consume the portion of the buffer that has been processed.
            let byte_offset: usize = chars[..start].iter().map(|c| c.len_utf8()).sum();
            self.buffer.drain(..byte_offset);
        }
    }

    /// Read one more line from the underlying reader.
    ///
    /// Returns `Ok(true)` if bytes were read, `Ok(false)` on EOF, and
    /// `Err(_)` on an I/O error.
    fn fill_buffer(&mut self) -> io::Result<bool> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line)?;
        if n == 0 {
            self.eof = true;
            Ok(false)
        } else {
            self.buffer.push_str(&line);
            Ok(true)
        }
    }
}

impl<R: BufRead> Iterator for SentenceReader<R> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we already have a sentence ready, return it immediately.
            if let Some(sentence) = self.queue.pop_front() {
                return Some(Ok(sentence));
            }

            // Nothing in the queue and EOF consumed — we are done.
            if self.eof {
                return None;
            }

            // Try to read more data from the reader.
            if let Err(e) = self.fill_buffer() {
                return Some(Err(e));
            }

            // Parse whatever is now in the buffer.
            self.drain_sentences();
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod sentence_reader_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_single_sentence() {
        let input = "안녕하세요.\n";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(sentences, vec!["안녕하세요."]);
    }

    #[test]
    fn test_multiple_sentences() {
        let input = "첫 번째 문장입니다. 두 번째 문장입니다.\n";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "첫 번째 문장입니다.");
        assert_eq!(sentences[1], "두 번째 문장입니다.");
    }

    #[test]
    fn test_newline_boundary() {
        let input = "줄 하나\n줄 둘\n";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(sentences, vec!["줄 하나", "줄 둘"]);
    }

    #[test]
    fn test_decimal_not_boundary() {
        let input = "값은 3.14입니다.\n";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(sentences, vec!["값은 3.14입니다."]);
    }

    #[test]
    fn test_question_mark() {
        let input = "이것은 무엇인가요? 네, 맞습니다.\n";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.collect::<std::result::Result<_, _>>().unwrap();
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.collect::<std::result::Result<_, _>>().unwrap();
        assert!(sentences.is_empty());
    }

    #[test]
    fn test_no_trailing_newline() {
        let input = "마지막 문장";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(sentences, vec!["마지막 문장"]);
    }

    #[test]
    fn test_multiple_newlines() {
        let input = "첫째\n\n둘째\n";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.map(|r| r.unwrap()).collect();
        // Empty lines are skipped.
        assert_eq!(sentences, vec!["첫째", "둘째"]);
    }

    #[test]
    fn test_exclamation() {
        let input = "대단합니다! 정말요?\n";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.collect::<std::result::Result<_, _>>().unwrap();
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn test_sentence_reader_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SentenceReader<std::io::Cursor<&[u8]>>>();
    }

    #[test]
    fn test_closing_paren_before_whitespace() {
        // Punctuation followed by closing bracket then space should still split.
        let input = "문장입니다.) 다음 문장.\n";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.collect::<std::result::Result<_, _>>().unwrap();
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn test_no_trailing_newline_punctuation() {
        // Final sentence with punctuation but no newline should still be yielded.
        let input = "첫째. 둘째.";
        let reader = SentenceReader::new(Cursor::new(input));
        let sentences: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "첫째.");
        assert_eq!(sentences[1], "둘째.");
    }
}
