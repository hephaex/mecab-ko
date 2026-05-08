//! # 토크나이저 모듈
//!
//! 형태소 분석의 메인 인터페이스입니다.
//!
//! ## 개요
//!
//! Tokenizer는 다음 컴포넌트들을 통합하여 형태소 분석을 수행합니다:
//! - **Trie**: 사전 검색 (mecab-ko-dict)
//! - **Matrix**: 연접 비용 계산
//! - **Lattice**: 후보 그래프 구축
//! - **Viterbi**: 최적 경로 탐색
//! - `UnknownHandler`: 미등록어 처리
//!
//! ## 분석 과정
//!
//! 1. **입력 텍스트 전처리**: 공백 제거 및 위치 정보 생성
//! 2. **Lattice 구축**: 각 위치에서 사전 검색 및 노드 추가
//! 3. **미등록어 처리**: 사전에 없는 부분에 대해 미등록어 노드 추가
//! 4. **Viterbi 탐색**: 최소 비용 경로 계산
//! 5. **Token 변환**: 최적 경로의 노드를 Token으로 변환
//!
//! ## Example
//!
//! ```rust,no_run
//! use mecab_ko_core::tokenizer::Tokenizer;
//!
//! // 기본 사전으로 초기화
//! let mut tokenizer = Tokenizer::new().unwrap();
//!
//! // 형태소 분석
//! let tokens = tokenizer.tokenize("아버지가방에들어가신다");
//! for token in tokens {
//!     println!("{}: {} ({}~{})", token.surface, token.pos, token.start_pos, token.end_pos);
//! }
//! ```

use std::borrow::Cow;
use std::path::Path;

use mecab_ko_dict::{SystemDictionary, UserDictionary};

use crate::error::Result;
use crate::lattice::{Lattice, Node, NodeBuilder, NodeType};
use crate::normalizer::{NormalizationConfig, Normalizer};
use crate::pool::{PoolManager, PoolStats};
use crate::pos_tag::PosTag;
use crate::unknown::UnknownHandler;
use crate::viterbi::{SpacePenalty, ViterbiSearcher};

/// 토큰
///
/// 형태소 분석 결과의 개별 토큰을 표현합니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// 표면형 (원본 텍스트의 형태)
    pub surface: String,

    /// 품사 태그
    pub pos: String,

    /// 시작 위치 (문자 단위, 0-based)
    pub start_pos: usize,

    /// 끝 위치 (문자 단위, exclusive)
    pub end_pos: usize,

    /// 시작 위치 (바이트 단위)
    pub start_byte: usize,

    /// 끝 위치 (바이트 단위)
    pub end_byte: usize,

    /// 읽기 (발음)
    pub reading: Option<String>,

    /// 원형 (기본형)
    pub lemma: Option<String>,

    /// 비용
    pub cost: i32,

    /// 전체 품사 정보 (CSV feature string)
    pub features: String,

    /// 정규화된 형태 (외래어 정규화 활성화 시)
    pub normalized: Option<String>,
}

impl Token {
    /// 새 토큰 생성
    #[must_use]
    pub const fn new(
        surface: String,
        pos: String,
        start_pos: usize,
        end_pos: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Self {
            surface,
            pos,
            start_pos,
            end_pos,
            start_byte,
            end_byte,
            reading: None,
            lemma: None,
            cost: 0,
            features: String::new(),
            normalized: None,
        }
    }

    /// Lattice 노드에서 토큰 생성
    ///
    /// # Arguments
    ///
    /// * `node` - Lattice 노드
    #[must_use]
    pub fn from_node(node: &Node) -> Self {
        let features = node.feature.to_string();
        let (pos, reading, lemma) = parse_features(&features);

        Self {
            surface: node.surface.to_string(),
            pos: pos.to_string(),
            start_pos: node.start_pos,
            end_pos: node.end_pos,
            start_byte: node.start_byte,
            end_byte: node.end_byte,
            reading,
            lemma,
            cost: node.total_cost,
            features,
            normalized: None,
        }
    }

