//! # mecab-ko-core
//!
//! 한국어 형태소 분석 핵심 엔진
//!
//! ## 주요 기능
//!
//! - Lattice 구축
//! - Viterbi 알고리즘
//! - N-best 경로 탐색
//! - 미등록어 처리
//!
//! ## 예제
//!
//! ```rust,ignore
//! use mecab_ko_core::Tokenizer;
//!
//! let tokenizer = Tokenizer::new()?;
//! let tokens = tokenizer.tokenize("안녕하세요");
//!
//! for token in tokens {
//!     println!("{}: {}", token.surface, token.pos);
//! }
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod error;
pub mod lattice;
pub mod tokenizer;
pub mod viterbi;

pub use error::{Error, Result};
pub use tokenizer::{Token, Tokenizer};

/// 에러 모듈
pub mod error {
    use thiserror::Error;
    
    /// 핵심 엔진 에러 타입
    #[derive(Error, Debug)]
    pub enum Error {
        /// 사전 에러
        #[error("Dictionary error: {0}")]
        Dict(#[from] mecab_ko_dict::error::DictError),
        
        /// 분석 에러
        #[error("Analysis error: {0}")]
        Analysis(String),
        
        /// 초기화 에러
        #[error("Initialization error: {0}")]
        Init(String),
    }
    
    /// Result 타입 별칭
    pub type Result<T> = std::result::Result<T, Error>;
}

/// Lattice 모듈 (스텁)
pub mod lattice {
    //! Lattice 자료구조
    //!
    //! 형태소 분석을 위한 격자(Lattice) 구조를 제공합니다.
    
    /// Lattice 노드
    #[derive(Debug, Clone)]
    pub struct Node {
        /// 표면형
        pub surface: String,
        /// 시작 위치
        pub start: usize,
        /// 끝 위치
        pub end: usize,
        /// 좌문맥 ID
        pub left_id: u16,
        /// 우문맥 ID
        pub right_id: u16,
        /// 비용
        pub cost: i32,
        /// 품사 정보
        pub feature: String,
    }
    
    /// Lattice 구조
    pub struct Lattice {
        /// 입력 텍스트
        pub text: String,
        /// 노드들
        pub nodes: Vec<Vec<Node>>,
    }
    
    impl Lattice {
        /// 새 Lattice 생성
        pub fn new(text: &str) -> Self {
            let len = text.chars().count();
            Self {
                text: text.to_string(),
                nodes: vec![Vec::new(); len + 1],
            }
        }
        
        /// 노드 추가
        pub fn add_node(&mut self, pos: usize, node: Node) {
            if pos < self.nodes.len() {
                self.nodes[pos].push(node);
            }
        }
    }
}

/// Viterbi 모듈 (스텁)
pub mod viterbi {
    //! Viterbi 알고리즘
    //!
    //! 최적 경로 탐색을 위한 Viterbi 알고리즘 구현
    
    use super::lattice::{Lattice, Node};
    
    /// Viterbi 탐색기
    pub struct ViterbiSearcher {
        // TODO: 구현
    }
    
    impl ViterbiSearcher {
        /// 새 탐색기 생성
        pub fn new() -> Self {
            Self {}
        }
        
        /// 최적 경로 탐색
        pub fn search(&self, _lattice: &Lattice) -> Vec<Node> {
            todo!("Viterbi 알고리즘 구현 예정")
        }
        
        /// N-best 경로 탐색
        pub fn search_nbest(&self, _lattice: &Lattice, _n: usize) -> Vec<Vec<Node>> {
            todo!("N-best 탐색 구현 예정")
        }
    }
    
    impl Default for ViterbiSearcher {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// 토크나이저 모듈
pub mod tokenizer {
    //! 토크나이저
    //!
    //! 형태소 분석의 메인 인터페이스
    
    use super::Result;
    
    /// 토큰
    #[derive(Debug, Clone, PartialEq)]
    pub struct Token {
        /// 표면형
        pub surface: String,
        /// 품사 태그
        pub pos: String,
        /// 시작 위치 (바이트)
        pub start: usize,
        /// 끝 위치 (바이트)
        pub end: usize,
        /// 읽기
        pub reading: Option<String>,
        /// 원형
        pub lemma: Option<String>,
    }
    
    /// 토크나이저
    pub struct Tokenizer {
        // TODO: 사전 등 필드 추가
    }
    
    impl Tokenizer {
        /// 기본 사전으로 토크나이저 생성
        pub fn new() -> Result<Self> {
            Ok(Self {})
        }
        
        /// 사전 경로 지정하여 생성
        pub fn with_dict(_dict_path: &str) -> Result<Self> {
            todo!("사전 로딩 구현 예정")
        }
        
        /// 형태소 분석
        pub fn tokenize(&self, text: &str) -> Vec<Token> {
            // TODO: 실제 분석 구현
            // 현재는 더미 구현
            vec![Token {
                surface: text.to_string(),
                pos: "UNK".to_string(),
                start: 0,
                end: text.len(),
                reading: None,
                lemma: None,
            }]
        }
        
        /// 분리만 수행 (wakati)
        pub fn wakati(&self, text: &str) -> Vec<String> {
            self.tokenize(text)
                .into_iter()
                .map(|t| t.surface)
                .collect()
        }
        
        /// 명사만 추출
        pub fn nouns(&self, text: &str) -> Vec<String> {
            self.tokenize(text)
                .into_iter()
                .filter(|t| t.pos.starts_with("NN"))
                .map(|t| t.surface)
                .collect()
        }
        
        /// 형태소만 추출
        pub fn morphs(&self, text: &str) -> Vec<String> {
            self.wakati(text)
        }
        
        /// 품사 태깅
        pub fn pos(&self, text: &str) -> Vec<(String, String)> {
            self.tokenize(text)
                .into_iter()
                .map(|t| (t.surface, t.pos))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenizer_creation() {
        let tokenizer = Tokenizer::new();
        assert!(tokenizer.is_ok());
    }
    
    #[test]
    fn test_basic_tokenize() {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("테스트");
        assert!(!tokens.is_empty());
    }
}
