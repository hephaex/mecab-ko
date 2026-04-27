//! Dictionary entry converter from external formats to MeCab-Ko format.

use std::collections::HashMap;

use crate::{Error, Result};

/// Dictionary entry from an external source (e.g., NIKL Open Dictionary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConverterEntry {
    /// Surface form (표면형).
    pub surface: String,

    /// POS tag in source format (e.g., "명사", "동사").
    pub pos: String,

    /// Reading/pronunciation (optional).
    pub reading: Option<String>,

    /// Frequency or usage count (optional, used for cost calculation).
    pub frequency: Option<u32>,
}

/// User dictionary entry in MeCab-Ko format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserEntry {
    /// Surface form (표면형).
    pub surface: String,

    /// Left context ID (0 = auto-determine).
    pub left_id: i16,

    /// Right context ID (0 = auto-determine).
    pub right_id: i16,

    /// Cost (lower = higher priority).
    pub cost: i16,

    /// POS tag in MeCab-Ko format (e.g., "NNG", "NNP").
    pub pos: String,

    /// Reading/pronunciation (optional).
    pub reading: Option<String>,
}

impl UserEntry {
    /// Convert to MeCab-Ko CSV line format.
    ///
    /// Format: `표면형,좌ID,우ID,비용,품사,*,*,*,읽기,원형,읽기,*`
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::UserEntry;
    ///
    /// let entry = UserEntry {
    ///     surface: "챗GPT".to_string(),
    ///     left_id: 0,
    ///     right_id: 0,
    ///     cost: 0,
    ///     pos: "NNP".to_string(),
    ///     reading: Some("챗지피티".to_string()),
    /// };
    ///
    /// let csv_line = entry.to_csv_line();
    /// assert_eq!(csv_line, "챗GPT,0,0,0,NNP,*,*,*,챗지피티,챗GPT,챗지피티,*");
    /// ```
    #[must_use]
    pub fn to_csv_line(&self) -> String {
        let reading = self.reading.as_deref().unwrap_or("*");
        format!(
            "{},{},{},{},{},*,*,*,{},{},{},*",
            self.surface,
            self.left_id,
            self.right_id,
            self.cost,
            self.pos,
            reading,
            self.surface,
            reading
        )
    }
}

/// Dictionary converter that maps external dictionary formats to MeCab-Ko format.
pub struct DictConverter {
    pos_mapping: HashMap<String, String>,
}

impl DictConverter {
    /// Create a new converter with default POS mappings.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::DictConverter;
    ///
    /// let converter = DictConverter::new();
    /// assert_eq!(converter.map_pos("명사"), Ok("NNG"));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let mut pos_mapping = HashMap::new();

        // 명사 (Nouns)
        pos_mapping.insert("명사".to_string(), "NNG".to_string());
        pos_mapping.insert("일반명사".to_string(), "NNG".to_string());
        pos_mapping.insert("보통명사".to_string(), "NNG".to_string());
        pos_mapping.insert("고유명사".to_string(), "NNP".to_string());
        pos_mapping.insert("고유_명사".to_string(), "NNP".to_string());
        pos_mapping.insert("의존명사".to_string(), "NNB".to_string());
        pos_mapping.insert("단위명사".to_string(), "NNBC".to_string());

        // 대명사 (Pronouns)
        pos_mapping.insert("대명사".to_string(), "NP".to_string());

        // 수사 (Numerals)
        pos_mapping.insert("수사".to_string(), "NR".to_string());

        // 동사 (Verbs)
        pos_mapping.insert("동사".to_string(), "VV".to_string());
        pos_mapping.insert("일반동사".to_string(), "VV".to_string());
        pos_mapping.insert("보조동사".to_string(), "VX".to_string());

        // 형용사 (Adjectives)
        pos_mapping.insert("형용사".to_string(), "VA".to_string());
        pos_mapping.insert("일반형용사".to_string(), "VA".to_string());

        // 관형사 (Determiners)
        pos_mapping.insert("관형사".to_string(), "MM".to_string());

        // 부사 (Adverbs)
        pos_mapping.insert("부사".to_string(), "MAG".to_string());
        pos_mapping.insert("일반부사".to_string(), "MAG".to_string());
        pos_mapping.insert("접속부사".to_string(), "MAJ".to_string());