    /// 토큰 길이 (문자 단위)
    #[inline]
    #[must_use]
    pub const fn char_len(&self) -> usize {
        self.end_pos - self.start_pos
    }

    /// 토큰 길이 (바이트 단위)
    #[inline]
    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.end_byte - self.start_byte
    }

    /// 품사 태그를 `PosTag` 타입으로 파싱
    #[must_use]
    pub fn pos_tag(&self) -> Option<PosTag> {
        self.pos.parse().ok()
    }
}

/// Feature 문자열 파싱
///
/// `MeCab` feature 포맷: `품사,의미분류,종성유무,읽기,타입,첫번째품사,마지막품사,표현`
///
/// # Returns
///
/// (품사, 읽기, 원형)
fn parse_features(features: &str) -> (Cow<'_, str>, Option<String>, Option<String>) {
    // Avoid allocating a Vec – iterate the splits directly.
    let mut split = features.splitn(5, ',');

    let pos = split.next().unwrap_or("*");

    // indices: 0=pos, 1=semantic, 2=jongseong, 3=reading
    let reading = split
        .nth(2) // skip indices 1 and 2, land on index 3
        .filter(|s| !s.is_empty() && *s != "*")
        .map(std::string::ToString::to_string);

    let lemma = reading.clone();

    (Cow::Borrowed(pos), reading, lemma)
}

/// 토크나이저
///
/// 형태소 분석의 메인 인터페이스입니다.
/// 시스템 사전, 사용자 사전, 미등록어 처리기를 통합하여 형태소 분석을 수행합니다.
///
/// # 메모리 최적화
///
/// - `lattice` 재사용으로 매 분석마다 재할당 방지
/// - `pool_manager`로 Token, Node 객체 재사용
/// - String interning으로 중복 문자열 제거
pub struct Tokenizer {
    /// 시스템 사전
    dictionary: SystemDictionary,

    /// 미등록어 처리기
    unknown_handler: UnknownHandler,

    /// Viterbi 탐색기
    viterbi_searcher: ViterbiSearcher,

    /// 재사용 가능한 Lattice (성능 최적화)
    lattice: Lattice,

    /// 외래어 정규화기 (옵션)
    normalizer: Option<Normalizer>,

    /// 정규화 활성화 여부
    enable_normalization: bool,

    /// 메모리 풀 관리자
    pool_manager: PoolManager,
}

impl Tokenizer {
    /// 기본 사전으로 토크나이저 생성
    ///
    /// 환경변수 `MECAB_DICDIR`이나 기본 경로에서 시스템 사전을 로드합니다.
    ///
    /// # Errors
    ///
    /// - 사전을 찾을 수 없는 경우
    /// - 사전 파일 포맷이 잘못된 경우
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mecab_ko_core::tokenizer::Tokenizer;
    ///
    /// let mut tokenizer = Tokenizer::new().unwrap();
    /// let tokens = tokenizer.tokenize("안녕하세요");
    /// ```
    pub fn new() -> Result<Self> {
        let dictionary = SystemDictionary::load_default()?;
        let unknown_handler = UnknownHandler::korean_default();
        let viterbi_searcher = ViterbiSearcher::new();

        // 초기 Lattice 생성 (빈 텍스트)
        let lattice = Lattice::new("");

        Ok(Self {
            dictionary,
            unknown_handler,
            viterbi_searcher,
            lattice,
            normalizer: None,
            enable_normalization: false,
            pool_manager: PoolManager::new(),
        })
    }

