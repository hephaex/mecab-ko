//! # Dictionary Integration Module
//!
//! 시스템 사전과 사용자 사전을 통합하여 형태소 검색을 제공합니다.
//!
//! ## 구조
//!
//! - **`SystemDictionary`**: Trie + Matrix + Features를 통합한 시스템 사전
//! - **`DictionaryLoader`**: 사전 경로 탐색 및 로딩
//! - 환경변수 기반 사전 경로 지원 (`MECAB_DICDIR`)
//! - 메모리 맵 기반 효율적 로딩
//!
//! ## 예제
//!
//! ```rust,ignore
//! use mecab_ko_dict::dictionary::SystemDictionary;
//!
//! // 기본 경로에서 로드
//! let dict = SystemDictionary::load_default()?;
//!
//! // 특정 경로에서 로드
//! let dict = SystemDictionary::load("/usr/local/lib/mecab/dic/mecab-ko-dic")?;
//!
//! // 형태소 검색
//! let entries = dict.lookup("안녕");
//! for entry in entries {
//!     println!("{}: {}", entry.surface, entry.feature);
//! }
//! ```

use std::io::{BufRead, BufReader, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::error::{DictError, Result};
use crate::matrix::{ConnectionMatrix, Matrix};
use crate::trie::Trie;
use crate::user_dict::UserDictionary;
use crate::{Dictionary, Entry};

/// 기본 사전 디렉토리 경로 (환경변수가 없을 때)
const DEFAULT_DICDIR_PATHS: &[&str] = &[
    "/usr/local/lib/mecab/dic/mecab-ko-dic",
    "/usr/lib/mecab/dic/mecab-ko-dic",
    "/opt/mecab/dic/mecab-ko-dic",
    "./dic/mecab-ko-dic",
];

/// 사전 파일 이름
const TRIE_FILE: &str = "sys.dic";
const MATRIX_FILE: &str = "matrix.bin";
const ENTRIES_BIN_FILE: &str = "entries.bin";
const ENTRIES_CSV_FILE: &str = "entries.csv";

/// entries.bin 매직 넘버
const ENTRIES_MAGIC: &[u8; 4] = b"MKED";
/// entries.bin 버전
const ENTRIES_VERSION: u32 = 1;

/// 시스템 사전
///
/// Trie, Matrix, Features를 통합하여 형태소 검색과 연접 비용 계산을 제공합니다.
/// 메모리 맵 기반으로 로드되어 효율적이며, 여러 인스턴스 간 메모리 공유가 가능합니다.
pub struct SystemDictionary {
    /// 사전 디렉토리 경로
    dicdir: PathBuf,
    /// Trie (형태소 검색)
    trie: Trie<'static>,
    /// 연접 비용 행렬
    matrix: ConnectionMatrix,
    /// 엔트리 배열 (Trie의 value를 인덱스로 사용)
    entries: Vec<DictEntry>,
    /// 사용자 사전 (선택)
    user_dict: Option<Arc<UserDictionary>>,
}

/// 사전 엔트리 (내부 표현)
///
/// 메모리 효율을 위해 feature는 인덱스로 저장합니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictEntry {
    /// 표면형
    pub surface: String,
    /// 좌문맥 ID
    pub left_id: u16,
    /// 우문맥 ID
    pub right_id: u16,
    /// 비용
    pub cost: i16,
    /// 품사 정보 (feature string)
    pub feature: String,
}

impl DictEntry {
    /// 새 사전 엔트리 생성
    pub fn new(
        surface: impl Into<String>,
        left_id: u16,
        right_id: u16,
        cost: i16,
        feature: impl Into<String>,
    ) -> Self {
        Self {
            surface: surface.into(),
            left_id,
            right_id,
            cost,
            feature: feature.into(),
        }
    }

    /// Entry로 변환
    #[must_use]
    pub fn to_entry(&self) -> Entry {
        Entry {
            surface: self.surface.clone(),
            left_id: self.left_id,
            right_id: self.right_id,
            cost: self.cost,
            feature: self.feature.clone(),
        }
    }
}

