//! # 연접 비용 행렬 (Connection Cost Matrix)
//!
//! 형태소 간 연접 비용을 저장하고 조회하는 모듈입니다.
//!
//! ## 포맷 지원
//!
//! - **텍스트 포맷** (`matrix.def`): `MeCab` 표준 형식
//! - **바이너리 포맷** (`matrix.bin`): 고정 크기 i16 배열
//! - **압축 포맷** (`matrix.bin.zst`): Zstd 압축 바이너리
//!
//! ## 예제
//!
//! ```rust,ignore
//! use mecab_ko_dict::matrix::ConnectionMatrix;
//!
//! // 텍스트 파일에서 로드
//! let matrix = ConnectionMatrix::from_def_file("matrix.def").unwrap();
//!
//! // 연접 비용 조회 (left_id=0, right_id=0)
//! let cost = matrix.get(0, 0);
//! ```
//!
//! ## 행렬 구조
//!
//! 연접 비용 행렬은 `lsize x rsize` 크기의 2차원 배열입니다.
//! - `lsize`: 좌문맥 ID 개수
//! - `rsize`: 우문맥 ID 개수
//! - 접근: `matrix[right_id + lsize * left_id]`

use std::io::{self};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::error::{DictError, Result};

mod dense;
mod mmap;
mod sparse;

pub use dense::DenseMatrix;
pub use mmap::MmapMatrix;
pub use sparse::SparseMatrix;

// SIMD 최적화 모듈
#[cfg(feature = "simd")]
pub mod simd;

#[cfg(feature = "simd")]
pub use simd::SimdMatrix;

pub(super) const MATRIX_HEADER_SIZE: usize = 4;

pub(super) const MKM3_MAGIC: &[u8; 4] = b"MKM3";
pub(super) const MKM3_HEADER_SIZE: usize = 16;

/// 행렬 헤더 정보
pub(super) struct MatrixHeader {
    /// 좌문맥 크기
    pub(super) lsize: usize,
    /// 우문맥 크기
    pub(super) rsize: usize,
    /// 헤더 크기 (v2: 4, v3: 16)
    pub(super) header_size: usize,
}

/// 행렬 헤더를 파싱하는 내부 함수
///
/// v2/v3 포맷을 자동 감지하고 헤더 정보를 추출합니다.
///
/// # Arguments
///
/// * `data` - 파싱할 바이트 데이터 (헤더 크기 이상)
///
/// # Returns
///
/// 성공 시 `MatrixHeader`, 형식 오류 시 에러
pub(super) fn parse_matrix_header(data: &[u8]) -> Result<MatrixHeader> {
    let is_v3 = data.len() >= 4 && &data[..4] == MKM3_MAGIC;
    let header_size = if is_v3 {
        MKM3_HEADER_SIZE
    } else {
        MATRIX_HEADER_SIZE
    };

    if data.len() < header_size {
        return Err(DictError::Format(
            "Matrix binary too short for header".to_string(),
        ));
    }

    let mut cursor = io::Cursor::new(data);

    let (lsize, rsize) = if is_v3 {
        cursor.set_position(4);
        let _version = cursor.read_u8().map_err(DictError::Io)?;
        let _flags = cursor.read_u8().map_err(DictError::Io)?;
        let _reserved = cursor.read_u16::<LittleEndian>().map_err(DictError::Io)?;
        let l = cursor.read_u32::<LittleEndian>().map_err(DictError::Io)? as usize;
        let r = cursor.read_u32::<LittleEndian>().map_err(DictError::Io)? as usize;
        (l, r)
    } else {
        let l = cursor.read_u16::<LittleEndian>().map_err(DictError::Io)? as usize;
        let r = cursor.read_u16::<LittleEndian>().map_err(DictError::Io)? as usize;
        (l, r)
    };

    Ok(MatrixHeader {
        lsize,
        rsize,
        header_size,
    })
}

/// 기본 비용 (연결 불가능한 경우)
pub const INVALID_CONNECTION_COST: i32 = i32::MAX;