    /// 사전 경로를 지정하여 토크나이저 생성
    ///
    /// # Arguments
    ///
    /// * `dict_path` - 사전 디렉토리 경로
    ///
    /// # Errors
    ///
    /// - 사전을 찾을 수 없는 경우
    /// - 사전 파일 포맷이 잘못된 경우
    pub fn with_dict<P: AsRef<Path>>(dict_path: P) -> Result<Self> {
        let dictionary = SystemDictionary::load(dict_path)?;
        let unknown_handler = UnknownHandler::korean_default();
        let viterbi_searcher = ViterbiSearcher::new();

        let lattice = Lattice::new("");

        Ok(Self {
            dictionary,
            unknown_handler,
            viterbi_searcher,
            lattice,
            normalizer: None,
            enable_normalization: false,
            pool_manager: PoolManager::new(),
        })
    }

    /// 사용자 사전 추가
    ///
    /// # Arguments
    ///
    /// * `user_dict` - 사용자 사전
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mecab_ko_core::tokenizer::Tokenizer;
    /// use mecab_ko_dict::UserDictionary;
    ///
    /// let mut user_dict = UserDictionary::new();
    /// user_dict.add_entry("딥러닝", "NNG", Some(-1000), None);
    ///
    /// let tokenizer = Tokenizer::new().unwrap()
    ///     .with_user_dict(user_dict);
    /// ```
    #[must_use]
    pub fn with_user_dict(mut self, user_dict: UserDictionary) -> Self {
        self.dictionary.set_user_dictionary(user_dict);
        self
    }

    /// 사용자 사전 설정 (in-place)
    ///
    /// 이미 생성된 토크나이저에 사용자 사전을 설정합니다.
    /// 빌더 패턴이 필요 없는 경우 사용합니다.
    ///
    /// # Arguments
    ///
    /// * `user_dict` - 사용자 사전
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mecab_ko_core::Tokenizer;
    /// use mecab_ko_dict::UserDictionary;
    ///
    /// let mut tokenizer = Tokenizer::new().unwrap();
    ///
    /// let mut user_dict = UserDictionary::new();
    /// user_dict.add_entry("챗GPT", "NNP", Some(-2000), None);
    /// tokenizer.set_user_dict(user_dict);
    /// ```
    pub fn set_user_dict(&mut self, user_dict: UserDictionary) {
        self.dictionary.set_user_dictionary(user_dict);
    }

    /// Hot-reload v2 사전 설정 (in-place)
    ///
    /// 이미 생성된 토크나이저에 `HotReloadDictV2` 인스턴스를 설정합니다.
    /// 도메인 사전의 동적 리로드를 활성화합니다.
    ///
    /// # Arguments
    ///
    /// * `hr` - `HotReloadDictV2` 인스턴스
    #[cfg(feature = "hot-reload-v2")]
    pub fn set_hot_reload(
        &mut self,
        hr: std::sync::Arc<mecab_ko_dict::hot_reload_v2::HotReloadDictV2>,
    ) {
        self.dictionary.set_hot_reload(hr);
    }

    /// 띄어쓰기 패널티 설정
    ///
    /// # Arguments
    ///
    /// * `penalty` - 띄어쓰기 패널티 설정
    #[must_use]
    pub fn with_space_penalty(mut self, penalty: SpacePenalty) -> Self {
        self.viterbi_searcher = ViterbiSearcher::new().with_space_penalty(penalty);
        self
    }

    /// 형태소 분석
    ///
    /// 입력 텍스트를 형태소 단위로 분석하여 Token 목록을 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트
    ///
    /// # Returns
    ///
    /// 토큰 목록
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use mecab_ko_core::tokenizer::Tokenizer;
    /// # let mut tokenizer = Tokenizer::new().unwrap();
    /// let tokens = tokenizer.tokenize("아버지가방에들어가신다");
    /// for token in tokens {
    ///     println!("{}: {}", token.surface, token.pos);
    /// }
    /// ```
    pub fn tokenize(&mut self, text: &str) -> Vec<Token> {
        if text.is_empty() {
            return Vec::new();
        }

        // Lattice 재설정
        self.lattice.reset(text);

        // Lattice 구축
        self.build_lattice();

        // Viterbi 탐색
        let path = self
            .viterbi_searcher
            .search(&mut self.lattice, self.dictionary.matrix());

        // Token 변환
        path.iter()
            .filter_map(|&node_id| self.lattice.node(node_id))
            .map(Token::from_node)
            .collect()
    }

