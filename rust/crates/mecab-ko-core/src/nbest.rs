//! 개선된 N-best Viterbi 알고리즘
//!
//! 진정한 N-best 경로 탐색을 위한 알고리즘을 제공합니다.
//!
//! # 개요
//!
//! 기존 N-best 구현은 1-best forward pass 후 단일 경로만 추적했습니다.
//! 이 모듈은 각 노드에서 K개의 최선 후보를 유지하여 진정한 N-best 결과를 제공합니다.
//!
//! # 알고리즘
//!
//! 1. **Forward Pass (K-best)**: 각 노드에서 상위 K개의 경로 후보를 유지
//! 2. **Backward Pass (N-best)**: EOS에서 시작하여 N개의 최적 경로를 추출
//!
//! # Example
//!
//! ```rust,no_run
//! use mecab_ko_core::nbest::{ImprovedNbestSearcher, NbestPath};
//! use mecab_ko_core::lattice::Lattice;
//! use mecab_ko_core::viterbi::ZeroConnectionCost;
//!
//! let mut lattice = Lattice::new("한국어");
//! // ... 노드 추가 ...
//!
//! let searcher = ImprovedNbestSearcher::new(5);
//! let conn_cost = ZeroConnectionCost;
//! let results = searcher.search(&mut lattice, &conn_cost);
//!
//! for path in results.iter() {
//!     println!("Cost: {}, Tokens: {:?}", path.cost(), path.surfaces(&lattice));
//! }
//! ```

use crate::lattice::{Lattice, Node, NodeId, NodeType, INVALID_NODE_ID};
use crate::viterbi::{ConnectionCost, SpacePenalty};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// N-best 경로 하나를 표현
#[derive(Debug, Clone)]
pub struct NbestPath {
    /// 경로의 노드 ID 목록 (BOS, EOS 제외)
    pub node_ids: Vec<NodeId>,
    /// 총 비용
    pub total_cost: i32,
    /// 경로 순위 (0-based)
    pub rank: usize,
}

impl NbestPath {
    /// 새 N-best 경로 생성
    #[must_use]
    pub const fn new(node_ids: Vec<NodeId>, total_cost: i32, rank: usize) -> Self {
        Self {
            node_ids,
            total_cost,
            rank,
        }
    }

    /// 경로가 비어있는지 확인
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.node_ids.is_empty()
    }

    /// 경로의 노드 수
    #[must_use]
    pub fn len(&self) -> usize {
        self.node_ids.len()
    }

    /// 총 비용
    #[must_use]
    pub const fn cost(&self) -> i32 {
        self.total_cost
    }

    /// 경로의 노드들 반복자
    pub fn nodes<'a>(&'a self, lattice: &'a Lattice) -> impl Iterator<Item = &'a Node> + 'a {
        self.node_ids.iter().filter_map(|&id| lattice.node(id))
    }

    /// 표면형 목록 반환
    #[must_use]
    pub fn surfaces<'a>(&'a self, lattice: &'a Lattice) -> Vec<&'a str> {
        self.nodes(lattice).map(|n| n.surface.as_ref()).collect()
    }

    /// 품사 태그 목록 반환
    #[must_use]
    pub fn pos_tags<'a>(&'a self, lattice: &'a Lattice) -> Vec<&'a str> {
        self.nodes(lattice)
            .map(|n| {
                n.feature
                    .split(',')
                    .next()
                    .unwrap_or_default()
            })
            .collect()
    }
}

/// N-best 검색 결과
#[derive(Debug, Clone, Default)]
pub struct NbestResult {
    /// 경로 목록 (비용 오름차순)
    paths: Vec<NbestPath>,
}

impl NbestResult {
    /// 새 결과 생성
    #[must_use]
    pub const fn new(paths: Vec<NbestPath>) -> Self {
        Self { paths }
    }

