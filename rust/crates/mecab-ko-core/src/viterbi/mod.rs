//! Viterbi 알고리즘
//!
//! 최적 형태소 분석 경로를 찾는 Viterbi 알고리즘을 구현합니다.
//!
//! # 개요
//!
//! Viterbi 알고리즘은 Lattice에서 최소 비용 경로를 찾는 동적 프로그래밍 알고리즘입니다.
//!
//! ```text
//! 총 비용 = Σ(단어 비용) + Σ(연접 비용) + Σ(띄어쓰기 패널티)
//! ```
//!
//! # 알고리즘
//!
//! 1. **Forward Pass**: BOS에서 시작하여 각 노드까지의 최소 비용 계산
//! 2. **Backward Pass**: EOS에서 BOS까지 역추적하여 최적 경로 추출
//!
//! # 한국어 특화
//!
//! - `left-space-penalty-factor`: 띄어쓰기 후 특정 품사 시작 시 페널티 부여
//! - 조사(JK*), 어미(E*) 등이 띄어쓰기 직후 시작하면 높은 페널티
//!
//! # Example
//!
//! ```rust,ignore
//! use mecab_ko_core::viterbi::{ViterbiSearcher, SpacePenalty};
//! use mecab_ko_core::lattice::Lattice;
//!
//! let mut lattice = Lattice::new("아버지가방에");
//! // ... 노드 추가 ...
//!
//! let searcher = ViterbiSearcher::new()
//!     .with_space_penalty(SpacePenalty::korean_default());
//!
//! let path = searcher.search(&mut lattice, &conn_cost);
//! ```

use crate::lattice::{Lattice, Node, NodeId, NodeType, INVALID_NODE_ID};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

// SIMD 최적화 모듈
#[cfg(feature = "simd")]
pub mod simd;

#[cfg(feature = "simd")]
pub use simd::{simd_forward_pass_position, simd_update_node_cost};

/// 연접 비용 조회 인터페이스
///
/// 두 형태소 간의 연결 비용을 반환합니다.
/// 이 비용은 matrix.def에서 학습된 값입니다.
pub trait ConnectionCost {
    /// 두 문맥 ID 간의 연접 비용 반환
    ///
    /// # Arguments
    ///
    /// * `right_id` - 이전 노드의 우문맥 ID
    /// * `left_id` - 현재 노드의 좌문맥 ID
    ///
    /// # Returns
    ///
    /// 연접 비용 (낮을수록 좋음)
    fn cost(&self, right_id: u16, left_id: u16) -> i32;
}

/// 더미 연접 비용 (테스트용)
///
/// 모든 연접에 대해 0을 반환합니다.
#[derive(Debug, Clone, Default)]
pub struct ZeroConnectionCost;

impl ConnectionCost for ZeroConnectionCost {
    fn cost(&self, _right_id: u16, _left_id: u16) -> i32 {
        0
    }
}

/// 고정 연접 비용 (테스트용)
#[derive(Debug, Clone)]
pub struct FixedConnectionCost {
    /// 기본 비용
    pub default_cost: i32,
}

impl FixedConnectionCost {
    /// 새 고정 비용 생성
    #[must_use]
    pub const fn new(cost: i32) -> Self {
        Self { default_cost: cost }
    }
}

impl ConnectionCost for FixedConnectionCost {
    fn cost(&self, _right_id: u16, _left_id: u16) -> i32 {
        self.default_cost
    }
}

/// mecab-ko-dict의 `Matrix` trait에 대한 `ConnectionCost` 구현
///
/// 사전 모듈의 연접 비용 행렬을 Viterbi 알고리즘에서 직접 사용할 수 있습니다.
impl<T: mecab_ko_dict::Matrix> ConnectionCost for T {
    fn cost(&self, right_id: u16, left_id: u16) -> i32 {
        self.get(right_id, left_id)
    }
}

