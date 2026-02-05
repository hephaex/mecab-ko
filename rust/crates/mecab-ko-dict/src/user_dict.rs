//! # 사용자 정의 사전 모듈
//!
//! 사용자가 커스텀 단어를 추가할 수 있는 기능을 제공합니다.
//!
//! ## 포맷
//!
//! CSV 형식의 사용자 사전을 지원합니다:
//!
//! ```csv
//! # 주석 라인 (# 으로 시작)
//! 표면형,품사,비용,읽기
//! 형태소분석,NNG,-1000,형태소분석
//! 딥러닝,NNG,-500,딥러닝
//! ```
//!
//! ## 예제
//!
//! ```rust,ignore
//! use mecab_ko_dict::user_dict::UserDictionary;
//!
//! let mut user_dict = UserDictionary::new();
//! user_dict.add_entry("딥러닝", "NNG", -500, None)?;
//! user_dict.load_from_csv("user.csv")?;
//! ```

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::{DictError, Result};
use crate::trie::{Trie, TrieBuilder};
use crate::Entry;

/// 사용자 사전 엔트리
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserEntry {
    /// 표면형
    pub surface: String,
    /// 좌문맥 ID
    pub left_id: u16,
    /// 우문맥 ID
    pub right_id: u16,
    /// 비용 (낮을수록 우선)
    pub cost: i16,
    /// 품사 태그
    pub pos: String,
    /// 읽기 (발음)
    pub reading: Option<String>,
    /// 원형 (기본형)
    pub lemma: Option<String>,
    /// 전체 품사 정보 (feature string, 캐시)
    pub feature: String,
}

impl UserEntry {
    /// 새 사용자 엔트리 생성
    pub fn new(
        surface: impl Into<String>,
        pos: impl Into<String>,
        cost: i16,
        reading: Option<String>,
    ) -> Self {
        let surface = surface.into();
        let pos = pos.into();
        let feature = format!("{},*,*,{},*,*,*,*", pos, reading.as_deref().unwrap_or("*"));
        Self {
            surface,
            left_id: 0, // 기본 컨텍스트 ID
            right_id: 0,
            cost,
            pos,
            reading,
            lemma: None,
            feature,
        }
    }

    /// 컨텍스트 ID 설정
    #[must_use]
    pub const fn with_context_ids(mut self, left_id: u16, right_id: u16) -> Self {
        self.left_id = left_id;
        self.right_id = right_id;
        self
    }

    /// 원형 설정
    #[must_use]
    pub fn with_lemma(mut self, lemma: impl Into<String>) -> Self {
        self.lemma = Some(lemma.into());
        self
    }

    /// Entry로 변환
    #[must_use]
    pub fn to_entry(&self) -> Entry {
        let feature = format!(
            "{},*,*,*,*,*,{},*",
            self.pos,
            self.reading.as_deref().unwrap_or("*")
        );

        Entry {
            surface: self.surface.clone(),
            left_id: self.left_id,
            right_id: self.right_id,
            cost: self.cost,
            feature,
        }
    }
}

/// 사용자 정의 사전
///
/// 사용자가 커스텀 단어를 추가하여 형태소 분석을 개선할 수 있습니다.
#[derive(Clone)]
pub struct UserDictionary {
    /// 엔트리 목록
    entries: Vec<UserEntry>,
    /// 표면형 -> 엔트리 인덱스 맵
    surface_map: HashMap<String, Vec<usize>>,
    /// 빌드된 Trie (캐시)
    trie_cache: Option<Vec<u8>>,
    /// 기본 비용
    default_cost: i16,
}