    /// 결과가 비어있는지 확인
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// 결과 수
    #[must_use]
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// 최선 경로 (1-best)
    #[must_use]
    pub fn best(&self) -> Option<&NbestPath> {
        self.paths.first()
    }

    /// 인덱스로 경로 조회
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&NbestPath> {
        self.paths.get(index)
    }

    /// 경로 반복자
    pub fn iter(&self) -> impl Iterator<Item = &NbestPath> {
        self.paths.iter()
    }

    /// 경로 벡터로 변환
    #[must_use]
    pub fn into_paths(self) -> Vec<NbestPath> {
        self.paths
    }

    /// (노드 ID 목록, 비용) 쌍으로 변환 (호환성용)
    #[must_use]
    pub fn to_pairs(&self) -> Vec<(Vec<NodeId>, i32)> {
        self.paths
            .iter()
            .map(|p| (p.node_ids.clone(), p.total_cost))
            .collect()
    }
}

impl IntoIterator for NbestResult {
    type Item = NbestPath;
    type IntoIter = std::vec::IntoIter<NbestPath>;

    fn into_iter(self) -> Self::IntoIter {
        self.paths.into_iter()
    }
}

/// 노드별 K-best 후보 저장
#[derive(Debug, Clone)]
struct NodeCandidate {
    /// 이 후보까지의 총 비용
    cost: i32,
    /// 이전 노드 ID
    prev_node_id: NodeId,
    /// 이전 노드에서의 후보 인덱스
    prev_candidate_idx: usize,
}

impl Eq for NodeCandidate {}

impl PartialEq for NodeCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Ord for NodeCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // 비용이 낮은 것이 우선 (Min-heap처럼 동작하도록 역순)
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for NodeCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Backward pass용 경로 후보
#[derive(Debug, Clone)]
struct BackwardCandidate {
    /// 현재 노드 ID
    node_id: NodeId,
    /// 현재 노드에서의 후보 인덱스
    candidate_idx: usize,
    /// 총 비용
    cost: i32,
    /// 지금까지의 경로 (역순, BOS 제외)
    path: Vec<NodeId>,
}

impl Eq for BackwardCandidate {}

impl PartialEq for BackwardCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Ord for BackwardCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: 비용이 낮은 것이 우선
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for BackwardCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 개선된 N-best Viterbi 탐색기
///
/// 각 노드에서 K개의 최선 후보를 유지하여 진정한 N-best 결과를 제공합니다.
#[derive(Debug, Clone)]
pub struct ImprovedNbestSearcher {
    /// 최대 결과 수 (N)
    max_results: usize,
    /// 각 노드에서 유지할 최대 후보 수 (K)
    /// 일반적으로 N보다 크거나 같아야 좋은 결과를 얻음
    max_candidates_per_node: usize,
    /// 띄어쓰기 패널티 설정
    space_penalty: SpacePenalty,
}

