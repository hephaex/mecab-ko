//! char.def 파서
//!
//! 문자 타입 정의 파일을 파싱합니다.

use crate::{BuildError, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// 문자 타입 정의
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharType {
    /// 타입 이름
    pub name: String,
    /// Invoke 플래그 (형태소 분석 호출 여부)
    pub invoke: bool,
    /// Group 플래그 (그룹화 여부)
    pub group: bool,
    /// Length (길이 제한)
    pub length: u32,
}

/// 문자 타입 매핑
#[derive(Debug, Clone)]
pub struct CharMapping {
    /// 문자 코드
    pub code: u32,
    /// 타입 이름
    pub type_name: String,
}

/// char.def 전체 정의
#[derive(Debug, Clone)]
pub struct CharDef {
    /// 문자 타입 목록
    pub types: Vec<CharType>,
    /// 문자별 타입 매핑
    pub mappings: Vec<CharMapping>,
    /// 기본 타입 (DEFAULT)
    pub default_type: Option<String>,
}

impl CharDef {
    /// char.def 파일 파싱
    ///
    /// # Format
    /// ```text
    /// # 타입 정의
    /// DEFAULT invoke group length
    /// SPACE   invoke group length
    ///
    /// # 문자 매핑
    /// 0x0020 SPACE    # 공백
    /// 0x0009 SPACE    # 탭
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened or read
    /// - The format is invalid (malformed type definitions or mappings)
    /// - Hex codes cannot be parsed
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref()).map_err(BuildError::Io)?;
        let reader = BufReader::new(file);

        let mut types = Vec::new();
        let mut mappings = Vec::new();
        let mut default_type = None;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(BuildError::Io)?;
            let line = line.trim();

            // 주석과 빈 줄 무시
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 공백으로 분리
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            // 0x로 시작하면 매핑, 아니면 타입 정의
            if parts[0].starts_with("0x") || parts[0].starts_with("0X") {
                // 문자 매핑: 0xXXXX TYPE_NAME
                if parts.len() < 2 {
                    return Err(BuildError::Format(format!(
                        "Invalid character mapping at line {}: {}",
                        line_num + 1,
                        line
                    )));
                }

                let code_str = parts[0].trim_start_matches("0x").trim_start_matches("0X");
                let code = u32::from_str_radix(code_str, 16).map_err(|_| {
                    BuildError::Format(format!(
                        "Invalid hex code at line {}: {}",
                        line_num + 1,
                        parts[0]
                    ))
                })?;

                mappings.push(CharMapping {
                    code,
                    type_name: parts[1].to_string(),
                });
            } else {
                // 타입 정의: TYPE_NAME invoke group length
                if parts.len() < 4 {
                    return Err(BuildError::Format(format!(
                        "Invalid type definition at line {}: {}",
                        line_num + 1,
                        line
                    )));
                }

                let name = parts[0].to_string();
                let invoke = parts[1] != "0";
                let group = parts[2] != "0";
                let length = parts[3].parse::<u32>().map_err(|_| {
                    BuildError::Format(format!(
                        "Invalid length at line {}: {}",
                        line_num + 1,
                        parts[3]
                    ))
                })?;

                if name == "DEFAULT" {
                    default_type = Some(name.clone());
                }

                types.push(CharType {
                    name,
                    invoke,
                    group,
                    length,
                });
            }
        }

        Ok(Self {
            types,
            mappings,
            default_type,
        })
    }

    /// 타입 이름으로 타입 정의 조회
    #[must_use]
    pub fn get_type(&self, name: &str) -> Option<&CharType> {
        self.types.iter().find(|t| t.name == name)
    }

    /// 문자 코드로 타입 조회
    #[must_use]
    pub fn get_type_for_char(&self, ch: char) -> Option<&CharType> {
        let code = ch as u32;

        // 매핑에서 찾기
        if let Some(mapping) = self.mappings.iter().find(|m| m.code == code) {
            return self.get_type(&mapping.type_name);
        }

        // 기본 타입 반환
        if let Some(default) = &self.default_type {
            return self.get_type(default);
        }

        None
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

        // 타입 개수
        // SAFETY: Writing to Vec<u8> in memory should never fail
        buf.write_u32::<LittleEndian>(self.types.len() as u32)
            .unwrap_or_else(|_| unreachable!("Vec write failed"));

        // 타입 정의
        for typ in &self.types {
            // 이름 길이 + 이름
            buf.write_u32::<LittleEndian>(typ.name.len() as u32)
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
            buf.write_all(typ.name.as_bytes())
                .unwrap_or_else(|_| unreachable!("Vec write failed"));

            // 플래그
            buf.write_u8(u8::from(typ.invoke))
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
            buf.write_u8(u8::from(typ.group))
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
            buf.write_u32::<LittleEndian>(typ.length)
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
        }

        // 매핑 개수
        buf.write_u32::<LittleEndian>(self.mappings.len() as u32)
            .unwrap_or_else(|_| unreachable!("Vec write failed"));

        // 매핑 데이터
        for mapping in &self.mappings {
            buf.write_u32::<LittleEndian>(mapping.code)
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
            buf.write_u32::<LittleEndian>(mapping.type_name.len() as u32)
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
            buf.write_all(mapping.type_name.as_bytes())
                .unwrap_or_else(|_| unreachable!("Vec write failed"));
        }

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_char_def() {
        let content = r#"
# Character type definition
DEFAULT 1 0 0
SPACE   0 1 0
ALPHA   1 1 0

# Character mappings
0x0020 SPACE
0x0009 SPACE
0x0041 ALPHA
"#;

        let mut tmp = NamedTempFile::new().expect("create temp file");
        tmp.write_all(content.as_bytes()).expect("write");

        let char_def = CharDef::from_file(tmp.path()).expect("parse failed");

        assert_eq!(char_def.types.len(), 3);
        assert_eq!(char_def.mappings.len(), 3);
        assert_eq!(char_def.default_type, Some("DEFAULT".to_string()));

        let space_type = char_def.get_type("SPACE").expect("SPACE type");
        assert!(!space_type.invoke);
        assert!(space_type.group);

        let alpha_type = char_def.get_type("ALPHA").expect("ALPHA type");
        assert!(alpha_type.invoke);
        assert!(alpha_type.group);
    }

    #[test]
    fn test_get_type_for_char() {
        let content = "DEFAULT 1 0 0\nSPACE 0 1 0\n0x0020 SPACE\n";
        let mut tmp = NamedTempFile::new().expect("create temp file");
        tmp.write_all(content.as_bytes()).expect("write");

        let char_def = CharDef::from_file(tmp.path()).expect("parse failed");

        let space_char = ' ';
        let typ = char_def.get_type_for_char(space_char).expect("type");
        assert_eq!(typ.name, "SPACE");

        let other_char = 'a';
        let typ = char_def.get_type_for_char(other_char).expect("type");
        assert_eq!(typ.name, "DEFAULT");
    }

    #[test]
    fn test_serialization() {
        let content = "DEFAULT 1 0 0\nSPACE 0 1 0\n0x0020 SPACE\n";
        let mut tmp = NamedTempFile::new().expect("create temp file");
        tmp.write_all(content.as_bytes()).expect("write");

        let char_def = CharDef::from_file(tmp.path()).expect("parse failed");
        let bytes = char_def.to_bytes();

        assert!(!bytes.is_empty());
        // 기본적인 직렬화 검증
        assert!(bytes.len() > 20); // 최소 크기
    }
}