impl Default for UserDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl UserDictionary {
    /// 새 사용자 사전 생성
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            surface_map: HashMap::new(),
            trie_cache: None,
            default_cost: -1000, // 낮은 비용으로 우선 선택
        }
    }

    /// 기본 비용 설정
    #[must_use]
    pub const fn with_default_cost(mut self, cost: i16) -> Self {
        self.default_cost = cost;
        self
    }

    /// 엔트리 추가
    ///
    /// # Arguments
    ///
    /// * `surface` - 표면형
    /// * `pos` - 품사 태그 (예: "NNG", "NNP", "VV")
    /// * `cost` - 비용 (낮을수록 우선, None이면 기본값 사용)
    /// * `reading` - 읽기 (발음, 선택)
    pub fn add_entry(
        &mut self,
        surface: impl Into<String>,
        pos: impl Into<String>,
        cost: Option<i16>,
        reading: Option<String>,
    ) -> &mut Self {
        let surface = surface.into();
        let cost = cost.unwrap_or(self.default_cost);
        let entry = UserEntry::new(surface.clone(), pos, cost, reading);

        let idx = self.entries.len();
        self.entries.push(entry);

        self.surface_map.entry(surface).or_default().push(idx);

        // Trie 캐시 무효화
        self.trie_cache = None;

        self
    }

    /// 컨텍스트 ID와 함께 엔트리 추가
    pub fn add_entry_with_ids(
        &mut self,
        surface: impl Into<String>,
        pos: impl Into<String>,
        cost: i16,
        left_id: u16,
        right_id: u16,
        reading: Option<String>,
    ) -> &mut Self {
        let surface = surface.into();
        let entry =
            UserEntry::new(surface.clone(), pos, cost, reading).with_context_ids(left_id, right_id);

        let idx = self.entries.len();
        self.entries.push(entry);

        self.surface_map.entry(surface).or_default().push(idx);

        self.trie_cache = None;

        self
    }

    /// CSV 파일에서 사전 로드
    ///
    /// # 포맷
    ///
    /// ```csv
    /// # 주석 라인
    /// 표면형,품사,비용,읽기
    /// ```
    ///
    /// - 표면형: 필수
    /// - 품사: 필수 (예: NNG, NNP, VV)
    /// - 비용: 선택 (기본값: -1000)
    /// - 읽기: 선택
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 파싱할 수 없는 경우 에러를 반환합니다.
    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Self> {
        let file = std::fs::File::open(path.as_ref()).map_err(DictError::Io)?;
        let reader = BufReader::new(file);

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(DictError::Io)?;
            let line = line.trim();

            // 빈 줄이나 주석 건너뛰기
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            self.parse_csv_line(line, line_num + 1)?;
        }

        Ok(self)
    }

    /// CSV 문자열에서 사전 로드
    ///
    /// # Errors
    ///
    /// 파싱 오류가 발생한 경우 에러를 반환합니다.
    pub fn load_from_str(&mut self, content: &str) -> Result<&mut Self> {
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            self.parse_csv_line(line, line_num + 1)?;
        }

        Ok(self)
    }

    /// CSV 라인 파싱
    fn parse_csv_line(&mut self, line: &str, line_num: usize) -> Result<()> {
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() < 2 {
            return Err(DictError::Format(format!(
                "Invalid user dictionary format at line {line_num}: expected at least 2 fields"
            )));
        }

        let surface = parts[0].trim();
        let pos = parts[1].trim();

        if surface.is_empty() || pos.is_empty() {
            return Err(DictError::Format(format!(
                "Empty surface or POS at line {line_num}"
            )));
        }

        let cost = if parts.len() > 2 && !parts[2].trim().is_empty() {
            parts[2].trim().parse::<i16>().map_err(|_| {
                DictError::Format(format!("Invalid cost at line {}: {}", line_num, parts[2]))
            })?
        } else {
            self.default_cost
        };

        let reading = if parts.len() > 3 && !parts[3].trim().is_empty() {
            Some(parts[3].trim().to_string())
        } else {
            None
        };

        self.add_entry(surface, pos, Some(cost), reading);

        Ok(())
    }

    /// 표면형으로 엔트리 검색
    #[must_use]
    pub fn lookup(&self, surface: &str) -> Vec<&UserEntry> {
        self.surface_map
            .get(surface)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.entries.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 공통 접두사 검색
    ///
    /// 주어진 텍스트의 접두사와 일치하는 모든 엔트리를 찾습니다.
    ///
    /// # Arguments
    ///
    /// * `text` - 검색할 텍스트
    ///
    /// # Returns
    ///
    /// 일치하는 엔트리의 벡터
    #[must_use]
    pub fn common_prefix_search(&self, text: &str) -> Vec<&UserEntry> {
        let mut results = Vec::new();

        // 각 엔트리의 표면형을 텍스트의 접두사로 확인
        for entry in &self.entries {
            if text.starts_with(&entry.surface) {
                results.push(entry);
            }
        }

        results
    }

    /// 모든 엔트리 반환
    #[must_use]
    pub fn entries(&self) -> &[UserEntry] {
        &self.entries
    }

    /// 엔트리 수 반환
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 사전이 비어있는지 확인
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Trie 빌드
    ///
    /// 사전 검색을 위한 Double-Array Trie를 빌드합니다.
    ///
    /// # Errors
    ///
    /// 사전이 비어있거나 Trie 빌드에 실패한 경우 에러를 반환합니다.
    pub fn build_trie(&mut self) -> Result<&[u8]> {
        if let Some(ref cache) = self.trie_cache {
            return Ok(cache);
        }

        if self.entries.is_empty() {
            return Err(DictError::Format(
                "Cannot build Trie from empty user dictionary".to_string(),
            ));
        }

        // 표면형과 첫 번째 인덱스로 엔트리 생성
        #[allow(clippy::cast_possible_truncation)]
        let mut trie_entries: Vec<(&str, u32)> = self
            .surface_map
            .iter()
            .filter_map(|(surface, indices)| {
                indices.first().map(|&idx| (surface.as_str(), idx as u32))
            })
            .collect();

        // 바이트 순으로 정렬
        trie_entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

        let bytes = TrieBuilder::build(&trie_entries)?;
        self.trie_cache = Some(bytes);

        // SAFETY: We just inserted the value above
        Ok(self.trie_cache.as_ref().unwrap_or_else(|| unreachable!()))
    }

    /// 빌드된 Trie 가져오기
    #[must_use]
    pub fn get_trie(&self) -> Option<Trie<'_>> {
        self.trie_cache.as_ref().map(|bytes| Trie::new(bytes))
    }

    /// Entry 목록으로 변환
    #[must_use]
    pub fn to_entries(&self) -> Vec<Entry> {
        self.entries.iter().map(UserEntry::to_entry).collect()
    }

    /// 사전 초기화 (모든 엔트리 제거)
    pub fn clear(&mut self) {
        self.entries.clear();
        self.surface_map.clear();
        self.trie_cache = None;
    }

    /// 파일로 저장 (CSV 형식)
    ///
    /// # Errors
    ///
    /// 파일을 쓸 수 없는 경우 에러를 반환합니다.
    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        use std::io::Write;

        let mut file = std::fs::File::create(path.as_ref()).map_err(DictError::Io)?;

        writeln!(file, "# 사용자 정의 사전").map_err(DictError::Io)?;
        writeln!(file, "# 표면형,품사,비용,읽기").map_err(DictError::Io)?;

        for entry in &self.entries {
            let reading = entry.reading.as_deref().unwrap_or("");
            writeln!(
                file,
                "{},{},{},{}",
                entry.surface, entry.pos, entry.cost, reading
            )
            .map_err(DictError::Io)?;
        }

        Ok(())
    }
}

