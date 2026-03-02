//! Lattice 자료구조
//!
//! 형태소 분석을 위한 격자(Lattice) 구조를 제공합니다.
//!
//! # 개요
//!
//! Lattice는 입력 텍스트의 모든 가능한 형태소 분석 결과를 DAG(Directed Acyclic Graph)
//! 형태로 표현합니다. Viterbi 알고리즘을 통해 최적 경로를 찾습니다.
//!
//! # 구조
//!
//! ```text
//! 입력: "아버지가"
//!
//!     BOS ─→ [아버지] ─→ [가] ─→ EOS
//!        │           │
//!        └→ [아버] ─→ [지가] ─┘
//!        │
//!        └→ [아] ─→ [버지가] ─┘
//! ```
//!
//! # 한국어 특화
//!
//! - 띄어쓰기 패널티 지원 (`left-space-penalty-factor`)
//! - UTF-8 문자 위치 정확한 처리
//! - 종성 기반 조사 연결 규칙
//!
//! # Example
//!
//! ```rust
//! use mecab_ko_core::lattice::{Lattice, NodeBuilder};
//!
//! let mut lattice = Lattice::new("안녕하세요");
//!
//! // 노드 추가 (사전에서 검색된 결과)
//! lattice.add_node(
//!     NodeBuilder::new("안녕", 0, 2)
//!         .left_id(1)
//!         .right_id(1)
//!         .word_cost(1000)
//!         .feature("NNG,*,F,안녕,*,*,*,*")
//!         .build()
//! );
//!
//! assert_eq!(lattice.node_count(), 3); // BOS + 추가노드 + EOS
//! ```

use std::borrow::Cow;

/// 노드 ID 타입
pub type NodeId = u32;

/// 특수 노드 ID
pub const BOS_NODE_ID: NodeId = 0;
/// EOS 노드는 마지막에 동적 할당
pub const INVALID_NODE_ID: NodeId = u32::MAX;

/// BOS (Beginning of Sentence) 컨텍스트 ID
pub const BOS_CONTEXT_ID: u16 = 0;

/// EOS (End of Sentence) 컨텍스트 ID
pub const EOS_CONTEXT_ID: u16 = 0;

/// 노드 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NodeType {
    /// 문장 시작 (Beginning of Sentence)
    Bos,
    /// 문장 끝 (End of Sentence)
    Eos,
    /// 사전에서 찾은 알려진 단어
    #[default]
    Known,
    /// 미등록어 (Unknown word)
    Unknown,
    /// 사용자 정의 사전
    User,
}

/// Lattice 노드
///
/// 형태소 후보를 나타내는 노드입니다.
#[derive(Debug, Clone)]
pub struct Node {
    /// 노드 ID (Lattice 내에서 유일)
    pub id: NodeId,

    /// 표면형 (surface form)
    pub surface: Cow<'static, str>,

    /// 시작 위치 (문자 단위, 0-based)
    pub start_pos: usize,

    /// 끝 위치 (문자 단위, exclusive)
    pub end_pos: usize,

    /// 시작 위치 (바이트 단위)
    pub start_byte: usize,

    /// 끝 위치 (바이트 단위)
    pub end_byte: usize,

    /// 좌문맥 ID (연접 비용 계산용)
    pub left_id: u16,

    /// 우문맥 ID (연접 비용 계산용)
    pub right_id: u16,

    /// 단어 비용 (사전에 기록된 비용)
    pub word_cost: i32,

    /// 누적 비용 (Viterbi 계산용)
    pub total_cost: i32,

    /// 최적 경로의 이전 노드 ID (backtrack용)
    pub prev_node_id: NodeId,

    /// 노드 타입
    pub node_type: NodeType,

    /// 품사 및 부가 정보 (CSV feature string)
    pub feature: Cow<'static, str>,

    /// 띄어쓰기 앞에 있는지 여부 (space penalty 적용용)
    pub has_space_before: bool,
}

