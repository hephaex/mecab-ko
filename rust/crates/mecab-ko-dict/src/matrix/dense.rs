//! 밀집 연접 비용 행렬 (Dense Connection Cost Matrix)

use std::io::{self, BufRead, BufReader};
#[cfg(feature = "zstd")]
use std::io::{Read, Write as IoWrite};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::error::{DictError, Result};

use super::{
    parse_matrix_header, Matrix, INVALID_CONNECTION_COST, MATRIX_HEADER_SIZE, MKM3_HEADER_SIZE,
    MKM3_MAGIC,
};

/// 밀집 연접 비용 행렬 (Dense Matrix)
///
/// 모든 연접 비용을 메모리에 저장하는 구현입니다.
/// 희소 행렬이 아닌 경우에 적합합니다.
#[derive(Debug, Clone)]
pub struct DenseMatrix {
    /// 좌문맥 크기
    pub(super) lsize: usize,
    /// 우문맥 크기
    pub(super) rsize: usize,
    /// 비용 배열 (row-major: costs[`right_id` + lsize * `left_id`])
    pub(super) costs: Vec<i16>,
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