/// 사용자 사전 빌더 (빌더 패턴)
pub struct UserDictionaryBuilder {
    dict: UserDictionary,
}

impl Default for UserDictionaryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl UserDictionaryBuilder {
    /// 새 빌더 생성
    #[must_use]
    pub fn new() -> Self {
        Self {
            dict: UserDictionary::new(),
        }
    }

    /// 기본 비용 설정
    #[must_use]
    pub fn default_cost(mut self, cost: i16) -> Self {
        self.dict = self.dict.with_default_cost(cost);
        self
    }

    /// 엔트리 추가
    #[must_use]
    pub fn add(mut self, surface: &str, pos: &str) -> Self {
        self.dict.add_entry(surface, pos, None, None);
        self
    }

    /// 비용과 함께 엔트리 추가
    #[must_use]
    pub fn add_with_cost(mut self, surface: &str, pos: &str, cost: i16) -> Self {
        self.dict.add_entry(surface, pos, Some(cost), None);
        self
    }

    /// 모든 정보와 함께 엔트리 추가
    #[must_use]
    pub fn add_full(mut self, surface: &str, pos: &str, cost: i16, reading: Option<&str>) -> Self {
        self.dict
            .add_entry(surface, pos, Some(cost), reading.map(String::from));
        self
    }

    /// CSV 파일에서 로드
    ///
    /// # Errors
    ///
    /// 파일을 읽거나 파싱할 수 없는 경우 에러를 반환합니다.
    pub fn load_csv<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        self.dict.load_from_csv(path)?;
        Ok(self)
    }

    /// CSV 문자열에서 로드
    ///
    /// # Errors
    ///
    /// 파싱 오류가 발생한 경우 에러를 반환합니다.
    pub fn load_str(mut self, content: &str) -> Result<Self> {
        self.dict.load_from_str(content)?;
        Ok(self)
    }

    /// 사전 빌드
    #[must_use]
    pub fn build(self) -> UserDictionary {
        self.dict
    }

    /// Trie와 함께 빌드
    ///
    /// # Errors
    ///
    /// Trie 빌드에 실패한 경우 에러를 반환합니다.
    pub fn build_with_trie(mut self) -> Result<UserDictionary> {
        if !self.dict.is_empty() {
            self.dict.build_trie()?;
        }
        Ok(self.dict)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_add_entry() {
        let mut dict = UserDictionary::new();
        dict.add_entry("딥러닝", "NNG", Some(-500), None);
        dict.add_entry("머신러닝", "NNG", None, Some("머신러닝".to_string()));

        assert_eq!(dict.len(), 2);
    }

    #[test]
    fn test_lookup() {
        let mut dict = UserDictionary::new();
        dict.add_entry("딥러닝", "NNG", Some(-500), None);
        dict.add_entry("딥러닝", "NNP", Some(-300), None); // 같은 표면형, 다른 품사

        let entries = dict.lookup("딥러닝");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].pos, "NNG");
        assert_eq!(entries[1].pos, "NNP");
    }

    #[test]
    fn test_load_from_str() {
        let csv = r"
# 사용자 사전
형태소분석,NNG,-1000,형태소분석
딥러닝,NNG,-500,
자연어처리,NNG,,자연어처리
";
        let mut dict = UserDictionary::new();
        dict.load_from_str(csv).expect("should load");

        assert_eq!(dict.len(), 3);

        let entries = dict.lookup("형태소분석");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].cost, -1000);
        assert_eq!(entries[0].reading.as_deref(), Some("형태소분석"));

        let entries = dict.lookup("딥러닝");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].cost, -500);

        let entries = dict.lookup("자연어처리");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].cost, -1000); // 기본 비용
    }

    #[test]
    fn test_build_trie() {
        let mut dict = UserDictionary::new();
        dict.add_entry("가", "NNG", Some(0), None);
        dict.add_entry("가다", "VV", Some(0), None);
        dict.add_entry("가방", "NNG", Some(0), None);

        let bytes = dict.build_trie().expect("should build");
        assert!(!bytes.is_empty());

        let trie = dict.get_trie().expect("should have trie");
        assert!(trie.exact_match("가").is_some());
        assert!(trie.exact_match("가다").is_some());
        assert!(trie.exact_match("가방").is_some());
        assert!(trie.exact_match("없음").is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let dict = UserDictionaryBuilder::new()
            .default_cost(-500)
            .add("딥러닝", "NNG")
            .add_with_cost("머신러닝", "NNG", -300)
            .add_full("자연어처리", "NNG", -400, Some("자연어처리"))
            .build();

        assert_eq!(dict.len(), 3);

        let entries = dict.lookup("딥러닝");
        assert_eq!(entries[0].cost, -500); // 기본 비용

        let entries = dict.lookup("머신러닝");
        assert_eq!(entries[0].cost, -300);
    }

    #[test]
    fn test_to_entry() {
        let user_entry = UserEntry::new("테스트", "NNG", -100, Some("테스트".to_string()));
        let entry = user_entry.to_entry();

        assert_eq!(entry.surface, "테스트");
        assert_eq!(entry.cost, -100);
        assert!(entry.feature.contains("NNG"));
        assert!(entry.feature.contains("테스트"));
    }

    #[test]
    fn test_korean_entries() {
        let mut dict = UserDictionary::new();
        dict.add_entry("챗GPT", "NNP", Some(-1000), Some("챗지피티".to_string()));
        dict.add_entry("클로드", "NNP", Some(-1000), None);
        dict.add_entry("라마", "NNP", Some(-1000), None);
        dict.add_entry("메타", "NNP", Some(-800), None);
        dict.add_entry("앤트로픽", "NNP", Some(-1000), None);

        assert_eq!(dict.len(), 5);

        let entries = dict.lookup("챗GPT");
        assert_eq!(entries[0].reading.as_deref(), Some("챗지피티"));
    }

    #[test]
    fn test_clear() {
        let mut dict = UserDictionary::new();
        dict.add_entry("테스트", "NNG", None, None);
        assert_eq!(dict.len(), 1);

        dict.clear();
        assert!(dict.is_empty());
    }

    #[test]
    fn test_invalid_csv() {
        let csv = "표면형만";
        let mut dict = UserDictionary::new();
        let result = dict.load_from_str(csv);
        assert!(result.is_err());
    }

    #[test]
    fn test_common_prefix_search() {
        let mut dict = UserDictionary::new();
        dict.add_entry("형태", "NNG", Some(0), None);
        dict.add_entry("형태소", "NNG", Some(0), None);
        dict.add_entry("형태소분석", "NNG", Some(0), None);

        dict.build_trie().expect("should build");

        let trie = dict.get_trie().expect("should have trie");

        // "형태소분석기" 에서 공통 접두사 검색
        assert_eq!(trie.common_prefix_search("형태소분석기").count(), 3); // 형태, 형태소, 형태소분석
    }

    #[test]
    fn test_with_context_ids() {
        let mut dict = UserDictionary::new();
        dict.add_entry_with_ids("테스트", "NNG", -100, 1234, 5678, None);

        let entries = dict.lookup("테스트");
        assert_eq!(entries[0].left_id, 1234);
        assert_eq!(entries[0].right_id, 5678);
    }
}