/// 연접 비용 행렬 인터페이스
///
/// 형태소 간 연접 비용을 조회하는 인터페이스입니다.
/// mecab-ko-core의 `ConnectionCost` trait과 호환됩니다.
pub trait Matrix {
    /// 연접 비용 조회
    ///
    /// # Arguments
    ///
    /// * `right_id` - 이전 노드의 우문맥 ID (right context ID)
    /// * `left_id` - 현재 노드의 좌문맥 ID (left context ID)
    ///
    /// # Returns
    ///
    /// 연접 비용 (i32). 연결 불가능한 경우 `INVALID_CONNECTION_COST` 반환
    fn get(&self, right_id: u16, left_id: u16) -> i32;

    /// 좌문맥 크기
    fn left_size(&self) -> usize;

    /// 우문맥 크기
    fn right_size(&self) -> usize;

    /// 전체 엔트리 수
    fn entry_count(&self) -> usize {
        self.left_size() * self.right_size()
    }
}

/// 연접 비용 행렬 로더
///
/// 다양한 포맷에서 연접 비용 행렬을 로드합니다.
pub struct MatrixLoader;

impl MatrixLoader {
    /// 자동 포맷 감지 로드
    ///
    /// 파일 확장자에 따라 적절한 로더를 선택합니다.
    /// - `.def`: 텍스트 포맷
    /// - `.bin`: 바이너리 포맷
    /// - `.bin.zst`, `.zst`: 압축 바이너리 포맷
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 파싱할 수 없는 경우 에러를 반환합니다.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<DenseMatrix> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy();

        if path_str.ends_with(".def") {
            DenseMatrix::from_def_file(path)
        } else if path_str.ends_with(".zst") || path_str.ends_with(".bin.zst") {
            DenseMatrix::from_compressed_file(path)
        } else if path_str.ends_with(".bin") {
            DenseMatrix::from_bin_file(path)
        } else {
            // 기본: 바이너리 시도 후 텍스트 시도
            DenseMatrix::from_bin_file(path).or_else(|_| DenseMatrix::from_def_file(path))
        }
    }

    /// 메모리 맵으로 로드 (바이너리 파일만 지원)
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 메모리 맵을 생성할 수 없는 경우 에러를 반환합니다.
    pub fn load_mmap<P: AsRef<Path>>(path: P) -> Result<MmapMatrix> {
        MmapMatrix::from_file(path)
    }
}

/// 연접 비용 행렬을 위한 통합 타입
///
/// 다양한 행렬 구현을 하나의 타입으로 사용할 수 있습니다.
pub enum ConnectionMatrix {
    /// 밀집 행렬
    Dense(DenseMatrix),
    /// 희소 행렬
    Sparse(SparseMatrix),
    /// 메모리 맵 행렬
    Mmap(MmapMatrix),
}

impl ConnectionMatrix {
    /// 텍스트 파일에서 로드
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 파싱할 수 없는 경우 에러를 반환합니다.
    pub fn from_def_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self::Dense(DenseMatrix::from_def_file(path)?))
    }

    /// 바이너리 파일에서 로드
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 파싱할 수 없는 경우 에러를 반환합니다.
    pub fn from_bin_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self::Dense(DenseMatrix::from_bin_file(path)?))
    }

    /// 메모리 맵으로 로드
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 메모리 맵을 생성할 수 없는 경우 에러를 반환합니다.
    pub fn from_mmap_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self::Mmap(MmapMatrix::from_file(path)?))
    }

    /// 압축된 바이너리 파일에서 로드 (.zst)
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 압축 해제/파싱할 수 없는 경우 에러를 반환합니다.
    pub fn from_compressed_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self::Dense(DenseMatrix::from_compressed_file(path)?))
    }

    /// 자동 포맷 감지 로드
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 파싱할 수 없는 경우 에러를 반환합니다.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self::Dense(MatrixLoader::load(path)?))
    }
}

impl Matrix for ConnectionMatrix {
    #[inline(always)]
    fn get(&self, right_id: u16, left_id: u16) -> i32 {
        match self {
            Self::Dense(m) => m.get(right_id, left_id),
            Self::Sparse(m) => m.get(right_id, left_id),
            Self::Mmap(m) => m.get(right_id, left_id),
        }
    }

    fn left_size(&self) -> usize {
        match self {
            Self::Dense(m) => m.left_size(),
            Self::Sparse(m) => m.left_size(),
            Self::Mmap(m) => m.left_size(),
        }
    }

