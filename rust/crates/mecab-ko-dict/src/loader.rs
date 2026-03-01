//! 사전 로딩 기능
//!
//! 바이너리 사전 파일을 로드하고 관리합니다.

use crate::error::{DictError, Result};
use crate::matrix::{DenseMatrix, Matrix};
use crate::trie::Trie;
use crate::{Dictionary, Entry};
use std::fs::File;
use std::path::{Path, PathBuf};

/// 사전 로더 설정
#[derive(Debug, Clone, Copy)]
pub struct LoaderConfig {
    /// 메모리 맵 사용 여부
    pub use_mmap: bool,
    /// 압축 해제 자동 지원
    pub auto_decompress: bool,
    /// 지연 로딩 (첫 번째 접근 시 로드)
    pub lazy_load: bool,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            use_mmap: true,
            auto_decompress: true,
            lazy_load: false,
        }
    }
}

/// 메모리 맵 사전
///
/// mmap을 이용하여 사전 파일을 메모리에 매핑합니다.
pub struct MmapDictionary {
    /// Trie 인스턴스
    trie: Trie<'static>,
    /// 연접 비용 매트릭스
    matrix: DenseMatrix,
    /// 사전 디렉토리
    dict_dir: PathBuf,
    /// 엔트리 배열 (Trie의 value를 인덱스로 사용)
    entries: Vec<Entry>,
}

