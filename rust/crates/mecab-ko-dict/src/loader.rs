//! 사전 로딩 기능
//!
//! 바이너리 사전 파일을 로드하고 관리합니다.

use crate::error::{DictError, Result};
use crate::matrix::{DenseMatrix, Matrix, MatrixLoader};
use crate::trie::Trie;
use crate::{Dictionary, Entry};
use memmap2::Mmap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 사전 로더 설정
#[derive(Debug, Clone)]
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
    /// Trie 데이터
    trie_data: Arc<Vec<u8>>,
    /// Trie 인스턴스
    trie: Trie,
    /// 연접 비용 매트릭스
    matrix: DenseMatrix,
    /// 사전 디렉토리
    dict_dir: PathBuf,
}

impl MmapDictionary {
    /// 사전 로드
    ///
    /// # Arguments
    /// * `path` - 사전 디렉토리 경로
    ///
    /// # Examples
    /// ```rust,ignore
    /// use mecab_ko_dict::loader::MmapDictionary;
    ///
    /// let dict = MmapDictionary::load("./dict")?;
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::load_with_config(path, LoaderConfig::default())
    }

    /// 설정과 함께 사전 로드
    pub fn load_with_config<P: AsRef<Path>>(path: P, config: LoaderConfig) -> Result<Self> {
        let dict_dir = path.as_ref().to_path_buf();

        // Trie 로드
        let trie_data = Self::load_trie(&dict_dir, &config)?;
        let trie_data = Arc::new(trie_data);
        let trie = Trie::new(&trie_data);

        // Matrix 로드
        let matrix = Self::load_matrix(&dict_dir, &config)?;

        Ok(Self {
            trie_data,
            trie,
            matrix,
            dict_dir,
        })
    }

    /// Trie 데이터 로드
    fn load_trie(dict_dir: &Path, config: &LoaderConfig) -> Result<Vec<u8>> {
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
            // 압축되지 않은 파일
            if config.use_mmap {
                let file = File::open(&uncompressed_path)?;
                let mmap = unsafe { Mmap::map(&file)? };
                Ok(mmap.to_vec())
            } else {
                Ok(std::fs::read(&uncompressed_path)?)
            }
        } else {
            Err(DictError::Format(
                "sys.dic or sys.dic.zst not found".to_string(),
            ))
        }
    }

    /// Matrix 데이터 로드
    fn load_matrix(dict_dir: &Path, config: &LoaderConfig) -> Result<DenseMatrix> {
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

    /// Trie 참조
    pub fn trie(&self) -> &Trie {
        &self.trie
    }

    /// Matrix 참조
    pub fn matrix(&self) -> &DenseMatrix {
        &self.matrix
    }

    /// 사전 디렉토리 경로
    pub fn dict_dir(&self) -> &Path {
        &self.dict_dir
    }
}

impl Dictionary for MmapDictionary {
    fn lookup(&self, surface: &str) -> Vec<Entry> {
        // Trie에서 검색
        if let Some(_index) = self.trie.exact_match(surface) {
            // TODO: 실제 엔트리 로드 (현재는 스텁)
            vec![Entry {
                surface: surface.to_string(),
                left_id: 0,
                right_id: 0,
                cost: 0,
                feature: "UNK,*,*,*,*,*,*,*".to_string(),
            }]
        } else {
            Vec::new()
        }
    }

    fn get_connection_cost(&self, left_id: u16, right_id: u16) -> i16 {
        self.matrix.get(left_id, right_id)
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

        if dict.is_none() {
            *dict = Some(MmapDictionary::load_with_config(
                &self.dict_path,
                self.config.clone(),
            )?);
        }

        Ok(())
    }
}

impl Dictionary for LazyDictionary {
    fn lookup(&self, surface: &str) -> Vec<Entry> {
        if self.ensure_loaded().is_err() {
            return Vec::new();
        }

        let dict = match self.dict.lock() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        match dict.as_ref() {
            Some(d) => d.lookup(surface),
            None => Vec::new(),
        }
    }

    fn get_connection_cost(&self, left_id: u16, right_id: u16) -> i16 {
        if self.ensure_loaded().is_err() {
            return 0;
        }

        let dict = match self.dict.lock() {
            Ok(d) => d,
            Err(_) => return 0,
        };

        dict.as_ref()
            .map(|d| d.get_connection_cost(left_id, right_id))
            .unwrap_or(0)
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
    pub fn use_mmap(mut self, use_mmap: bool) -> Self {
        self.config.use_mmap = use_mmap;
        self
    }

    /// 자동 압축 해제 설정
    pub fn auto_decompress(mut self, auto: bool) -> Self {
        self.config.auto_decompress = auto;
        self
    }

    /// 지연 로딩 설정
    pub fn lazy_load(mut self, lazy: bool) -> Self {
        self.config.lazy_load = lazy;
        self
    }

    /// 사전 로드
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
mod tests {
    use super::*;
    use crate::matrix::Matrix;
    use crate::trie::TrieBuilder;
    use tempfile::TempDir;

    fn create_test_dict() -> TempDir {
        let temp_dir = TempDir::new().expect("create temp dir");

        // 테스트 Trie 생성
        let entries = vec![("가", 0u32), ("가다", 1u32), ("가방", 2u32)];
        let trie_bytes = TrieBuilder::build(&entries).expect("build trie");
        std::fs::write(temp_dir.path().join("sys.dic"), trie_bytes).expect("write trie");

        // 테스트 Matrix 생성
        let matrix = DenseMatrix::new(2, 2);
        matrix
            .to_bin_file(&temp_dir.path().join("matrix.bin"))
            .expect("write matrix");

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

        let no_entries = dict.lookup("없음");
        assert!(no_entries.is_empty());
    }

    #[test]
    fn test_connection_cost() {
        let temp_dir = create_test_dict();
        let dict = MmapDictionary::load(temp_dir.path()).expect("load failed");

        let cost = dict.get_connection_cost(0, 0);
        assert_eq!(cost, 0); // 기본값
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
        let temp_dir = TempDir::new().expect("create temp dir");
        let result = MmapDictionary::load(temp_dir.path());
        assert!(result.is_err());
    }
}