impl Node {
    /// BOS 노드 생성
    #[must_use]
    pub const fn bos() -> Self {
        Self {
            id: BOS_NODE_ID,
            surface: Cow::Borrowed("BOS"),
            start_pos: 0,
            end_pos: 0,
            start_byte: 0,
            end_byte: 0,
            left_id: BOS_CONTEXT_ID,
            right_id: BOS_CONTEXT_ID,
            word_cost: 0,
            total_cost: 0,
            prev_node_id: INVALID_NODE_ID,
            node_type: NodeType::Bos,
            feature: Cow::Borrowed("BOS/EOS,*,*,*,*,*,*,*"),
            has_space_before: false,
        }
    }

    /// EOS 노드 생성
    #[must_use]
    pub const fn eos(id: NodeId, char_len: usize, byte_len: usize) -> Self {
        Self {
            id,
            surface: Cow::Borrowed("EOS"),
            start_pos: char_len,
            end_pos: char_len,
            start_byte: byte_len,
            end_byte: byte_len,
            left_id: EOS_CONTEXT_ID,
            right_id: EOS_CONTEXT_ID,
            word_cost: 0,
            total_cost: i32::MAX,
            prev_node_id: INVALID_NODE_ID,
            node_type: NodeType::Eos,
            feature: Cow::Borrowed("BOS/EOS,*,*,*,*,*,*,*"),
            has_space_before: false,
        }
    }

    /// 노드가 BOS인지 확인
    #[inline]
    #[must_use]
    pub fn is_bos(&self) -> bool {
        self.node_type == NodeType::Bos
    }

    /// 노드가 EOS인지 확인
    #[inline]
    #[must_use]
    pub fn is_eos(&self) -> bool {
        self.node_type == NodeType::Eos
    }

    /// 노드 길이 (문자 단위)
    #[inline]
    #[must_use]
    pub const fn char_len(&self) -> usize {
        self.end_pos - self.start_pos
    }

    /// 노드 길이 (바이트 단위)
    #[inline]
    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.end_byte - self.start_byte
    }
}

/// 노드 빌더 (Builder 패턴)
#[derive(Debug, Clone)]
pub struct NodeBuilder {
    surface: String,
    start_pos: usize,
    end_pos: usize,
    start_byte: usize,
    end_byte: usize,
    left_id: u16,
    right_id: u16,
    word_cost: i32,
    node_type: NodeType,
    feature: String,
    has_space_before: bool,
}

impl NodeBuilder {
    /// 새 빌더 생성
    ///
    /// # Arguments
    ///
    /// * `surface` - 표면형
    /// * `start_pos` - 시작 위치 (문자 단위)
    /// * `end_pos` - 끝 위치 (문자 단위)
    #[must_use]
    pub fn new(surface: &str, start_pos: usize, end_pos: usize) -> Self {
        Self {
            surface: surface.to_string(),
            start_pos,
            end_pos,
            start_byte: 0,
            end_byte: 0,
            left_id: 0,
            right_id: 0,
            word_cost: 0,
            node_type: NodeType::Known,
            feature: String::new(),
            has_space_before: false,
        }
    }

    /// 바이트 위치 설정
    #[must_use]
    pub const fn byte_positions(mut self, start: usize, end: usize) -> Self {
        self.start_byte = start;
        self.end_byte = end;
        self
    }

    /// 좌문맥 ID 설정
    #[must_use]
    pub const fn left_id(mut self, id: u16) -> Self {
        self.left_id = id;
        self
    }

    /// 우문맥 ID 설정
    #[must_use]
    pub const fn right_id(mut self, id: u16) -> Self {
        self.right_id = id;
        self
    }

    /// 단어 비용 설정
    #[must_use]
    pub const fn word_cost(mut self, cost: i32) -> Self {
        self.word_cost = cost;
        self
    }

    /// 노드 타입 설정
    #[must_use]
    pub const fn node_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }

    /// 품사 정보 설정
    #[must_use]
    pub fn feature(mut self, feature: &str) -> Self {
        self.feature = feature.to_string();
        self
    }

    /// 띄어쓰기 앞 여부 설정
    #[must_use]
    pub const fn has_space_before(mut self, value: bool) -> Self {
        self.has_space_before = value;
        self
    }

    /// Node 빌드 (ID는 Lattice에서 할당)
    #[must_use]
    pub const fn build(self) -> Self {
        self
    }
}