/// 띄어쓰기 패널티 설정
///
/// mecab-ko의 `left-space-penalty-factor` 기능을 구현합니다.
/// 띄어쓰기 직후에 특정 품사가 오면 페널티를 부여하여 오분석을 방지합니다.
///
/// # Example
///
/// ```rust
/// use mecab_ko_core::viterbi::SpacePenalty;
///
/// // mecab-ko 기본 설정
/// let penalty = SpacePenalty::korean_default();
///
/// // dicrc 형식에서 생성
/// let penalty = SpacePenalty::from_dicrc("1785,6000;1786,6000");
/// ```
#[derive(Debug, Clone, Default)]
pub struct SpacePenalty {
    /// 페널티를 적용할 품사 ID 목록과 페널티 값
    /// `(left_id, penalty)`
    penalties: Vec<(u16, i32)>,
}

impl SpacePenalty {
    /// 빈 페널티 설정 생성
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 한국어 기본 페널티 설정
    ///
    /// 조사(JK*)와 어미(E*)가 띄어쓰기 직후 나타나면 높은 페널티를 부여합니다.
    /// 이는 "아버지가방에" → "아버지가 방에"로 분석하는 데 도움이 됩니다.
    #[must_use]
    pub fn korean_default() -> Self {
        // mecab-ko-dic의 left-id 기준 (실제 값은 사전에 따라 다름)
        // 여기서는 대표적인 조사/어미 ID 범위를 사용
        let mut penalties = Vec::new();

        // 조사 계열 (JKS, JKC, JKG, JKO, JKB, JKV, JKQ, JX, JC)
        // 일반적으로 1780~1800 범위
        for id in 1780..1810 {
            penalties.push((id, 6000));
        }

        // 어미 계열 (EP, EF, EC, ETN, ETM)
        // 일반적으로 1700~1750 범위
        for id in 1700..1760 {
            penalties.push((id, 6000));
        }

        Self { penalties }
    }

    /// mecab-ko의 dicrc 설정에서 생성
    ///
    /// # Format
    ///
    /// `left_id,penalty;left_id,penalty;...`
    ///
    /// # Example
    ///
    /// ```rust
    /// use mecab_ko_core::viterbi::SpacePenalty;
    ///
    /// let penalty = SpacePenalty::from_dicrc("1785,6000;1786,6000;1787,5000");
    /// assert_eq!(penalty.get(1785), 6000);
    /// assert_eq!(penalty.get(1786), 6000);
    /// assert_eq!(penalty.get(9999), 0);  // 미등록
    /// ```
    #[must_use]
    pub fn from_dicrc(config: &str) -> Self {
        let mut penalties = Vec::new();

        for part in config.split(';') {
            let parts: Vec<&str> = part.trim().split(',').collect();
            if parts.len() == 2 {
                if let (Ok(id), Ok(penalty)) = (
                    parts[0].trim().parse::<u16>(),
                    parts[1].trim().parse::<i32>(),
                ) {
                    penalties.push((id, penalty));
                }
            }
        }

        Self { penalties }
    }

    /// 페널티 추가
    pub fn add(&mut self, left_id: u16, penalty: i32) {
        self.penalties.push((left_id, penalty));
    }

    /// 특정 품사 ID에 대한 페널티 조회
    ///
    /// # Returns
    ///
    /// 해당 ID에 설정된 페널티, 없으면 0
    #[must_use]
    #[inline]
    pub fn get(&self, left_id: u16) -> i32 {
        for &(id, penalty) in &self.penalties {
            if id == left_id {
                return penalty;
            }
        }
        0
    }

    /// 페널티가 설정되어 있는지 확인
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.penalties.is_empty()
    }

    /// 설정된 페널티 개수
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.penalties.len()
    }
}

/// Viterbi 탐색기
///
/// Lattice에서 최적 경로를 찾는 Viterbi 알고리즘을 구현합니다.
#[derive(Debug, Clone)]
pub struct ViterbiSearcher {
    /// 띄어쓰기 패널티 설정
    pub space_penalty: SpacePenalty,
}

