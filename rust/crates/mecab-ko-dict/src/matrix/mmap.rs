//! 메모리 맵 연접 비용 행렬 (Memory-Mapped Connection Cost Matrix)

use std::path::Path;

use crate::error::{DictError, Result};

use super::{dense::DenseMatrix, parse_matrix_header, Matrix, INVALID_CONNECTION_COST};

/// 메모리 맵 연접 비용 행렬 (Memory-Mapped Matrix)
///
/// 대용량 행렬을 메모리 맵으로 로드하여 효율적으로 접근합니다.
/// 프로세스 간 메모리 공유가 가능합니다.
///
/// # Safety
///
/// 이 구조체는 메모리 맵을 사용하므로 내부적으로 unsafe 코드가 필요합니다.
/// 파일이 외부에서 수정되지 않아야 합니다.
pub struct MmapMatrix {
    /// 좌문맥 크기
    lsize: usize,
    /// 우문맥 크기
    rsize: usize,
    /// 헤더 크기 (v2: 4, v3: 16)
    header_size: usize,
    /// 메모리 맵
    mmap: memmap2::Mmap,
}

impl MmapMatrix {
    /// 바이너리 파일에서 메모리 맵으로 로드
    ///
    /// # Safety
    ///
    /// 파일이 외부에서 수정되지 않아야 합니다.
    /// memmap2는 파일을 메모리에 매핑하며, 이는 본질적으로 unsafe입니다.
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 메모리 맵을 생성할 수 없는 경우 에러를 반환합니다.
    #[allow(unsafe_code)]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref()).map_err(DictError::Io)?;

        // SAFETY: 파일이 열려 있는 동안 수정되지 않는다고 가정
        // memmap2::Mmap::map은 파일 내용이 변경되지 않을 때 안전합니다.
        let mmap = unsafe { memmap2::Mmap::map(&file).map_err(DictError::Io)? };

        // 헤더만 파싱할 만큼만 전달
        let header = parse_matrix_header(&mmap)?;

        let expected_size = header.header_size + header.lsize * header.rsize * 2;
        if mmap.len() != expected_size {
            return Err(DictError::Format(format!(
                "Matrix file size mismatch: expected {} bytes, got {}",
                expected_size,
                mmap.len()
            )));
        }

        Ok(Self {
            lsize: header.lsize,
            rsize: header.rsize,
            header_size: header.header_size,
            mmap,
        })
    }

    /// 압축된 파일에서 로드 (메모리에 전체 압축 해제)
    ///
    /// 압축 파일은 메모리 맵이 아닌 전체 압축 해제 후 로드됩니다.
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 압축 해제할 수 없는 경우 에러를 반환합니다.
    pub fn from_compressed_file<P: AsRef<Path>>(path: P) -> Result<DenseMatrix> {
        // 압축 파일은 DenseMatrix로 로드
        DenseMatrix::from_compressed_file(path)
    }

    #[inline]
    const fn offset(&self, right_id: u16, left_id: u16) -> usize {
        self.header_size + (right_id as usize + self.lsize * left_id as usize) * 2
    }
}

impl Matrix for MmapMatrix {
    #[inline(always)]
    fn get(&self, right_id: u16, left_id: u16) -> i32 {
        let offset = self.offset(right_id, left_id);
        if offset + 2 <= self.mmap.len() {
            let bytes = [self.mmap[offset], self.mmap[offset + 1]];
            i32::from(i16::from_le_bytes(bytes))
        } else {
            INVALID_CONNECTION_COST
        }
    }

    fn left_size(&self) -> usize {
        self.lsize
    }

    fn right_size(&self) -> usize {
        self.rsize
    }
}