    /// Lattice 구축
    ///
    /// 입력 텍스트의 각 위치에서 사전 검색 및 미등록어 처리를 수행하여
    /// Lattice에 노드를 추가합니다.
    fn build_lattice(&mut self) {
        let char_len = self.lattice.char_len();

        // 각 문자 위치에서 사전 검색 및 미등록어 처리
        for pos in 0..char_len {
            // 사전 검색
            let has_dict_entry = self.add_dict_nodes(pos);

            // 미등록어 처리
            self.unknown_handler
                .add_unknown_nodes(&mut self.lattice, pos, has_dict_entry);
        }
    }

    /// 사전 노드 추가
    ///
    /// 특정 위치에서 시작하는 모든 사전 엔트리를 Lattice에 추가합니다.
    ///
    /// # Arguments
    ///
    /// * `start_pos` - 시작 위치 (문자 단위)
    ///
    /// # Returns
    ///
    /// 사전 엔트리가 하나라도 있으면 true
    fn add_dict_nodes(&mut self, start_pos: usize) -> bool {
        // Get the byte range for the suffix starting at `start_pos` without
        // allocating a new String.  We collect only the trie-match indices
        // (small integers) before any lattice mutation, so the immutable borrow
        // of `self.lattice` is released before we call `add_node`.
        let char_len = self.lattice.char_len();
        let search_text: &str = self.lattice.substring(start_pos, char_len);

        if search_text.is_empty() {
            return false;
        }

        // Use dictionary.common_prefix_search which returns all entries for
        // the same surface (not just the first one). This is essential for
        // the Viterbi algorithm to consider all possible POS tags and select
        // the best path based on connection costs.
        let dict_entries: Vec<_> = self
            .dictionary
            .common_prefix_search(search_text)
            .unwrap_or_default();

        // Collect user-dict entries as owned data before mutating lattice.
        // user_dict.common_prefix_search returns owned UserEntry values so
        // this is already allocation-minimal; we just need to separate the
        // immutable borrow from the mutable one.
        let user_entries: Vec<_> = self
            .dictionary
            .user_dictionary()
            .map(|ud| ud.common_prefix_search(search_text))
            .unwrap_or_default();

        // Immutable borrows on self.lattice are now finished; we can mutate.
        let mut found = false;

        for (entry, byte_len) in dict_entries {
            // Use the trie-provided byte_len to compute end_pos via
            // binary search on char_positions, avoiding chars().count().
            let end_pos = self
                .lattice
                .char_pos_from_start_and_byte_len(start_pos, byte_len);

            self.lattice.add_node(
                NodeBuilder::new(&entry.surface, start_pos, end_pos)
                    .left_id(entry.left_id)
                    .right_id(entry.right_id)
                    .word_cost(i32::from(entry.cost))
                    .node_type(NodeType::Known)
                    .feature(&entry.feature),
            );

            found = true;
        }

        for user_entry in user_entries {
            let surface_char_len = user_entry.surface.chars().count();
            let end_pos = start_pos + surface_char_len;

            self.lattice.add_node(
                NodeBuilder::new(&user_entry.surface, start_pos, end_pos)
                    .left_id(user_entry.left_id)
                    .right_id(user_entry.right_id)
                    .word_cost(i32::from(user_entry.cost))
                    .node_type(NodeType::User)
                    .feature(&user_entry.feature),
            );

            found = true;
        }

        found
    }