impl ImprovedNbestSearcher {
    /// 새 N-best 탐색기 생성
    ///
    /// # Arguments
    ///
    /// * `n` - 반환할 최대 경로 수
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            max_results: n,
            // K는 N의 2배로 설정 (더 많은 후보 탐색)
            max_candidates_per_node: n.max(2) * 2,
            space_penalty: SpacePenalty::default(),
        }
    }

    /// 노드당 최대 후보 수 설정
    #[must_use]
    pub const fn with_max_candidates(mut self, k: usize) -> Self {
        self.max_candidates_per_node = k;
        self
    }

    /// 띄어쓰기 패널티 설정
    #[must_use]
    pub fn with_space_penalty(mut self, penalty: SpacePenalty) -> Self {
        self.space_penalty = penalty;
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
    /// N-best 검색 결과
    pub fn search<C: ConnectionCost>(
        &self,
        lattice: &mut Lattice,
        conn_cost: &C,
    ) -> NbestResult {
        if lattice.node_count() <= 2 {
            // BOS, EOS만 있는 경우
            return NbestResult::default();
        }

        // 1. Forward pass: 각 노드에서 K-best 후보 계산
        let candidates = self.forward_pass_kbest(lattice, conn_cost);

        // 2. Backward pass: EOS에서 시작하여 N-best 경로 추출
        self.backward_pass_nbest(lattice, &candidates)
    }

    /// K-best Forward Pass
    ///
    /// 각 노드에서 상위 K개의 경로 후보를 유지합니다.
    fn forward_pass_kbest<C: ConnectionCost>(
        &self,
        lattice: &mut Lattice,
        conn_cost: &C,
    ) -> Vec<Vec<NodeCandidate>> {
        let node_count = lattice.node_count();
        let char_len = lattice.char_len();

        // 각 노드별 K-best 후보 저장
        let mut candidates: Vec<Vec<NodeCandidate>> = vec![Vec::new(); node_count];

        // BOS 노드의 초기 후보
        let bos_id = lattice.bos().id;
        candidates[bos_id as usize].push(NodeCandidate {
            cost: 0,
            prev_node_id: INVALID_NODE_ID,
            prev_candidate_idx: 0,
        });

        // 재사용 가능한 버퍼
        let mut starting_ids: Vec<NodeId> = Vec::new();
        let mut ending_data: Vec<(NodeId, u16)> = Vec::new();

        // 위치 0부터 끝까지 순회
        for pos in 0..=char_len {
            // 이 위치에서 시작하는 노드들
            starting_ids.clear();
            starting_ids.extend(lattice.nodes_starting_at(pos).map(|n| n.id));

            // 이 위치에서 끝나는 노드들의 정보
            ending_data.clear();
            ending_data.extend(
                lattice
                    .nodes_ending_at(pos)
                    .map(|n| (n.id, n.right_id)),
            );

            for &node_id in &starting_ids {
                let (left_id, word_cost, has_space) = {
                    let Some(node) = lattice.node(node_id) else {
                        continue;
                    };
                    (node.left_id, node.word_cost, node.has_space_before)
                };

                // 띄어쓰기 패널티
                let space_penalty = if has_space {
                    self.space_penalty.get(left_id)
                } else {
                    0
                };

                // 이 노드로 올 수 있는 모든 후보 수집
                let mut new_candidates: BinaryHeap<NodeCandidate> = BinaryHeap::new();

                for &(prev_id, prev_right_id) in &ending_data {
                    let prev_candidates = &candidates[prev_id as usize];
                    if prev_candidates.is_empty() {
                        continue;
                    }

                    let connection = conn_cost.cost(prev_right_id, left_id);

                    for (idx, prev_cand) in prev_candidates.iter().enumerate() {
                        if prev_cand.cost == i32::MAX {
                            continue;
                        }

                        let total = prev_cand
                            .cost
                            .saturating_add(connection)
                            .saturating_add(word_cost)
                            .saturating_add(space_penalty);

                        new_candidates.push(NodeCandidate {
                            cost: total,
                            prev_node_id: prev_id,
                            prev_candidate_idx: idx,
                        });
                    }
                }

                // 상위 K개만 유지
                let k = self.max_candidates_per_node;
                let mut selected: Vec<NodeCandidate> = Vec::with_capacity(k);

                while selected.len() < k {
                    if let Some(cand) = new_candidates.pop() {
                        selected.push(cand);
                    } else {
                        break;
                    }
                }

                candidates[node_id as usize] = selected;

                // 1-best 정보를 Lattice에도 업데이트 (기존 호환성)
                if let Some(best) = candidates[node_id as usize].first() {
                    if let Some(node) = lattice.node_mut(node_id) {
                        node.total_cost = best.cost;
                        node.prev_node_id = best.prev_node_id;
                    }
                }
            }
        }

        candidates
    }

    /// N-best Backward Pass
    ///
    /// EOS에서 시작하여 N개의 최적 경로를 추출합니다.
    fn backward_pass_nbest(
        &self,
        lattice: &Lattice,
        candidates: &[Vec<NodeCandidate>],
    ) -> NbestResult {
        let eos = lattice.eos();
        let eos_candidates = &candidates[eos.id as usize];

        if eos_candidates.is_empty() {
            return NbestResult::default();
        }

        let mut results: Vec<NbestPath> = Vec::with_capacity(self.max_results);
        let mut heap: BinaryHeap<BackwardCandidate> = BinaryHeap::new();

        // EOS의 모든 K-best 후보를 시작점으로 추가
        for (idx, cand) in eos_candidates.iter().enumerate() {
            heap.push(BackwardCandidate {
                node_id: eos.id,
                candidate_idx: idx,
                cost: cand.cost,
                path: Vec::new(),
            });
        }

        while let Some(current) = heap.pop() {
            if results.len() >= self.max_results {
                break;
            }

            let node_cands = &candidates[current.node_id as usize];
            if current.candidate_idx >= node_cands.len() {
                continue;
            }

            let cand = &node_cands[current.candidate_idx];

            // BOS에 도달했으면 경로 완성
            if cand.prev_node_id == INVALID_NODE_ID {
                // 경로를 뒤집어서 정상 순서로
                let mut path = current.path;
                path.reverse();

                results.push(NbestPath::new(path, current.cost, results.len()));
                continue;
            }

            // 이전 노드로 이동
            let Some(node) = lattice.node(current.node_id) else {
                continue;
            };

            let mut new_path = current.path.clone();
            // BOS, EOS가 아닌 노드만 경로에 추가
            if node.node_type != NodeType::Bos && node.node_type != NodeType::Eos {
                new_path.push(current.node_id);
            }

            heap.push(BackwardCandidate {
                node_id: cand.prev_node_id,
                candidate_idx: cand.prev_candidate_idx,
                cost: current.cost,
                path: new_path,
            });
        }

        NbestResult::new(results)
    }
}

