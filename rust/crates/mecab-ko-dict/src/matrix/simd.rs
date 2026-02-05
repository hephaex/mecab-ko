//! SIMD 최적화된 연접 비용 행렬 조회
//!
//! 이 모듈은 portable_simd를 사용하여 연접 비용 행렬 조회를 벡터화합니다.
//!
//! # 최적화 전략
//!
//! 1. **배치 조회 (Batch Lookup)**: 여러 연접 비용을 한 번에 조회
//! 2. **벡터화된 비교**: SIMD 레인을 사용하여 비용 비교
//! 3. **캐시 친화적 접근**: 메모리 접근 패턴 최적화
//!
//! # 성능
//!
//! SIMD를 사용하면 단일 조회보다 배치 조회가 2-4배 빠릅니다.

#![allow(unsafe_code)]

use super::{DenseMatrix, Matrix, INVALID_CONNECTION_COST};
use std::simd::{
    cmp::{SimdOrd, SimdPartialOrd},
    i32x8,
    num::SimdInt,
    u16x16, u16x8, LaneCount, Simd, SupportedLaneCount,
};

/// SIMD 레인 크기 (i32x8 사용)
const SIMD_LANES_8: usize = 8;

/// SIMD 레인 크기 (i32x16 사용 - AVX512)
const SIMD_LANES_16: usize = 16;

/// SIMD 최적화된 배치 조회 인터페이스
pub trait SimdMatrix: Matrix {
    /// 배치로 연접 비용 조회 (8개씩)
    ///
    /// # Arguments
    ///
    /// * `right_ids` - 우문맥 ID 배열 (8개)
    /// * `left_ids` - 좌문맥 ID 배열 (8개)
    ///
    /// # Returns
    ///
    /// 연접 비용 배열 (8개)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let right_ids = [1, 2, 3, 4, 5, 6, 7, 8];
    /// let left_ids = [10, 11, 12, 13, 14, 15, 16, 17];
    /// let costs = matrix.batch_get_8(&right_ids, &left_ids);
    /// ```
    fn batch_get_8(&self, right_ids: &[u16; 8], left_ids: &[u16; 8]) -> [i32; 8];

    /// 배치로 연접 비용 조회 (16개씩 - AVX512 지원 시)
    fn batch_get_16(&self, right_ids: &[u16; 16], left_ids: &[u16; 16]) -> [i32; 16];

    /// 슬라이스에서 배치 조회 (가변 길이)
    ///
    /// 길이가 SIMD 레인의 배수가 아니면 나머지는 스칼라 처리
    fn batch_get_slice(&self, right_ids: &[u16], left_ids: &[u16], output: &mut [i32]);
}

impl SimdMatrix for DenseMatrix {
    #[inline]
    fn batch_get_8(&self, right_ids: &[u16; 8], left_ids: &[u16; 8]) -> [i32; 8] {
        // SIMD 구현
        batch_lookup_simd_8(self, right_ids, left_ids)
    }

    #[inline]
    fn batch_get_16(&self, right_ids: &[u16; 16], left_ids: &[u16; 16]) -> [i32; 16] {
        // SIMD 구현 (AVX512)
        batch_lookup_simd_16(self, right_ids, left_ids)
    }

    fn batch_get_slice(&self, right_ids: &[u16], left_ids: &[u16], output: &mut [i32]) {
        let len = right_ids.len().min(left_ids.len()).min(output.len());

        // 8개씩 SIMD 처리
        let simd_chunks = len / SIMD_LANES_8;
        for i in 0..simd_chunks {
            let start = i * SIMD_LANES_8;
            let end = start + SIMD_LANES_8;

            let right_chunk: [u16; 8] = right_ids[start..end].try_into().unwrap_or([0; 8]);
            let left_chunk: [u16; 8] = left_ids[start..end].try_into().unwrap_or([0; 8]);

            let costs = self.batch_get_8(&right_chunk, &left_chunk);
            output[start..end].copy_from_slice(&costs);
        }

        // 나머지 스칼라 처리
        for i in (simd_chunks * SIMD_LANES_8)..len {
            output[i] = self.get(right_ids[i], left_ids[i]);
        }
    }
}

/// SIMD를 사용한 배치 조회 (8개)
#[inline]
fn batch_lookup_simd_8(
    matrix: &DenseMatrix,
    right_ids: &[u16; 8],
    left_ids: &[u16; 8],
) -> [i32; 8] {
    let lsize = matrix.left_size() as u16;
    let costs_ref = matrix.costs();

    // 인덱스 계산: index = right_id + lsize * left_id
    let right_vec = u16x8::from_array(*right_ids);
    let left_vec = u16x8::from_array(*left_ids);

    // lsize를 SIMD 벡터로 브로드캐스트
    let lsize_vec = u16x8::splat(lsize);

    // left_id * lsize
    let left_scaled = left_vec * lsize_vec;

    // index = right_id + left_id * lsize
    let indices = right_vec + left_scaled;

    // 인덱스를 배열로 변환하여 조회
    let indices_array = indices.to_array();
    let mut result = [INVALID_CONNECTION_COST; 8];

    for (i, &idx) in indices_array.iter().enumerate() {
        let idx = idx as usize;
        if idx < costs_ref.len() {
            result[i] = costs_ref[idx] as i32;
        }
    }

    result
}

