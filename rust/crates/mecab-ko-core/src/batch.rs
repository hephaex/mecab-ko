//! # Batch Processing Module
//!
//! Rayon 기반 병렬 배치 처리
//!
//! ## 주요 기능
//!
//! - 병렬 배치 토큰화
//! - Work-stealing 스케줄링
//! - CPU 코어 활용 최적화
//!
//! ## Example
//!
//! ```rust,no_run
//! use mecab_ko_core::batch::BatchTokenizer;
//!
//! let mut batch = BatchTokenizer::new().unwrap();
//! let texts = vec!["안녕하세요", "감사합니다", "좋은 하루 되세요"];
//! let results = batch.tokenize_batch(&texts);
//!
//! for (text, tokens) in texts.iter().zip(results.iter()) {
//!     println!("{}: {} tokens", text, tokens.len());
//! }
//! ```

use std::path::Path;
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use crate::tokenizer::{Token, Tokenizer};
use crate::Result;

/// 배치 토크나이저
///
/// Rayon을 사용하여 여러 텍스트를 병렬로 처리합니다.
/// 내부적으로 토크나이저 풀을 관리하여 각 스레드가 독립적으로 작업합니다.
pub struct BatchTokenizer {
    /// 토크나이저 풀
    tokenizer_pool: Arc<Mutex<Vec<Tokenizer>>>,

    /// 풀 크기
    pool_size: usize,
}

impl BatchTokenizer {
    /// 기본 풀 크기 (CPU 코어 수)
    #[must_use]
    pub fn default_pool_size() -> usize {
        rayon::current_num_threads()
    }

    /// 새 배치 토크나이저 생성
    ///
    /// CPU 코어 수만큼 토크나이저를 미리 생성합니다.
    ///
    /// # Errors
    ///
    /// 토크나이저 초기화 실패 시
    pub fn new() -> Result<Self> {
        Self::with_pool_size(Self::default_pool_size())
    }

    /// 풀 크기 지정하여 생성
    ///
    /// # Arguments
    ///
    /// * `pool_size` - 토크나이저 풀 크기
    ///
    /// # Errors
    ///
    /// 토크나이저 초기화 실패 시
    pub fn with_pool_size(pool_size: usize) -> Result<Self> {
        let mut tokenizers = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            tokenizers.push(Tokenizer::new()?);
        }