/// 문자 위치 정보
///
/// UTF-8에서 문자 위치와 바이트 위치의 매핑
#[derive(Debug, Clone)]
pub struct CharPositions {
    /// 각 문자의 바이트 시작 위치
    char_to_byte: Vec<usize>,
    /// 총 바이트 길이
    total_bytes: usize,
}

impl CharPositions {
    /// 문자열에서 위치 정보 생성
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut char_to_byte = Vec::with_capacity(text.chars().count() + 1);
        let mut byte_pos = 0;

        for c in text.chars() {
            char_to_byte.push(byte_pos);
            byte_pos += c.len_utf8();
        }
        char_to_byte.push(byte_pos); // 마지막 위치 (EOS용)

        Self {
            char_to_byte,
            total_bytes: byte_pos,
        }
    }

    /// 문자 위치 → 바이트 위치 변환
    #[inline]
    #[must_use]
    pub fn char_to_byte(&self, char_pos: usize) -> usize {
        self.char_to_byte
            .get(char_pos)
            .copied()
            .unwrap_or(self.total_bytes)
    }

    /// 문자 개수
    #[inline]
    #[must_use]
    pub fn char_count(&self) -> usize {
        if self.char_to_byte.is_empty() {
            0
        } else {
            self.char_to_byte.len() - 1
        }
    }

    /// 바이트 위치 → 문자 위치 변환 (binary search)
    ///
    /// Returns the char index whose byte start equals `byte_pos`, or
    /// `char_count()` if not found (e.g. `byte_pos` == `total_bytes`).
    #[inline]
    #[must_use]
    pub fn byte_to_char(&self, byte_pos: usize) -> usize {
        self.char_to_byte
            .binary_search(&byte_pos)
            .unwrap_or_else(|_| self.char_count())
    }

    /// 총 바이트 수
    #[inline]
    #[must_use]
    pub const fn byte_count(&self) -> usize {
        self.total_bytes
    }
}

/// 띄어쓰기 위치 정보
///
/// Stores positions as a sorted `Vec` rather than a `HashSet`.  For typical
/// sentences the number of spaces is small, so binary search in a sorted `Vec`
/// is cheaper than hashing and avoids `HashMap` overhead.
#[derive(Debug, Clone, Default)]
pub struct SpacePositions {
    /// 띄어쓰기 직후 문자 위치 목록 (정렬된 상태)
    positions: Vec<usize>,
}

impl SpacePositions {
    /// 문자열에서 띄어쓰기 위치 추출
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut positions = Vec::new();
        let mut char_pos = 0;
        let mut prev_is_space = false;

        for c in text.chars() {
            if prev_is_space && !c.is_whitespace() {
                positions.push(char_pos);
            }
            prev_is_space = c.is_whitespace();
            if !c.is_whitespace() {
                char_pos += 1;
            }
        }

        // positions are already in ascending order since we iterate left→right
        Self { positions }
    }

    /// 해당 위치가 띄어쓰기 직후인지 확인
    #[inline]
    #[must_use]
    pub fn has_space_before(&self, char_pos: usize) -> bool {
        self.positions.binary_search(&char_pos).is_ok()
    }
}

/// Lattice 구조
///
/// 입력 텍스트의 모든 형태소 분석 후보를 담는 격자 구조입니다.
///
/// # 메모리 최적화
///
/// - `nodes` 벡터는 재사용을 위해 `clear()` 시 용량 유지
/// - `ends_at`, `starts_at`도 용량 유지하여 재할당 최소화
#[derive(Debug)]
pub struct Lattice {
    /// 원본 텍스트 (공백 제거 전)
    original_text: String,

    /// 분석용 텍스트 (공백 제거 후)
    text: String,

    /// 문자 위치 정보
    char_positions: CharPositions,

    /// 띄어쓰기 위치 정보
    space_positions: SpacePositions,

    /// 모든 노드 (ID로 인덱싱)
    /// 메모리 최적화: 재사용을 위해 용량 유지
    nodes: Vec<Node>,

    /// 각 문자 위치에서 끝나는 노드 ID 목록
    /// `ends_at[pos]` = pos에서 끝나는 노드들의 ID
    ends_at: Vec<Vec<NodeId>>,