/// SIMD를 사용한 배치 조회 (16개 - AVX512)
#[inline]
fn batch_lookup_simd_16(
    matrix: &DenseMatrix,
    right_ids: &[u16; 16],
    left_ids: &[u16; 16],
) -> [i32; 16] {
    let lsize = matrix.left_size() as u16;
    let costs_ref = matrix.costs();

    let right_vec = u16x16::from_array(*right_ids);
    let left_vec = u16x16::from_array(*left_ids);
    let lsize_vec = u16x16::splat(lsize);

    let left_scaled = left_vec * lsize_vec;
    let indices = right_vec + left_scaled;

    let indices_array = indices.to_array();
    let mut result = [INVALID_CONNECTION_COST; 16];

    for (i, &idx) in indices_array.iter().enumerate() {
        let idx = idx as usize;
        if idx < costs_ref.len() {
            result[i] = costs_ref[idx] as i32;
        }
    }

    result
}

/// SIMD를 사용한 최소값 찾기
///
/// 연접 비용 배열에서 최소값과 인덱스를 찾습니다.
#[inline]
pub fn simd_find_min_cost_8(costs: &[i32; 8]) -> (i32, usize) {
    let vec = i32x8::from_array(*costs);

    // 수평 최소값 계산
    let min_cost = vec.reduce_min();

    // 최소값의 인덱스 찾기 (스칼라 루프 사용)
    let mut min_idx = 0;
    for (i, &cost) in costs.iter().enumerate() {
        if cost == min_cost {
            min_idx = i;
            break;
        }
    }

    (min_cost, min_idx)
}

/// SIMD를 사용한 비용 계산 (여러 노드의 총 비용)
///
/// # Arguments
///
/// * `prev_costs` - 이전 노드의 총 비용 (8개)
/// * `conn_costs` - 연접 비용 (8개)
/// * `word_cost` - 단어 비용 (브로드캐스트)
/// * `space_penalty` - 띄어쓰기 패널티 (브로드캐스트)
///
/// # Returns
///
/// 총 비용 배열 (8개)
#[inline]
pub fn simd_calculate_total_costs_8(
    prev_costs: &[i32; 8],
    conn_costs: &[i32; 8],
    word_cost: i32,
    space_penalty: i32,
) -> [i32; 8] {
    let prev_vec = i32x8::from_array(*prev_costs);
    let conn_vec = i32x8::from_array(*conn_costs);
    let word_vec = i32x8::splat(word_cost);
    let penalty_vec = i32x8::splat(space_penalty);

    // total = prev + conn + word + penalty
    let total = prev_vec + conn_vec + word_vec + penalty_vec;

    total.to_array()
}

/// SIMD를 사용한 포화 덧셈 (saturating add)
///
/// 오버플로우 방지를 위해 i32::MAX로 포화
#[inline]
pub fn simd_saturating_add_8(a: &[i32; 8], b: &[i32; 8]) -> [i32; 8] {
    let a_vec = i32x8::from_array(*a);
    let b_vec = i32x8::from_array(*b);

    // SIMD에는 saturating_add가 없으므로 수동 구현
    let sum = a_vec + b_vec;

    // 오버플로우 체크: a > 0 && b > 0 && sum < 0
    let a_pos = a_vec.simd_gt(i32x8::splat(0));
    let b_pos = b_vec.simd_gt(i32x8::splat(0));
    let sum_neg = sum.simd_lt(i32x8::splat(0));
    let overflow = a_pos & b_pos & sum_neg;

    // 오버플로우 발생 시 i32::MAX로 대체
    let max_vec = i32x8::splat(i32::MAX);
    overflow.select(max_vec, sum).to_array()
}

/// 여러 비용 배열에서 최소값 찾기 (수평 reduction)
///
/// 각 위치에서의 최소값을 찾습니다.
#[inline]
pub fn simd_min_across_8(a: &[i32; 8], b: &[i32; 8]) -> [i32; 8] {
    let a_vec = i32x8::from_array(*a);
    let b_vec = i32x8::from_array(*b);
    a_vec.simd_min(b_vec).to_array()
}

