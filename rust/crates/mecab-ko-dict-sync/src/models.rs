//! Data models for dictionary entries.

use serde::{Deserialize, Serialize};

/// A dictionary entry from the search results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictEntry {
    /// Target code (unique identifier)
    pub target_code: String,

    /// Word or phrase
    pub word: String,

    /// Part of speech
    pub pos: String,

    /// Definition
    pub definition: String,

    /// Reading/pronunciation (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading: Option<String>,
}

impl DictEntry {
    /// Creates a new dictionary entry.
    ///
    /// # Arguments
    ///
    /// * `target_code` - Unique identifier for the entry
    /// * `word` - The word or phrase
    /// * `pos` - Part of speech
    /// * `definition` - Definition of the word
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::models::DictEntry;
    ///
    /// let entry = DictEntry::new(
    ///     "12345",
    ///     "사랑",
    ///     "명사",
    ///     "남을 이성적으로 아끼는 마음"
    /// );
    /// assert_eq!(entry.word, "사랑");
    /// ```
    pub fn new(
        target_code: impl Into<String>,
        word: impl Into<String>,
        pos: impl Into<String>,
        definition: impl Into<String>,
    ) -> Self {
        Self {
            target_code: target_code.into(),
            word: word.into(),
            pos: pos.into(),
            definition: definition.into(),
            reading: None,
        }
    }

    /// Sets the reading/pronunciation for this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::models::DictEntry;
    ///
    /// let entry = DictEntry::new("123", "愛", "명사", "사랑")
    ///     .with_reading("애");
    /// assert_eq!(entry.reading, Some("애".to_string()));
    /// ```
    #[must_use]
    pub fn with_reading(mut self, reading: impl Into<String>) -> Self {
        self.reading = Some(reading.into());
        self
    }
}

/// Detailed information about a dictionary entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictDetail {
    /// Target code (unique identifier)
    pub target_code: String,

    /// Word or phrase
    pub word: String,

    /// Part of speech
    pub pos: String,

    /// Definition
    pub definition: String,

    /// Reading/pronunciation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading: Option<String>,

    /// Usage examples
    #[serde(default)]
    pub examples: Vec<String>,

    /// Etymology information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etymology: Option<String>,

    /// Related words
    #[serde(default)]
    pub related_words: Vec<String>,

    /// Original language (for loanwords)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_language: Option<String>,
}

impl DictDetail {
    /// Creates a new detailed dictionary entry.
    ///
    /// # Arguments
    ///
    /// * `target_code` - Unique identifier
    /// * `word` - The word or phrase
    /// * `pos` - Part of speech
    /// * `definition` - Definition
    ///
    /// # Examples
    ///
    /// ```
    /// use mecab_ko_dict_sync::models::DictDetail;
    ///
    /// let detail = DictDetail::new("123", "컴퓨터", "명사", "전자 계산기");
    /// assert_eq!(detail.word, "컴퓨터");
    /// ```
    pub fn new(
        target_code: impl Into<String>,
        word: impl Into<String>,
        pos: impl Into<String>,
        definition: impl Into<String>,
    ) -> Self {
        Self {
            target_code: target_code.into(),
            word: word.into(),
            pos: pos.into(),
            definition: definition.into(),
            reading: None,
            examples: Vec::new(),
            etymology: None,
            related_words: Vec::new(),
            original_language: None,
        }
    }

    /// Sets the reading/pronunciation.
    #[must_use]
    pub fn with_reading(mut self, reading: impl Into<String>) -> Self {
        self.reading = Some(reading.into());
        self
    }

    /// Adds an example sentence.
    #[must_use]
    pub fn add_example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }

    /// Sets the etymology.
    #[must_use]
    pub fn with_etymology(mut self, etymology: impl Into<String>) -> Self {
        self.etymology = Some(etymology.into());
        self
    }

    /// Adds a related word.
    #[must_use]
    pub fn add_related_word(mut self, word: impl Into<String>) -> Self {
        self.related_words.push(word.into());
        self
    }

    /// Sets the original language.
    #[must_use]
    pub fn with_original_language(mut self, lang: impl Into<String>) -> Self {
        self.original_language = Some(lang.into());
        self
    }
}

/// Search response from the `OpenDict` API.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponse {
    #[serde(rename = "channel")]
    pub channel: Channel,
}

/// Channel containing search results.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Channel {
    #[serde(rename = "total")]
    pub total: u32,

    #[serde(rename = "num")]
    pub num: u32,

    #[serde(rename = "item", default)]
    pub items: Vec<Item>,
}

/// Individual item in search results.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Item {
    #[serde(rename = "target_code")]
    pub target_code: String,

    #[serde(rename = "word")]
    pub word: String,

    #[serde(rename = "pos")]
    pub pos: String,

    #[serde(rename = "definition")]
    pub definition: String,

    #[serde(rename = "pronunciation")]
    pub pronunciation: Option<String>,
}

impl From<Item> for DictEntry {
    fn from(item: Item) -> Self {
        Self {
            target_code: item.target_code,
            word: item.word,
            pos: item.pos,
            definition: item.definition,
            reading: item.pronunciation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict_entry_new() {
        let entry = DictEntry::new("123", "테스트", "명사", "시험");
        assert_eq!(entry.target_code, "123");
        assert_eq!(entry.word, "테스트");
        assert_eq!(entry.pos, "명사");
        assert_eq!(entry.definition, "시험");
        assert_eq!(entry.reading, None);
    }

    #[test]
    fn test_dict_entry_with_reading() {
        let entry = DictEntry::new("123", "愛", "명사", "사랑")
            .with_reading("애");
        assert_eq!(entry.reading, Some("애".to_string()));
    }

    #[test]
    fn test_dict_detail_builder() {
        let detail = DictDetail::new("123", "컴퓨터", "명사", "전자 계산기")
            .with_reading("컴퓨터")
            .add_example("컴퓨터로 작업한다")
            .add_example("컴퓨터를 켠다")
            .with_etymology("영어 computer")
            .add_related_word("전산기")
            .with_original_language("영어");

        assert_eq!(detail.word, "컴퓨터");
        assert_eq!(detail.examples.len(), 2);
        assert_eq!(detail.etymology, Some("영어 computer".to_string()));
        assert_eq!(detail.related_words, vec!["전산기"]);
        assert_eq!(detail.original_language, Some("영어".to_string()));
    }

    #[test]
    fn test_item_to_dict_entry() {
        let item = Item {
            target_code: "123".to_string(),
            word: "사랑".to_string(),
            pos: "명사".to_string(),
            definition: "애정".to_string(),
            pronunciation: Some("사랑".to_string()),
        };

        let entry: DictEntry = item.into();
        assert_eq!(entry.target_code, "123");
        assert_eq!(entry.word, "사랑");
        assert_eq!(entry.reading, Some("사랑".to_string()));
    }
}