    /// Lattice를 반환하여 검사
    ///
    /// Viterbi 탐색 전의 Lattice 상태를 반환합니다. (디버깅/테스트용)
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트
    ///
    /// # Returns
    ///
    /// 구축된 Lattice
    pub fn tokenize_to_lattice(&mut self, text: &str) -> &Lattice {
        if !text.is_empty() {
            self.lattice.reset(text);
            self.build_lattice();
        }
        &self.lattice
    }

    /// 표면형만 추출 (wakati)
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트
    ///
    /// # Returns
    ///
    /// 분리된 표면형 목록 (wakati gaki)
    ///
    /// 일본어 형태소 분석기의 wakati gaki 모드와 동일합니다.
    /// 형태소로 분리된 표면형만 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트
    ///
    /// # Returns
    ///
    /// 분리된 표면형 목록
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mecab_ko_core::Tokenizer;
    ///
    /// let mut tokenizer = Tokenizer::new().unwrap();
    /// let surfaces = tokenizer.wakati("아버지가방에들어가신다");
    /// // ["아버지", "가", "방", "에", "들어가", "신다"]
    /// ```
    pub fn wakati(&mut self, text: &str) -> Vec<String> {
        self.tokenize(text).into_iter().map(|t| t.surface).collect()
    }

    /// 명사만 추출
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트
    ///
    /// # Returns
    ///
    /// 명사 목록
    pub fn nouns(&mut self, text: &str) -> Vec<String> {
        self.tokenize(text)
            .into_iter()
            .filter(|t| t.pos.starts_with("NN"))
            .map(|t| t.surface)
            .collect()
    }

    /// 형태소 목록 추출
    ///
    /// [`wakati`](Self::wakati)와 동일한 기능입니다.
    /// Python의 `KoNLPy` 인터페이스와 호환됩니다.
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트
    ///
    /// # Returns
    ///
    /// 형태소 목록
    pub fn morphs(&mut self, text: &str) -> Vec<String> {
        self.wakati(text)
    }

    /// 품사 태깅
    ///
    /// 형태소와 품사 태그 쌍을 반환합니다.
    /// Python의 `KoNLPy` 인터페이스와 호환됩니다.
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트
    ///
    /// # Returns
    ///
    /// `(표면형, 품사)` 쌍의 벡터
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mecab_ko_core::Tokenizer;
    ///
    /// let mut tokenizer = Tokenizer::new().unwrap();
    /// let tagged = tokenizer.pos("아버지가방에들어가신다");
    /// // [("아버지", "NNG"), ("가", "JKS"), ("방", "NNG"), ...]
    /// ```
    pub fn pos(&mut self, text: &str) -> Vec<(String, String)> {
        self.tokenize(text)
            .into_iter()
            .map(|t| (t.surface, t.pos))
            .collect()
    }

    /// 시스템 사전 참조 반환
    ///
    /// 내부 시스템 사전에 대한 읽기 전용 참조를 반환합니다.
    /// 사전 정보 조회나 디버깅에 유용합니다.
    #[must_use]
    pub const fn dictionary(&self) -> &SystemDictionary {
        &self.dictionary
    }

    /// Lattice 통계 정보
    ///
    /// 마지막 분석에서 생성된 Lattice의 통계 정보를 반환합니다.
    /// 노드 수, 엣지 수 등 디버깅 및 프로파일링에 유용합니다.
    #[must_use]
    pub fn lattice_stats(&self) -> crate::lattice::LatticeStats {
        self.lattice.stats()
    }

    /// 메모리 풀 통계 정보
    ///
    /// 메모리 풀의 사용 현황을 반환합니다.
    #[must_use]
    pub fn pool_stats(&self) -> PoolStats {
        self.pool_manager.stats()
    }

