//! # mecab-ko-dict-builder
//!
//! 한국어 형태소 사전 빌더
//!
//! mecab-ko-dic CSV 형식에서 바이너리 사전을 생성합니다.
//!
//! ## 주요 기능
//!
//! - CSV 파싱 (12컬럼 형식)
//! - Double Array Trie 구축
//! - 연접 비용 매트릭스 생성
//! - 미등록어 정의 처리
//! - 바이너리 사전 압축
//!
//! ## 사용법
//!
//! ```bash
//! mecab-ko-dict-builder --input ./mecab-ko-dic --output ./dict.bin
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub use builder::DictionaryBuilder;
pub use error::{BuildError, Result};

pub mod char_def_parser;
pub mod unk_def_parser;

/// 빌더 에러 모듈
pub mod error {
    use thiserror::Error;

    /// 사전 빌드 에러
    #[derive(Error, Debug)]
    pub enum BuildError {
        /// IO 에러
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),

        /// CSV 파싱 에러
        #[error("CSV parsing error: {0}")]
        Csv(#[from] csv::Error),

        /// 포맷 에러
        #[error("Invalid format: {0}")]
        Format(String),

        /// 인코딩 에러
        #[error("Encoding error: {0}")]
        Encoding(String),

        /// Trie 빌드 에러
        #[error("Trie build error: {0}")]
        Trie(String),

        /// Dictionary 에러
        #[error("Dictionary error: {0}")]
        Dict(#[from] mecab_ko_dict::error::DictError),
    }

    /// Result 타입 별칭
    pub type Result<T> = std::result::Result<T, BuildError>;
}

/// mecab-ko-dic CSV 형식 파서
///
/// 12컬럼 CSV 형식을 파싱합니다.
#[allow(clippy::mixed_attributes_style)]
pub mod csv_parser {
    //! mecab-ko-dic CSV 형식 파서 구현

    use super::{BuildError, Result};
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    use encoding_rs::{EUC_KR, UTF_8};
    use mecab_ko_hangul::has_jongseong;

    /// CSV 엔트리 (12컬럼)
    ///
    /// mecab-ko-dic 형식:
    /// 표면형,좌ID,우ID,비용,품사,품사세분류,종성유무,읽기,타입,첫품사,마지막품사,표현
    #[derive(Debug, Clone)]
    pub struct CsvEntry {
        /// 표면형
        pub surface: String,
        /// 좌문맥 ID
        pub left_id: u16,
        /// 우문맥 ID
        pub right_id: u16,
        /// 비용
        pub cost: i16,
        /// 품사
        pub pos: String,
        /// 품사 세분류
        pub pos_detail: String,
        /// 종성 유무 (T/F/*)
        pub jongseong: String,
        /// 읽기
        pub reading: String,
        /// 타입
        pub entry_type: String,
        /// 첫 품사
        pub first_pos: String,
        /// 마지막 품사
        pub last_pos: String,
        /// 표현 (복합어 분석)
        pub expression: String,
    }

    impl CsvEntry {
        /// Feature 문자열 생성 (`MeCab` 형식)
        ///
        /// 형식: 품사,품사세분류,종성유무,읽기,타입,첫품사,마지막품사,표현
        #[must_use]
        pub fn to_feature(&self) -> String {
            format!(
                "{},{},{},{},{},{},{},{}",
                self.pos,
                self.pos_detail,
                self.jongseong,
                self.reading,
                self.entry_type,
                self.first_pos,
                self.last_pos,
                self.expression
            )
        }

        /// 종성 유무 자동 설정
        ///
        /// CSV에서 '*'로 되어있는 경우 표면형에서 자동 판별
        pub fn normalize_jongseong(&mut self) {
            if self.jongseong == "*" && !self.surface.is_empty() {
                // 마지막 문자의 종성 확인
                if let Some(last_char) = self.surface.chars().last() {
                    self.jongseong = match has_jongseong(last_char) {
                        Some(true) => "T".to_string(),
                        Some(false) => "F".to_string(),
                        None => "*".to_string(), // 한글이 아닌 경우
                    };
                }
            }
        }
    }

    /// CSV 파일 파서
    pub struct CsvParser {
        /// 디렉토리 경로
        dir_path: String,
        /// 인코딩 (UTF-8 또는 EUC-KR)
        encoding: Encoding,
        /// 진행 상황 로깅
        verbose: bool,
    }