impl Default for ViterbiSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ViterbiSearcher {
    /// 새 탐색기 생성
    #[must_use]
    pub fn new() -> Self {
        Self {
            space_penalty: SpacePenalty::default(),
        }
    }

    /// 띄어쓰기 패널티 설정
    #[must_use]
    pub fn with_space_penalty(mut self, penalty: SpacePenalty) -> Self {
        self.space_penalty = penalty;
        self
    }

    /// 최적 경로 탐색 (Forward-Backward)
    ///
    /// # Arguments
    ///
    /// * `lattice` - 노드가 추가된 Lattice
    /// * `conn_cost` - 연접 비용 조회 인터페이스
    ///
    /// # Returns
    ///
    /// 최적 경로의 노드 ID 목록 (BOS, EOS 제외)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let path = searcher.search(&mut lattice, &conn_cost);
    /// for node_id in path {
    ///     let node = lattice.node(node_id).unwrap();
    ///     println!("{}: {}", node.surface, node.word_cost);
    /// }
    /// ```
    pub fn search<C: ConnectionCost>(&self, lattice: &mut Lattice, conn_cost: &C) -> Vec<NodeId> {
        // Forward pass
        self.forward_pass(lattice, conn_cost);

        // Backward pass
        Self::backward_pass(lattice)
    }

    /// Forward Pass: 각 노드의 최소 비용 계산
    ///
    /// BOS에서 시작하여 각 위치의 노드들에 대해 최소 비용을 계산합니다.
    fn forward_pass<C: ConnectionCost>(&self, lattice: &mut Lattice, conn_cost: &C) {
        let char_len = lattice.char_len();

        // 위치 0부터 끝까지 순회
        for pos in 0..=char_len {
            // 이 위치에서 시작하는 모든 노드에 대해 최소 비용 계산
            let starting_ids: Vec<NodeId> = lattice.nodes_starting_at(pos).map(|n| n.id).collect();

            for node_id in starting_ids {
                self.update_node_cost(lattice, conn_cost, node_id, pos);
            }
        }
    }

    /// 단일 노드의 최소 비용 계산 및 업데이트
    fn update_node_cost<C: ConnectionCost>(
        &self,
        lattice: &mut Lattice,
        conn_cost: &C,
        node_id: NodeId,
        pos: usize,
    ) {
        // 현재 노드 정보 추출
        let (left_id, word_cost, has_space) = {
            let Some(node) = lattice.node(node_id) else {
                return;
            };
            (node.left_id, node.word_cost, node.has_space_before)
        };

        // 이 노드로 연결될 수 있는 이전 노드들 (pos에서 끝나는 노드들)
        let ending_nodes: Vec<(NodeId, i32, u16)> = lattice
            .nodes_ending_at(pos)
            .map(|n| (n.id, n.total_cost, n.right_id))
            .collect();

        let mut best_cost = i32::MAX;
        let mut best_prev = INVALID_NODE_ID;

        for (prev_id, prev_cost, prev_right_id) in ending_nodes {
            // 이전 노드까지의 비용이 무한대면 스킵
            if prev_cost == i32::MAX {
                continue;
            }

            // 연접 비용 계산
            let connection = conn_cost.cost(prev_right_id, left_id);

            // 띄어쓰기 패널티 (공백 뒤에서 시작하는 경우)
            let space_penalty = if has_space {
                self.space_penalty.get(left_id)
            } else {
                0
            };

            // 총 비용 = 이전 비용 + 연접 비용 + 단어 비용 + 띄어쓰기 패널티
            let total = prev_cost
                .saturating_add(connection)
                .saturating_add(word_cost)
                .saturating_add(space_penalty);

            if total < best_cost {
                best_cost = total;
                best_prev = prev_id;
            }
        }

        // 노드 업데이트
        if let Some(node) = lattice.node_mut(node_id) {
            node.total_cost = best_cost;
            node.prev_node_id = best_prev;
        }
    }

