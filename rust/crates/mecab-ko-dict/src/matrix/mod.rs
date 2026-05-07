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

use std::io::{self, BufRead, BufReader};
#[cfg(feature = "zstd")]
use std::io::{Read, Write as IoWrite};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::error::{DictError, Result};

// SIMD 최적화 모듈
#[cfg(feature = "simd")]
pub mod simd;

#[cfg(feature = "simd")]
pub use simd::SimdMatrix;

const MATRIX_HEADER_SIZE: usize = 4;

const MKM3_MAGIC: &[u8; 4] = b"MKM3";
const MKM3_HEADER_SIZE: usize = 16;

/// 행렬 헤더 정보
struct MatrixHeader {
    /// 좌문맥 크기
    lsize: usize,
    /// 우문맥 크기
    rsize: usize,
    /// 헤더 크기 (v2: 4, v3: 16)
    header_size: usize,
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
fn parse_matrix_header(data: &[u8]) -> Result<MatrixHeader> {
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

/// 밀집 연접 비용 행렬 (Dense Matrix)
///
/// 모든 연접 비용을 메모리에 저장하는 구현입니다.
/// 희소 행렬이 아닌 경우에 적합합니다.
#[derive(Debug, Clone)]
pub struct DenseMatrix {
    /// 좌문맥 크기
    lsize: usize,
    /// 우문맥 크기
    rsize: usize,
    /// 비용 배열 (row-major: costs[`right_id` + lsize * `left_id`])
    costs: Vec<i16>,
}

impl DenseMatrix {
    /// 새로운 밀집 행렬 생성 (모든 값을 기본값으로 초기화)
    ///
    /// # Arguments
    ///
    /// * `lsize` - 좌문맥 크기
    /// * `rsize` - 우문맥 크기
    /// * `default_cost` - 기본 비용 값
    #[must_use]
    pub fn new(lsize: usize, rsize: usize, default_cost: i16) -> Self {
        let costs = vec![default_cost; lsize * rsize];
        Self {
            lsize,
            rsize,
            costs,
        }
    }

    /// 기존 비용 벡터로 밀집 행렬 생성
    ///
    /// # Arguments
    ///
    /// * `lsize` - 좌문맥 크기
    /// * `rsize` - 우문맥 크기
    /// * `costs` - 비용 배열
    ///
    /// # Returns
    ///
    /// 성공 시 `DenseMatrix`, 크기 불일치 시 에러
    ///
    /// # Errors
    ///
    /// 비용 배열의 길이가 `lsize * rsize`와 일치하지 않으면 에러를 반환합니다.
    pub fn from_vec(lsize: usize, rsize: usize, costs: Vec<i16>) -> Result<Self> {
        let expected_size = lsize * rsize;
        if costs.len() != expected_size {
            return Err(DictError::Format(format!(
                "Matrix size mismatch: expected {} entries, got {}",
                expected_size,
                costs.len()
            )));
        }
        Ok(Self {
            lsize,
            rsize,
            costs,
        })
    }

    /// 비용 설정
    ///
    /// # Arguments
    ///
    /// * `right_id` - 우문맥 ID
    /// * `left_id` - 좌문맥 ID
    /// * `cost` - 비용 값
    pub fn set(&mut self, right_id: u16, left_id: u16, cost: i16) {
        let index = right_id as usize + self.lsize * left_id as usize;
        if index < self.costs.len() {
            self.costs[index] = cost;
        }
    }

    /// 텍스트 파일(matrix.def)에서 로드
    ///
    /// # Format
    ///
    /// ```text
    /// <lsize> <rsize>
    /// <right_id> <left_id> <cost>
    /// ...
    /// ```
    ///
    /// # Arguments
    ///
    /// * `path` - matrix.def 파일 경로
    ///
    /// # Errors
    ///
    /// 파일을 읽을 수 없거나 형식이 잘못된 경우 에러를 반환합니다.
    pub fn from_def_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref()).map_err(DictError::Io)?;
        let reader = BufReader::new(file);
        Self::from_def_reader(reader)
    }

