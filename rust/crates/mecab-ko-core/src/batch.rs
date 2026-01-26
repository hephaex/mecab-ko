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
//! ```rust,ignore
//! use mecab_ko_core::batch::BatchTokenizer;
//!
//! let batch = BatchTokenizer::new()?;
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
    /// ```rust,ignore
    /// let batch = BatchTokenizer::new()?;
    /// let texts = vec!["안녕하세요", "감사합니다"];
    /// let results = batch.tokenize_batch(&texts);
    /// ```
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
        let mut pool = self
            .tokenizer_pool
            .lock()
            .expect("tokenizer pool lock poisoned");

        if let Some(mut tokenizer) = pool.pop() {
            // 풀 락 해제
            drop(pool);

            // 토큰화 수행
            let tokens = tokenizer.tokenize(text);

            // 토크나이저 반환
            self.tokenizer_pool
                .lock()
                .expect("tokenizer pool lock poisoned")
                .push(tokenizer);

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
    pub fn tokenize_chunked(&self, text: &str, chunk_size: usize) -> Vec<Token> {
        let chunks = self.split_into_chunks(text, chunk_size);

        let results: Vec<Vec<Token>> = chunks
            .par_iter()
            .map(|chunk| self.tokenize_single(chunk))
            .collect();

        // 결과 병합
        results.into_iter().flatten().collect()
    }

    /// 텍스트를 청크로 분할
    fn split_into_chunks(&self, text: &str, chunk_size: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current = String::new();

        for ch in text.chars() {
            current.push(ch);

            if current.len() >= chunk_size {
                chunks.push(std::mem::take(&mut current));
            }
        }

        if !current.is_empty() {
            chunks.push(current);
        }

        chunks
    }

    /// 풀 크기 조회
    #[must_use]
    pub fn pool_size(&self) -> usize {
        self.pool_size
    }

    /// 현재 사용 가능한 토크나이저 수
    #[must_use]
    pub fn available_tokenizers(&self) -> usize {
        self.tokenizer_pool
            .lock()
            .map(|pool| pool.len())
            .unwrap_or(0)
    }
}

impl Default for BatchTokenizer {
    fn default() -> Self {
        Self::new().expect("Failed to create default batch tokenizer")
    }
}

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
    pub fn with_chunk_size(mut self, size: usize) -> Self {
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

impl Default for ParallelStreamProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to create default parallel stream processor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
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
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_tokenize_batch() {
        let batch = BatchTokenizer::new().expect("should create");
        let texts = vec!["안녕하세요", "감사합니다"];

        let results = batch.tokenize_batch(&texts);

        assert_eq!(results.len(), 2);
        assert!(!results[0].is_empty());
        assert!(!results[1].is_empty());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_tokenize_batch_owned() {
        let batch = BatchTokenizer::new().expect("should create");
        let texts = vec!["안녕하세요".to_string(), "감사합니다".to_string()];

        let results = batch.tokenize_batch_owned(&texts);

        assert_eq!(results.len(), 2);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_tokenize_chunked() {
        let batch = BatchTokenizer::new().expect("should create");
        let text = "안녕하세요 감사합니다 좋은 하루 되세요";

        let tokens = batch.tokenize_chunked(text, 10);

        assert!(!tokens.is_empty());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_split_into_chunks() {
        let batch = BatchTokenizer::new().expect("should create");
        let text = "안녕하세요 감사합니다";

        let chunks = batch.split_into_chunks(text, 5);

        assert!(chunks.len() > 1);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_pool_size() {
        let batch = BatchTokenizer::new().expect("should create");
        assert_eq!(batch.pool_size(), BatchTokenizer::default_pool_size());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_available_tokenizers() {
        let batch = BatchTokenizer::new().expect("should create");
        let available = batch.available_tokenizers();
        assert_eq!(available, batch.pool_size());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_with_pool_size() {
        let batch = BatchTokenizer::with_pool_size(4).expect("should create");
        assert_eq!(batch.pool_size(), 4);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_parallel_stream_processor_creation() {
        let processor = ParallelStreamProcessor::new();
        assert!(processor.is_ok());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_with_chunk_size() {
        let processor = ParallelStreamProcessor::new()
            .expect("should create")
            .with_chunk_size(8192);

        assert_eq!(processor.chunk_size, 8192);
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_empty_batch() {
        let batch = BatchTokenizer::new().expect("should create");
        let texts: Vec<&str> = vec![];

        let results = batch.tokenize_batch(&texts);

        assert!(results.is_empty());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_single_item_batch() {
        let batch = BatchTokenizer::new().expect("should create");
        let texts = vec!["안녕하세요"];

        let results = batch.tokenize_batch(&texts);

        assert_eq!(results.len(), 1);
        assert!(!results[0].is_empty());
    }

    #[test]
    #[ignore = "requires dictionary - install mecab-ko-dic or set MECAB_DICDIR"]
    fn test_large_batch() {
        let batch = BatchTokenizer::new().expect("should create");
        let texts: Vec<&str> = (0..100).map(|_| "안녕하세요").collect();

        let results = batch.tokenize_batch(&texts);

        assert_eq!(results.len(), 100);
    }
}