    /// Backward Pass: EOS에서 BOS까지 역추적
    ///
    /// 최적 경로의 노드 ID 목록을 반환합니다 (BOS, EOS 제외).
    fn backward_pass(lattice: &Lattice) -> Vec<NodeId> {
        let mut path = Vec::new();
        let mut current_id = lattice.eos().id;

        while current_id != INVALID_NODE_ID {
            if let Some(node) = lattice.node(current_id) {
                // BOS, EOS는 결과에서 제외
                if node.node_type != NodeType::Bos && node.node_type != NodeType::Eos {
                    path.push(current_id);
                }
                current_id = node.prev_node_id;
            } else {
                break;
            }
        }

        path.reverse();
        path
    }

    /// 최적 경로의 총 비용 조회
    #[must_use]
    pub fn get_best_cost(&self, lattice: &Lattice) -> i32 {
        lattice.eos().total_cost
    }

    /// 경로가 유효한지 확인
    ///
    /// EOS까지의 경로가 존재하는지 확인합니다.
    #[must_use]
    pub fn has_valid_path(&self, lattice: &Lattice) -> bool {
        lattice.eos().total_cost != i32::MAX && lattice.eos().prev_node_id != INVALID_NODE_ID
    }
}

// ============================================
// N-best 지원
// ============================================

/// N-best 경로 후보
#[derive(Debug, Clone)]
struct NbestCandidate {
    /// 노드 ID
    node_id: NodeId,
    /// 총 비용
    cost: i32,
    /// 이전 노드로의 경로 인덱스
    prev_path_index: usize,
}

impl Eq for NbestCandidate {}

impl PartialEq for NbestCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Ord for NbestCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: 비용이 낮은 것이 우선
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for NbestCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// N-best 탐색기
///
/// 상위 N개의 최적 경로를 찾습니다.
#[derive(Debug, Clone)]
pub struct NbestSearcher {
    /// 기본 Viterbi 탐색기
    viterbi: ViterbiSearcher,
    /// 최대 결과 수
    max_results: usize,
}

impl NbestSearcher {
    /// 새 N-best 탐색기 생성
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            viterbi: ViterbiSearcher::new(),
            max_results: n,
        }
    }

    /// 띄어쓰기 패널티 설정
    #[must_use]
    pub fn with_space_penalty(mut self, penalty: SpacePenalty) -> Self {
        self.viterbi.space_penalty = penalty;
        self
    }

    /// N-best 경로 탐색
    ///
    /// # Arguments
    ///
    /// * `lattice` - 노드가 추가된 Lattice
    /// * `conn_cost` - 연접 비용 조회 인터페이스
    ///
    /// # Returns
    ///
    /// (경로, 비용) 쌍의 벡터, 비용 오름차순
    pub fn search<C: ConnectionCost>(
        &self,
        lattice: &mut Lattice,
        conn_cost: &C,
    ) -> Vec<(Vec<NodeId>, i32)> {
        // 먼저 Forward pass 실행
        self.viterbi.forward_pass(lattice, conn_cost);

        // 최적 경로가 없으면 빈 결과 반환
        if !self.viterbi.has_valid_path(lattice) {
            return Vec::new();
        }

        // 1-best인 경우 단순 backward pass
        if self.max_results == 1 {
            let path = ViterbiSearcher::backward_pass(lattice);
            let cost = self.viterbi.get_best_cost(lattice);
            return vec![(path, cost)];
        }

        // N-best: A* 유사 알고리즘
        self.search_nbest(lattice, conn_cost)
    }

    /// N-best 경로 탐색 (A* 기반)
    fn search_nbest<C: ConnectionCost>(
        &self,
        lattice: &Lattice,
        _conn_cost: &C,
    ) -> Vec<(Vec<NodeId>, i32)> {
        let mut results: Vec<(Vec<NodeId>, i32)> = Vec::new();
        let mut heap: BinaryHeap<NbestCandidate> = BinaryHeap::new();

        // EOS에서 시작
        let eos = lattice.eos();
        if eos.total_cost == i32::MAX {
            return results;
        }

        heap.push(NbestCandidate {
            node_id: eos.id,
            cost: eos.total_cost,
            prev_path_index: 0,
        });

        // 경로 추적을 위한 저장소
        let mut paths: Vec<Vec<NodeId>> = vec![vec![]];

        while let Some(candidate) = heap.pop() {
            if results.len() >= self.max_results {
                break;
            }

            let Some(node) = lattice.node(candidate.node_id) else {
                continue;
            };

            // 현재까지의 경로
            let mut current_path = paths[candidate.prev_path_index].clone();

            // BOS, EOS가 아니면 경로에 추가
            if node.node_type != NodeType::Bos && node.node_type != NodeType::Eos {
                current_path.push(candidate.node_id);
            }

            // BOS에 도달하면 결과에 추가
            if node.node_type == NodeType::Bos {
                current_path.reverse();
                results.push((current_path, candidate.cost));
                continue;
            }

            // 이전 노드로 계속 탐색
            if node.prev_node_id != INVALID_NODE_ID {
                let path_index = paths.len();
                paths.push(current_path);

                heap.push(NbestCandidate {
                    node_id: node.prev_node_id,
                    cost: candidate.cost,
                    prev_path_index: path_index,
                });
            }
        }

        results
    }
}