    /// 텍스트 리더에서 로드
    ///
    /// # Errors
    ///
    /// 리더에서 데이터를 읽을 수 없거나 형식이 잘못된 경우 에러를 반환합니다.
    pub fn from_def_reader<R: BufRead>(mut reader: R) -> Result<Self> {
        // 첫 줄: 크기 정보
        let mut first_line = String::new();
        reader.read_line(&mut first_line).map_err(DictError::Io)?;

        let sizes: Vec<usize> = first_line
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        if sizes.len() != 2 {
            return Err(DictError::Format(
                "Invalid matrix header: expected 'lsize rsize'".to_string(),
            ));
        }

        let lsize = sizes[0];
        let rsize = sizes[1];

        // 기본값으로 초기화 (i16::MAX는 연결 불가능을 의미)
        let mut matrix = Self::new(lsize, rsize, i16::MAX);

        // 나머지 줄: 연접 비용
        for line in reader.lines() {
            let line = line.map_err(DictError::Io)?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 3 {
                continue;
            }

            let right_id: u16 = parts[0]
                .parse()
                .map_err(|_| DictError::Format(format!("Invalid right_id: {}", parts[0])))?;
            let left_id: u16 = parts[1]
                .parse()
                .map_err(|_| DictError::Format(format!("Invalid left_id: {}", parts[1])))?;
            let cost: i16 = parts[2]
                .parse()
                .map_err(|_| DictError::Format(format!("Invalid cost: {}", parts[2])))?;

            matrix.set(right_id, left_id, cost);
        }

        Ok(matrix)
    }