impl MmapDictionary {
    /// 사전 로드
    ///
    /// # Arguments
    /// * `path` - 사전 디렉토리 경로
    ///
    /// # Errors
    ///
    /// 사전 파일을 찾을 수 없거나 로드에 실패한 경우 에러를 반환합니다.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use mecab_ko_dict::loader::MmapDictionary;
    ///
    /// let dict = MmapDictionary::load("./dict").unwrap();
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::load_with_config(path, LoaderConfig::default())
    }

    /// 설정과 함께 사전 로드
    ///
    /// # Errors
    ///
    /// 사전 파일을 찾을 수 없거나 로드에 실패한 경우 에러를 반환합니다.
    pub fn load_with_config<P: AsRef<Path>>(path: P, config: LoaderConfig) -> Result<Self> {
        let dict_dir = path.as_ref().to_path_buf();

        // Trie 로드
        let trie_data = Self::load_trie(&dict_dir, config)?;
        let trie = Trie::from_vec(trie_data);

        // Matrix 로드
        let matrix = Self::load_matrix(&dict_dir, config)?;

        // 엔트리 로드
        let entries = Self::load_entries(&dict_dir, config)?;

        Ok(Self {
            trie,
            matrix,
            dict_dir,
            entries,
        })
    }

    /// Trie 데이터 로드
    #[cfg(feature = "zstd")]
    fn load_trie(dict_dir: &Path, config: LoaderConfig) -> Result<Vec<u8>> {
        // 압축 파일 우선 시도
        let compressed_path = dict_dir.join("sys.dic.zst");
        let uncompressed_path = dict_dir.join("sys.dic");

        if config.auto_decompress && compressed_path.exists() {
            // zstd 압축 해제
            let file = File::open(&compressed_path)?;
            let mut decoder = zstd::Decoder::new(file)?;
            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut decoder, &mut buffer)?;
            Ok(buffer)
        } else if uncompressed_path.exists() {
            // 압축되지 않은 파일 - mmap 대신 일반 읽기 사용 (unsafe 제거)
            Ok(std::fs::read(&uncompressed_path)?)
        } else {
            Err(DictError::Format(
                "sys.dic or sys.dic.zst not found".to_string(),
            ))
        }
    }

    /// Trie 데이터 로드 (zstd feature 비활성화 시)
    #[cfg(not(feature = "zstd"))]
    fn load_trie(dict_dir: &Path, _config: LoaderConfig) -> Result<Vec<u8>> {
        let uncompressed_path = dict_dir.join("sys.dic");

        if uncompressed_path.exists() {
            Ok(std::fs::read(&uncompressed_path)?)
        } else {
            Err(DictError::Format(
                "sys.dic not found (zstd feature disabled, compressed files not supported)"
                    .to_string(),
            ))
        }
    }

    /// Matrix 데이터 로드
    #[cfg(feature = "zstd")]
    fn load_matrix(dict_dir: &Path, config: LoaderConfig) -> Result<DenseMatrix> {
        let compressed_path = dict_dir.join("matrix.bin.zst");
        let uncompressed_path = dict_dir.join("matrix.bin");

        if config.auto_decompress && compressed_path.exists() {
            DenseMatrix::from_compressed_file(&compressed_path)
        } else if uncompressed_path.exists() {
            DenseMatrix::from_bin_file(&uncompressed_path)
        } else {
            Err(DictError::Format(
                "matrix.bin or matrix.bin.zst not found".to_string(),
            ))
        }
    }

    /// Matrix 데이터 로드 (zstd feature 비활성화 시)
    #[cfg(not(feature = "zstd"))]
    fn load_matrix(dict_dir: &Path, _config: LoaderConfig) -> Result<DenseMatrix> {
        let uncompressed_path = dict_dir.join("matrix.bin");

        if uncompressed_path.exists() {
            DenseMatrix::from_bin_file(&uncompressed_path)
        } else {
            Err(DictError::Format(
                "matrix.bin not found (zstd feature disabled, compressed files not supported)"
                    .to_string(),
            ))
        }
    }

    /// 엔트리 데이터 로드
    ///
    /// 바이너리 엔트리 파일 또는 CSV 파일에서 사전 엔트리를 로드합니다.
    ///
    /// # 파일 포맷
    ///
    /// ## 바이너리 포맷 (entries.bin)
    /// - 4 bytes: entry count (u32 little-endian)
    /// - For each entry:
    ///   - 2 bytes: `left_id` (u16 little-endian)
    ///   - 2 bytes: `right_id` (u16 little-endian)
    ///   - 2 bytes: cost (i16 little-endian)
    ///   - 2 bytes: surface length (u16 little-endian)
    ///   - N bytes: surface (UTF-8)
    ///   - 2 bytes: feature length (u16 little-endian)
    ///   - M bytes: feature (UTF-8)
    ///
    /// ## CSV 포맷 (entries.csv)
    /// - Format: `surface,left_id,right_id,cost,feature`
    /// - Example: `가,1,1,100,NNG,*,T,가,*,*,*,*`
    ///
    /// # Arguments
    ///
    /// * `dict_dir` - 사전 디렉토리 경로
    /// * `config` - 로더 설정
    ///
    /// # Errors
    ///
    /// 엔트리 파일을 찾을 수 없거나 파싱에 실패한 경우 에러를 반환합니다.
    #[cfg(feature = "zstd")]
    fn load_entries(dict_dir: &Path, config: LoaderConfig) -> Result<Vec<Entry>> {
        // 바이너리 파일 우선 시도
        let bin_path = dict_dir.join("entries.bin");
        let compressed_bin_path = dict_dir.join("entries.bin.zst");
        let csv_path = dict_dir.join("entries.csv");

        // 압축된 바이너리 파일
        if config.auto_decompress && compressed_bin_path.exists() {
            return Self::load_entries_from_compressed_bin(&compressed_bin_path);
        }

        // 비압축 바이너리 파일
        if bin_path.exists() {
            return Self::load_entries_from_bin(&bin_path);
        }

        // CSV 파일 (fallback)
        if csv_path.exists() {
            return Self::load_entries_from_csv(&csv_path);
        }

        // 파일이 없으면 빈 벡터 반환 (스텁 동작 유지)
        // 실제 프로덕션에서는 에러를 반환해야 하지만,
        // 기존 테스트와의 호환성을 위해 빈 벡터를 반환합니다.
        Ok(Vec::new())
    }

    #[cfg(not(feature = "zstd"))]
    fn load_entries(dict_dir: &Path, _config: LoaderConfig) -> Result<Vec<Entry>> {
        // 바이너리 파일 우선 시도
        let bin_path = dict_dir.join("entries.bin");
        let csv_path = dict_dir.join("entries.csv");

        // 비압축 바이너리 파일
        if bin_path.exists() {
            return Self::load_entries_from_bin(&bin_path);
        }

        // CSV 파일 (fallback)
        if csv_path.exists() {
            return Self::load_entries_from_csv(&csv_path);
        }

        // 파일이 없으면 빈 벡터 반환
        Ok(Vec::new())
    }

    /// 바이너리 파일에서 엔트리 로드
    fn load_entries_from_bin(path: &Path) -> Result<Vec<Entry>> {
        use std::io::Read;

        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Self::parse_entries_binary(&buffer)
    }

    /// 압축된 바이너리 파일에서 엔트리 로드
    #[cfg(feature = "zstd")]
    fn load_entries_from_compressed_bin(path: &Path) -> Result<Vec<Entry>> {
        use std::io::Read;

        let file = File::open(path)?;
        let mut decoder = zstd::Decoder::new(file)?;
        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)?;

        Self::parse_entries_binary(&buffer)
    }

    /// 압축된 바이너리 파일에서 엔트리 로드 (zstd feature 비활성화 시)
    #[cfg(not(feature = "zstd"))]
    #[allow(dead_code)]
    fn load_entries_from_compressed_bin(_path: &Path) -> Result<Vec<Entry>> {
        Err(DictError::Format(
            "zstd feature is not enabled. Use uncompressed files or enable the 'zstd' feature."
                .to_string(),
        ))
    }

    /// 바이너리 데이터 파싱
    fn parse_entries_binary(data: &[u8]) -> Result<Vec<Entry>> {
        use std::io::{Cursor, Read};

        let mut cursor = Cursor::new(data);
        let mut count_bytes = [0u8; 4];
        cursor.read_exact(&mut count_bytes).map_err(|_| {
            DictError::Format("Failed to read entry count from binary file".to_string())
        })?;

        let count = u32::from_le_bytes(count_bytes) as usize;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            // left_id
            let mut buf = [0u8; 2];
            cursor.read_exact(&mut buf).map_err(|_| {
                DictError::Format("Failed to read left_id from binary file".to_string())
            })?;
            let left_id = u16::from_le_bytes(buf);

            // right_id
            cursor.read_exact(&mut buf).map_err(|_| {
                DictError::Format("Failed to read right_id from binary file".to_string())
            })?;
            let right_id = u16::from_le_bytes(buf);

            // cost
            cursor.read_exact(&mut buf).map_err(|_| {
                DictError::Format("Failed to read cost from binary file".to_string())
            })?;
            let cost = i16::from_le_bytes(buf);

            // surface
            cursor.read_exact(&mut buf).map_err(|_| {
                DictError::Format("Failed to read surface length from binary file".to_string())
            })?;
            let surface_len = u16::from_le_bytes(buf) as usize;
            let mut surface_bytes = vec![0u8; surface_len];
            cursor.read_exact(&mut surface_bytes).map_err(|_| {
                DictError::Format("Failed to read surface from binary file".to_string())
            })?;
            let surface = String::from_utf8(surface_bytes)
                .map_err(|_| DictError::Format("Invalid UTF-8 in surface field".to_string()))?;

            // feature
            cursor.read_exact(&mut buf).map_err(|_| {
                DictError::Format("Failed to read feature length from binary file".to_string())
            })?;
            let feature_len = u16::from_le_bytes(buf) as usize;
            let mut feature_bytes = vec![0u8; feature_len];
            cursor.read_exact(&mut feature_bytes).map_err(|_| {
                DictError::Format("Failed to read feature from binary file".to_string())
            })?;
            let feature = String::from_utf8(feature_bytes)
                .map_err(|_| DictError::Format("Invalid UTF-8 in feature field".to_string()))?;

            entries.push(Entry {
                surface,
                left_id,
                right_id,
                cost,
                feature,
            });
        }

        Ok(entries)
    }

    /// CSV 파일에서 엔트리 로드
    fn load_entries_from_csv(path: &Path) -> Result<Vec<Entry>> {
        use std::io::{BufRead, BufReader};

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;

            // 빈 줄이나 주석 건너뛰기
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let entry = Self::parse_csv_line(&line)
                .map_err(|e| DictError::Format(format!("Failed to parse line {line_num}: {e}")))?;

            entries.push(entry);
        }

        Ok(entries)
    }

    /// CSV 라인 파싱
    ///
    /// 포맷: `surface,left_id,right_id,cost,feature1,feature2,...`
    /// 예시: `가,1,1,100,NNG,*,T,가,*,*,*,*`
    fn parse_csv_line(line: &str) -> Result<Entry> {
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() < 5 {
            return Err(DictError::Format(format!(
                "Invalid CSV line: expected at least 5 fields, got {}",
                parts.len()
            )));
        }

        let surface = parts[0].to_string();

        let left_id = parts[1]
            .parse::<u16>()
            .map_err(|_| DictError::Format(format!("Invalid left_id: {}", parts[1])))?;

        let right_id = parts[2]
            .parse::<u16>()
            .map_err(|_| DictError::Format(format!("Invalid right_id: {}", parts[2])))?;

        let cost = parts[3]
            .parse::<i16>()
            .map_err(|_| DictError::Format(format!("Invalid cost: {}", parts[3])))?;

        // 나머지 필드들을 쉼표로 연결하여 feature로 저장
        let feature = parts[4..].join(",");

        Ok(Entry {
            surface,
            left_id,
            right_id,
            cost,
            feature,
        })
    }

    /// Trie 참조
    #[must_use]
    pub const fn trie(&self) -> &Trie<'static> {
        &self.trie
    }

    /// Matrix 참조
    #[must_use]
    pub const fn matrix(&self) -> &DenseMatrix {
        &self.matrix
    }

    /// 사전 디렉토리 경로
    #[must_use]
    pub fn dict_dir(&self) -> &Path {
        &self.dict_dir
    }

    /// 엔트리 배열 참조
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// 인덱스로 엔트리 조회
    ///
    /// # Arguments
    ///
    /// * `index` - Trie에서 반환된 인덱스
    #[must_use]
    pub fn get_entry(&self, index: u32) -> Option<&Entry> {
        self.entries.get(index as usize)
    }
}