impl From<Entry> for DictEntry {
    fn from(entry: Entry) -> Self {
        Self {
            surface: entry.surface,
            left_id: entry.left_id,
            right_id: entry.right_id,
            cost: entry.cost,
            feature: entry.feature,
        }
    }
}

impl SystemDictionary {
    /// 기본 경로에서 사전 로드
    ///
    /// 다음 순서로 사전 경로를 탐색합니다:
    /// 1. `MECAB_DICDIR` 환경변수
    /// 2. 기본 경로 목록 (`DEFAULT_DICDIR_PATHS`)
    ///
    /// # Errors
    ///
    /// - 사전 파일을 찾을 수 없는 경우
    /// - 사전 파일 포맷이 잘못된 경우
    pub fn load_default() -> Result<Self> {
        let dicdir = DictionaryLoader::find_dicdir()?;
        Self::load(dicdir)
    }

    /// 특정 경로에서 사전 로드
    ///
    /// # Arguments
    ///
    /// * `dicdir` - 사전 디렉토리 경로
    ///
    /// # Errors
    ///
    /// - 사전 파일을 찾을 수 없는 경우
    /// - 사전 파일 포맷이 잘못된 경우
    pub fn load<P: AsRef<Path>>(dicdir: P) -> Result<Self> {
        let dicdir = dicdir.as_ref().to_path_buf();

        // Trie 로드
        let trie_path = dicdir.join(TRIE_FILE);
        let trie = if trie_path.exists() {
            Trie::from_file(&trie_path)?
        } else {
            // 압축 파일 시도
            let compressed_path = dicdir.join(format!("{TRIE_FILE}.zst"));
            if compressed_path.exists() {
                Trie::from_compressed_file(&compressed_path)?
            } else {
                return Err(DictError::Format(format!(
                    "Trie file not found: {}",
                    trie_path.display()
                )));
            }
        };

        // Matrix 로드
        let matrix_path = dicdir.join(MATRIX_FILE);
        let matrix = if matrix_path.exists() {
            ConnectionMatrix::from_bin_file(&matrix_path)?
        } else {
            // .def 파일 시도
            let def_path = dicdir.join("matrix.def");
            if def_path.exists() {
                ConnectionMatrix::from_def_file(&def_path)?
            } else {
                return Err(DictError::Format(format!(
                    "Matrix file not found: {}",
                    matrix_path.display()
                )));
            }
        };

        // 엔트리 로드 (entries.bin → entries.csv 순서로 시도)
        let entries = Self::load_entries(&dicdir)?;

        Ok(Self {
            dicdir,
            trie,
            matrix,
            entries,
            user_dict: None,
        })
    }

    /// 엔트리 로드 (entries.bin → entries.csv 순서로 시도)
    ///
    /// # Arguments
    ///
    /// * `dicdir` - 사전 디렉토리 경로
    fn load_entries(dicdir: &Path) -> Result<Vec<DictEntry>> {
        // 1. entries.bin 바이너리 파일 시도
        let bin_path = dicdir.join(ENTRIES_BIN_FILE);
        if bin_path.exists() {
            return Self::load_entries_bin(&bin_path);
        }

        // 2. entries.csv 텍스트 파일 시도
        let csv_path = dicdir.join(ENTRIES_CSV_FILE);
        if csv_path.exists() {
            return Self::load_entries_csv(&csv_path);
        }

        // 3. 엔트리 파일이 없으면 빈 벡터 (Trie + Matrix만으로 동작)
        Ok(Vec::new())
    }