    /// 각 문자 위치에서 시작하는 노드 ID 목록
    /// `starts_at[pos]` = pos에서 시작하는 노드들의 ID
    starts_at: Vec<Vec<NodeId>>,

    /// BOS 노드 ID
    bos_id: NodeId,

    /// EOS 노드 ID
    eos_id: NodeId,
}

impl Lattice {
    /// 새 Lattice 생성
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트
    ///
    /// # Example
    ///
    /// ```rust
    /// use mecab_ko_core::lattice::Lattice;
    ///
    /// let lattice = Lattice::new("안녕하세요");
    /// assert_eq!(lattice.text(), "안녕하세요");
    /// ```
    #[must_use]
    pub fn new(text: &str) -> Self {
        // 공백 제거 (분석용)
        let original_text = text.to_string();
        let text_no_space: String = text.chars().filter(|c| !c.is_whitespace()).collect();

        let char_positions = CharPositions::new(&text_no_space);
        let space_positions = SpacePositions::new(text);

        let char_len = char_positions.char_count();
        let byte_len = char_positions.byte_count();

        // BOS 노드
        let bos = Node::bos();
        let bos_id = bos.id;

        // EOS 노드 (ID = 1)
        let eos_id = 1;
        let eos = Node::eos(eos_id, char_len, byte_len);

        // 노드 벡터 초기화 (BOS, EOS)
        let nodes = vec![bos, eos];

        // 위치별 노드 목록 초기화
        let mut ends_at = vec![Vec::new(); char_len + 1];
        let mut starts_at = vec![Vec::new(); char_len + 1];

        // BOS는 위치 0에서 끝남
        ends_at[0].push(bos_id);
        // EOS는 마지막 위치에서 시작
        starts_at[char_len].push(eos_id);

        Self {
            original_text,
            text: text_no_space,
            char_positions,
            space_positions,
            nodes,
            ends_at,
            starts_at,
            bos_id,
            eos_id,
        }
    }

    /// 분석용 텍스트 반환 (공백 제거됨)
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 원본 텍스트 반환
    #[inline]
    #[must_use]
    pub fn original_text(&self) -> &str {
        &self.original_text
    }

    /// 문자 개수
    #[inline]
    #[must_use]
    pub fn char_len(&self) -> usize {
        self.char_positions.char_count()
    }

    /// 특정 위치에서 시작하는 바이트 오프셋을 주어진 바이트 길이만큼 더한 뒤
    /// 해당 위치의 문자 인덱스를 반환합니다.
    ///
    /// `start_pos`의 바이트 시작 위치에 `byte_len`을 더한 결과에 대응하는
    /// 문자 인덱스를 binary search로 빠르게 구합니다.
    /// 이를 통해 `entry.surface.chars().count()` 비용을 줄일 수 있습니다.
    #[inline]
    #[must_use]
    pub fn char_pos_from_start_and_byte_len(&self, start_pos: usize, byte_len: usize) -> usize {
        let start_byte = self.char_positions.char_to_byte(start_pos);
        self.char_positions.byte_to_char(start_byte + byte_len)
    }

