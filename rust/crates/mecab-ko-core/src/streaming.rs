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
//! ```rust,ignore
//! use mecab_ko_core::streaming::StreamingTokenizer;
//!
//! let mut stream = StreamingTokenizer::new(tokenizer);
//!
//! // 청크 단위로 처리
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

use std::io::{BufRead, BufReader, Read};

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
    /// ```rust,ignore
    /// let tokenizer = Tokenizer::new()?;
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

    /// 부분 버퍼 강제 flush (문장 경계가 없을 때)
    fn force_flush_partial(&mut self) -> Vec<Token> {
        // 절반까지만 처리
        let split_pos = self.buffer.len() / 2;
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
    pub fn total_chars_processed(&self) -> usize {
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
pub struct TokenStream<I>
where
    I: Iterator<Item = String>,
{
    /// 청크 iterator
    chunks: I,

    /// 스트리밍 토크나이저
    streaming: StreamingTokenizer,

    /// 현재 처리 중인 토큰 버퍼
    token_buffer: Vec<Token>,

    /// 스트림 종료 여부
    finished: bool,
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
            token_buffer: Vec::new(),
            finished: false,
        }
    }

    /// 청크 크기 설정
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.streaming = self.streaming.with_chunk_size(size);
        self
    }
}

impl<I> Iterator for TokenStream<I>
where
    I: Iterator<Item = String>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // 버퍼에서 토큰 반환
        if !self.token_buffer.is_empty() {
            return Some(self.token_buffer.remove(0));
        }

        // 스트림이 끝났으면 None
        if self.finished {
            return None;
        }

        // 다음 청크 처리
        while let Some(chunk) = self.chunks.next() {
            let tokens = self.streaming.process_chunk(&chunk);

            if !tokens.is_empty() {
                self.token_buffer.extend(tokens);
                return Some(self.token_buffer.remove(0));
            }
        }

        // 청크가 더 이상 없으면 flush
        self.finished = true;
        let remaining = self.streaming.flush();

        if !remaining.is_empty() {
            self.token_buffer.extend(remaining);
            return Some(self.token_buffer.remove(0));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tokenizer() -> Tokenizer {
        Tokenizer::new().expect("should create tokenizer")
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_streaming_tokenizer_creation() {
        let tokenizer = create_test_tokenizer();
        let stream = StreamingTokenizer::new(tokenizer);

        assert_eq!(stream.buffer_len(), 0);
        assert_eq!(stream.total_chars_processed(), 0);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_process_chunk_with_delimiter() {
        let tokenizer = create_test_tokenizer();
        let mut stream = StreamingTokenizer::new(tokenizer);

        let tokens = stream.process_chunk("안녕하세요.\n");
        assert!(!tokens.is_empty() || stream.buffer_len() > 0);

        // Flush로 남은 토큰 확인
        let remaining = stream.flush();
        let total_tokens = tokens.len() + remaining.len();
        assert!(total_tokens > 0);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_process_chunk_without_delimiter() {
        let tokenizer = create_test_tokenizer();
        let mut stream = StreamingTokenizer::new(tokenizer);

        let tokens = stream.process_chunk("안녕하세요");
        // 구분자가 없으면 버퍼에 저장
        assert!(tokens.is_empty() || stream.buffer_len() > 0);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_flush() {
        let tokenizer = create_test_tokenizer();
        let mut stream = StreamingTokenizer::new(tokenizer);

        stream.process_chunk("안녕하세요");
        let tokens = stream.flush();

        assert!(!tokens.is_empty());
        assert_eq!(stream.buffer_len(), 0);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_multiple_chunks() {
        let tokenizer = create_test_tokenizer();
        let mut stream = StreamingTokenizer::new(tokenizer);

        let _tokens1 = stream.process_chunk("안녕하세요.\n");
        let _tokens2 = stream.process_chunk("감사합니다.\n");
        let _remaining = stream.flush();

        assert!(stream.total_chars_processed() > 0);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_reset() {
        let tokenizer = create_test_tokenizer();
        let mut stream = StreamingTokenizer::new(tokenizer);

        stream.process_chunk("안녕하세요");
        stream.reset();

        assert_eq!(stream.buffer_len(), 0);
        assert_eq!(stream.total_chars_processed(), 0);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_custom_chunk_size() {
        let tokenizer = create_test_tokenizer();
        let stream = StreamingTokenizer::new(tokenizer).with_chunk_size(1024);

        assert_eq!(stream.chunk_size, 1024);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_custom_delimiters() {
        let tokenizer = create_test_tokenizer();
        let stream =
            StreamingTokenizer::new(tokenizer).with_sentence_delimiters(vec!['.', '!', '?']);

        assert_eq!(stream.sentence_delimiters.len(), 3);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_token_stream_creation() {
        let tokenizer = create_test_tokenizer();
        let chunks = vec!["안녕하세요.\n".to_string(), "감사합니다.\n".to_string()];
        let stream = TokenStream::new(chunks.into_iter(), tokenizer);

        assert!(!stream.finished);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_token_stream_iteration() {
        let tokenizer = create_test_tokenizer();
        let chunks = vec!["안녕하세요.\n".to_string(), "감사합니다.\n".to_string()];
        let stream = TokenStream::new(chunks.into_iter(), tokenizer);

        let tokens: Vec<_> = stream.collect();
        assert!(!tokens.is_empty());
    }
}