    /// CSV 엔트리 파일 로드
    ///
    /// 형식: `surface,left_id,right_id,cost,feature_fields...`
    fn load_entries_csv(path: &Path) -> Result<Vec<DictEntry>> {
        let file = std::fs::File::open(path).map_err(DictError::Io)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(DictError::Io)?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 최소 5필드: surface, left_id, right_id, cost, feature...
            let mut fields = line.splitn(5, ',');
            let surface = fields
                .next()
                .ok_or_else(|| {
                    DictError::Format(format!("line {}: missing surface", line_num + 1))
                })?
                .to_string();
            let left_id: u16 = fields
                .next()
                .ok_or_else(|| {
                    DictError::Format(format!("line {}: missing left_id", line_num + 1))
                })?
                .parse()
                .map_err(|_| {
                    DictError::Format(format!("line {}: invalid left_id", line_num + 1))
                })?;
            let right_id: u16 = fields
                .next()
                .ok_or_else(|| {
                    DictError::Format(format!("line {}: missing right_id", line_num + 1))
                })?
                .parse()
                .map_err(|_| {
                    DictError::Format(format!("line {}: invalid right_id", line_num + 1))
                })?;
            let cost: i16 = fields
                .next()
                .ok_or_else(|| {
                    DictError::Format(format!("line {}: missing cost", line_num + 1))
                })?
                .parse()
                .map_err(|_| {
                    DictError::Format(format!("line {}: invalid cost", line_num + 1))
                })?;
            let feature = fields.next().unwrap_or("").to_string();

            entries.push(DictEntry {
                surface,
                left_id,
                right_id,
                cost,
                feature,
            });
        }