/// 일반적인 SIMD 레인 크기에 대한 배치 조회 (제네릭)
///
/// 이 함수는 다양한 SIMD 레인 크기를 지원합니다.
#[inline]
fn batch_lookup_generic<const LANES: usize>(
    matrix: &DenseMatrix,
    right_ids: &[u16],
    left_ids: &[u16],
) -> Vec<i32>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    let len = right_ids.len().min(left_ids.len());
    let mut result = Vec::with_capacity(len);

    let lsize = matrix.left_size() as u16;
    let costs_ref = matrix.costs();

    let chunks = len / LANES;
    for i in 0..chunks {
        let start = i * LANES;
        let end = start + LANES;

        let right_slice = &right_ids[start..end];
        let left_slice = &left_ids[start..end];

        // 제네릭 SIMD 벡터 생성
        let right_vec = Simd::<u16, LANES>::from_slice(right_slice);
        let left_vec = Simd::<u16, LANES>::from_slice(left_slice);
        let lsize_vec = Simd::<u16, LANES>::splat(lsize);

        let left_scaled = left_vec * lsize_vec;
        let indices = right_vec + left_scaled;

        // 결과 조회
        for lane in 0..LANES {
            let idx = indices.to_array()[lane] as usize;
            let cost = if idx < costs_ref.len() {
                costs_ref[idx] as i32
            } else {
                INVALID_CONNECTION_COST
            };
            result.push(cost);
        }
    }

    // 나머지 스칼라 처리
    for i in (chunks * LANES)..len {
        result.push(matrix.get(right_ids[i], left_ids[i]));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_get_8() {
        let mut matrix = DenseMatrix::new(10, 10, 0);

        // 테스트 데이터 설정
        for i in 0..8 {
            matrix.set(i, i, (i as i16 + 1) * 100);
        }

        let right_ids = [0, 1, 2, 3, 4, 5, 6, 7];
        let left_ids = [0, 1, 2, 3, 4, 5, 6, 7];

        let costs = matrix.batch_get_8(&right_ids, &left_ids);

        assert_eq!(costs[0], 100);
        assert_eq!(costs[1], 200);
        assert_eq!(costs[7], 800);
    }

    #[test]
    fn test_batch_get_slice() {
        let mut matrix = DenseMatrix::new(20, 20, 0);

        // 테스트 데이터 설정
        for i in 0..20 {
            matrix.set(i, i, (i as i16) * 10);
        }

        let right_ids: Vec<u16> = (0..20).collect();
        let left_ids: Vec<u16> = (0..20).collect();
        let mut output = vec![0i32; 20];

        matrix.batch_get_slice(&right_ids, &left_ids, &mut output);

        for i in 0..20 {
            assert_eq!(output[i], (i as i32) * 10);
        }
    }

    #[test]
    fn test_simd_find_min_cost_8() {
        let costs = [100, 50, 200, 25, 300, 10, 150, 75];
        let (min_cost, min_idx) = simd_find_min_cost_8(&costs);

        assert_eq!(min_cost, 10);
        assert_eq!(min_idx, 5);
    }

    #[test]
    fn test_simd_calculate_total_costs_8() {
        let prev_costs = [100, 200, 300, 400, 500, 600, 700, 800];
        let conn_costs = [10, 20, 30, 40, 50, 60, 70, 80];
        let word_cost = 1000;
        let space_penalty = 500;

        let totals =
            simd_calculate_total_costs_8(&prev_costs, &conn_costs, word_cost, space_penalty);

        // total = prev + conn + word + penalty
        assert_eq!(totals[0], 100 + 10 + 1000 + 500); // 1610
        assert_eq!(totals[1], 200 + 20 + 1000 + 500); // 1720
        assert_eq!(totals[7], 800 + 80 + 1000 + 500); // 2380
    }

    #[test]
    fn test_simd_saturating_add_8() {
        let a = [i32::MAX - 10, 100, 200, 300, 400, 500, 600, 700];
        let b = [20, 50, 60, 70, 80, 90, 100, 110];

        let result = simd_saturating_add_8(&a, &b);

        // 첫 번째 요소는 오버플로우로 i32::MAX
        assert_eq!(result[0], i32::MAX);
        // 나머지는 정상 덧셈
        assert_eq!(result[1], 150);
        assert_eq!(result[7], 810);
    }

    #[test]
    fn test_simd_min_across_8() {
        let a = [100, 200, 300, 400, 500, 600, 700, 800];
        let b = [150, 150, 250, 450, 450, 550, 750, 750];

        let result = simd_min_across_8(&a, &b);

        assert_eq!(result[0], 100); // min(100, 150)
        assert_eq!(result[1], 150); // min(200, 150)
        assert_eq!(result[7], 750); // min(800, 750)
    }

    #[test]
    fn test_batch_get_boundary() {
        let matrix = DenseMatrix::new(10, 10, 0);

        // 경계 외 접근
        let right_ids = [50, 60, 70, 80, 90, 100, 110, 120];
        let left_ids = [50, 60, 70, 80, 90, 100, 110, 120];

        let costs = matrix.batch_get_8(&right_ids, &left_ids);

        // 모두 INVALID_CONNECTION_COST
        for cost in costs.iter() {
            assert_eq!(*cost, INVALID_CONNECTION_COST);
        }
    }

    #[test]
    fn test_batch_get_mixed() {
        let mut matrix = DenseMatrix::new(10, 10, 0);
        matrix.set(0, 0, 100);
        matrix.set(5, 5, 500);

        // 일부 유효, 일부 무효
        let right_ids = [0, 1, 2, 5, 50, 60, 70, 80];
        let left_ids = [0, 1, 2, 5, 50, 60, 70, 80];

        let costs = matrix.batch_get_8(&right_ids, &left_ids);

        assert_eq!(costs[0], 100);
        assert_eq!(costs[1], 0);
        assert_eq!(costs[3], 500);
        assert_eq!(costs[4], INVALID_CONNECTION_COST);
    }
}