/// Viterbi 결과를 Token으로 변환하는 헬퍼
pub struct ViterbiResult<'a> {
    /// Lattice 참조
    lattice: &'a Lattice,
    /// 최적 경로 노드 ID
    path: Vec<NodeId>,
    /// 총 비용
    total_cost: i32,
}

impl<'a> ViterbiResult<'a> {
    /// 결과 생성
    #[must_use]
    pub const fn new(lattice: &'a Lattice, path: Vec<NodeId>, total_cost: i32) -> Self {
        Self {
            lattice,
            path,
            total_cost,
        }
    }

    /// 경로의 노드들 반복
    pub fn nodes(&self) -> impl Iterator<Item = &'a Node> + '_ {
        self.path.iter().filter_map(|&id| self.lattice.node(id))
    }

    /// 총 비용
    #[must_use]
    pub const fn cost(&self) -> i32 {
        self.total_cost
    }

    /// 노드 개수
    #[must_use]
    pub fn len(&self) -> usize {
        self.path.len()
    }

    /// 비어있는지 확인
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    /// 표면형 목록
    #[must_use]
    pub fn surfaces(&self) -> Vec<&str> {
        self.nodes().map(|n| n.surface.as_ref()).collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::lattice::NodeBuilder;

    /// 테스트용 연접 비용 행렬
    struct TestConnectionCost {
        costs: std::collections::HashMap<(u16, u16), i32>,
        default: i32,
    }

    impl TestConnectionCost {
        fn new(default: i32) -> Self {
            Self {
                costs: std::collections::HashMap::new(),
                default,
            }
        }

        fn set(&mut self, right_id: u16, left_id: u16, cost: i32) {
            self.costs.insert((right_id, left_id), cost);
        }
    }

    impl ConnectionCost for TestConnectionCost {
        fn cost(&self, right_id: u16, left_id: u16) -> i32 {
            self.costs
                .get(&(right_id, left_id))
                .copied()
                .unwrap_or(self.default)
        }
    }

    #[test]
    fn test_space_penalty_default() {
        let penalty = SpacePenalty::default();
        assert!(penalty.is_empty());
        assert_eq!(penalty.get(100), 0);
    }

    #[test]
    fn test_space_penalty_from_dicrc() {
        let penalty = SpacePenalty::from_dicrc("100,5000;200,3000;300,1000");

        assert_eq!(penalty.len(), 3);
        assert_eq!(penalty.get(100), 5000);
        assert_eq!(penalty.get(200), 3000);
        assert_eq!(penalty.get(300), 1000);
        assert_eq!(penalty.get(999), 0); // 미등록
    }

    #[test]
    fn test_space_penalty_korean_default() {
        let penalty = SpacePenalty::korean_default();
        assert!(!penalty.is_empty());

        // 조사 범위에 대해 페널티가 설정되어 있어야 함
        assert!(penalty.get(1785) > 0);
    }

    #[test]
    fn test_viterbi_simple_path() {
        // 간단한 Lattice: "AB"
        // BOS -> [A] -> [B] -> EOS
        let mut lattice = Lattice::new("AB");

        // A 노드 (위치 0-1)
        lattice.add_node(
            NodeBuilder::new("A", 0, 1)
                .left_id(1)
                .right_id(1)
                .word_cost(100),
        );

        // B 노드 (위치 1-2)
        lattice.add_node(
            NodeBuilder::new("B", 1, 2)
                .left_id(2)
                .right_id(2)
                .word_cost(200),
        );

        let conn_cost = ZeroConnectionCost;
        let searcher = ViterbiSearcher::new();

        let path = searcher.search(&mut lattice, &conn_cost);

        assert_eq!(path.len(), 2);

        // 첫 번째 노드는 "A"
        let first = lattice.node(path[0]).unwrap();
        assert_eq!(first.surface.as_ref(), "A");

        // 두 번째 노드는 "B"
        let second = lattice.node(path[1]).unwrap();
        assert_eq!(second.surface.as_ref(), "B");

        // 총 비용 확인
        let total_cost = searcher.get_best_cost(&lattice);
        assert_eq!(total_cost, 300); // 100 + 200
    }

    #[test]
    fn test_viterbi_choose_best_path() {
        // 두 가지 경로가 있는 Lattice: "AB"
        // 경로 1: BOS -> [AB] -> EOS (비용: 500)
        // 경로 2: BOS -> [A] -> [B] -> EOS (비용: 100 + 200 = 300)
        let mut lattice = Lattice::new("AB");

        // AB 노드 (위치 0-2) - 비용 높음
        lattice.add_node(
            NodeBuilder::new("AB", 0, 2)
                .left_id(1)
                .right_id(1)
                .word_cost(500),
        );

        // A 노드 (위치 0-1)
        lattice.add_node(
            NodeBuilder::new("A", 0, 1)
                .left_id(2)
                .right_id(2)
                .word_cost(100),
        );

        // B 노드 (위치 1-2)
        lattice.add_node(
            NodeBuilder::new("B", 1, 2)
                .left_id(3)
                .right_id(3)
                .word_cost(200),
        );

        let conn_cost = ZeroConnectionCost;
        let searcher = ViterbiSearcher::new();

        let path = searcher.search(&mut lattice, &conn_cost);

        // 더 낮은 비용의 경로 선택: A + B
        assert_eq!(path.len(), 2);
        assert_eq!(lattice.node(path[0]).unwrap().surface.as_ref(), "A");
        assert_eq!(lattice.node(path[1]).unwrap().surface.as_ref(), "B");
    }

    #[test]
    fn test_viterbi_with_connection_cost() {
        // 연접 비용이 경로 선택에 영향
        // 경로 1: BOS -> [AB] -> EOS (단어: 300, 연접: 0)
        // 경로 2: BOS -> [A] -> [B] -> EOS (단어: 100+100=200, 연접: 500)
        let mut lattice = Lattice::new("AB");

        // AB 노드
        lattice.add_node(
            NodeBuilder::new("AB", 0, 2)
                .left_id(1)
                .right_id(1)
                .word_cost(300),
        );

        // A 노드
        lattice.add_node(
            NodeBuilder::new("A", 0, 1)
                .left_id(2)
                .right_id(2)
                .word_cost(100),
        );

        // B 노드
        lattice.add_node(
            NodeBuilder::new("B", 1, 2)
                .left_id(3)
                .right_id(3)
                .word_cost(100),
        );

        let mut conn_cost = TestConnectionCost::new(0);
        // A -> B 연접에 높은 비용 설정
        conn_cost.set(2, 3, 500);

        let searcher = ViterbiSearcher::new();
        let path = searcher.search(&mut lattice, &conn_cost);

        // 연접 비용 때문에 AB 선택: 300 < 200 + 500
        assert_eq!(path.len(), 1);
        assert_eq!(lattice.node(path[0]).unwrap().surface.as_ref(), "AB");
    }

    #[test]
    fn test_viterbi_with_space_penalty() {
        // 띄어쓰기 패널티 테스트
        // "A B" (공백 있음)
        // B의 left_id에 패널티가 있으면 다른 경로 선택
        let mut lattice = Lattice::new("A B");
        // 공백 제거 후 "AB"

        // AB 노드 (전체)
        lattice.add_node(
            NodeBuilder::new("AB", 0, 2)
                .left_id(1)
                .right_id(1)
                .word_cost(500),
        );

        // A 노드
        lattice.add_node(
            NodeBuilder::new("A", 0, 1)
                .left_id(2)
                .right_id(2)
                .word_cost(100),
        );

        // B 노드 (공백 뒤에서 시작)
        lattice.add_node(
            NodeBuilder::new("B", 1, 2)
                .left_id(100) // 페널티가 적용될 ID
                .right_id(3)
                .word_cost(100)
                .has_space_before(true),
        );

        // B의 left_id에 높은 페널티 설정
        let mut penalty = SpacePenalty::new();
        penalty.add(100, 1000);

        let conn_cost = ZeroConnectionCost;
        let searcher = ViterbiSearcher::new().with_space_penalty(penalty);

        let path = searcher.search(&mut lattice, &conn_cost);

        // 페널티 때문에 AB 선택: 500 < 100 + 100 + 1000
        assert_eq!(path.len(), 1);
        assert_eq!(lattice.node(path[0]).unwrap().surface.as_ref(), "AB");
    }

    #[test]
    fn test_viterbi_korean_example() {
        // 한국어 예시: "아버지가"
        let mut lattice = Lattice::new("아버지가");

        // 경로 1: "아버지" + "가" (조사)
        lattice.add_node(
            NodeBuilder::new("아버지", 0, 3)
                .left_id(1)
                .right_id(1)
                .word_cost(1000),
        );
        lattice.add_node(
            NodeBuilder::new("가", 3, 4)
                .left_id(100) // 조사
                .right_id(100)
                .word_cost(500),
        );

        // 경로 2: "아버" + "지가"
        lattice.add_node(
            NodeBuilder::new("아버", 0, 2)
                .left_id(2)
                .right_id(2)
                .word_cost(3000),
        );
        lattice.add_node(
            NodeBuilder::new("지가", 2, 4)
                .left_id(3)
                .right_id(3)
                .word_cost(3000),
        );

        let conn_cost = ZeroConnectionCost;
        let searcher = ViterbiSearcher::new();

        let path = searcher.search(&mut lattice, &conn_cost);

        // "아버지" + "가" 선택 (비용: 1500 < 6000)
        assert_eq!(path.len(), 2);
        assert_eq!(lattice.node(path[0]).unwrap().surface.as_ref(), "아버지");
        assert_eq!(lattice.node(path[1]).unwrap().surface.as_ref(), "가");
    }

    #[test]
    fn test_viterbi_empty_lattice() {
        let mut lattice = Lattice::new("");

        let conn_cost = ZeroConnectionCost;
        let searcher = ViterbiSearcher::new();

        let path = searcher.search(&mut lattice, &conn_cost);

        // 빈 텍스트는 빈 경로
        assert!(path.is_empty());
    }

    #[test]
    fn test_viterbi_no_path() {
        // 노드가 연결되지 않는 경우
        let mut lattice = Lattice::new("ABC");

        // A만 있고 B, C 없음 -> EOS에 도달 불가
        lattice.add_node(
            NodeBuilder::new("A", 0, 1)
                .left_id(1)
                .right_id(1)
                .word_cost(100),
        );

        let conn_cost = ZeroConnectionCost;
        let searcher = ViterbiSearcher::new();

        let path = searcher.search(&mut lattice, &conn_cost);

        // 유효한 경로 없음
        assert!(!searcher.has_valid_path(&lattice));
        assert!(path.is_empty());
    }

    #[test]
    fn test_nbest_single() {
        let mut lattice = Lattice::new("AB");

        lattice.add_node(
            NodeBuilder::new("A", 0, 1)
                .left_id(1)
                .right_id(1)
                .word_cost(100),
        );
        lattice.add_node(
            NodeBuilder::new("B", 1, 2)
                .left_id(2)
                .right_id(2)
                .word_cost(200),
        );

        let conn_cost = ZeroConnectionCost;
        let searcher = NbestSearcher::new(1);

        let results = searcher.search(&mut lattice, &conn_cost);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, 300); // 비용
    }