        Ok(entries)
    }

    /// 바이너리 엔트리 파일 로드
    ///
    /// 형식: `[magic:4][version:u32][count:u32][entries...]`
    fn load_entries_bin(path: &Path) -> Result<Vec<DictEntry>> {
        let data = std::fs::read(path).map_err(DictError::Io)?;
        let mut cursor = std::io::Cursor::new(&data);

        // 매직 넘버 검증
        let mut magic = [0u8; 4];
        cursor
            .read_exact(&mut magic)
            .map_err(|e| DictError::Format(format!("entries.bin magic: {e}")))?;
        if &magic != ENTRIES_MAGIC {
            return Err(DictError::Format("entries.bin: invalid magic number".into()));
        }

        // 버전 검증
        let version = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("entries.bin version: {e}")))?;
        if version != ENTRIES_VERSION {
            return Err(DictError::Format(format!(
                "entries.bin: unsupported version {version}"
            )));
        }

        // 엔트리 수
        let count = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("entries.bin count: {e}")))?;

        let mut entries = Vec::with_capacity(count as usize);
        for i in 0..count {
            let left_id = cursor.read_u16::<LittleEndian>().map_err(|e| {
                DictError::Format(format!("entries.bin entry {i} left_id: {e}"))
            })?;
            let right_id = cursor.read_u16::<LittleEndian>().map_err(|e| {
                DictError::Format(format!("entries.bin entry {i} right_id: {e}"))
            })?;
            let cost = cursor.read_i16::<LittleEndian>().map_err(|e| {
                DictError::Format(format!("entries.bin entry {i} cost: {e}"))
            })?;
            let surface_len = cursor.read_u16::<LittleEndian>().map_err(|e| {
                DictError::Format(format!("entries.bin entry {i} surface_len: {e}"))
            })? as usize;
            let feature_len = cursor.read_u16::<LittleEndian>().map_err(|e| {
                DictError::Format(format!("entries.bin entry {i} feature_len: {e}"))
            })? as usize;

            let mut surface_bytes = vec![0u8; surface_len];
            cursor.read_exact(&mut surface_bytes).map_err(|e| {
                DictError::Format(format!("entries.bin entry {i} surface: {e}"))
            })?;
            let surface = String::from_utf8(surface_bytes).map_err(|e| {
                DictError::Format(format!("entries.bin entry {i} surface utf8: {e}"))
            })?;

            let mut feature_bytes = vec![0u8; feature_len];
            cursor.read_exact(&mut feature_bytes).map_err(|e| {
                DictError::Format(format!("entries.bin entry {i} feature: {e}"))
            })?;
            let feature = String::from_utf8(feature_bytes).map_err(|e| {
                DictError::Format(format!("entries.bin entry {i} feature utf8: {e}"))
            })?;

            entries.push(DictEntry {
                surface,
                left_id,
                right_id,
                cost,
                feature,
            });
        }

        Ok(entries)
    }

    /// 엔트리를 바이너리 파일로 저장
    ///
    /// # Errors
    ///
    /// 파일 쓰기 실패 시 에러 반환
    pub fn save_entries_bin(entries: &[DictEntry], path: &Path) -> Result<()> {
        let mut file = std::fs::File::create(path).map_err(DictError::Io)?;

        file.write_all(ENTRIES_MAGIC).map_err(DictError::Io)?;
        file.write_u32::<LittleEndian>(ENTRIES_VERSION)
            .map_err(DictError::Io)?;

        let count = u32::try_from(entries.len())
            .map_err(|_| DictError::Format("too many entries".into()))?;
        file.write_u32::<LittleEndian>(count)
            .map_err(DictError::Io)?;

        for entry in entries {
            file.write_u16::<LittleEndian>(entry.left_id)
                .map_err(DictError::Io)?;
            file.write_u16::<LittleEndian>(entry.right_id)
                .map_err(DictError::Io)?;
            file.write_i16::<LittleEndian>(entry.cost)
                .map_err(DictError::Io)?;

            let surface_bytes = entry.surface.as_bytes();
            let surface_len = u16::try_from(surface_bytes.len())
                .map_err(|_| DictError::Format("surface too long".into()))?;
            file.write_u16::<LittleEndian>(surface_len)
                .map_err(DictError::Io)?;

            let feature_bytes = entry.feature.as_bytes();
            let feature_len = u16::try_from(feature_bytes.len())
                .map_err(|_| DictError::Format("feature too long".into()))?;
            file.write_u16::<LittleEndian>(feature_len)
                .map_err(DictError::Io)?;

            file.write_all(surface_bytes).map_err(DictError::Io)?;
            file.write_all(feature_bytes).map_err(DictError::Io)?;
        }

        Ok(())
    }

    /// 엔트리를 CSV 파일로 저장
    ///
    /// # Errors
    ///
    /// 파일 쓰기 실패 시 에러 반환
    pub fn save_entries_csv(entries: &[DictEntry], path: &Path) -> Result<()> {
        let mut file = std::fs::File::create(path).map_err(DictError::Io)?;

        for entry in entries {
            writeln!(
                file,
                "{},{},{},{},{}",
                entry.surface, entry.left_id, entry.right_id, entry.cost, entry.feature
            )
            .map_err(DictError::Io)?;
        }

        Ok(())
    }

    /// 인덱스에서 시작하여 같은 surface를 가진 연속된 모든 엔트리 반환
    ///
    /// 사전 빌더가 같은 surface의 엔트리를 연속으로 배치하므로,
    /// `first_index`부터 surface가 같은 동안 모든 엔트리를 수집합니다.
    fn get_entries_at(&self, first_index: u32, surface: &str) -> Vec<&DictEntry> {
        let start = first_index as usize;
        let mut results = Vec::new();
        for entry in self.entries.get(start..).unwrap_or(&[]) {
            if entry.surface == surface {
                results.push(entry);
            } else {
                break;
            }
        }
        results
    }

    /// 사용자 사전 추가
    ///
    /// # Arguments
    ///
    /// * `user_dict` - 사용자 사전
    #[must_use]
    pub fn with_user_dictionary(mut self, user_dict: UserDictionary) -> Self {
        self.user_dict = Some(Arc::new(user_dict));
        self
    }

    /// 사용자 사전 설정
    pub fn set_user_dictionary(&mut self, user_dict: UserDictionary) {
        self.user_dict = Some(Arc::new(user_dict));
    }

    /// 사전 디렉토리 경로 반환
    #[must_use]
    pub fn dicdir(&self) -> &Path {
        &self.dicdir
    }

    /// Trie 참조 반환
    #[must_use]
    pub const fn trie(&self) -> &Trie<'static> {
        &self.trie
    }

    /// Matrix 참조 반환
    #[must_use]
    pub const fn matrix(&self) -> &ConnectionMatrix {
        &self.matrix
    }

    /// 엔트리 배열 참조 반환
    #[must_use]
    pub fn entries(&self) -> &[DictEntry] {
        &self.entries
    }

    /// 사용자 사전 참조 반환
    #[must_use]
    pub fn user_dictionary(&self) -> Option<&UserDictionary> {
        self.user_dict.as_deref()
    }

    /// 인덱스로 엔트리 조회
    ///
    /// # Arguments
    ///
    /// * `index` - Trie에서 반환된 인덱스
    #[must_use]
    pub fn get_entry(&self, index: u32) -> Option<&DictEntry> {
        self.entries.get(index as usize)
    }

    /// 공통 접두사 검색
    ///
    /// 주어진 텍스트의 접두사와 일치하는 모든 엔트리를 찾습니다.
    /// 같은 surface에 복수 엔트리가 있으면 모두 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `text` - 검색할 텍스트
    ///
    /// # Returns
    ///
    /// 일치하는 엔트리와 바이트 길이의 벡터
    #[must_use]
    pub fn common_prefix_search(&self, text: &str) -> Vec<(&DictEntry, usize)> {
        let mut results = Vec::new();
        for (index, byte_len) in self.trie.common_prefix_search(text) {
            let surface = &text[..byte_len];
            let entries = self.get_entries_at(index, surface);
            for entry in entries {
                results.push((entry, byte_len));
            }
        }
        results
    }

    /// 특정 위치에서 공통 접두사 검색
    ///
    /// # Arguments
    ///
    /// * `text` - 전체 텍스트
    /// * `start_byte` - 검색 시작 바이트 위치
    #[must_use]
    pub fn common_prefix_search_at(
        &self,
        text: &str,
        start_byte: usize,
    ) -> Vec<(&DictEntry, usize)> {
        let mut results = Vec::new();
        for (index, end_byte) in self.trie.common_prefix_search_at(text, start_byte) {
            let byte_len = end_byte - start_byte;
            let surface = &text[start_byte..end_byte];
            let entries = self.get_entries_at(index, surface);
            for entry in entries {
                results.push((entry, byte_len));
            }
        }
        results
    }

    /// 시스템 사전과 사용자 사전을 통합하여 검색
    ///
    /// # Arguments
    ///
    /// * `surface` - 검색할 표면형
    #[must_use]
    pub fn lookup_combined(&self, surface: &str) -> Vec<Entry> {
        let mut results = self.lookup(surface);

        // 사용자 사전 검색
        if let Some(user_dict) = &self.user_dict {
            let user_entries = user_dict.lookup(surface);
            results.extend(user_entries.iter().map(|e| e.to_entry()));
        }

        results
    }

    /// 엔트리 추가 (테스트용)
    ///
    /// 실제 사전에서는 파일에서 로드되므로, 이 메서드는 테스트에서만 사용됩니다.
    #[cfg(test)]
    pub fn add_entry(&mut self, entry: DictEntry) {
        self.entries.push(entry);
    }

    /// 테스트용 생성자 (외부 crate의 test에서도 사용 가능)
    #[doc(hidden)]
    #[must_use]
    pub const fn new_test(
        dicdir: PathBuf,
        trie: Trie<'static>,
        matrix: ConnectionMatrix,
        entries: Vec<DictEntry>,
    ) -> Self {
        Self {
            dicdir,
            trie,
            matrix,
            entries,
            user_dict: None,
        }
    }
}