    /// 파일 인코딩
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Encoding {
        /// UTF-8
        Utf8,
        /// EUC-KR (cp949)
        EucKr,
        /// 자동 감지
        Auto,
    }

    impl CsvParser {
        /// 새 파서 생성
        #[must_use]
        pub fn new<P: AsRef<Path>>(dir_path: P) -> Self {
            Self {
                dir_path: dir_path.as_ref().to_string_lossy().to_string(),
                encoding: Encoding::Auto,
                verbose: false,
            }
        }

        /// 인코딩 설정
        #[must_use]
        pub const fn with_encoding(mut self, encoding: Encoding) -> Self {
            self.encoding = encoding;
            self
        }

        /// 자세한 출력 설정
        #[must_use]
        pub const fn verbose(mut self, verbose: bool) -> Self {
            self.verbose = verbose;
            self
        }

        /// 모든 CSV 파일 파싱
        ///
        /// 디렉토리 내의 모든 *.csv 파일을 파싱합니다.
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - The directory cannot be read
        /// - Any CSV file cannot be opened or parsed
        /// - CSV entries have invalid format or field values
        pub fn parse_all(&self) -> Result<Vec<CsvEntry>> {
            let dir = Path::new(&self.dir_path);
            if !dir.is_dir() {
                return Err(BuildError::Format(format!(
                    "Directory not found: {}",
                    self.dir_path
                )));
            }

            let mut all_entries = Vec::new();
            let csv_files = Self::find_csv_files(dir)?;

            if self.verbose {
                tracing::info!("Found {} CSV files", csv_files.len());
            }

            for csv_file in csv_files {
                if self.verbose {
                    tracing::debug!("Parsing {}", csv_file.display());
                }

                let entries = self.parse_file(&csv_file)?;
                all_entries.extend(entries);
            }

            if self.verbose {
                tracing::info!("Parsed {} total entries", all_entries.len());
            }

            Ok(all_entries)
        }

        /// CSV 파일 목록 찾기
        fn find_csv_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
            let mut csv_files = Vec::new();

            for entry in std::fs::read_dir(dir).map_err(BuildError::Io)? {
                let entry = entry.map_err(BuildError::Io)?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "csv" {
                            csv_files.push(path);
                        }
                    }
                }
            }

            csv_files.sort();
            Ok(csv_files)
        }

        /// 단일 CSV 파일 파싱
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - The file cannot be opened or read
        /// - The file encoding cannot be detected or decoded
        /// - CSV content is malformed or has invalid field values
        pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvEntry>> {
            let file = File::open(path.as_ref()).map_err(BuildError::Io)?;
            let content = self.read_with_encoding(file)?;

            self.parse_csv_content(&content)
        }

        /// 인코딩을 고려하여 파일 읽기
        fn read_with_encoding(&self, file: File) -> Result<String> {
            use std::io::Read;

            let mut reader = BufReader::new(file);
            let mut first_bytes = vec![0u8; 1024];
            let n = reader
                .by_ref()
                .take(1024)
                .read(&mut first_bytes)
                .map_err(BuildError::Io)?;
            first_bytes.truncate(n);

            // 전체 파일 읽기
            let mut all_bytes = first_bytes.clone();
            reader.read_to_end(&mut all_bytes).map_err(BuildError::Io)?;

            // 인코딩 감지 및 변환
            let encoding = match self.encoding {
                Encoding::Utf8 => UTF_8,
                Encoding::EucKr => EUC_KR,
                Encoding::Auto => {
                    // UTF-8 유효성 검사
                    if std::str::from_utf8(&first_bytes).is_ok() {
                        UTF_8
                    } else {
                        EUC_KR
                    }
                }
            };

            let (decoded, _, had_errors) = encoding.decode(&all_bytes);
            if had_errors && self.verbose {
                tracing::warn!("Encoding errors detected during decoding");
            }

            Ok(decoded.into_owned())
        }

        /// CSV 내용 파싱
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - CSV content is malformed
        /// - Fields cannot be parsed to the expected types (`left_id`, `right_id`, cost)
        pub fn parse_csv_content(&self, content: &str) -> Result<Vec<CsvEntry>> {
            let mut entries = Vec::new();
            let mut csv_reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .flexible(true)
                .comment(Some(b'#'))
                .from_reader(content.as_bytes());

            for (line_num, result) in csv_reader.records().enumerate() {
                let record = result.map_err(BuildError::Csv)?;

                if record.len() < 12 {
                    if self.verbose {
                        tracing::warn!(
                            "Line {}: Expected 12 fields, got {}. Skipping.",
                            line_num + 1,
                            record.len()
                        );
                    }
                    continue;
                }

                let mut entry = CsvEntry {
                    surface: record[0].to_string(),
                    left_id: record[1].parse().map_err(|_| {
                        BuildError::Format(format!("Invalid left_id at line {}", line_num + 1))
                    })?,
                    right_id: record[2].parse().map_err(|_| {
                        BuildError::Format(format!("Invalid right_id at line {}", line_num + 1))
                    })?,
                    cost: record[3].parse().map_err(|_| {
                        BuildError::Format(format!("Invalid cost at line {}", line_num + 1))
                    })?,
                    pos: record[4].to_string(),
                    pos_detail: record[5].to_string(),
                    jongseong: record[6].to_string(),
                    reading: record[7].to_string(),
                    entry_type: record[8].to_string(),
                    first_pos: record[9].to_string(),
                    last_pos: record[10].to_string(),
                    expression: record[11].to_string(),
                };

                entry.normalize_jongseong();
                entries.push(entry);
            }

            Ok(entries)
        }
    }
}