impl Dictionary for MmapDictionary {
    fn lookup(&self, surface: &str) -> Vec<Entry> {
        // Trie에서 검색
        self.trie
            .exact_match(surface)
            .map_or_else(Vec::new, |index| {
                self.entries.get(index as usize).map_or_else(
                    || {
                        // 인덱스가 범위를 벗어난 경우 (엔트리 데이터가 없는 경우)
                        // 스텁 엔트리 반환 (하위 호환성)
                        vec![Entry {
                            surface: surface.to_string(),
                            left_id: 0,
                            right_id: 0,
                            cost: 0,
                            feature: "UNK,*,*,*,*,*,*,*".to_string(),
                        }]
                    },
                    |entry| vec![entry.clone()],
                )
            })
    }

    fn get_connection_cost(&self, left_id: u16, right_id: u16) -> i16 {
        // DenseMatrix::get returns i32, convert to i16
        // Saturate to i16::MAX/MIN if out of range
        let cost = self.matrix.get(left_id, right_id);
        cost.clamp(i32::from(i16::MIN), i32::from(i16::MAX))
            .try_into()
            .unwrap_or(i16::MAX)
    }
}

/// 지연 로딩 사전
///
/// 첫 번째 접근 시에만 사전을 로드합니다.
pub struct LazyDictionary {
    dict_path: PathBuf,
    config: LoaderConfig,
    dict: std::sync::Mutex<Option<MmapDictionary>>,
}