impl Dictionary for SystemDictionary {
    fn lookup(&self, surface: &str) -> Vec<Entry> {
        // Trie exact match로 검색 → 같은 surface의 모든 엔트리 반환
        if let Some(index) = self.trie.exact_match(surface) {
            let entries = self.get_entries_at(index, surface);
            if !entries.is_empty() {
                return entries.iter().map(|e| e.to_entry()).collect();
            }
        }

        Vec::new()
    }

    fn get_connection_cost(&self, left_id: u16, right_id: u16) -> i16 {
        i16::try_from(self.matrix.get(right_id, left_id)).unwrap_or(i16::MAX)
    }
}

/// 사전 로더
///
/// 사전 경로 탐색 및 로딩을 담당합니다.
pub struct DictionaryLoader;

impl DictionaryLoader {
    /// 사전 디렉토리 경로 탐색
    ///
    /// 다음 순서로 탐색합니다:
    /// 1. `MECAB_DICDIR` 환경변수
    /// 2. 기본 경로 목록
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary directory cannot be found.
    pub fn find_dicdir() -> Result<PathBuf> {
        // 환경변수 확인
        if let Ok(dicdir) = std::env::var("MECAB_DICDIR") {
            let path = PathBuf::from(dicdir);
            if path.is_dir() {
                return Ok(path);
            }
        }

        // 기본 경로 탐색
        for &path_str in DEFAULT_DICDIR_PATHS {
            let path = PathBuf::from(path_str);
            if path.is_dir() {
                return Ok(path);
            }
        }

        Err(DictError::Format(
            "Dictionary directory not found. Set MECAB_DICDIR environment variable or install mecab-ko-dic to default location".to_string(),
        ))
    }