    /// 바이트 길이
    #[inline]
    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.char_positions.byte_count()
    }

    /// 노드 개수 (BOS, EOS 포함)
    #[inline]
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// BOS 노드 참조
    #[inline]
    #[must_use]
    pub fn bos(&self) -> &Node {
        &self.nodes[self.bos_id as usize]
    }

    /// EOS 노드 참조
    #[inline]
    #[must_use]
    pub fn eos(&self) -> &Node {
        &self.nodes[self.eos_id as usize]
    }

    /// EOS 노드 가변 참조
    #[inline]
    pub fn eos_mut(&mut self) -> &mut Node {
        let eos_id = self.eos_id as usize;
        &mut self.nodes[eos_id]
    }

    /// ID로 노드 참조
    #[inline]
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id as usize)
    }

    /// ID로 노드 가변 참조
    #[inline]
    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id as usize)
    }

    /// 모든 노드 반복자
    #[inline]
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
    }

    /// 특정 위치에서 끝나는 노드들
    #[inline]
    pub fn nodes_ending_at(&self, pos: usize) -> impl Iterator<Item = &Node> {
        self.ends_at
            .get(pos)
            .map(|ids| ids.iter())
            .into_iter()
            .flatten()
            .filter_map(|&id| self.nodes.get(id as usize))
    }

    /// 특정 위치에서 시작하는 노드들
    #[inline]
    pub fn nodes_starting_at(&self, pos: usize) -> impl Iterator<Item = &Node> {
        self.starts_at
            .get(pos)
            .map(|ids| ids.iter())
            .into_iter()
            .flatten()
            .filter_map(|&id| self.nodes.get(id as usize))
    }

    /// 노드 추가
    ///
    /// # Arguments
    ///
    /// * `builder` - `NodeBuilder`로 구성된 노드 정보
    ///
    /// # Returns
    ///
    /// 추가된 노드의 ID
    #[allow(clippy::cast_possible_truncation)]
    pub fn add_node(&mut self, builder: NodeBuilder) -> NodeId {
        let id = self.nodes.len() as NodeId;

        // 바이트 위치 계산
        let start_byte = self.char_positions.char_to_byte(builder.start_pos);
        let end_byte = self.char_positions.char_to_byte(builder.end_pos);

        // 띄어쓰기 앞 여부 확인
        let has_space_before =
            builder.has_space_before || self.space_positions.has_space_before(builder.start_pos);

        let node = Node {
            id,
            surface: Cow::Owned(builder.surface),
            start_pos: builder.start_pos,
            end_pos: builder.end_pos,
            start_byte,
            end_byte,
            left_id: builder.left_id,
            right_id: builder.right_id,
            word_cost: builder.word_cost,
            total_cost: i32::MAX, // Viterbi에서 계산
            prev_node_id: INVALID_NODE_ID,
            node_type: builder.node_type,
            feature: Cow::Owned(builder.feature),
            has_space_before,
        };

        // 위치 인덱스 업데이트
        if builder.start_pos < self.starts_at.len() {
            self.starts_at[builder.start_pos].push(id);
        }
        if builder.end_pos < self.ends_at.len() {
            self.ends_at[builder.end_pos].push(id);
        }

        self.nodes.push(node);
        id
    }

    /// 문자 위치에서 부분 문자열 추출
    #[must_use]
    pub fn substring(&self, start: usize, end: usize) -> &str {
        let start_byte = self.char_positions.char_to_byte(start);
        let end_byte = self.char_positions.char_to_byte(end);
        &self.text[start_byte..end_byte]
    }

    /// 특정 위치에 띄어쓰기가 있는지 확인
    #[inline]
    #[must_use]
    pub fn has_space_at(&self, char_pos: usize) -> bool {
        self.space_positions.has_space_before(char_pos)
    }

    /// Lattice 초기화 (노드 재사용)
    pub fn clear(&mut self) {
        // BOS, EOS 유지하고 나머지 제거
        self.nodes.truncate(2);

        // 위치 인덱스 초기화
        for v in &mut self.ends_at {
            v.clear();
        }
        for v in &mut self.starts_at {
            v.clear();
        }

        // BOS, EOS 재등록
        if !self.ends_at.is_empty() {
            self.ends_at[0].push(self.bos_id);
        }
        let char_len = self.char_len();
        if char_len < self.starts_at.len() {
            self.starts_at[char_len].push(self.eos_id);
        }

        // EOS 노드 리셋
        if let Some(eos) = self.nodes.get_mut(self.eos_id as usize) {
            eos.total_cost = i32::MAX;
            eos.prev_node_id = INVALID_NODE_ID;
        }
    }

    /// 새 텍스트로 Lattice 재설정
    pub fn reset(&mut self, text: &str) {
        // Reuse the String allocation for original_text/text instead of
        // dropping and recreating.
        self.original_text.clear();
        self.original_text.push_str(text);

        self.text.clear();
        for c in text.chars().filter(|c| !c.is_whitespace()) {
            self.text.push(c);
        }

        self.char_positions = CharPositions::new(&self.text);
        self.space_positions = SpacePositions::new(text);

        let char_len = self.char_positions.char_count();
        let byte_len = self.char_positions.byte_count();

        // Resize the position index vectors without dropping inner Vec capacity.
        // If new size <= old size, clear existing slots and truncate.
        // If new size > old size, clear existing slots and push new empty vecs.
        let new_len = char_len + 1;
        let old_ends_len = self.ends_at.len();
        let old_starts_len = self.starts_at.len();

        // Clear the slots we will reuse (preserving their heap capacity).
        for v in self.ends_at.iter_mut().take(new_len.min(old_ends_len)) {
            v.clear();
        }
        for v in self.starts_at.iter_mut().take(new_len.min(old_starts_len)) {
            v.clear();
        }

        // Resize (truncate or extend).
        self.ends_at.truncate(new_len);
        self.starts_at.truncate(new_len);
        while self.ends_at.len() < new_len {
            self.ends_at.push(Vec::new());
        }
        while self.starts_at.len() < new_len {
            self.starts_at.push(Vec::new());
        }

        // 노드 리셋
        self.nodes.truncate(2);

        // EOS 업데이트
        if let Some(eos) = self.nodes.get_mut(self.eos_id as usize) {
            eos.start_pos = char_len;
            eos.end_pos = char_len;
            eos.start_byte = byte_len;
            eos.end_byte = byte_len;
            eos.total_cost = i32::MAX;
            eos.prev_node_id = INVALID_NODE_ID;
        }

        // BOS, EOS 재등록
        self.ends_at[0].push(self.bos_id);
        self.starts_at[char_len].push(self.eos_id);
    }

    /// 최적 경로 추출 (Viterbi 실행 후 호출)
    ///
    /// EOS에서 BOS까지 역추적하여 최적 경로의 노드들을 반환합니다.
    #[must_use]
    pub fn best_path(&self) -> Vec<&Node> {
        let mut path = Vec::new();
        let mut current_id = self.eos_id;

        while current_id != INVALID_NODE_ID {
            if let Some(node) = self.nodes.get(current_id as usize) {
                if node.node_type != NodeType::Bos && node.node_type != NodeType::Eos {
                    path.push(node);
                }
                current_id = node.prev_node_id;
            } else {
                break;
            }
        }

        path.reverse();
        path
    }

    /// 디버그용: Lattice 시각화
    #[cfg(test)]
    #[must_use]
    #[allow(clippy::format_push_string, clippy::uninlined_format_args)]
    pub fn visualize(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Lattice for: \"{}\"\n", self.text));
        output.push_str(&format!("Nodes: {}\n", self.node_count()));

        for pos in 0..=self.char_len() {
            let ending: Vec<_> = self.nodes_ending_at(pos).collect();
            if !ending.is_empty() {
                output.push_str(&format!("\nPosition {}: ", pos));
                for node in ending {
                    output.push_str(&format!(
                        "[{}: {} ({}-{})]",
                        node.id, node.surface, node.start_pos, node.end_pos
                    ));
                }
            }
        }

        output
    }
}