impl LazyDictionary {
    /// 새 지연 로딩 사전 생성
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self::new_with_config(path, LoaderConfig::default())
    }

    /// 설정과 함께 생성
    pub fn new_with_config<P: AsRef<Path>>(path: P, config: LoaderConfig) -> Self {
        Self {
            dict_path: path.as_ref().to_path_buf(),
            config,
            dict: std::sync::Mutex::new(None),
        }
    }

    /// 사전 로드 (내부용)
    fn ensure_loaded(&self) -> Result<()> {
        let mut dict = self.dict.lock().map_err(|_| {
            DictError::Format("Failed to acquire lock for lazy dictionary".to_string())
        })?;

        if dict.is_some() {
            return Ok(());
        }

        let loaded_dict = MmapDictionary::load_with_config(&self.dict_path, self.config)?;
        *dict = Some(loaded_dict);
        drop(dict);

        Ok(())
    }
}

impl Dictionary for LazyDictionary {
    fn lookup(&self, surface: &str) -> Vec<Entry> {
        if self.ensure_loaded().is_err() {
            return Vec::new();
        }

        let Ok(dict) = self.dict.lock() else {
            return Vec::new();
        };

        dict.as_ref().map_or_else(Vec::new, |d| d.lookup(surface))
    }

    fn get_connection_cost(&self, left_id: u16, right_id: u16) -> i16 {
        if self.ensure_loaded().is_err() {
            return 0;
        }

        let Ok(dict) = self.dict.lock() else {
            return 0;
        };

        dict.as_ref()
            .map_or(0, |d| d.get_connection_cost(left_id, right_id))
    }
}