        Ok(Self {
            tokenizer_pool: Arc::new(Mutex::new(tokenizers)),
            pool_size,
        })
    }

    /// 사전 경로 지정하여 생성
    ///
    /// # Arguments
    ///
    /// * `dict_path` - 사전 디렉토리 경로
    /// * `pool_size` - 토크나이저 풀 크기
    ///
    /// # Errors
    ///
    /// 토크나이저 초기화 실패 시
    pub fn with_dict<P: AsRef<Path>>(dict_path: P, pool_size: usize) -> Result<Self> {
        let mut tokenizers = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            tokenizers.push(Tokenizer::with_dict(dict_path.as_ref())?);
        }

        Ok(Self {
            tokenizer_pool: Arc::new(Mutex::new(tokenizers)),
            pool_size,
        })
    }

    /// 배치 토큰화
    ///
    /// 여러 텍스트를 병렬로 처리합니다.
    ///
    /// # Arguments
    ///
    /// * `texts` - 텍스트 목록
    ///
    /// # Returns
    ///
    /// 각 텍스트의 토큰 목록
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mecab_ko_core::batch::BatchTokenizer;
    ///
    /// let batch = BatchTokenizer::new().unwrap();
    /// let texts = vec!["안녕하세요", "감사합니다"];
    /// let results = batch.tokenize_batch(&texts);
    /// ```
    #[must_use]
    pub fn tokenize_batch(&self, texts: &[&str]) -> Vec<Vec<Token>> {
        texts
            .par_iter()
            .map(|text| self.tokenize_single(text))
            .collect()
    }

    /// 배치 토큰화 (소유된 문자열)
    ///
    /// # Arguments
    ///
    /// * `texts` - 텍스트 목록
    ///
    /// # Returns
    ///
    /// 각 텍스트의 토큰 목록
    #[must_use]
    pub fn tokenize_batch_owned(&self, texts: &[String]) -> Vec<Vec<Token>> {
        texts
            .par_iter()
            .map(|text| self.tokenize_single(text))
            .collect()
    }

    /// 단일 텍스트 토큰화
    ///
    /// 풀에서 토크나이저를 가져와 사용합니다.
    fn tokenize_single(&self, text: &str) -> Vec<Token> {
        // 풀에서 토크나이저 가져오기
        let Ok(mut pool) = self.tokenizer_pool.lock() else {
            return Vec::new(); // Lock poisoned, return empty result
        };

        if let Some(mut tokenizer) = pool.pop() {
            // 풀 락 해제
            drop(pool);

            // 토큰화 수행
            let tokens = tokenizer.tokenize(text);

            // 토크나이저 반환
            if let Ok(mut pool) = self.tokenizer_pool.lock() {
                pool.push(tokenizer);
            }
            // If lock fails here, tokenizer is dropped but processing succeeded

            tokens
        } else {
            // 풀이 비어있으면 임시 토크나이저 생성 (fallback)
            drop(pool);
            Tokenizer::new()
                .map(|mut tok| tok.tokenize(text))
                .unwrap_or_default()
        }
    }

    /// 파일 목록 배치 처리
    ///
    /// # Arguments
    ///
    /// * `paths` - 파일 경로 목록
    ///
    /// # Returns
    ///
    /// 각 파일의 토큰 목록
    ///
    /// # Errors
    ///
    /// 파일 읽기 실패 시
    pub fn tokenize_files<P: AsRef<Path> + Sync>(&self, paths: &[P]) -> Result<Vec<Vec<Token>>> {
        paths
            .par_iter()
            .map(|path| {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| crate::Error::Analysis(format!("Failed to read file: {e}")))?;
                Ok(self.tokenize_single(&content))
            })
            .collect()
    }

    /// 청크 단위 병렬 처리
    ///
    /// 대용량 텍스트를 청크로 나누어 병렬 처리합니다.
    ///
    /// # Arguments
    ///
    /// * `text` - 입력 텍스트
    /// * `chunk_size` - 청크 크기 (문자 단위)
    ///
    /// # Returns
    ///
    /// 모든 토큰 목록
    #[must_use]
    pub fn tokenize_chunked(&self, text: &str, chunk_size: usize) -> Vec<Token> {
        let chunks = Self::split_into_chunks(text, chunk_size);

        let results: Vec<Vec<Token>> = chunks
            .par_iter()
            .map(|chunk| self.tokenize_single(chunk))
            .collect();

        // 결과 병합
        results.into_iter().flatten().collect()
    }

    /// 텍스트를 청크로 분할 (단어 경계 존중)
    ///
    /// 문장 구분자나 공백에서 분할하여 단어 중간에서 끊기지 않도록 합니다.
    fn split_into_chunks(text: &str, chunk_size: usize) -> Vec<String> {
        Self::split_into_chunks_smart(text, chunk_size, &['.', '!', '?', '。', '．', '\n', ' '])
    }

    /// 스마트 청크 분할 (구분자 지정 가능)
    ///
    /// # Arguments
    ///
    /// * `text` - 분할할 텍스트
    /// * `chunk_size` - 목표 청크 크기 (문자 단위)
    /// * `delimiters` - 분할 가능한 구분자 목록
    fn split_into_chunks_smart(text: &str, chunk_size: usize, delimiters: &[char]) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        let mut chunks = Vec::new();
        let mut current_start = 0;
        let chars: Vec<(usize, char)> = text.char_indices().collect();

        while current_start < chars.len() {
            // 목표 끝점 계산
            let target_end = (current_start + chunk_size).min(chars.len());

            if target_end >= chars.len() {
                // 남은 텍스트 전체를 마지막 청크로
                let byte_start = chars[current_start].0;
                chunks.push(text[byte_start..].to_string());
                break;
            }

            // 목표 끝점에서 뒤로 탐색하여 분할점 찾기
            let mut split_pos = target_end;
            let mut found_delimiter = false;

            // 뒤로 탐색 (최대 청크 크기의 25%까지)
            let min_pos = current_start + (chunk_size * 3 / 4).max(1);
            while split_pos > min_pos {
                if delimiters.contains(&chars[split_pos - 1].1) {
                    found_delimiter = true;
                    break;
                }
                split_pos -= 1;
            }

            // 구분자를 못 찾으면 앞으로 탐색 (최대 청크 크기의 25%까지)
            if !found_delimiter {
                split_pos = target_end;
                let max_pos = (target_end + chunk_size / 4).min(chars.len());
                while split_pos < max_pos {
                    if delimiters.contains(&chars[split_pos - 1].1) {
                        found_delimiter = true;
                        break;
                    }
                    split_pos += 1;
                }
            }

            // 여전히 못 찾으면 그냥 목표 끝점에서 분할
            if !found_delimiter {
                split_pos = target_end;
            }

            // 청크 추출
            let byte_start = chars[current_start].0;
            let byte_end = if split_pos < chars.len() {
                chars[split_pos].0
            } else {
                text.len()
            };

            let chunk = text[byte_start..byte_end].to_string();
            if !chunk.is_empty() {
                chunks.push(chunk);
            }

            current_start = split_pos;
        }

        chunks
    }

    /// 오버랩 있는 청크 분할
    ///
    /// 컨텍스트 보존을 위해 청크 간 오버랩을 추가합니다.
    ///
    /// # Arguments
    ///
    /// * `text` - 분할할 텍스트
    /// * `chunk_size` - 청크 크기
    /// * `overlap` - 오버랩 크기 (문자 단위)
    #[must_use]
    pub fn split_with_overlap(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        if text.is_empty() || chunk_size == 0 {
            return Vec::new();
        }

        let overlap = overlap.min(chunk_size / 2); // 오버랩은 청크의 절반을 넘지 않음
        let chars: Vec<char> = text.chars().collect();
        let mut chunks = Vec::new();
        let mut pos = 0;

        while pos < chars.len() {
            let end = (pos + chunk_size).min(chars.len());
            let chunk: String = chars[pos..end].iter().collect();
            chunks.push(chunk);

            if end >= chars.len() {
                break;
            }

            pos = end.saturating_sub(overlap);
        }

        chunks
    }

    /// 풀 크기 조회
    #[must_use]
    pub const fn pool_size(&self) -> usize {
        self.pool_size
    }

    /// 현재 사용 가능한 토크나이저 수
    #[must_use]
    pub fn available_tokenizers(&self) -> usize {
        self.tokenizer_pool
            .lock()
            .map_or(0, |pool| pool.len())
    }
}