/// 사전 빌드 파이프라인
///
/// CSV → 바이너리 사전 변환
#[allow(clippy::mixed_attributes_style)]
pub mod builder {
    //! 사전 빌더 구현

    use super::char_def_parser::CharDef;
    use super::csv_parser::{CsvEntry, CsvParser, Encoding};
    use super::unk_def_parser::UnkDef;
    use super::{BuildError, Result};
    use mecab_ko_dict::dictionary::DictEntry;
    use mecab_ko_dict::matrix::{DenseMatrix, Matrix};
    use mecab_ko_dict::trie::TrieBuilder;
    use std::collections::HashMap;
    use std::path::Path;

    /// 빌드 설정
    #[derive(Debug, Clone)]
    pub struct BuildConfig {
        /// 입력 디렉토리
        pub input_dir: String,
        /// 출력 디렉토리
        pub output_dir: String,
        /// 압축 레벨 (0-22, 0=압축 안 함)
        pub compression_level: i32,
        /// 인코딩
        pub encoding: Encoding,
        /// 자세한 출력
        pub verbose: bool,
    }

    impl Default for BuildConfig {
        fn default() -> Self {
            Self {
                input_dir: ".".to_string(),
                output_dir: "./dict".to_string(),
                compression_level: 3,
                encoding: Encoding::Auto,
                verbose: false,
            }
        }
    }

    /// 사전 빌더
    pub struct DictionaryBuilder {
        config: BuildConfig,
    }

    impl DictionaryBuilder {
        /// 새 빌더 생성
        #[must_use]
        pub const fn new(config: BuildConfig) -> Self {
            Self { config }
        }

        /// 사전 빌드 실행
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - CSV files cannot be parsed
        /// - `matrix.def` is missing or malformed
        /// - Trie building fails
        /// - Output files cannot be written
        pub fn build(&self) -> Result<BuildResult> {
            if self.config.verbose {
                tracing::info!("Starting dictionary build");
                tracing::info!("  Input: {}", self.config.input_dir);
                tracing::info!("  Output: {}", self.config.output_dir);
            }

            // 1. CSV 파싱
            let csv_entries = self.parse_csv_files()?;

            // 2. matrix.def 파싱
            let matrix = self.build_matrix()?;

            // 3. char.def 파싱 (선택적)
            let char_def = self.build_char_def().ok();

            // 4. unk.def 파싱 (선택적)
            let unk_def = self.build_unk_def().ok();

            // 5. Trie 및 엔트리 빌드
            let (trie_bytes, dict_entries) = self.build_trie_and_entries(&csv_entries)?;

            // 6. 바이너리 출력
            self.save_dictionary(
                &trie_bytes,
                &matrix,
                &dict_entries,
                char_def.as_ref(),
                unk_def.as_ref(),
            )?;

            Ok(BuildResult {
                entry_count: dict_entries.len(),
                trie_size: trie_bytes.len(),
                matrix_size: matrix.left_size() * matrix.right_size(),
            })
        }