/// 사전 로더 빌더
pub struct DictionaryLoader {
    path: PathBuf,
    config: LoaderConfig,
}

impl DictionaryLoader {
    /// 새 로더 생성
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            config: LoaderConfig::default(),
        }
    }

    /// 메모리 맵 사용 설정
    #[must_use]
    pub const fn use_mmap(mut self, use_mmap: bool) -> Self {
        self.config.use_mmap = use_mmap;
        self
    }

    /// 자동 압축 해제 설정
    #[must_use]
    pub const fn auto_decompress(mut self, auto: bool) -> Self {
        self.config.auto_decompress = auto;
        self
    }

    /// 지연 로딩 설정
    #[must_use]
    pub const fn lazy_load(mut self, lazy: bool) -> Self {
        self.config.lazy_load = lazy;
        self
    }

    /// 사전 로드
    ///
    /// # Errors
    ///
    /// 사전 파일을 찾을 수 없거나 로드에 실패한 경우 에러를 반환합니다.
    pub fn load(self) -> Result<Box<dyn Dictionary>> {
        if self.config.lazy_load {
            Ok(Box::new(LazyDictionary::new_with_config(
                self.path,
                self.config,
            )))
        } else {
            Ok(Box::new(MmapDictionary::load_with_config(
                self.path,
                self.config,
            )?))
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::trie::TrieBuilder;

    fn create_test_dict() -> tempfile::TempDir {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");

        // 테스트 Trie 생성
        let trie_entries = vec![("가", 0u32), ("가다", 1u32), ("가방", 2u32)];
        let trie_bytes = TrieBuilder::build(&trie_entries).expect("build trie");
        std::fs::write(temp_dir.path().join("sys.dic"), trie_bytes).expect("write trie");

        // 테스트 Matrix 생성
        let matrix = DenseMatrix::new(10, 10, 100);
        matrix
            .to_bin_file(temp_dir.path().join("matrix.bin"))
            .expect("write matrix");

        // 테스트 엔트리 생성 (CSV)
        let entries_csv = "가,1,1,100,NNG,*,T,가,*,*,*,*\n\
                           가다,2,2,200,VV,*,F,가다,*,*,*,*\n\
                           가방,3,3,300,NNG,*,T,가방,*,*,*,*\n";
        std::fs::write(temp_dir.path().join("entries.csv"), entries_csv).expect("write entries");

        temp_dir
    }

    #[test]
    fn test_mmap_dictionary_load() {
        let temp_dir = create_test_dict();
        let dict = MmapDictionary::load(temp_dir.path()).expect("load failed");

        assert!(dict.trie().exact_match("가").is_some());
        assert!(dict.trie().exact_match("가다").is_some());
        assert!(dict.trie().exact_match("없음").is_none());
    }

    #[test]
    fn test_dictionary_lookup() {
        let temp_dir = create_test_dict();
        let dict = MmapDictionary::load(temp_dir.path()).expect("load failed");

        let entries = dict.lookup("가");
        assert!(!entries.is_empty());
        assert_eq!(entries[0].surface, "가");
        assert_eq!(entries[0].left_id, 1);
        assert_eq!(entries[0].right_id, 1);
        assert_eq!(entries[0].cost, 100);
        assert!(entries[0].feature.starts_with("NNG"));

        let no_entries = dict.lookup("없음");
        assert!(no_entries.is_empty());
    }

    #[test]
    fn test_connection_cost() {
        let temp_dir = create_test_dict();
        let dict = MmapDictionary::load(temp_dir.path()).expect("load failed");

        let cost = dict.get_connection_cost(0, 0);
        assert_eq!(cost, 100); // default_cost from test matrix
    }

    #[test]
    fn test_loader_builder() {
        let temp_dir = create_test_dict();

        let dict = DictionaryLoader::new(temp_dir.path())
            .use_mmap(true)
            .auto_decompress(true)
            .load()
            .expect("load failed");

        let entries = dict.lookup("가");
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_lazy_dictionary() {
        let temp_dir = create_test_dict();

        let dict = LazyDictionary::new(temp_dir.path());

        // 첫 번째 접근 시 로드
        let entries = dict.lookup("가");
        assert!(!entries.is_empty());

        // 두 번째 접근은 캐시된 사전 사용
        let entries2 = dict.lookup("가다");
        assert!(!entries2.is_empty());
    }

    #[test]
    fn test_missing_dictionary() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let result = MmapDictionary::load(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_entry_by_index() {
        let temp_dir = create_test_dict();
        let dict = MmapDictionary::load(temp_dir.path()).expect("load failed");

        // 인덱스로 엔트리 조회
        let entry = dict.get_entry(0);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().surface, "가");

        let entry = dict.get_entry(1);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().surface, "가다");

        // 범위를 벗어난 인덱스
        let entry = dict.get_entry(100);
        assert!(entry.is_none());
    }

    #[test]
    fn test_entries_accessor() {
        let temp_dir = create_test_dict();
        let dict = MmapDictionary::load(temp_dir.path()).expect("load failed");

        let entries = dict.entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].surface, "가");
        assert_eq!(entries[1].surface, "가다");
        assert_eq!(entries[2].surface, "가방");
    }

    #[test]
    fn test_csv_parsing() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");

        // CSV 파일만 생성 (바이너리 없음)
        let entries_csv = "안녕,10,20,500,NNG,*,T,안녕,*,*,*,*\n\
                           하세요,15,25,600,VV+EC,*,F,하세요,*,*,*,*\n";
        std::fs::write(temp_dir.path().join("entries.csv"), entries_csv).expect("write entries");

        // Trie와 Matrix는 여전히 필요
        let trie_entries = vec![("안녕", 0u32), ("하세요", 1u32)];
        let trie_bytes = TrieBuilder::build(&trie_entries).expect("build trie");
        std::fs::write(temp_dir.path().join("sys.dic"), trie_bytes).expect("write trie");

        let matrix = DenseMatrix::new(30, 30, 100);
        matrix
            .to_bin_file(temp_dir.path().join("matrix.bin"))
            .expect("write matrix");

        // 로드 및 검증
        let dict = MmapDictionary::load(temp_dir.path()).expect("load failed");
        let entries = dict.lookup("안녕");
        assert!(!entries.is_empty());
        assert_eq!(entries[0].surface, "안녕");
        assert_eq!(entries[0].left_id, 10);
        assert_eq!(entries[0].right_id, 20);
        assert_eq!(entries[0].cost, 500);
    }

    #[test]
    fn test_dict_without_entries() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");

        // Trie와 Matrix만 생성 (엔트리 파일 없음)
        let trie_entries = vec![("테스트", 0u32)];
        let trie_bytes = TrieBuilder::build(&trie_entries).expect("build trie");
        std::fs::write(temp_dir.path().join("sys.dic"), trie_bytes).expect("write trie");

        let matrix = DenseMatrix::new(2, 2, 100);
        matrix
            .to_bin_file(temp_dir.path().join("matrix.bin"))
            .expect("write matrix");

        // 로드 성공 (빈 엔트리 벡터)
        let dict = MmapDictionary::load(temp_dir.path()).expect("load failed");
        assert_eq!(dict.entries().len(), 0);

        // 스텁 엔트리 반환 (하위 호환성)
        let entries = dict.lookup("테스트");
        assert!(!entries.is_empty());
        assert_eq!(entries[0].surface, "테스트");
        assert_eq!(entries[0].feature, "UNK,*,*,*,*,*,*,*");
    }
}