/// 기존 `NbestSearcher`와의 호환성을 위한 래퍼
impl ImprovedNbestSearcher {
    /// 기존 API 호환: `(Vec<NodeId>, i32)` 쌍의 벡터 반환
    pub fn search_pairs<C: ConnectionCost>(
        &self,
        lattice: &mut Lattice,
        conn_cost: &C,
    ) -> Vec<(Vec<NodeId>, i32)> {
        self.search(lattice, conn_cost).to_pairs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::NodeBuilder;
    use crate::viterbi::ZeroConnectionCost;

    #[test]
    fn test_nbest_single_path() {
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

        let searcher = ImprovedNbestSearcher::new(5);
        let conn_cost = ZeroConnectionCost;
        let results = searcher.search(&mut lattice, &conn_cost);

        assert_eq!(results.len(), 1);
        assert_eq!(results.best().unwrap().cost(), 300);
    }

    #[test]
    fn test_nbest_multiple_paths() {
        // 두 가지 경로가 있는 Lattice
        // 경로 1: A -> B (비용: 100 + 200 = 300)
        // 경로 2: AB (비용: 350)
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
        lattice.add_node(
            NodeBuilder::new("AB", 0, 2)
                .left_id(3)
                .right_id(3)
                .word_cost(350),
        );

        let searcher = ImprovedNbestSearcher::new(5);
        let conn_cost = ZeroConnectionCost;
        let results = searcher.search(&mut lattice, &conn_cost);

        // 두 가지 경로가 있어야 함
        assert_eq!(results.len(), 2);

        // 1-best는 A + B (300)
        assert_eq!(results.get(0).unwrap().cost(), 300);

        // 2-best는 AB (350)
        assert_eq!(results.get(1).unwrap().cost(), 350);
    }

    #[test]
    fn test_nbest_korean_example() {
        // "아버지가" 예시
        // 경로 1: "아버지" + "가" (1000 + 500 = 1500)
        // 경로 2: "아버" + "지가" (3000 + 3000 = 6000)
        let mut lattice = Lattice::new("아버지가");

        // 경로 1
        lattice.add_node(
            NodeBuilder::new("아버지", 0, 3)
                .left_id(1)
                .right_id(1)
                .word_cost(1000),
        );
        lattice.add_node(
            NodeBuilder::new("가", 3, 4)
                .left_id(2)
                .right_id(2)
                .word_cost(500),
        );

        // 경로 2
        lattice.add_node(
            NodeBuilder::new("아버", 0, 2)
                .left_id(3)
                .right_id(3)
                .word_cost(3000),
        );
        lattice.add_node(
            NodeBuilder::new("지가", 2, 4)
                .left_id(4)
                .right_id(4)
                .word_cost(3000),
        );

        let searcher = ImprovedNbestSearcher::new(3);
        let conn_cost = ZeroConnectionCost;
        let results = searcher.search(&mut lattice, &conn_cost);

        assert!(results.len() >= 2);

        // 1-best는 "아버지" + "가"
        let best = results.best().unwrap();
        assert_eq!(best.cost(), 1500);
        assert_eq!(best.surfaces(&lattice), vec!["아버지", "가"]);

        // 2-best는 "아버" + "지가"
        let second = results.get(1).unwrap();
        assert_eq!(second.cost(), 6000);
        assert_eq!(second.surfaces(&lattice), vec!["아버", "지가"]);
    }

    #[test]
    fn test_nbest_result_api() {
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

        let searcher = ImprovedNbestSearcher::new(5);
        let conn_cost = ZeroConnectionCost;
        let results = searcher.search(&mut lattice, &conn_cost);

        // Iterator API
        for path in results.iter() {
            assert!(!path.is_empty());
            assert!(path.cost() > 0);
        }

        // IntoIterator API
        let results2 = searcher.search(&mut lattice, &conn_cost);
        for path in results2 {
            assert!(!path.is_empty());
        }
    }

    #[test]
    fn test_nbest_empty_lattice() {
        let mut lattice = Lattice::new("");
        let searcher = ImprovedNbestSearcher::new(5);
        let conn_cost = ZeroConnectionCost;
        let results = searcher.search(&mut lattice, &conn_cost);

        assert!(results.is_empty());
    }

    #[test]
    fn test_nbest_compatibility_pairs() {
        let mut lattice = Lattice::new("AB");

        lattice.add_node(
            NodeBuilder::new("AB", 0, 2)
                .left_id(1)
                .right_id(1)
                .word_cost(300),
        );

        let searcher = ImprovedNbestSearcher::new(5);
        let conn_cost = ZeroConnectionCost;
        let pairs = searcher.search_pairs(&mut lattice, &conn_cost);

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].1, 300);
    }

    #[test]
    fn test_nbest_with_max_candidates() {
        let mut lattice = Lattice::new("ABC");

        // 다양한 경로 추가
        lattice.add_node(NodeBuilder::new("A", 0, 1).word_cost(100));
        lattice.add_node(NodeBuilder::new("B", 1, 2).word_cost(100));
        lattice.add_node(NodeBuilder::new("C", 2, 3).word_cost(100));
        lattice.add_node(NodeBuilder::new("AB", 0, 2).word_cost(180));
        lattice.add_node(NodeBuilder::new("BC", 1, 3).word_cost(180));
        lattice.add_node(NodeBuilder::new("ABC", 0, 3).word_cost(250));

        let searcher = ImprovedNbestSearcher::new(5).with_max_candidates(10);
        let conn_cost = ZeroConnectionCost;
        let results = searcher.search(&mut lattice, &conn_cost);

        // 여러 경로가 있어야 함
        assert!(results.len() >= 2);

        // 비용이 오름차순으로 정렬되어야 함
        let costs: Vec<i32> = results.iter().map(|p| p.cost()).collect();
        for i in 1..costs.len() {
            assert!(costs[i] >= costs[i - 1]);
        }
    }
}