/// Lattice 통계 정보
#[derive(Debug, Clone, Default)]
pub struct LatticeStats {
    /// 총 노드 수
    pub total_nodes: usize,
    /// Known 노드 수
    pub known_nodes: usize,
    /// Unknown 노드 수
    pub unknown_nodes: usize,
    /// User 노드 수
    pub user_nodes: usize,
    /// 문자 길이
    pub char_length: usize,
}

impl Lattice {
    /// 통계 정보 계산
    #[must_use]
    pub fn stats(&self) -> LatticeStats {
        let mut stats = LatticeStats {
            total_nodes: self.nodes.len(),
            char_length: self.char_len(),
            ..Default::default()
        };

        for node in &self.nodes {
            match node.node_type {
                NodeType::Known => stats.known_nodes += 1,
                NodeType::Unknown => stats.unknown_nodes += 1,
                NodeType::User => stats.user_nodes += 1,
                _ => {}
            }
        }

        stats
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::needless_collect)]
mod tests {
    use super::*;

    #[test]
    fn test_lattice_creation() {
        let lattice = Lattice::new("안녕하세요");

        assert_eq!(lattice.text(), "안녕하세요");
        assert_eq!(lattice.char_len(), 5);
        assert_eq!(lattice.node_count(), 2); // BOS + EOS
    }