    /// 바이너리 파일(matrix.bin)에서 로드
    ///
    /// # Format
    ///
    /// ```text
    /// [2 bytes] lsize (little-endian u16)
    /// [2 bytes] rsize (little-endian u16)
    /// [lsize * rsize * 2 bytes] costs (little-endian i16 array)
    /// ```
    ///
    /// # Errors
    ///
    /// 파일을 읽을 수 없거나 형식이 잘못된 경우 에러를 반환합니다.
    pub fn from_bin_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path.as_ref()).map_err(DictError::Io)?;
        Self::from_bin_bytes(&data)
    }

    /// 바이너리 바이트에서 로드
    ///
    /// # Errors
    ///
    /// 데이터가 유효한 바이너리 형식이 아닌 경우 에러를 반환합니다.
    pub fn from_bin_bytes(data: &[u8]) -> Result<Self> {
        let header = parse_matrix_header(data)?;

        let expected_size = header.lsize * header.rsize * 2;
        let data_size = data.len() - header.header_size;

        if data_size != expected_size {
            return Err(DictError::Format(format!(
                "Matrix data size mismatch: expected {expected_size} bytes, got {data_size}"
            )));
        }

        let mut cursor = io::Cursor::new(data);
        cursor.set_position(header.header_size as u64);

        let mut costs = Vec::with_capacity(header.lsize * header.rsize);
        for _ in 0..(header.lsize * header.rsize) {
            costs.push(cursor.read_i16::<LittleEndian>().map_err(DictError::Io)?);
        }

        Ok(Self {
            lsize: header.lsize,
            rsize: header.rsize,
            costs,
        })
    }

    /// v3 포맷(MKM3)으로 직렬화
    #[must_use]
    pub fn to_bin_bytes_v3(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(MKM3_HEADER_SIZE + self.costs.len() * 2);

        buf.extend_from_slice(MKM3_MAGIC);
        buf.push(1);
        buf.push(0);
        buf.write_u16::<LittleEndian>(0).ok();
        #[allow(clippy::cast_possible_truncation)]
        buf.write_u32::<LittleEndian>(self.lsize as u32).ok();
        #[allow(clippy::cast_possible_truncation)]
        buf.write_u32::<LittleEndian>(self.rsize as u32).ok();

        for &cost in &self.costs {
            buf.write_i16::<LittleEndian>(cost).ok();
        }

        buf
    }

    /// 압축된 바이너리 파일(matrix.bin.zst)에서 로드
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 압축 해제할 수 없는 경우 에러를 반환합니다.
    #[cfg(feature = "zstd")]
    pub fn from_compressed_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref()).map_err(DictError::Io)?;
        let decoder = zstd::Decoder::new(file).map_err(DictError::Io)?;
        let mut data = Vec::new();
        BufReader::new(decoder)
            .read_to_end(&mut data)
            .map_err(DictError::Io)?;
        Self::from_bin_bytes(&data)
    }

    /// 압축된 바이너리 파일에서 로드 (zstd feature 비활성화 시)
    ///
    /// # Errors
    ///
    /// zstd feature가 비활성화된 경우 항상 에러를 반환합니다.
    #[cfg(not(feature = "zstd"))]
    pub fn from_compressed_file<P: AsRef<Path>>(_path: P) -> Result<Self> {
        Err(DictError::Format(
            "zstd feature is not enabled. Use uncompressed files or enable the 'zstd' feature."
                .to_string(),
        ))
    }

    /// 바이너리 형식으로 저장
    #[must_use]
    pub fn to_bin_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(MATRIX_HEADER_SIZE + self.costs.len() * 2);

        // 헤더
        #[allow(clippy::cast_possible_truncation)]
        buf.write_u16::<LittleEndian>(self.lsize as u16).ok();
        #[allow(clippy::cast_possible_truncation)]
        buf.write_u16::<LittleEndian>(self.rsize as u16).ok();

        // 데이터
        for &cost in &self.costs {
            buf.write_i16::<LittleEndian>(cost).ok();
        }

        buf
    }

    /// 바이너리 파일로 저장
    ///
    /// # Errors
    ///
    /// 파일을 쓸 수 없는 경우 에러를 반환합니다.
    pub fn to_bin_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = self.to_bin_bytes();
        std::fs::write(path.as_ref(), data).map_err(DictError::Io)
    }

    /// 압축된 바이너리 파일로 저장
    ///
    /// # Errors
    ///
    /// 파일을 쓰거나 압축할 수 없는 경우 에러를 반환합니다.
    #[cfg(feature = "zstd")]
    pub fn to_compressed_file<P: AsRef<Path>>(&self, path: P, level: i32) -> Result<()> {
        let data = self.to_bin_bytes();
        let file = std::fs::File::create(path.as_ref()).map_err(DictError::Io)?;
        let mut encoder = zstd::Encoder::new(file, level).map_err(DictError::Io)?;
        encoder.write_all(&data).map_err(DictError::Io)?;
        encoder.finish().map_err(DictError::Io)?;
        Ok(())
    }

    /// 압축된 바이너리 파일로 저장 (zstd feature 비활성화 시)
    ///
    /// # Errors
    ///
    /// zstd feature가 비활성화된 경우 항상 에러를 반환합니다.
    #[cfg(not(feature = "zstd"))]
    pub fn to_compressed_file<P: AsRef<Path>>(&self, _path: P, _level: i32) -> Result<()> {
        Err(DictError::Format(
            "zstd feature is not enabled. Use uncompressed files or enable the 'zstd' feature."
                .to_string(),
        ))
    }

    /// 원본 비용 배열 참조
    #[must_use]
    pub fn costs(&self) -> &[i16] {
        &self.costs
    }

    /// 메모리 사용량 (바이트)
    #[must_use]
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>() + self.costs.len() * std::mem::size_of::<i16>()
    }
}

impl Matrix for DenseMatrix {
    #[inline(always)]
    fn get(&self, right_id: u16, left_id: u16) -> i32 {
        let index = right_id as usize + self.lsize * left_id as usize;
        if index < self.costs.len() {
            i32::from(self.costs[index])
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