        /// CSV 파일 파싱
        fn parse_csv_files(&self) -> Result<Vec<CsvEntry>> {
            if self.config.verbose {
                tracing::info!("Parsing CSV files...");
            }

            let parser = CsvParser::new(&self.config.input_dir)
                .with_encoding(self.config.encoding)
                .verbose(self.config.verbose);

            parser.parse_all()
        }

        /// 연접 비용 행렬 빌드
        fn build_matrix(&self) -> Result<DenseMatrix> {
            let matrix_path = Path::new(&self.config.input_dir).join("matrix.def");

            if self.config.verbose {
                tracing::info!("Loading connection matrix from {}", matrix_path.display());
            }

            if !matrix_path.exists() {
                return Err(BuildError::Format(format!(
                    "matrix.def not found: {}",
                    matrix_path.display()
                )));
            }

            DenseMatrix::from_def_file(&matrix_path).map_err(BuildError::Dict)
        }

        /// char.def 파싱
        fn build_char_def(&self) -> Result<CharDef> {
            let char_def_path = Path::new(&self.config.input_dir).join("char.def");

            if self.config.verbose {
                tracing::info!("Loading char.def from {}", char_def_path.display());
            }

            if !char_def_path.exists() {
                if self.config.verbose {
                    tracing::warn!("char.def not found, skipping");
                }
                return Err(BuildError::Format("char.def not found".to_string()));
            }

            CharDef::from_file(&char_def_path)
        }

        /// unk.def 파싱
        fn build_unk_def(&self) -> Result<UnkDef> {
            let unk_def_path = Path::new(&self.config.input_dir).join("unk.def");

            if self.config.verbose {
                tracing::info!("Loading unk.def from {}", unk_def_path.display());
            }

            if !unk_def_path.exists() {
                if self.config.verbose {
                    tracing::warn!("unk.def not found, skipping");
                }
                return Err(BuildError::Format("unk.def not found".to_string()));
            }

            UnkDef::from_file(&unk_def_path)
        }

        /// Trie 및 사전 엔트리 빌드
        ///
        /// # Errors
        ///
        /// Returns an error if the trie cannot be built from the entries.
        pub fn build_trie_and_entries(
            &self,
            csv_entries: &[CsvEntry],
        ) -> Result<(Vec<u8>, Vec<DictEntry>)> {
            if self.config.verbose {
                tracing::info!("Building trie and dictionary entries...");
            }

            // 표면형별로 엔트리 그룹화
            let mut surface_map: HashMap<String, Vec<&CsvEntry>> = HashMap::new();
            for entry in csv_entries {
                surface_map
                    .entry(entry.surface.clone())
                    .or_default()
                    .push(entry);
            }

            // DictEntry 생성 (정렬하여 인덱스 일관성 보장)
            let mut dict_entries = Vec::new();
            let mut trie_entries = Vec::new();

            let mut surfaces: Vec<_> = surface_map.keys().collect();
            surfaces.sort();

            for surface in surfaces {
                let entries = &surface_map[surface];

                // 이 표면형의 첫 번째 인덱스를 Trie 값으로 사용
                #[allow(clippy::cast_possible_truncation)]
                let first_index = dict_entries.len() as u32;
                trie_entries.push((surface.as_str(), first_index));

                // 모든 엔트리를 DictEntry로 변환
                for csv_entry in entries {
                    let dict_entry = DictEntry::new(
                        csv_entry.surface.clone(),
                        csv_entry.left_id,
                        csv_entry.right_id,
                        csv_entry.cost,
                        csv_entry.to_feature(),
                    );
                    dict_entries.push(dict_entry);
                }
            }

            if self.config.verbose {
                tracing::info!("  Unique surfaces: {}", trie_entries.len());
                tracing::info!("  Total entries: {}", dict_entries.len());
            }

            // Trie 빌드 (이미 정렬됨)
            let trie_bytes = TrieBuilder::build(&trie_entries)
                .map_err(|e| BuildError::Trie(format!("Failed to build trie: {e}")))?;

            if self.config.verbose {
                tracing::info!("  Trie size: {} bytes", trie_bytes.len());
            }

            Ok((trie_bytes, dict_entries))
        }