    #[test]
    fn test_lattice_with_spaces() {
        let lattice = Lattice::new("안녕 하세요");

        // 공백 제거된 텍스트
        assert_eq!(lattice.text(), "안녕하세요");
        assert_eq!(lattice.original_text(), "안녕 하세요");
        assert_eq!(lattice.char_len(), 5);

        // 띄어쓰기 위치 확인
        assert!(!lattice.has_space_at(0));
        assert!(!lattice.has_space_at(1));
        assert!(lattice.has_space_at(2)); // "하" 앞에 공백
    }

    #[test]
    fn test_add_node() {
        let mut lattice = Lattice::new("안녕하세요");

        let node_id = lattice.add_node(
            NodeBuilder::new("안녕", 0, 2)
                .left_id(100)
                .right_id(100)
                .word_cost(1000)
                .feature("NNG,*,F,안녕,*,*,*,*"),
        );

        assert_eq!(node_id, 2); // BOS=0, EOS=1, 새 노드=2
        assert_eq!(lattice.node_count(), 3);

        let node = lattice.node(node_id).unwrap();
        assert_eq!(node.surface.as_ref(), "안녕");
        assert_eq!(node.start_pos, 0);
        assert_eq!(node.end_pos, 2);
        assert_eq!(node.left_id, 100);
        assert_eq!(node.word_cost, 1000);
    }

    #[test]
    fn test_nodes_at_position() {
        let mut lattice = Lattice::new("안녕하세요");

        // "안녕" (0-2)
        lattice.add_node(NodeBuilder::new("안녕", 0, 2));
        // "안" (0-1)
        lattice.add_node(NodeBuilder::new("안", 0, 1));
        // "녕하" (1-3)
        lattice.add_node(NodeBuilder::new("녕하", 1, 3));

        // 위치 0에서 시작하는 노드들
        let starting_at_0: Vec<_> = lattice.nodes_starting_at(0).collect();
        assert_eq!(starting_at_0.len(), 2); // "안녕", "안"

        // 위치 2에서 끝나는 노드들
        let ending_at_2: Vec<_> = lattice.nodes_ending_at(2).collect();
        assert_eq!(ending_at_2.len(), 1); // "안녕"
    }

    #[test]
    fn test_char_positions() {
        let positions = CharPositions::new("한글test");

        assert_eq!(positions.char_count(), 6);
        assert_eq!(positions.char_to_byte(0), 0); // '한' 시작
        assert_eq!(positions.char_to_byte(1), 3); // '글' 시작 (한글 3바이트)
        assert_eq!(positions.char_to_byte(2), 6); // 't' 시작
        assert_eq!(positions.char_to_byte(3), 7); // 'e' 시작
    }

    #[test]
    fn test_substring() {
        let lattice = Lattice::new("안녕하세요");

        assert_eq!(lattice.substring(0, 2), "안녕");
        assert_eq!(lattice.substring(2, 5), "하세요");
        assert_eq!(lattice.substring(0, 5), "안녕하세요");
    }

    #[test]
    fn test_bos_eos() {
        let lattice = Lattice::new("테스트");

        let bos = lattice.bos();
        assert!(bos.is_bos());
        assert_eq!(bos.id, BOS_NODE_ID);

        let eos = lattice.eos();
        assert!(eos.is_eos());
        assert_eq!(eos.start_pos, 3);
    }

    #[test]
    fn test_lattice_reset() {
        let mut lattice = Lattice::new("안녕");
        lattice.add_node(NodeBuilder::new("안녕", 0, 2));
        assert_eq!(lattice.node_count(), 3);

        lattice.reset("하세요");
        assert_eq!(lattice.text(), "하세요");
        assert_eq!(lattice.char_len(), 3);
        assert_eq!(lattice.node_count(), 2); // BOS + EOS만
    }

    #[test]
    fn test_space_before_detection() {
        let mut lattice = Lattice::new("아버지가 방에");

        // "방" 노드 추가 (공백 뒤)
        let node_id = lattice.add_node(NodeBuilder::new("방에", 4, 6));

        let node = lattice.node(node_id).unwrap();
        assert!(node.has_space_before);

        // "아버지가" 노드 추가 (공백 앞)
        let node_id2 = lattice.add_node(NodeBuilder::new("아버지가", 0, 4));
        let node2 = lattice.node(node_id2).unwrap();
        assert!(!node2.has_space_before);
    }
}