    /// 특정 경로에서 시스템 사전 로드
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary files cannot be loaded.
    pub fn load_system<P: AsRef<Path>>(dicdir: P) -> Result<SystemDictionary> {
        SystemDictionary::load(dicdir)
    }

    /// 기본 경로에서 시스템 사전 로드
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary files cannot be loaded.
    pub fn load_default() -> Result<SystemDictionary> {
        SystemDictionary::load_default()
    }

    /// 사전 경로가 유효한지 확인
    ///
    /// # Arguments
    ///
    /// * `dicdir` - 확인할 디렉토리 경로
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary directory is invalid or required files are missing.
    pub fn validate_dicdir<P: AsRef<Path>>(dicdir: P) -> Result<()> {
        let dicdir = dicdir.as_ref();

        if !dicdir.is_dir() {
            return Err(DictError::Format(format!(
                "Dictionary directory does not exist: {}",
                dicdir.display()
            )));
        }

        // 필수 파일 확인 (Trie와 Matrix 중 하나는 있어야 함)
        let has_trie =
            dicdir.join(TRIE_FILE).exists() || dicdir.join(format!("{TRIE_FILE}.zst")).exists();

        let has_matrix = dicdir.join(MATRIX_FILE).exists() || dicdir.join("matrix.def").exists();

        if !has_trie {
            return Err(DictError::Format(format!(
                "Trie file not found in {}",
                dicdir.display()
            )));
        }

        if !has_matrix {
            return Err(DictError::Format(format!(
                "Matrix file not found in {}",
                dicdir.display()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::matrix::DenseMatrix;
    use crate::trie::TrieBuilder;

    fn create_test_dictionary() -> SystemDictionary {
        // 테스트용 Trie 생성
        let entries = vec![
            ("가", 0u32),
            ("가다", 1),
            ("가방", 2),
            ("나", 3),
            ("나다", 4),
        ];
        let trie_bytes = TrieBuilder::build(&entries).expect("should build trie");
        let trie = Trie::from_vec(trie_bytes);

        // 테스트용 Matrix 생성
        let matrix = DenseMatrix::new(10, 10, 100);
        let matrix = ConnectionMatrix::Dense(matrix);

        // 테스트용 엔트리 생성
        let dict_entries = vec![
            DictEntry::new("가", 1, 1, 100, "NNG,*,T,가,*,*,*,*"),
            DictEntry::new("가다", 2, 2, 200, "VV,*,F,가다,*,*,*,*"),
            DictEntry::new("가방", 3, 3, 300, "NNG,*,T,가방,*,*,*,*"),
            DictEntry::new("나", 4, 4, 400, "NP,*,F,나,*,*,*,*"),
            DictEntry::new("나다", 5, 5, 500, "VV,*,F,나다,*,*,*,*"),
        ];

        SystemDictionary {
            dicdir: PathBuf::from("./test_dic"),
            trie,
            matrix,
            entries: dict_entries,
            user_dict: None,
        }
    }

    #[test]
    fn test_dict_entry_creation() {
        let entry = DictEntry::new("안녕", 1, 1, 100, "NNG,*,T,안녕,*,*,*,*");
        assert_eq!(entry.surface, "안녕");
        assert_eq!(entry.left_id, 1);
        assert_eq!(entry.right_id, 1);
        assert_eq!(entry.cost, 100);
    }

    #[test]
    fn test_dict_entry_to_entry() {
        let dict_entry = DictEntry::new("테스트", 5, 5, 200, "NNG,*,T,테스트,*,*,*,*");
        let entry = dict_entry.to_entry();

        assert_eq!(entry.surface, "테스트");
        assert_eq!(entry.left_id, 5);
        assert_eq!(entry.cost, 200);
    }

    #[test]
    fn test_system_dictionary_lookup() {
        let dict = create_test_dictionary();

        let entries = dict.lookup("가");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].surface, "가");

        let entries = dict.lookup("가다");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].surface, "가다");

        let entries = dict.lookup("없음");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_system_dictionary_get_connection_cost() {
        let dict = create_test_dictionary();
        let cost = dict.get_connection_cost(1, 2);
        assert_eq!(cost, 100); // 기본값
    }

    #[test]
    fn test_common_prefix_search() {
        let dict = create_test_dictionary();

        // "가방에" 검색 -> "가", "가방" 매칭
        let results = dict.common_prefix_search("가방에");
        assert_eq!(results.len(), 2);

        let surfaces: Vec<_> = results.iter().map(|(e, _)| e.surface.as_str()).collect();
        assert!(surfaces.contains(&"가"));
        assert!(surfaces.contains(&"가방"));
    }

    #[test]
    fn test_common_prefix_search_at() {
        let dict = create_test_dictionary();

        let text = "나가다";
        let start = "나".len(); // 3 bytes

        let results = dict.common_prefix_search_at(text, start);
        assert_eq!(results.len(), 2); // "가", "가다"

        let surfaces: Vec<_> = results.iter().map(|(e, _)| e.surface.as_str()).collect();
        assert!(surfaces.contains(&"가"));
        assert!(surfaces.contains(&"가다"));
    }

    #[test]
    fn test_with_user_dictionary() {
        let mut dict = create_test_dictionary();

        let mut user_dict = UserDictionary::new();
        user_dict.add_entry("딥러닝", "NNG", Some(-1000), None);
        user_dict.add_entry("머신러닝", "NNG", Some(-1000), None);

        dict.set_user_dictionary(user_dict);

        let entries = dict.lookup_combined("딥러닝");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].surface, "딥러닝");
    }