    /// 메모리 사용량 통계
    ///
    /// 토크나이저의 메모리 사용 현황을 반환합니다.
    #[must_use]
    pub fn memory_stats(&self) -> crate::memory::MemoryStats {
        crate::memory::MemoryStats {
            dictionary_bytes: 0, // 사전 크기는 별도 측정 필요
            lattice_bytes: self.lattice.memory_usage(),
            pool_bytes: self.pool_manager.total_memory_usage(),
            cache_bytes: 0,
            interner_bytes: 0,
            token_bytes: 0,
        }
    }

    /// 메모리 풀 초기화
    ///
    /// 모든 풀을 비워 메모리를 해제합니다.
    /// 장기 실행 프로세스에서 주기적으로 호출하여 메모리 누수 방지.
    pub fn clear_pools(&self) {
        self.pool_manager.clear_all();
    }

    /// 외래어 정규화 활성화
    ///
    /// # Arguments
    ///
    /// * `enable` - 정규화 활성화 여부
    /// * `config` - 정규화 설정 (None이면 기본 설정 사용)
    ///
    /// # Errors
    ///
    /// 정규화기 초기화 실패 시 에러 반환
    pub fn set_normalization(
        &mut self,
        enable: bool,
        config: Option<NormalizationConfig>,
    ) -> Result<()> {
        self.enable_normalization = enable;

        if enable {
            let normalizer_config = config.unwrap_or_default();
            self.normalizer = Some(Normalizer::new(normalizer_config)?);
        } else {
            self.normalizer = None;
        }

        Ok(())
    }

    /// 외래어 정규화기 참조 반환
    #[must_use]
    pub const fn normalizer(&self) -> Option<&Normalizer> {
        self.normalizer.as_ref()
    }

    /// 정규화가 활성화되어 있는지 확인
    #[must_use]
    pub const fn is_normalization_enabled(&self) -> bool {
        self.enable_normalization
    }

    /// 정규화 적용 형태소 분석
    ///
    /// 토큰의 표면형에 대해 정규화를 적용하고, 정규화된 형태도 함께 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트
    ///
    /// # Returns
    ///
    /// 정규화 정보가 포함된 토큰 목록
    pub fn tokenize_with_normalization(&mut self, text: &str) -> Vec<Token> {
        let mut tokens = self.tokenize(text);

        // 정규화 적용
        if let Some(normalizer) = &self.normalizer {
            for token in &mut tokens {
                token.normalized = Some(normalizer.normalize(&token.surface));
            }
        }

        tokens
    }

    /// 변이형 확장 검색
    ///
    /// 입력 단어의 변이형들을 모두 고려하여 사전 검색을 수행합니다.
    ///
    /// # Arguments
    ///
    /// * `word` - 검색할 단어
    ///
    /// # Returns
    ///
    /// `(표준형, [변이형들])` 튜플
    #[must_use]
    pub fn get_word_variants(&self, word: &str) -> (String, Vec<String>) {
        self.normalizer.as_ref().map_or_else(
            || (word.to_string(), Vec::new()),
            |normalizer| {
                let standard = normalizer.normalize(word);
                let variants = normalizer.get_variants(&standard);
                (standard, variants)
            },
        )
    }
}

// Note: Default implementation is not provided for Tokenizer because initialization
// can fail (dictionary loading, etc.). Use Tokenizer::new() explicitly instead.

#[cfg(test)]
#[allow(clippy::expect_used, clippy::vec_init_then_push)]
mod tests {
    use super::*;
    use mecab_ko_dict::{matrix::DenseMatrix, trie::TrieBuilder, DictEntry};