    #[test]
    fn test_viterbi_result_helper() {
        let mut lattice = Lattice::new("AB");

        let _id1 = lattice.add_node(
            NodeBuilder::new("A", 0, 1)
                .left_id(1)
                .right_id(1)
                .word_cost(100),
        );
        let _id2 = lattice.add_node(
            NodeBuilder::new("B", 1, 2)
                .left_id(2)
                .right_id(2)
                .word_cost(200),
        );

        let conn_cost = ZeroConnectionCost;
        let searcher = ViterbiSearcher::new();
        let path = searcher.search(&mut lattice, &conn_cost);
        let cost = searcher.get_best_cost(&lattice);

        let result = ViterbiResult::new(&lattice, path, cost);

        assert_eq!(result.len(), 2);
        assert_eq!(result.cost(), 300);
        assert_eq!(result.surfaces(), vec!["A", "B"]);
    }

    #[test]
    fn test_viterbi_with_dense_matrix() {
        use mecab_ko_dict::DenseMatrix;

        // 3x3 연접 비용 행렬 생성
        // left_id: 0=BOS, 1=명사, 2=조사
        // right_id: 0=EOS, 1=명사, 2=조사
        let mut matrix = DenseMatrix::new(3, 3, 0);

        // 연접 비용 설정
        // BOS -> 명사: 낮은 비용 (자연스러움)
        matrix.set(0, 1, 100);
        // 명사 -> 조사: 낮은 비용 (자연스러움)
        matrix.set(1, 2, 50);
        // 조사 -> EOS: 낮은 비용
        matrix.set(2, 0, 30);

        // BOS -> 조사: 높은 비용 (부자연스러움)
        matrix.set(0, 2, 5000);
        // 명사 -> EOS: 중간 비용
        matrix.set(1, 0, 200);

        let mut lattice = Lattice::new("책을");

        // "책" (명사) - 문자 위치 0..1
        lattice.add_node(
            NodeBuilder::new("책", 0, 1)
                .left_id(1) // 명사 left_id
                .right_id(1) // 명사 right_id
                .word_cost(500),
        );

        // "을" (조사) - 문자 위치 1..2
        lattice.add_node(
            NodeBuilder::new("을", 1, 2)
                .left_id(2) // 조사 left_id
                .right_id(2) // 조사 right_id
                .word_cost(100),
        );

        let searcher = ViterbiSearcher::new();
        let path = searcher.search(&mut lattice, &matrix);

        // BOS -> 명사 -> 조사 -> EOS 경로 확인
        assert!(!path.is_empty());

        let result = ViterbiResult::new(&lattice, path, searcher.get_best_cost(&lattice));
        assert_eq!(result.surfaces(), vec!["책", "을"]);

        // 총 비용: BOS->명사(100) + 명사비용(500) + 명사->조사(50) + 조사비용(100) + 조사->EOS(30)
        // = 100 + 500 + 50 + 100 + 30 = 780
        assert_eq!(result.cost(), 780);
    }
}