        /// 사전 파일 저장
        fn save_dictionary(
            &self,
            trie_bytes: &[u8],
            matrix: &DenseMatrix,
            _dict_entries: &[DictEntry],
            char_def: Option<&CharDef>,
            unk_def: Option<&UnkDef>,
        ) -> Result<()> {
            let output_dir = Path::new(&self.config.output_dir);
            std::fs::create_dir_all(output_dir).map_err(BuildError::Io)?;

            if self.config.verbose {
                tracing::info!("Saving dictionary files to {}", output_dir.display());
            }

            // Trie 저장
            let trie_path = output_dir.join("sys.dic");
            if self.config.compression_level > 0 {
                if self.config.verbose {
                    tracing::info!(
                        "  Saving compressed trie (level {})...",
                        self.config.compression_level
                    );
                }
                let compressed_path = output_dir.join("sys.dic.zst");
                TrieBuilder::save_to_compressed_file(
                    trie_bytes,
                    &compressed_path,
                    self.config.compression_level,
                )
                .map_err(BuildError::Dict)?;

                if self.config.verbose {
                    let compressed_size = std::fs::metadata(&compressed_path)
                        .map_err(BuildError::Io)?
                        .len();
                    #[allow(clippy::cast_precision_loss)]
                    let ratio = (compressed_size as f64 / trie_bytes.len() as f64) * 100.0;
                    tracing::info!(
                        "  Compressed trie: {} bytes (ratio: {:.2}%)",
                        compressed_size,
                        ratio
                    );
                }
            } else {
                TrieBuilder::save_to_file(trie_bytes, &trie_path).map_err(BuildError::Dict)?;
            }

            // Matrix 저장
            let matrix_path = output_dir.join("matrix.bin");
            if self.config.compression_level > 0 {
                if self.config.verbose {
                    tracing::info!("  Saving compressed matrix...");
                }
                let compressed_path = output_dir.join("matrix.bin.zst");
                matrix
                    .to_compressed_file(&compressed_path, self.config.compression_level)
                    .map_err(BuildError::Dict)?;
            } else {
                matrix.to_bin_file(&matrix_path).map_err(BuildError::Dict)?;
            }

            // char.def 저장
            if let Some(char_def) = char_def {
                if self.config.verbose {
                    tracing::info!("  Saving char.def...");
                }
                let char_def_bytes = char_def.to_bytes();
                let char_def_path = output_dir.join("char.bin");
                std::fs::write(&char_def_path, char_def_bytes).map_err(BuildError::Io)?;
            }

            // unk.def 저장
            if let Some(unk_def) = unk_def {
                if self.config.verbose {
                    tracing::info!("  Saving unk.def...");
                }
                let unk_def_bytes = unk_def.to_bytes();
                let unk_def_path = output_dir.join("unk.bin");
                std::fs::write(&unk_def_path, unk_def_bytes).map_err(BuildError::Io)?;
            }

            if self.config.verbose {
                tracing::info!("Dictionary build completed successfully");
            }

            Ok(())
        }

        /// 입력 디렉토리 설정
        #[must_use]
        pub fn input_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
            self.config.input_dir = path.as_ref().to_string_lossy().to_string();
            self
        }