    /// 테스트용 토크나이저 생성
    fn create_test_tokenizer() -> Tokenizer {
        // 테스트용 Trie 생성
        let mut trie_entries = vec![
            ("아버지", 0u32),
            ("가", 1),
            ("방", 2),
            ("에", 3),
            ("들어가", 4),
            ("신다", 5),
        ];
        let trie_bytes = TrieBuilder::build_unsorted(&mut trie_entries).expect("should build trie");
        let trie =
            mecab_ko_dict::TrieBackend::Owned(mecab_ko_dict::Trie::from_vec(trie_bytes));

        // 테스트용 Matrix 생성
        let matrix = DenseMatrix::new(10, 10, 100);
        let matrix = mecab_ko_dict::matrix::ConnectionMatrix::Dense(matrix);

        // 테스트용 엔트리 생성
        let mut entries = Vec::new();
        entries.push(DictEntry::new(
            "아버지",
            1,
            1,
            1000,
            "NNG,*,T,아버지,*,*,*,*",
        ));
        entries.push(DictEntry::new("가", 5, 5, 500, "JKS,*,F,가,*,*,*,*"));
        entries.push(DictEntry::new("방", 2, 2, 2000, "NNG,*,T,방,*,*,*,*"));
        entries.push(DictEntry::new("에", 6, 6, 400, "JKB,*,F,에,*,*,*,*"));
        entries.push(DictEntry::new(
            "들어가",
            3,
            3,
            1500,
            "VV,*,F,들어가다,*,*,*,*",
        ));
        entries.push(DictEntry::new("신다", 4, 4, 1800, "VV+EP,*,F,신다,*,*,*,*"));

        let dictionary = SystemDictionary::new_test(
            std::path::PathBuf::from("./test_dic"),
            trie,
            matrix,
            entries,
        );

        let unknown_handler = UnknownHandler::korean_default();
        let viterbi_searcher = ViterbiSearcher::new();
        let lattice = Lattice::new("");

        Tokenizer {
            dictionary,
            unknown_handler,
            viterbi_searcher,
            lattice,
            normalizer: None,
            enable_normalization: false,
            pool_manager: PoolManager::new(),
        }
    }

    #[test]
    fn test_token_creation() {
        let token = Token::new("안녕".to_string(), "NNG".to_string(), 0, 2, 0, 6);

        assert_eq!(token.surface, "안녕");
        assert_eq!(token.pos, "NNG");
        assert_eq!(token.start_pos, 0);
        assert_eq!(token.end_pos, 2);
        assert_eq!(token.char_len(), 2);
        assert_eq!(token.byte_len(), 6);
    }

    #[test]
    fn test_parse_features() {
        let features = "NNG,*,T,안녕,*,*,*,*";
        let (pos, reading, lemma) = parse_features(features);

        assert_eq!(pos, "NNG");
        assert_eq!(reading, Some("안녕".to_string()));
        assert_eq!(lemma, Some("안녕".to_string()));
    }

    #[test]
    fn test_parse_features_no_reading() {
        let features = "JKS,*,F,*,*,*,*,*";
        let (pos, reading, _lemma) = parse_features(features);

        assert_eq!(pos, "JKS");
        assert_eq!(reading, None);
    }