        // 감탄사 (Interjections)
        pos_mapping.insert("감탄사".to_string(), "IC".to_string());

        // 조사 (Particles)
        pos_mapping.insert("조사".to_string(), "JX".to_string());
        pos_mapping.insert("격조사".to_string(), "JKS".to_string());
        pos_mapping.insert("보조사".to_string(), "JX".to_string());
        pos_mapping.insert("접속조사".to_string(), "JC".to_string());

        // 어미 (Endings)
        pos_mapping.insert("어미".to_string(), "EF".to_string());
        pos_mapping.insert("선어말어미".to_string(), "EP".to_string());
        pos_mapping.insert("종결어미".to_string(), "EF".to_string());
        pos_mapping.insert("연결어미".to_string(), "EC".to_string());

        // 접두사/접미사 (Affixes)
        pos_mapping.insert("접두사".to_string(), "XPN".to_string());
        pos_mapping.insert("접미사".to_string(), "XSN".to_string());

        // 어근 (Roots)
        pos_mapping.insert("어근".to_string(), "XR".to_string());

        // 기호 (Symbols)
        pos_mapping.insert("기호".to_string(), "SF".to_string());
        pos_mapping.insert("외국어".to_string(), "SL".to_string());
        pos_mapping.insert("한자".to_string(), "SH".to_string());
        pos_mapping.insert("숫자".to_string(), "SN".to_string());