        /// 출력 디렉토리 설정
        #[must_use]
        pub fn output_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
            self.config.output_dir = path.as_ref().to_string_lossy().to_string();
            self
        }

        /// 압축 레벨 설정
        #[must_use]
        pub const fn compression_level(mut self, level: i32) -> Self {
            self.config.compression_level = level;
            self
        }
    }

    /// 빌드 결과
    #[derive(Debug, Clone)]
    pub struct BuildResult {
        /// 엔트리 수
        pub entry_count: usize,
        /// Trie 크기 (바이트)
        pub trie_size: usize,
        /// Matrix 크기 (엔트리 수)
        pub matrix_size: usize,
    }

    impl BuildResult {
        /// 결과 출력
        pub fn print_summary(&self) {
            println!("\n=== Build Summary ===");
            println!("Entries:      {}", self.entry_count);
            println!("Trie size:    {} bytes", self.trie_size);
            println!("Matrix size:  {} entries", self.matrix_size);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv_parser::{CsvParser, Encoding};
    use tempfile::TempDir;

    #[test]
    fn test_csv_entry_to_feature() {
        let entry = csv_parser::CsvEntry {
            surface: "가다".to_string(),
            left_id: 1,
            right_id: 1,
            cost: 100,
            pos: "VV".to_string(),
            pos_detail: "*".to_string(),
            jongseong: "F".to_string(),
            reading: "가다".to_string(),
            entry_type: "*".to_string(),
            first_pos: "VV".to_string(),
            last_pos: "VV".to_string(),
            expression: "*".to_string(),
        };

        let feature = entry.to_feature();
        assert!(feature.contains("VV"));
        assert!(feature.contains("가다"));
        assert_eq!(feature, "VV,*,F,가다,*,VV,VV,*");
    }

    #[test]
    fn test_csv_entry_normalize_jongseong() {
        let mut entry = csv_parser::CsvEntry {
            surface: "가방".to_string(),
            left_id: 1,
            right_id: 1,
            cost: 100,
            pos: "NNG".to_string(),
            pos_detail: "*".to_string(),
            jongseong: "*".to_string(),
            reading: "가방".to_string(),
            entry_type: "*".to_string(),
            first_pos: "NNG".to_string(),
            last_pos: "NNG".to_string(),
            expression: "*".to_string(),
        };

        entry.normalize_jongseong();
        assert_eq!(entry.jongseong, "T"); // 가방 has 종성

        entry.surface = "가다".to_string();
        entry.jongseong = "*".to_string();
        entry.normalize_jongseong();
        assert_eq!(entry.jongseong, "F"); // 가다 no 종성
    }

    #[test]
    fn test_csv_parser_basic() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let csv_path = temp_dir.path().join("test.csv");

        let csv_content = "가,1,2,100,NNG,*,T,가,*,NNG,NNG,*\n\
                          가다,2,3,200,VV,*,F,가다,*,VV,VV,*\n\
                          가방,3,4,300,NNG,*,T,가방,*,NNG,NNG,*\n";

        std::fs::write(&csv_path, csv_content).expect("failed to write test csv");

        let parser = CsvParser::new(temp_dir.path());
        let entries = parser.parse_file(&csv_path).expect("failed to parse");

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].surface, "가");
        assert_eq!(entries[0].left_id, 1);
        assert_eq!(entries[0].cost, 100);

        assert_eq!(entries[1].surface, "가다");
        assert_eq!(entries[2].surface, "가방");
    }

    #[test]
    fn test_csv_parser_encoding() {
        // 인코딩 설정은 private이므로 builder 패턴이 작동하는지만 확인
        let _parser = CsvParser::new(".").with_encoding(Encoding::Utf8);
        let _parser = CsvParser::new(".").with_encoding(Encoding::EucKr);
        let _parser = CsvParser::new(".");
        // 파서가 생성되면 테스트 성공
    }

    #[test]
    fn test_csv_parser_with_comments() {
        let csv_content = "# This is a comment\n\
                          가,1,2,100,NNG,*,T,가,*,NNG,NNG,*\n\
                          # Another comment\n\
                          가다,2,3,200,VV,*,F,가다,*,VV,VV,*\n";

        let parser = CsvParser::new(".");
        let entries = parser
            .parse_csv_content(csv_content)
            .expect("failed to parse");

        // 주석은 무시되어야 함
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_builder_creation() {
        let config = builder::BuildConfig::default();
        let _builder = DictionaryBuilder::new(config);
    }

    #[test]
    fn test_builder_config() {
        let config = builder::BuildConfig {
            input_dir: "./input".to_string(),
            output_dir: "./output".to_string(),
            compression_level: 5,
            encoding: Encoding::Utf8,
            verbose: true,
        };

        assert_eq!(config.input_dir, "./input");
        assert_eq!(config.output_dir, "./output");
        assert_eq!(config.compression_level, 5);
        assert!(config.verbose);
    }

    #[test]
    fn test_full_build_pipeline() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");

        // 테스트 CSV 파일 생성
        let csv_path = temp_dir.path().join("test.csv");
        let csv_content = "가,1,1,100,NNG,*,T,가,*,NNG,NNG,*\n\
                          가다,1,1,200,VV,*,F,가다,*,VV,VV,*\n\
                          가방,1,1,150,NNG,*,T,가방,*,NNG,NNG,*\n";
        std::fs::write(&csv_path, csv_content).expect("failed to write csv");

        // 테스트 matrix.def 파일 생성
        let matrix_path = temp_dir.path().join("matrix.def");
        let matrix_content = "2 2\n0 0 100\n0 1 200\n1 0 150\n1 1 50\n";
        std::fs::write(&matrix_path, matrix_content).expect("failed to write matrix");

        // 출력 디렉토리
        let output_dir = temp_dir.path().join("output");

        // 빌드 설정
        let config = builder::BuildConfig {
            input_dir: temp_dir.path().to_string_lossy().to_string(),
            output_dir: output_dir.to_string_lossy().to_string(),
            compression_level: 0, // 테스트에서는 압축 안 함
            encoding: Encoding::Utf8,
            verbose: false,
        };

        // 빌더 실행
        let builder = DictionaryBuilder::new(config);
        let result = builder.build().expect("build should succeed");

        // 결과 검증
        assert_eq!(result.entry_count, 3);
        assert!(result.trie_size > 0);
        assert_eq!(result.matrix_size, 4); // 2x2 matrix

        // 출력 파일 확인
        assert!(output_dir.join("sys.dic").exists());
        assert!(output_dir.join("matrix.bin").exists());
    }

    #[test]
    fn test_trie_and_entries_building() {
        use mecab_ko_dict::trie::Trie;

        let csv_entries = vec![
            csv_parser::CsvEntry {
                surface: "가".to_string(),
                left_id: 1,
                right_id: 1,
                cost: 100,
                pos: "NNG".to_string(),
                pos_detail: "*".to_string(),
                jongseong: "T".to_string(),
                reading: "가".to_string(),
                entry_type: "*".to_string(),
                first_pos: "NNG".to_string(),
                last_pos: "NNG".to_string(),
                expression: "*".to_string(),
            },
            csv_parser::CsvEntry {
                surface: "가다".to_string(),
                left_id: 2,
                right_id: 2,
                cost: 200,
                pos: "VV".to_string(),
                pos_detail: "*".to_string(),
                jongseong: "F".to_string(),
                reading: "가다".to_string(),
                entry_type: "*".to_string(),
                first_pos: "VV".to_string(),
                last_pos: "VV".to_string(),
                expression: "*".to_string(),
            },
        ];

        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let config = builder::BuildConfig {
            input_dir: temp_dir.path().to_string_lossy().to_string(),
            output_dir: temp_dir.path().to_string_lossy().to_string(),
            compression_level: 0,
            encoding: Encoding::Utf8,
            verbose: false,
        };

        let builder = DictionaryBuilder::new(config);
        let (trie_bytes, dict_entries) = builder
            .build_trie_and_entries(&csv_entries)
            .expect("should build trie");

        // Trie 검증
        assert!(!trie_bytes.is_empty());
        let trie = Trie::new(&trie_bytes);
        assert!(trie.exact_match("가").is_some());
        assert!(trie.exact_match("가다").is_some());
        assert!(trie.exact_match("없음").is_none());

        // 엔트리 검증
        assert_eq!(dict_entries.len(), 2);
        assert_eq!(dict_entries[0].surface, "가");
        assert_eq!(dict_entries[1].surface, "가다");
    }

    #[test]
    fn test_korean_text_processing() {
        // 한글 종성 감지 테스트
        let mut entry1 = csv_parser::CsvEntry {
            surface: "안녕".to_string(),
            left_id: 1,
            right_id: 1,
            cost: 100,
            pos: "NNG".to_string(),
            pos_detail: "*".to_string(),
            jongseong: "*".to_string(),
            reading: "안녕".to_string(),
            entry_type: "*".to_string(),
            first_pos: "NNG".to_string(),
            last_pos: "NNG".to_string(),
            expression: "*".to_string(),
        };

        entry1.normalize_jongseong();
        assert_eq!(entry1.jongseong, "T"); // 안녕 has 종성 (ㅇ)

        let mut entry2 = csv_parser::CsvEntry {
            surface: "하세요".to_string(),
            left_id: 1,
            right_id: 1,
            cost: 100,
            pos: "VV".to_string(),
            pos_detail: "*".to_string(),
            jongseong: "*".to_string(),
            reading: "하세요".to_string(),
            entry_type: "*".to_string(),
            first_pos: "VV".to_string(),
            last_pos: "VV".to_string(),
            expression: "*".to_string(),
        };

        entry2.normalize_jongseong();
        assert_eq!(entry2.jongseong, "F"); // 하세요 no 종성
    }

    #[test]
    fn test_build_result() {
        let result = builder::BuildResult {
            entry_count: 1000,
            trie_size: 50000,
            matrix_size: 400,
        };

        assert_eq!(result.entry_count, 1000);
        assert_eq!(result.trie_size, 50000);
        assert_eq!(result.matrix_size, 400);

        // print_summary는 단순 출력이므로 패닉이 없어야 함
        result.print_summary();
    }
}