    #[test]
    fn test_tokenize_simple() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize("아버지");

        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].surface, "아버지");
        assert_eq!(tokens[0].pos, "NNG");
    }

    #[test]
    fn test_tokenize_with_particle() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize("아버지가");

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].surface, "아버지");
        assert_eq!(tokens[0].pos, "NNG");
        assert_eq!(tokens[1].surface, "가");
        assert_eq!(tokens[1].pos, "JKS");
    }

    #[test]
    fn test_tokenize_complex() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize("아버지가방에들어가신다");

        // 최소한 "아버지", "가", "방", "에", ... 등이 분석되어야 함
        assert!(!tokens.is_empty());

        // 첫 토큰은 "아버지"
        assert_eq!(tokens[0].surface, "아버지");
    }

    #[test]
    fn test_tokenize_empty() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize("");

        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_with_spaces() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize("아버지 가방");

        // 공백은 제거되고 "아버지가방"으로 분석됨
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_wakati() {
        let mut tokenizer = create_test_tokenizer();
        let surfaces = tokenizer.wakati("아버지가");

        assert_eq!(surfaces.len(), 2);
        assert_eq!(surfaces[0], "아버지");
        assert_eq!(surfaces[1], "가");
    }

    #[test]
    fn test_nouns() {
        let mut tokenizer = create_test_tokenizer();
        let nouns = tokenizer.nouns("아버지가방에");

        // "아버지"와 "방"이 명사 (NNG)
        assert!(nouns.contains(&"아버지".to_string()));
        assert!(nouns.contains(&"방".to_string()));
        assert!(!nouns.contains(&"가".to_string())); // 조사는 제외
    }

    #[test]
    fn test_pos() {
        let mut tokenizer = create_test_tokenizer();
        let pos_tags = tokenizer.pos("아버지가");

        assert_eq!(pos_tags.len(), 2);
        assert_eq!(pos_tags[0], ("아버지".to_string(), "NNG".to_string()));
        assert_eq!(pos_tags[1], ("가".to_string(), "JKS".to_string()));
    }

    #[test]
    fn test_tokenize_to_lattice() {
        let mut tokenizer = create_test_tokenizer();
        let lattice = tokenizer.tokenize_to_lattice("아버지가");

        // Lattice에 노드가 추가되었는지 확인
        assert!(lattice.node_count() > 2); // BOS, EOS 외에 최소 1개 이상

        // 통계 확인
        let stats = lattice.stats();
        assert!(stats.total_nodes > 2);
    }

    #[test]
    fn test_lattice_stats() {
        let mut tokenizer = create_test_tokenizer();
        tokenizer.tokenize("아버지가");

        let stats = tokenizer.lattice_stats();
        assert!(stats.total_nodes > 0);
        assert!(stats.char_length > 0);
    }

    #[test]
    fn test_token_positions() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize("아버지가");

        // 첫 번째 토큰: "아버지"
        assert_eq!(tokens[0].start_pos, 0);
        assert_eq!(tokens[0].end_pos, 3);

        // 두 번째 토큰: "가"
        assert_eq!(tokens[1].start_pos, 3);
        assert_eq!(tokens[1].end_pos, 4);
    }

    #[test]
    fn test_multiple_tokenize_calls() {
        let mut tokenizer = create_test_tokenizer();

        // 첫 번째 분석
        let tokens1 = tokenizer.tokenize("아버지");
        assert!(!tokens1.is_empty());

        // 두 번째 분석 (Lattice 재사용)
        let tokens2 = tokenizer.tokenize("가방");
        assert!(!tokens2.is_empty());

        // 각 분석이 독립적으로 동작해야 함
        assert_ne!(tokens1[0].surface, tokens2[0].surface);
    }

    #[test]
    fn test_token_from_node() {
        use crate::lattice::Node;
        use std::borrow::Cow;

        let node = Node {
            id: 1,
            surface: Cow::Borrowed("테스트"),
            start_pos: 0,
            end_pos: 3,
            start_byte: 0,
            end_byte: 9,
            left_id: 1,
            right_id: 1,
            word_cost: 1000,
            total_cost: 1500,
            prev_node_id: 0,
            node_type: NodeType::Known,
            feature: Cow::Borrowed("NNG,*,T,테스트,*,*,*,*"),
            has_space_before: false,
        };

        let token = Token::from_node(&node);

        assert_eq!(token.surface, "테스트");
        assert_eq!(token.pos, "NNG");
        assert_eq!(token.start_pos, 0);
        assert_eq!(token.end_pos, 3);
        assert_eq!(token.reading, Some("테스트".to_string()));
        assert_eq!(token.cost, 1500);
    }

    #[test]
    fn test_with_user_dict() {
        let mut tokenizer = create_test_tokenizer();

        let mut user_dict = UserDictionary::new();
        user_dict.add_entry("딥러닝", "NNG", Some(-1000), None);

        tokenizer.set_user_dict(user_dict);

        // 사용자 사전이 설정되었는지 확인
        assert!(tokenizer.dictionary().user_dictionary().is_some());
    }
}