    #[test]
    fn test_lookup_combined_system_and_user() {
        let mut dict = create_test_dictionary();

        let mut user_dict = UserDictionary::new();
        user_dict.add_entry("가", "JKS", Some(-500), None); // "가" 조사 추가

        dict.set_user_dictionary(user_dict);

        let entries = dict.lookup_combined("가");
        // 시스템 사전 "가" (NNG) + 사용자 사전 "가" (JKS) = 2개
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_get_entry() {
        let dict = create_test_dictionary();

        let entry = dict.get_entry(0);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().surface, "가");

        let entry = dict.get_entry(100);
        assert!(entry.is_none());
    }

    #[test]
    fn test_dicdir() {
        let dict = create_test_dictionary();
        assert_eq!(dict.dicdir(), Path::new("./test_dic"));
    }

    #[test]
    fn test_trie_reference() {
        let dict = create_test_dictionary();
        let trie = dict.trie();
        assert!(trie.exact_match("가").is_some());
    }

    #[test]
    fn test_matrix_reference() {
        let dict = create_test_dictionary();
        let matrix = dict.matrix();
        assert_eq!(matrix.left_size(), 10);
        assert_eq!(matrix.right_size(), 10);
    }

    #[test]
    fn test_entries_reference() {
        let dict = create_test_dictionary();
        let entries = dict.entries();
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_dictionary_loader_find_dicdir() {
        // 환경변수나 기본 경로에 사전이 없으면 에러
        // 실제 시스템에 사전이 설치되어 있으면 성공할 수 있음
        let result = DictionaryLoader::find_dicdir();

        // 이 테스트는 환경에 따라 성공/실패할 수 있으므로,
        // 단순히 Result 타입이 올바르게 반환되는지만 확인
        match result {
            Ok(path) => {
                assert!(path.is_dir());
            }
            Err(e) => {
                // 에러 메시지가 적절한지 확인
                assert!(e.to_string().contains("Dictionary directory not found"));
            }
        }
    }

    #[test]
    fn test_dict_entry_from_entry() {
        let entry = Entry {
            surface: "테스트".to_string(),
            left_id: 10,
            right_id: 20,
            cost: 300,
            feature: "NNG,*,T,테스트,*,*,*,*".to_string(),
        };

        let dict_entry: DictEntry = entry.into();
        assert_eq!(dict_entry.surface, "테스트");
        assert_eq!(dict_entry.left_id, 10);
        assert_eq!(dict_entry.right_id, 20);
        assert_eq!(dict_entry.cost, 300);
    }

    #[test]
    fn test_entries_bin_roundtrip() {
        let entries = vec![
            DictEntry::new("안녕", 1, 1, 100, "NNG,*,T,안녕,*,*,*,*"),
            DictEntry::new("하세요", 2, 2, 50, "VV,*,F,하세요,*,*,*,*"),
            DictEntry::new("감사", 3, 3, 80, "NNG,*,F,감사,*,*,*,*"),
        ];

        let temp = tempfile::NamedTempFile::new().expect("create temp file");
        let path = temp.path();

        SystemDictionary::save_entries_bin(&entries, path).expect("save should work");
        let loaded = SystemDictionary::load_entries_bin(path).expect("load should work");

        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].surface, "안녕");
        assert_eq!(loaded[0].left_id, 1);
        assert_eq!(loaded[0].cost, 100);
        assert_eq!(loaded[0].feature, "NNG,*,T,안녕,*,*,*,*");
        assert_eq!(loaded[1].surface, "하세요");
        assert_eq!(loaded[2].surface, "감사");
    }

    #[test]
    fn test_entries_csv_roundtrip() {
        let entries = vec![
            DictEntry::new("형태소", 10, 20, 150, "NNG,*,F,형태소,*,*,*,*"),
            DictEntry::new("분석", 11, 21, 200, "NNG,*,T,분석,*,*,*,*"),
        ];

        let temp = tempfile::NamedTempFile::new().expect("create temp file");
        let path = temp.path();

        SystemDictionary::save_entries_csv(&entries, path).expect("save should work");
        let loaded = SystemDictionary::load_entries_csv(path).expect("load should work");

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].surface, "형태소");
        assert_eq!(loaded[0].left_id, 10);
        assert_eq!(loaded[0].right_id, 20);
        assert_eq!(loaded[0].cost, 150);
        assert_eq!(loaded[1].surface, "분석");
    }

    #[test]
    fn test_get_entries_at_multi() {
        // 같은 surface에 복수 엔트리가 있는 경우
        let trie_input = vec![("가", 0u32), ("나", 2u32)];
        let trie_bytes = TrieBuilder::build(&trie_input).expect("build trie");
        let trie = Trie::from_vec(trie_bytes);
        let matrix = ConnectionMatrix::Dense(DenseMatrix::new(5, 5, 100));

        let dict_entries = vec![
            DictEntry::new("가", 1, 1, 100, "VV,*,F,가,*,*,*,*"),
            DictEntry::new("가", 2, 2, 50, "JKS,*,F,가,*,*,*,*"),
            DictEntry::new("나", 3, 3, 200, "NP,*,F,나,*,*,*,*"),
        ];

        let dict = SystemDictionary {
            dicdir: PathBuf::from("./test"),
            trie,
            matrix,
            entries: dict_entries,
            user_dict: None,
        };

        // "가" 검색 → 2개 엔트리 반환
        let results = dict.get_entries_at(0, "가");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].feature, "VV,*,F,가,*,*,*,*");
        assert_eq!(results[1].feature, "JKS,*,F,가,*,*,*,*");

        // lookup도 복수 반환
        use crate::Dictionary;
        let entries = dict.lookup("가");
        assert_eq!(entries.len(), 2);
    }
}