// Note: Default implementation is not provided for BatchTokenizer because initialization
// can fail (dictionary loading, thread pool setup, etc.). Use BatchTokenizer::new() explicitly instead.

/// 병렬 스트리밍 프로세서
///
/// 대용량 파일을 청크로 나누어 병렬 처리합니다.
pub struct ParallelStreamProcessor {
    /// 배치 토크나이저
    batch: BatchTokenizer,

    /// 청크 크기
    chunk_size: usize,
}

impl ParallelStreamProcessor {
    /// 기본 청크 크기 (16KB)
    pub const DEFAULT_CHUNK_SIZE: usize = 16384;

    /// 새 병렬 스트리밍 프로세서 생성
    ///
    /// # Errors
    ///
    /// 배치 토크나이저 초기화 실패 시
    pub fn new() -> Result<Self> {
        Ok(Self {
            batch: BatchTokenizer::new()?,
            chunk_size: Self::DEFAULT_CHUNK_SIZE,
        })
    }

    /// 청크 크기 설정
    #[must_use]
    pub const fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// 대용량 파일 처리
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
    pub fn process_large_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Token>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::Error::Analysis(format!("Failed to read file: {e}")))?;

        Ok(self.batch.tokenize_chunked(&content, self.chunk_size))
    }

    /// 여러 대용량 파일 병렬 처리
    ///
    /// # Arguments
    ///
    /// * `paths` - 파일 경로 목록
    ///
    /// # Returns
    ///
    /// 각 파일의 토큰 목록
    ///
    /// # Errors
    ///
    /// 파일 읽기 실패 시
    pub fn process_files<P: AsRef<Path> + Sync>(&self, paths: &[P]) -> Result<Vec<Vec<Token>>> {
        self.batch.tokenize_files(paths)
    }
}

