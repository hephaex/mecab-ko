//! unk.def 파서
//!
//! 미등록어 정의 파일을 파싱합니다.

use crate::{BuildError, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// 미등록어 엔트리
#[derive(Debug, Clone)]
pub struct UnkEntry {
    /// 문자 타입 (예: DEFAULT, SPACE, ALPHA)
    pub char_type: String,
    /// 좌문맥 ID
    pub left_id: u16,
    /// 우문맥 ID
    pub right_id: u16,
    /// 비용
    pub cost: i16,
    /// Feature 문자열
    pub feature: String,
}

/// unk.def 정의
#[derive(Debug, Clone)]
pub struct UnkDef {
    /// 미등록어 엔트리 목록
    pub entries: Vec<UnkEntry>,
}

impl UnkDef {
    /// unk.def 파일 파싱
    ///
    /// # Format
    /// ```text
    /// # 문자타입,좌ID,우ID,비용,Feature
    /// DEFAULT,0,0,0,*
    /// SPACE,0,0,0,*
    /// ALPHA,1,2,100,UNK,*,*,*,*,*,*,*
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened or read
    /// - A line has fewer than 5 fields
    /// - The `left_id`, `right_id`, or cost fields cannot be parsed
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref()).map_err(BuildError::Io)?;
        let reader = BufReader::new(file);

        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(BuildError::Io)?;
            let line = line.trim();

            // 주석과 빈 줄 무시
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // CSV 파싱
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 5 {
                return Err(BuildError::Format(format!(
                    "Invalid unk.def entry at line {}: expected at least 5 fields, got {}",
                    line_num + 1,
                    parts.len()
                )));
            }

            let char_type = parts[0].trim().to_string();
            let left_id = parts[1].trim().parse::<u16>().map_err(|_| {
                BuildError::Format(format!("Invalid left_id at line {}", line_num + 1))
            })?;
            let right_id = parts[2].trim().parse::<u16>().map_err(|_| {
                BuildError::Format(format!("Invalid right_id at line {}", line_num + 1))
            })?;
            let cost = parts[3].trim().parse::<i16>().map_err(|_| {
                BuildError::Format(format!("Invalid cost at line {}", line_num + 1))
            })?;

            // 나머지는 feature (콤마로 재결합)
            let feature = parts[4..].join(",");

            entries.push(UnkEntry {
                char_type,
                left_id,
                right_id,
                cost,
                feature,
            });
        }

        Ok(Self { entries })
    }

    /// 문자 타입으로 엔트리 조회
    #[must_use]
    pub fn get_entry(&self, char_type: &str) -> Option<&UnkEntry> {
        self.entries.iter().find(|e| e.char_type == char_type)
    }

    /// 바이너리로 직렬화
    ///
    /// # Panics
    ///
    /// Writing to a `Vec<u8>` should never fail in practice.
    /// If it does, the process is in an unrecoverable state.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_bytes(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use std::io::Write;

        let mut buf = Vec::new();

        // 엔트리 개수
        // SAFETY: Writing to Vec<u8> in memory should never fail
        buf.write_u32::<LittleEndian>(self.entries.len() as u32)
            .unwrap_or_else(|_| unreachable!("Vec write failed"));

        // 엔트리 데이터
        for entry in &self.entries {
            // 문자 타입
            buf.write_u32::<LittleEndian>(entry.char_type.len() as u32)
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
            buf.write_all(entry.char_type.as_bytes())
                .unwrap_or_else(|_| unreachable!("Vec write failed"));

            // IDs와 비용
            buf.write_u16::<LittleEndian>(entry.left_id)
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
            buf.write_u16::<LittleEndian>(entry.right_id)
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
            buf.write_i16::<LittleEndian>(entry.cost)
                .unwrap_or_else(|_| unreachable!("Vec write failed"));

            // Feature
            buf.write_u32::<LittleEndian>(entry.feature.len() as u32)
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
            buf.write_all(entry.feature.as_bytes())
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
        }

        buf
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::needless_raw_string_hashes
)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_unk_def() {
        let content = r#"
# Unknown word definition
DEFAULT,0,0,0,*
SPACE,0,0,0,*
ALPHA,1,2,100,UNK,*,*,*,*,*,*,*
NUMERIC,1,3,150,SN,*,*,*,*,*,*,*
"#;

        let mut tmp = NamedTempFile::new().expect("create temp file");
        tmp.write_all(content.as_bytes()).expect("write");

        let unk_def = UnkDef::from_file(tmp.path()).expect("parse failed");

        assert_eq!(unk_def.entries.len(), 4);

        let default_entry = unk_def.get_entry("DEFAULT").expect("DEFAULT entry");
        assert_eq!(default_entry.left_id, 0);
        assert_eq!(default_entry.cost, 0);
        assert_eq!(default_entry.feature, "*");

        let alpha_entry = unk_def.get_entry("ALPHA").expect("ALPHA entry");
        assert_eq!(alpha_entry.left_id, 1);
        assert_eq!(alpha_entry.right_id, 2);
        assert_eq!(alpha_entry.cost, 100);
        assert!(alpha_entry.feature.contains("UNK"));
    }

    #[test]
    fn test_unk_entry_with_complex_feature() {
        let content = "SYMBOL,5,6,200,SF,*,*,*,*,*,*,*\n";
        let mut tmp = NamedTempFile::new().expect("create temp file");
        tmp.write_all(content.as_bytes()).expect("write");

        let unk_def = UnkDef::from_file(tmp.path()).expect("parse failed");

        assert_eq!(unk_def.entries.len(), 1);

        let entry = &unk_def.entries[0];
        assert_eq!(entry.char_type, "SYMBOL");
        assert_eq!(entry.feature, "SF,*,*,*,*,*,*,*");
    }

    #[test]
    fn test_serialization() {
        let content = "DEFAULT,0,0,0,*\nSPACE,0,0,0,*\n";
        let mut tmp = NamedTempFile::new().expect("create temp file");
        tmp.write_all(content.as_bytes()).expect("write");

        let unk_def = UnkDef::from_file(tmp.path()).expect("parse failed");
        let bytes = unk_def.to_bytes();

        assert!(!bytes.is_empty());
        assert!(bytes.len() > 10);
    }

    #[test]
    fn test_empty_unk_def() {
        let content = "# Only comments\n# No entries\n";
        let mut tmp = NamedTempFile::new().expect("create temp file");
        tmp.write_all(content.as_bytes()).expect("write");

        let unk_def = UnkDef::from_file(tmp.path()).expect("parse failed");
        assert_eq!(unk_def.entries.len(), 0);
    }
}