        Self { pos_mapping }
    }

    /// Map a POS tag from NIKL format to MeCab-Ko format.
    ///
    /// # Errors
    ///
    /// Returns `Error::UnknownPosTag` if the POS tag is not recognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::DictConverter;
    ///
    /// let converter = DictConverter::new();
    /// assert_eq!(converter.map_pos("명사").unwrap(), "NNG");
    /// assert_eq!(converter.map_pos("고유명사").unwrap(), "NNP");
    /// assert_eq!(converter.map_pos("동사").unwrap(), "VV");
    /// ```
    pub fn map_pos(&self, nikl_pos: &str) -> Result<&str> {
        self.pos_mapping
            .get(nikl_pos)
            .map(String::as_str)
            .ok_or_else(|| Error::UnknownPosTag(nikl_pos.to_string()))
    }

    /// Calculate cost for a dictionary entry.
    ///
    /// Cost calculation strategy:
    /// - Base cost: 0 for high-frequency words (frequency >= 1000)
    /// - Medium frequency (100-999): cost = 500
    /// - Low frequency (< 100): cost = 1000
    /// - No frequency data: cost = 500 (medium priority)
    /// - Longer words (> 5 chars): reduce cost by 100 (prefer longer matches)
    ///
    /// Lower cost = higher priority in `MeCab`'s lattice search.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::{DictConverter, ConverterEntry};
    ///
    /// let converter = DictConverter::new();
    ///
    /// // High frequency word
    /// let entry = ConverterEntry {
    ///     surface: "챗GPT".to_string(),
    ///     pos: "고유명사".to_string(),
    ///     reading: Some("챗지피티".to_string()),
    ///     frequency: Some(5000),
    /// };
    /// assert_eq!(converter.calculate_cost(&entry), 0);
    ///
    /// // Medium frequency word
    /// let entry = ConverterEntry {
    ///     surface: "메타버스".to_string(),
    ///     pos: "명사".to_string(),
    ///     reading: None,
    ///     frequency: Some(500),
    /// };
    /// assert_eq!(converter.calculate_cost(&entry), 500);
    ///
    /// // Low frequency word
    /// let entry = ConverterEntry {
    ///     surface: "갓생".to_string(),
    ///     pos: "명사".to_string(),
    ///     reading: None,
    ///     frequency: Some(50),
    /// };
    /// assert_eq!(converter.calculate_cost(&entry), 1000);
    /// ```
    #[must_use]
    pub fn calculate_cost(&self, entry: &ConverterEntry) -> i16 {
        // Base cost based on frequency
        let base_cost = match entry.frequency {
            Some(freq) if freq >= 1000 => 0,
            Some(freq) if freq >= 100 => 500,
            Some(_) => 1000,
            None => 500, // Default medium priority
        };

        // Adjust for word length (prefer longer matches)
        let length_adjustment = if entry.surface.chars().count() > 5 {
            -100
        } else {
            0
        };

        // Ensure cost stays within valid range
        (base_cost + length_adjustment).max(0)
    }

    /// Convert a dictionary entry to MeCab-Ko user entry format.
    ///
    /// # Errors
    ///
    /// Returns `Error::UnknownPosTag` if the POS tag cannot be mapped.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::{DictConverter, ConverterEntry};
    ///
    /// let converter = DictConverter::new();
    /// let entry = ConverterEntry {
    ///     surface: "챗GPT".to_string(),
    ///     pos: "고유명사".to_string(),
    ///     reading: Some("챗지피티".to_string()),
    ///     frequency: Some(1000),
    /// };
    ///
    /// let user_entry = converter.convert_entry(&entry).unwrap();
    /// assert_eq!(user_entry.surface, "챗GPT");
    /// assert_eq!(user_entry.pos, "NNP");
    /// assert_eq!(user_entry.cost, 0);
    /// assert_eq!(user_entry.reading, Some("챗지피티".to_string()));
    /// ```
    pub fn convert_entry(&self, entry: &ConverterEntry) -> Result<UserEntry> {
        let pos = self.map_pos(&entry.pos)?.to_string();
        let cost = self.calculate_cost(entry);

        Ok(UserEntry {
            surface: entry.surface.clone(),
            left_id: 0,  // Auto-determine
            right_id: 0, // Auto-determine
            cost,
            pos,
            reading: entry.reading.clone(),
        })
    }

    /// Convert multiple entries and return CSV lines.
    ///
    /// # Errors
    ///
    /// Returns an error if any entry fails to convert.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::{DictConverter, ConverterEntry};
    ///
    /// let converter = DictConverter::new();
    /// let entries = vec![
    ///     ConverterEntry {
    ///         surface: "챗GPT".to_string(),
    ///         pos: "고유명사".to_string(),
    ///         reading: Some("챗지피티".to_string()),
    ///         frequency: Some(1000),
    ///     },
    ///     ConverterEntry {
    ///         surface: "메타버스".to_string(),
    ///         pos: "명사".to_string(),
    ///         reading: Some("메타버스".to_string()),
    ///         frequency: Some(500),
    ///     },
    /// ];
    ///
    /// let csv_lines = converter.convert_to_csv(&entries).unwrap();
    /// assert_eq!(csv_lines.len(), 2);
    /// assert!(csv_lines[0].starts_with("챗GPT,0,0,0,NNP"));
    /// assert!(csv_lines[1].starts_with("메타버스,0,0,500,NNG"));
    /// ```
    pub fn convert_to_csv(&self, entries: &[ConverterEntry]) -> Result<Vec<String>> {
        entries
            .iter()
            .map(|entry| {
                self.convert_entry(entry)
                    .map(|user_entry| user_entry.to_csv_line())
            })
            .collect()
    }

    /// Add a custom POS mapping.
    ///
    /// This allows extending the converter with additional mappings not in the default set.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::DictConverter;
    ///
    /// let mut converter = DictConverter::new();
    /// converter.add_pos_mapping("커스텀명사".to_string(), "NNG".to_string());
    /// assert_eq!(converter.map_pos("커스텀명사").unwrap(), "NNG");
    /// ```
    pub fn add_pos_mapping(&mut self, nikl_pos: String, mecab_pos: String) {
        self.pos_mapping.insert(nikl_pos, mecab_pos);
    }

    /// Get all supported POS mappings.
    ///
    /// Returns an iterator over (NIKL POS, `MeCab` POS) pairs.
    pub fn pos_mappings(&self) -> impl Iterator<Item = (&str, &str)> {
        self.pos_mapping
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

impl Default for DictConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_pos_mapping_nouns() {
        let converter = DictConverter::new();
        assert_eq!(converter.map_pos("명사").unwrap(), "NNG");
        assert_eq!(converter.map_pos("일반명사").unwrap(), "NNG");
        assert_eq!(converter.map_pos("고유명사").unwrap(), "NNP");
        assert_eq!(converter.map_pos("의존명사").unwrap(), "NNB");
    }

    #[test]
    fn test_pos_mapping_verbs() {
        let converter = DictConverter::new();
        assert_eq!(converter.map_pos("동사").unwrap(), "VV");
        assert_eq!(converter.map_pos("형용사").unwrap(), "VA");
        assert_eq!(converter.map_pos("보조동사").unwrap(), "VX");
    }

    #[test]
    fn test_pos_mapping_adverbs() {
        let converter = DictConverter::new();
        assert_eq!(converter.map_pos("부사").unwrap(), "MAG");
        assert_eq!(converter.map_pos("일반부사").unwrap(), "MAG");
        assert_eq!(converter.map_pos("접속부사").unwrap(), "MAJ");
    }

    #[test]
    fn test_pos_mapping_interjections() {
        let converter = DictConverter::new();
        assert_eq!(converter.map_pos("감탄사").unwrap(), "IC");
    }

    #[test]
    fn test_pos_mapping_unknown() {
        let converter = DictConverter::new();
        let result = converter.map_pos("알수없는품사");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::UnknownPosTag(_)));
    }

    #[test]
    fn test_calculate_cost_high_frequency() {
        let converter = DictConverter::new();
        let entry = ConverterEntry {
            surface: "챗GPT".to_string(),
            pos: "고유명사".to_string(),
            reading: Some("챗지피티".to_string()),
            frequency: Some(5000),
        };
        assert_eq!(converter.calculate_cost(&entry), 0);
    }

    #[test]
    fn test_calculate_cost_medium_frequency() {
        let converter = DictConverter::new();
        let entry = ConverterEntry {
            surface: "메타버스".to_string(),
            pos: "명사".to_string(),
            reading: None,
            frequency: Some(500),
        };
        assert_eq!(converter.calculate_cost(&entry), 500);
    }

    #[test]
    fn test_calculate_cost_low_frequency() {
        let converter = DictConverter::new();
        let entry = ConverterEntry {
            surface: "갓생".to_string(),
            pos: "명사".to_string(),
            reading: None,
            frequency: Some(50),
        };
        assert_eq!(converter.calculate_cost(&entry), 1000);
    }

    #[test]
    fn test_calculate_cost_no_frequency() {
        let converter = DictConverter::new();
        let entry = ConverterEntry {
            surface: "워라밸".to_string(),
            pos: "명사".to_string(),
            reading: None,
            frequency: None,
        };
        assert_eq!(converter.calculate_cost(&entry), 500);
    }

    #[test]
    fn test_calculate_cost_long_word() {
        let converter = DictConverter::new();
        let entry = ConverterEntry {
            surface: "디지털노마드".to_string(), // 7 chars
            pos: "명사".to_string(),
            reading: None,
            frequency: Some(500),
        };
        assert_eq!(converter.calculate_cost(&entry), 400); // 500 - 100
    }

    #[test]
    fn test_calculate_cost_long_word_boundary() {
        let converter = DictConverter::new();
        let entry = ConverterEntry {
            surface: "메타버스".to_string(), // 5 chars (not > 5)
            pos: "명사".to_string(),
            reading: None,
            frequency: Some(500),
        };
        assert_eq!(converter.calculate_cost(&entry), 500); // No adjustment
    }

    #[test]
    fn test_convert_entry_basic() {
        let converter = DictConverter::new();
        let entry = ConverterEntry {
            surface: "챗GPT".to_string(),
            pos: "고유명사".to_string(),
            reading: Some("챗지피티".to_string()),
            frequency: Some(1000),
        };

        let user_entry = converter.convert_entry(&entry).unwrap();
        assert_eq!(user_entry.surface, "챗GPT");
        assert_eq!(user_entry.pos, "NNP");
        assert_eq!(user_entry.left_id, 0);
        assert_eq!(user_entry.right_id, 0);
        assert_eq!(user_entry.cost, 0);
        assert_eq!(user_entry.reading, Some("챗지피티".to_string()));
    }

    #[test]
    fn test_convert_entry_without_reading() {
        let converter = DictConverter::new();
        let entry = ConverterEntry {
            surface: "메타버스".to_string(),
            pos: "명사".to_string(),
            reading: None,
            frequency: Some(500),
        };

        let user_entry = converter.convert_entry(&entry).unwrap();
        assert_eq!(user_entry.surface, "메타버스");
        assert_eq!(user_entry.pos, "NNG");
        assert_eq!(user_entry.reading, None);
    }

    #[test]
    fn test_user_entry_to_csv_line() {
        let entry = UserEntry {
            surface: "챗GPT".to_string(),
            left_id: 0,
            right_id: 0,
            cost: 0,
            pos: "NNP".to_string(),
            reading: Some("챗지피티".to_string()),
        };

        let csv_line = entry.to_csv_line();
        assert_eq!(csv_line, "챗GPT,0,0,0,NNP,*,*,*,챗지피티,챗GPT,챗지피티,*");
    }

    #[test]
    fn test_user_entry_to_csv_line_no_reading() {
        let entry = UserEntry {
            surface: "메타버스".to_string(),
            left_id: 0,
            right_id: 0,
            cost: 500,
            pos: "NNG".to_string(),
            reading: None,
        };

        let csv_line = entry.to_csv_line();
        assert_eq!(csv_line, "메타버스,0,0,500,NNG,*,*,*,*,메타버스,*,*");
    }

    #[test]
    fn test_convert_to_csv() {
        let converter = DictConverter::new();
        let entries = vec![
            ConverterEntry {
                surface: "챗GPT".to_string(),
                pos: "고유명사".to_string(),
                reading: Some("챗지피티".to_string()),
                frequency: Some(1000),
            },
            ConverterEntry {
                surface: "메타버스".to_string(),
                pos: "명사".to_string(),
                reading: Some("메타버스".to_string()),
                frequency: Some(500),
            },
        ];

        let csv_lines = converter.convert_to_csv(&entries).unwrap();
        assert_eq!(csv_lines.len(), 2);
        assert_eq!(
            csv_lines[0],
            "챗GPT,0,0,0,NNP,*,*,*,챗지피티,챗GPT,챗지피티,*"
        );
        assert_eq!(
            csv_lines[1],
            "메타버스,0,0,500,NNG,*,*,*,메타버스,메타버스,메타버스,*"
        );
    }

    #[test]
    fn test_add_pos_mapping() {
        let mut converter = DictConverter::new();
        converter.add_pos_mapping("커스텀명사".to_string(), "NNG".to_string());
        assert_eq!(converter.map_pos("커스텀명사").unwrap(), "NNG");
    }

    #[test]
    fn test_pos_mappings_iter() {
        let converter = DictConverter::new();
        let mappings: Vec<_> = converter.pos_mappings().collect();
        assert!(!mappings.is_empty());
        assert!(mappings.contains(&("명사", "NNG")));
        assert!(mappings.contains(&("고유명사", "NNP")));
    }

    #[test]
    fn test_convert_entry_unknown_pos() {
        let converter = DictConverter::new();
        let entry = ConverterEntry {
            surface: "테스트".to_string(),
            pos: "알수없는품사".to_string(),
            reading: None,
            frequency: None,
        };

        let result = converter.convert_entry(&entry);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::UnknownPosTag(_)));
    }

    #[test]
    fn test_comprehensive_pos_categories() {
        let converter = DictConverter::new();

        // All major POS categories should be mappable
        let test_cases = vec![
            // Nouns
            ("명사", "NNG"),
            ("고유명사", "NNP"),
            ("의존명사", "NNB"),
            // Verbs
            ("동사", "VV"),
            ("형용사", "VA"),
            // Adverbs
            ("부사", "MAG"),
            // Interjections
            ("감탄사", "IC"),
            // Determiners
            ("관형사", "MM"),
            // Pronouns
            ("대명사", "NP"),
        ];

        for (nikl, expected) in test_cases {
            assert_eq!(converter.map_pos(nikl).unwrap(), expected);
        }
    }

    #[test]
    fn test_dict_entry_equality() {
        let entry1 = ConverterEntry {
            surface: "테스트".to_string(),
            pos: "명사".to_string(),
            reading: None,
            frequency: Some(100),
        };

        let entry2 = ConverterEntry {
            surface: "테스트".to_string(),
            pos: "명사".to_string(),
            reading: None,
            frequency: Some(100),
        };

        assert_eq!(entry1, entry2);
    }

    #[test]
    fn test_user_entry_equality() {
        let entry1 = UserEntry {
            surface: "테스트".to_string(),
            left_id: 0,
            right_id: 0,
            cost: 500,
            pos: "NNG".to_string(),
            reading: None,
        };

        let entry2 = UserEntry {
            surface: "테스트".to_string(),
            left_id: 0,
            right_id: 0,
            cost: 500,
            pos: "NNG".to_string(),
            reading: None,
        };

        assert_eq!(entry1, entry2);
    }
}