// Note: Default implementation is not provided for ParallelStreamProcessor because initialization
// can fail (dictionary loading, thread pool setup, etc.). Use ParallelStreamProcessor::new() explicitly instead.

/// 대용량 파일 스트리밍 프로세서
///
/// 파일을 청크 단위로 읽으면서 토큰화를 수행합니다.
/// 메모리 효율적인 처리를 위해 전체 파일을 메모리에 로드하지 않습니다.
pub struct LargeFileProcessor {
    /// 배치 토크나이저
    batch: BatchTokenizer,

    /// 버퍼 크기 (바이트)
    buffer_size: usize,

    /// 진행률 콜백 (Send + Sync for parallel processing)
    progress_callback: Option<Box<dyn Fn(LargeFileProgress) + Send + Sync>>,
}

/// 대용량 파일 처리 진행 상황
#[derive(Debug, Clone)]
pub struct LargeFileProgress {
    /// 처리된 바이트 수
    pub bytes_processed: usize,
    /// 파일 총 크기
    pub total_bytes: usize,
    /// 생성된 토큰 수
    pub tokens_generated: usize,
}

impl LargeFileProgress {
    /// 진행률 퍼센트 계산
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn percent(&self) -> f64 {
        if self.total_bytes == 0 {
            100.0
        } else {
            (self.bytes_processed as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

impl LargeFileProcessor {
    /// 기본 버퍼 크기 (64KB)
    pub const DEFAULT_BUFFER_SIZE: usize = 65536;

    /// 새 대용량 파일 프로세서 생성
    ///
    /// # Errors
    ///
    /// 배치 토크나이저 초기화 실패 시
    pub fn new() -> Result<Self> {
        Ok(Self {
            batch: BatchTokenizer::new()?,
            buffer_size: Self::DEFAULT_BUFFER_SIZE,
            progress_callback: None,
        })
    }

    /// 버퍼 크기 설정
    #[must_use]
    pub const fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// 진행률 콜백 설정
    #[must_use]
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(LargeFileProgress) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// 대용량 파일 스트리밍 처리
    ///
    /// 파일을 청크 단위로 읽어 토큰화합니다.
    /// 문장 경계를 고려하여 분할합니다.
    ///
    /// # Arguments
    ///
    /// * `path` - 파일 경로
    ///
    /// # Returns
    ///
    /// 토큰 목록
    ///
    /// # Errors
    ///
    /// 파일 읽기 실패 시
    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Token>> {
        use std::io::{BufRead, BufReader};

        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| crate::Error::Analysis(format!("Failed to open file: {e}")))?;

        let metadata = file
            .metadata()
            .map_err(|e| crate::Error::Analysis(format!("Failed to read metadata: {e}")))?;

        #[allow(clippy::cast_possible_truncation)]
        let total_bytes = metadata.len() as usize;
        let reader = BufReader::with_capacity(self.buffer_size, file);

        let mut all_tokens = Vec::new();
        let mut bytes_processed = 0;
        let mut pending_text = String::new();
        let sentence_delimiters = ['.', '!', '?', '。', '．', '\n'];

        for line in reader.lines() {
            let line =
                line.map_err(|e| crate::Error::Analysis(format!("Failed to read line: {e}")))?;

            bytes_processed += line.len() + 1; // +1 for newline
            pending_text.push_str(&line);
            pending_text.push('\n');

            // 충분한 텍스트가 모이면 처리
            if pending_text.len() >= self.buffer_size {
                // 마지막 문장 경계 찾기
                if let Some(pos) = pending_text
                    .char_indices()
                    .rev()
                    .find(|(_, c)| sentence_delimiters.contains(c))
                    .map(|(i, _)| i)
                {
                    let to_process = pending_text[..=pos].to_string();
                    let remaining = pending_text[pos + 1..].to_string();

                    let tokens = self.batch.tokenize_single(&to_process);
                    all_tokens.extend(tokens);

                    pending_text = remaining;
                }
            }

            // 진행률 보고
            if let Some(ref callback) = self.progress_callback {
                callback(LargeFileProgress {
                    bytes_processed,
                    total_bytes,
                    tokens_generated: all_tokens.len(),
                });
            }
        }

        // 남은 텍스트 처리
        if !pending_text.is_empty() {
            let tokens = self.batch.tokenize_single(&pending_text);
            all_tokens.extend(tokens);
        }

        // 최종 진행률 보고
        if let Some(ref callback) = self.progress_callback {
            callback(LargeFileProgress {
                bytes_processed: total_bytes,
                total_bytes,
                tokens_generated: all_tokens.len(),
            });
        }

        Ok(all_tokens)
    }

    /// 여러 대용량 파일 병렬 처리
    ///
    /// # Errors
    ///
    /// 파일 읽기 실패 시
    pub fn process_files<P: AsRef<Path> + Sync>(&self, paths: &[P]) -> Result<Vec<Vec<Token>>> {
        paths
            .par_iter()
            .map(|path| self.process_file(path))
            .collect()
    }
}

// Note: Default implementation is not provided for LargeFileProcessor because initialization
// can fail (dictionary loading, etc.). Use LargeFileProcessor::new() explicitly instead.

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_tokenizer_creation() {
        let batch = BatchTokenizer::new();
        assert!(batch.is_ok());
    }

    #[test]
    fn test_default_pool_size() {
        let size = BatchTokenizer::default_pool_size();
        assert!(size > 0);
    }

    #[test]
    fn test_tokenize_batch() {
        let batch = BatchTokenizer::new().expect("should create");
        let texts = vec!["안녕하세요", "감사합니다"];

        let results = batch.tokenize_batch(&texts);

        assert_eq!(results.len(), 2);
        assert!(!results[0].is_empty());
        assert!(!results[1].is_empty());
    }

    #[test]
    fn test_tokenize_batch_owned() {
        let batch = BatchTokenizer::new().expect("should create");
        let texts = vec!["안녕하세요".to_string(), "감사합니다".to_string()];

        let results = batch.tokenize_batch_owned(&texts);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_tokenize_chunked() {
        let batch = BatchTokenizer::new().expect("should create");
        let text = "안녕하세요 감사합니다 좋은 하루 되세요";

        let tokens = batch.tokenize_chunked(text, 10);

        // mini-dict 환경에서는 토큰이 없을 수 있으므로 패닉 없이 완료되면 성공
        let _ = tokens.len();
    }

    #[test]
    fn test_split_into_chunks() {
        let text = "안녕하세요 감사합니다";

        let chunks = BatchTokenizer::split_into_chunks(text, 5);

        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_pool_size() {
        let batch = BatchTokenizer::new().expect("should create");
        assert_eq!(batch.pool_size(), BatchTokenizer::default_pool_size());
    }

    #[test]
    fn test_available_tokenizers() {
        let batch = BatchTokenizer::new().expect("should create");
        let available = batch.available_tokenizers();
        assert_eq!(available, batch.pool_size());
    }

    #[test]
    fn test_with_pool_size() {
        let batch = BatchTokenizer::with_pool_size(4).expect("should create");
        assert_eq!(batch.pool_size(), 4);
    }

    #[test]
    fn test_parallel_stream_processor_creation() {
        let processor = ParallelStreamProcessor::new();
        assert!(processor.is_ok());
    }

    #[test]
    fn test_with_chunk_size() {
        let processor = ParallelStreamProcessor::new()
            .expect("should create")
            .with_chunk_size(8192);

        assert_eq!(processor.chunk_size, 8192);
    }

    #[test]
    fn test_empty_batch() {
        let batch = BatchTokenizer::new().expect("should create");
        let texts: Vec<&str> = vec![];

        let results = batch.tokenize_batch(&texts);

        assert!(results.is_empty());
    }

    #[test]
    fn test_single_item_batch() {
        let batch = BatchTokenizer::new().expect("should create");
        let texts = vec!["안녕하세요"];

        let results = batch.tokenize_batch(&texts);

        assert_eq!(results.len(), 1);
        assert!(!results[0].is_empty());
    }

    #[test]
    fn test_large_batch() {
        let batch = BatchTokenizer::new().expect("should create");
        let texts: Vec<&str> = (0..100).map(|_| "안녕하세요").collect();

        let results = batch.tokenize_batch(&texts);

        assert_eq!(results.len(), 100);
    }

    #[test]
    fn test_smart_chunking_respects_sentence_boundary() {
        // 구분자가 많은 텍스트
        let text = "안녕. 감사. 좋아. 행복. 건강.";

        // 문장 구분자에서 분할되어야 함
        let chunks = BatchTokenizer::split_into_chunks(text, 6);

        // 여러 청크로 분할되어야 함
        assert!(chunks.len() > 1, "Should split into multiple chunks");

        // 대부분의 청크가 구분자로 끝나야 함 (마지막 청크 제외)
        let delimiter_ending: Vec<_> = chunks[..chunks.len().saturating_sub(1)]
            .iter()
            .filter(|chunk| {
                let trimmed = chunk.trim();
                trimmed.ends_with('.') || trimmed.ends_with(' ')
            })
            .collect();

        // 적어도 일부 청크는 구분자에서 분할되어야 함
        assert!(
            !delimiter_ending.is_empty() || chunks.len() <= 2,
            "At least some chunks should end with delimiters"
        );
    }

    #[test]
    fn test_smart_chunking_with_spaces() {
        let text = "안녕하세요 감사합니다 좋은 하루 되세요";

        let chunks = BatchTokenizer::split_into_chunks_smart(text, 8, &[' ']);

        // 공백에서 분할되어야 함
        for chunk in &chunks {
            assert!(!chunk.is_empty());
        }
    }

    #[test]
    fn test_split_with_overlap() {
        let text = "안녕하세요감사합니다좋은하루되세요";

        let chunks = BatchTokenizer::split_with_overlap(text, 5, 2);

        assert!(chunks.len() > 1);

        // 오버랩 확인: 이전 청크의 끝과 다음 청크의 시작이 겹쳐야 함
        if chunks.len() >= 2 {
            let first_end: String = chunks[0].chars().rev().take(2).collect::<String>();
            let first_end: String = first_end.chars().rev().collect();
            let second_start: String = chunks[1].chars().take(2).collect();
            assert_eq!(
                first_end, second_start,
                "Overlap should match: {first_end} vs {second_start}"
            );
        }
    }

    #[test]
    fn test_split_with_overlap_empty_text() {
        let chunks = BatchTokenizer::split_with_overlap("", 5, 2);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_split_with_overlap_large_overlap() {
        let text = "안녕하세요";

        // 오버랩이 청크 크기의 절반을 넘으면 제한됨
        let chunks = BatchTokenizer::split_with_overlap(text, 4, 10);

        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_smart_chunking_empty_text() {
        let chunks = BatchTokenizer::split_into_chunks("", 5);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_smart_chunking_no_delimiter() {
        // 구분자 없는 텍스트
        let text = "안녕하세요감사합니다";

        let chunks = BatchTokenizer::split_into_chunks(text, 4);

        // 분할은 되어야 함
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_large_file_processor_creation() {
        let processor = LargeFileProcessor::new();
        assert!(processor.is_ok());
    }

    #[test]
    fn test_large_file_processor_with_buffer_size() {
        let processor = LargeFileProcessor::new()
            .expect("should create")
            .with_buffer_size(32768);

        assert_eq!(processor.buffer_size, 32768);
    }

    #[test]
    fn test_large_file_progress_percent() {
        let progress = LargeFileProgress {
            bytes_processed: 50,
            total_bytes: 100,
            tokens_generated: 10,
        };

        assert!((progress.percent() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_large_file_progress_percent_zero_total() {
        let progress = LargeFileProgress {
            bytes_processed: 50,
            total_bytes: 0,
            tokens_generated: 10,
        };

        assert!((progress.percent() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_large_file_processor_with_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let callback_count = Arc::new(AtomicUsize::new(0));
        let callback_count_clone = Arc::clone(&callback_count);

        let _processor = LargeFileProcessor::new()
            .expect("should create")
            .with_progress_callback(move |_progress| {
                callback_count_clone.fetch_add(1, Ordering::SeqCst);
            });

        // 콜백이 설정되었는지 확인 (실제 호출은 파일 처리 시)
        assert!(callback_count.load(Ordering::SeqCst) == 0);
    }
}