    fn right_size(&self) -> usize {
        match self {
            Self::Dense(m) => m.right_size(),
            Self::Sparse(m) => m.right_size(),
            Self::Mmap(m) => m.right_size(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_dense_matrix_new() {
        let matrix = DenseMatrix::new(10, 10, 0);
        assert_eq!(matrix.left_size(), 10);
        assert_eq!(matrix.right_size(), 10);
        assert_eq!(matrix.entry_count(), 100);
        assert_eq!(matrix.get(0, 0), 0);
    }

    #[test]
    fn test_dense_matrix_set_get() {
        let mut matrix = DenseMatrix::new(10, 10, 0);
        matrix.set(3, 5, 100);
        assert_eq!(matrix.get(3, 5), 100);
        assert_eq!(matrix.get(5, 3), 0);
    }

    #[test]
    fn test_dense_matrix_from_vec() {
        let costs = vec![1, 2, 3, 4, 5, 6];
        let matrix = DenseMatrix::from_vec(2, 3, costs).unwrap();
        // costs[right_id + lsize * left_id]
        // (0,0) = costs[0] = 1
        // (1,0) = costs[1] = 2
        // (0,1) = costs[2] = 3
        // (1,1) = costs[3] = 4
        // (0,2) = costs[4] = 5
        // (1,2) = costs[5] = 6
        assert_eq!(matrix.get(0, 0), 1);
        assert_eq!(matrix.get(1, 0), 2);
        assert_eq!(matrix.get(0, 1), 3);
        assert_eq!(matrix.get(1, 1), 4);
        assert_eq!(matrix.get(0, 2), 5);
        assert_eq!(matrix.get(1, 2), 6);
    }

    #[test]
    fn test_dense_matrix_from_vec_size_mismatch() {
        let costs = vec![1, 2, 3];
        let result = DenseMatrix::from_vec(2, 3, costs);
        assert!(result.is_err());
    }

    #[test]
    fn test_dense_matrix_boundary() {
        let matrix = DenseMatrix::new(10, 10, 0);
        // 경계 외 접근
        assert_eq!(matrix.get(100, 100), INVALID_CONNECTION_COST);
    }

    #[test]
    fn test_dense_matrix_def_reader() {
        let data = "3 3\n0 0 100\n1 1 200\n2 2 300\n";
        let reader = std::io::Cursor::new(data);
        let matrix = DenseMatrix::from_def_reader(reader).unwrap();

        assert_eq!(matrix.left_size(), 3);
        assert_eq!(matrix.right_size(), 3);
        assert_eq!(matrix.get(0, 0), 100);
        assert_eq!(matrix.get(1, 1), 200);
        assert_eq!(matrix.get(2, 2), 300);
        // 설정되지 않은 값은 i16::MAX
        assert_eq!(matrix.get(0, 1), i16::MAX as i32);
    }

    #[test]
    fn test_dense_matrix_binary_roundtrip() {
        let mut matrix = DenseMatrix::new(5, 5, 0);
        matrix.set(0, 0, 100);
        matrix.set(1, 2, -500);
        matrix.set(4, 4, 32767);

        let bytes = matrix.to_bin_bytes();
        let loaded = DenseMatrix::from_bin_bytes(&bytes).unwrap();

        assert_eq!(loaded.left_size(), 5);
        assert_eq!(loaded.right_size(), 5);
        assert_eq!(loaded.get(0, 0), 100);
        assert_eq!(loaded.get(1, 2), -500);
        assert_eq!(loaded.get(4, 4), 32767);
    }

    #[test]
    fn test_sparse_matrix() {
        let mut sparse = SparseMatrix::new(100, 100, 0);
        sparse.set(10, 20, 500);
        sparse.set(50, 50, -100);

        assert_eq!(sparse.get(10, 20), 500);
        assert_eq!(sparse.get(50, 50), -100);
        assert_eq!(sparse.get(0, 0), 0); // 기본값

        assert_eq!(sparse.entry_count_stored(), 2);
        assert!(sparse.sparsity() > 0.99); // 거의 희소
    }

    #[test]
    fn test_sparse_dense_conversion() {
        let mut dense = DenseMatrix::new(10, 10, 0);
        dense.set(3, 3, 100);
        dense.set(5, 7, 200);

        let sparse = SparseMatrix::from_dense(&dense, 0);
        assert_eq!(sparse.entry_count_stored(), 2);
        assert_eq!(sparse.get(3, 3), 100);
        assert_eq!(sparse.get(5, 7), 200);

        let converted = sparse.to_dense();
        assert_eq!(converted.get(3, 3), 100);
        assert_eq!(converted.get(5, 7), 200);
        assert_eq!(converted.get(0, 0), 0);
    }

    #[test]
    fn test_memory_size() {
        let dense = DenseMatrix::new(100, 100, 0);
        let mem_size = dense.memory_size();
        // 최소 20000 바이트 (100*100*2)
        assert!(mem_size >= 20000);

        let sparse = SparseMatrix::new(100, 100, 0);
        let sparse_size = sparse.memory_size();
        // 희소 행렬은 훨씬 작음
        assert!(sparse_size < mem_size);
    }

    #[test]
    fn test_connection_matrix_enum() {
        let dense = DenseMatrix::new(5, 5, 100);
        let matrix = ConnectionMatrix::Dense(dense);

        assert_eq!(matrix.left_size(), 5);
        assert_eq!(matrix.right_size(), 5);
        assert_eq!(matrix.get(0, 0), 100);
    }

    #[test]
    fn test_large_matrix() {
        // mecab-ko-dic의 실제 크기 (약 2800 x 2800)
        let matrix = DenseMatrix::new(178, 178, 0);
        assert_eq!(matrix.entry_count(), 178 * 178);
        assert_eq!(
            matrix.memory_size(),
            std::mem::size_of::<DenseMatrix>() + 178 * 178 * 2
        );
    }

    #[test]
    fn test_def_with_comments_and_empty_lines() {
        let data = "2 2\n# This is a comment\n\n0 0 10\n0 1 20\n\n1 0 30\n1 1 40\n";
        let reader = std::io::Cursor::new(data);
        let matrix = DenseMatrix::from_def_reader(reader).unwrap();

        assert_eq!(matrix.get(0, 0), 10);
        assert_eq!(matrix.get(0, 1), 20);
        assert_eq!(matrix.get(1, 0), 30);
        assert_eq!(matrix.get(1, 1), 40);
    }

    #[test]
    fn test_v3_header_roundtrip() {
        let mut matrix = DenseMatrix::new(5, 5, 0);
        matrix.set(0, 0, 42);
        matrix.set(2, 3, -999);
        matrix.set(4, 4, 32767);

        let bytes = matrix.to_bin_bytes_v3();
        assert_eq!(&bytes[..4], b"MKM3");
        assert_eq!(bytes[4], 1);
        assert_eq!(bytes[5], 0);

        let loaded = DenseMatrix::from_bin_bytes(&bytes).unwrap();
        assert_eq!(loaded.left_size(), 5);
        assert_eq!(loaded.right_size(), 5);
        assert_eq!(loaded.get(0, 0), 42);
        assert_eq!(loaded.get(2, 3), -999);
        assert_eq!(loaded.get(4, 4), 32767);
    }

    #[test]
    fn test_v2_backward_compat() {
        let mut matrix = DenseMatrix::new(4, 4, 0);
        matrix.set(1, 2, 777);

        let bytes = matrix.to_bin_bytes();
        assert_ne!(&bytes[..4], b"MKM3");

        let loaded = DenseMatrix::from_bin_bytes(&bytes).unwrap();
        assert_eq!(loaded.left_size(), 4);
        assert_eq!(loaded.right_size(), 4);
        assert_eq!(loaded.get(1, 2), 777);
    }

    #[test]
    fn test_v3_large_dimensions() {
        let lsize = (u16::MAX as usize) + 1;
        let rsize = 1;
        let costs = vec![0i16; lsize * rsize];
        let matrix = DenseMatrix::from_vec(lsize, rsize, costs).unwrap();

        let bytes = matrix.to_bin_bytes_v3();
        let loaded = DenseMatrix::from_bin_bytes(&bytes).unwrap();
        assert_eq!(loaded.left_size(), lsize);
        assert_eq!(loaded.right_size(), rsize);
    }
}
