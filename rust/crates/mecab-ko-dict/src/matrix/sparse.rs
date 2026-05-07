//! 희소 연접 비용 행렬 (Sparse Connection Cost Matrix)

use super::{dense::DenseMatrix, Matrix};

/// 희소 연접 비용 행렬 (Sparse Matrix)
///
/// 희소 행렬을 효율적으로 저장하는 구현입니다.
/// 대부분의 값이 기본값인 경우 메모리를 절약합니다.
#[derive(Debug, Clone)]
pub struct SparseMatrix {
    /// 좌문맥 크기
    lsize: usize,
    /// 우문맥 크기
    rsize: usize,
    /// 기본 비용 (희소 엔트리에 없는 경우)
    default_cost: i16,
    /// 희소 엔트리 (key: `right_id` + lsize * `left_id`, value: cost)
    entries: std::collections::HashMap<usize, i16>,
}

impl SparseMatrix {
    /// 새로운 희소 행렬 생성
    #[must_use]
    pub fn new(lsize: usize, rsize: usize, default_cost: i16) -> Self {
        Self {
            lsize,
            rsize,
            default_cost,
            entries: std::collections::HashMap::new(),
        }
    }

    /// 비용 설정
    pub fn set(&mut self, right_id: u16, left_id: u16, cost: i16) {
        let index = right_id as usize + self.lsize * left_id as usize;
        if cost == self.default_cost {
            self.entries.remove(&index);
        } else {
            self.entries.insert(index, cost);
        }
    }

    /// `DenseMatrix에서` 변환 (기본값과 다른 엔트리만 저장)
    #[must_use]
    pub fn from_dense(dense: &DenseMatrix, default_cost: i16) -> Self {
        let mut sparse = Self::new(dense.lsize, dense.rsize, default_cost);
        for (index, &cost) in dense.costs.iter().enumerate() {
            if cost != default_cost {
                sparse.entries.insert(index, cost);
            }
        }
        sparse
    }

    /// `DenseMatrix로` 변환
    #[must_use]
    pub fn to_dense(&self) -> DenseMatrix {
        let mut costs = vec![self.default_cost; self.lsize * self.rsize];
        for (&index, &cost) in &self.entries {
            if index < costs.len() {
                costs[index] = cost;
            }
        }
        DenseMatrix {
            lsize: self.lsize,
            rsize: self.rsize,
            costs,
        }
    }

    /// 엔트리 수
    #[must_use]
    pub fn entry_count_stored(&self) -> usize {
        self.entries.len()
    }

    /// 희소도 (0.0 ~ 1.0, 1.0 = 완전 희소)
    #[must_use]
    pub fn sparsity(&self) -> f64 {
        let total = self.lsize * self.rsize;
        if total == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let entries_len = self.entries.len() as f64;
        #[allow(clippy::cast_precision_loss)]
        let total_f64 = total as f64;
        1.0 - (entries_len / total_f64)
    }

    /// 메모리 사용량 (바이트, 대략적)
    #[must_use]
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.entries.capacity() * (std::mem::size_of::<usize>() + std::mem::size_of::<i16>())
    }
}

impl Matrix for SparseMatrix {
    #[inline(always)]
    fn get(&self, right_id: u16, left_id: u16) -> i32 {
        let index = right_id as usize + self.lsize * left_id as usize;
        self.entries
            .get(&index)
            .map_or_else(|| i32::from(self.default_cost), |&c| i32::from(c))
    }

    fn left_size(&self) -> usize {
        self.lsize
    }

    fn right_size(&self) -> usize {
        self.rsize
    }
}
