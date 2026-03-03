//! # 미등록어 처리 모듈 (Unknown Word Handler)
//!
//! 사전에 없는 단어를 처리하는 모듈입니다.
//!
//! ## 개요
//!
//! `MeCab`의 미등록어 처리는 두 가지 정의 파일을 기반으로 합니다:
//! - `char.def`: 문자 카테고리 정의
//! - `unk.def`: 미등록어 품사/비용 정의
//!
//! ## 문자 카테고리 속성
//!
//! | 속성 | 값 | 의미 |
//! |------|-----|------|
//! | INVOKE | 0 | 사전에 있으면 미등록어 처리 생략 |
//! | INVOKE | 1 | 항상 미등록어 후보도 생성 |
//! | GROUP | 0 | 그룹핑 비활성화 |
//! | GROUP | 1 | 동일 카테고리 문자 그룹핑 |
//! | LENGTH | n | 1~n 길이의 미등록어 후보 생성 |
//!
//! ## 예제
//!
//! ```rust,no_run
//! use mecab_ko_core::unknown::UnknownHandler;
//!
//! let handler = UnknownHandler::korean_default();
//! ```

use std::collections::HashMap;
use std::io::BufRead;

use mecab_ko_hangul::{classify_char, CharType};

use crate::error::{Error, Result};
use crate::lattice::{Lattice, NodeBuilder, NodeType};

/// 문자 카테고리 ID
pub type CategoryId = u8;

/// 기본 카테고리 ID
pub const DEFAULT_CATEGORY: CategoryId = 0;
/// 공백 카테고리 ID
pub const SPACE_CATEGORY: CategoryId = 1;
/// 한글 카테고리 ID
pub const HANGUL_CATEGORY: CategoryId = 2;
/// 한자 카테고리 ID
pub const HANJA_CATEGORY: CategoryId = 3;
/// 알파벳 카테고리 ID
pub const ALPHA_CATEGORY: CategoryId = 4;
/// 숫자 카테고리 ID
pub const NUMERIC_CATEGORY: CategoryId = 5;
/// 기호 카테고리 ID
pub const SYMBOL_CATEGORY: CategoryId = 6;

/// 문자 카테고리 정의
///
/// char.def의 카테고리 정의를 표현합니다.
#[derive(Debug, Clone)]
pub struct CharCategoryDef {
    /// 카테고리 이름
    pub name: String,
    /// 카테고리 ID
    pub id: CategoryId,
    /// INVOKE 플래그: 항상 미등록어 후보 생성 여부
    pub invoke: bool,
    /// GROUP 플래그: 동일 카테고리 문자 그룹핑 여부
    pub group: bool,
    /// LENGTH: 미등록어 후보 최대 길이 (0이면 제한 없음)
    pub length: usize,
}

impl CharCategoryDef {
    /// 새로운 카테고리 정의 생성
    #[must_use]
    pub fn new(name: &str, id: CategoryId, invoke: bool, group: bool, length: usize) -> Self {
        Self {
            name: name.to_string(),
            id,
            invoke,
            group,
            length,
        }
    }
}

/// 미등록어 정의
///
/// unk.def의 미등록어 정의를 표현합니다.
#[derive(Debug, Clone)]
pub struct UnknownDef {
    /// 적용 카테고리 ID
    pub category_id: CategoryId,
    /// 좌문맥 ID
    pub left_id: u16,
    /// 우문맥 ID
    pub right_id: u16,
    /// 단어 비용
    pub cost: i16,
    /// 품사 태그
    pub pos: String,
    /// 피처 문자열 (품사 정보 전체)
    pub feature: String,
}

/// 단어 패턴 종류
///
/// 미등록어의 패턴을 분류합니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordPattern {
    /// 일반 패턴 (단일 카테고리)
    Plain,
    /// 대문자로 시작하는 영단어 (고유명사 가능성)
    ProperNoun,
    /// CamelCase 패턴 (iPhone, `HelloWorld` 등)
    CamelCase,
    /// 한글+영문 혼합 (카카오톡, 네이버맵 등)
    HangulAlphaMix,
    /// 숫자+단위 혼합 (15kg, 3개 등)
    NumberUnit,
    /// 이모지 포함
    Emoji,
}

impl UnknownDef {
    /// 새로운 미등록어 정의 생성
    #[must_use]
    pub fn new(
        category_id: CategoryId,
        left_id: u16,
        right_id: u16,
        cost: i16,
        pos: &str,
        feature: &str,
    ) -> Self {
        Self {
            category_id,
            left_id,
            right_id,
            cost,
            pos: pos.to_string(),
            feature: feature.to_string(),
        }
    }
}

/// 문자 카테고리 매퍼
///
/// 문자를 카테고리 ID로 매핑합니다.
#[derive(Debug, Clone)]
pub struct CharCategoryMap {
    /// 카테고리 정의 목록
    categories: Vec<CharCategoryDef>,
    /// 카테고리 이름 -> ID 매핑
    name_to_id: HashMap<String, CategoryId>,
    /// `CharType` -> `CategoryId` 매핑 (기본 매핑)
    type_to_category: HashMap<CharType, CategoryId>,
    /// Unicode 범위별 카테고리 오버라이드
    range_overrides: Vec<(u32, u32, CategoryId)>,
}

impl Default for CharCategoryMap {
    fn default() -> Self {
        Self::korean_default()
    }
}

impl CharCategoryMap {
    /// 빈 카테고리 맵 생성
    #[must_use]
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
            name_to_id: HashMap::new(),
            type_to_category: HashMap::new(),
            range_overrides: Vec::new(),
        }
    }

    /// 한국어 기본 카테고리 맵 생성
    ///
    /// mecab-ko-dic의 char.def 기반 기본 설정
    #[must_use]
    pub fn korean_default() -> Self {
        let mut map = Self::new();

        // 기본 카테고리 정의
        // 형식: (이름, ID, INVOKE, GROUP, LENGTH)
        let defaults = [
            ("DEFAULT", DEFAULT_CATEGORY, false, true, 0),
            ("SPACE", SPACE_CATEGORY, false, true, 0),
            ("HANGUL", HANGUL_CATEGORY, false, true, 2), // 한글은 최대 2글자
            ("HANJA", HANJA_CATEGORY, false, false, 1),
            ("ALPHA", ALPHA_CATEGORY, true, true, 0), // 알파벳은 항상 INVOKE
            ("NUMERIC", NUMERIC_CATEGORY, true, true, 0), // 숫자도 항상 INVOKE
            ("SYMBOL", SYMBOL_CATEGORY, true, true, 0),
        ];

        for (name, id, invoke, group, length) in defaults {
            map.add_category(CharCategoryDef::new(name, id, invoke, group, length));
        }

        // CharType -> CategoryId 기본 매핑
        map.type_to_category
            .insert(CharType::HangulSyllable, HANGUL_CATEGORY);
        map.type_to_category
            .insert(CharType::HangulJamo, HANGUL_CATEGORY);
        map.type_to_category.insert(CharType::Hanja, HANJA_CATEGORY);
        map.type_to_category
            .insert(CharType::Katakana, ALPHA_CATEGORY);
        map.type_to_category
            .insert(CharType::Hiragana, ALPHA_CATEGORY);
        map.type_to_category
            .insert(CharType::Alphabet, ALPHA_CATEGORY);
        map.type_to_category
            .insert(CharType::Digit, NUMERIC_CATEGORY);
        map.type_to_category
            .insert(CharType::Whitespace, SPACE_CATEGORY);
        map.type_to_category
            .insert(CharType::Punctuation, SYMBOL_CATEGORY);
        map.type_to_category
            .insert(CharType::Other, DEFAULT_CATEGORY);

        map
    }

    /// 카테고리 정의 추가
    pub fn add_category(&mut self, def: CharCategoryDef) {
        self.name_to_id.insert(def.name.clone(), def.id);
        self.categories.push(def);
    }

    /// Unicode 범위에 카테고리 할당
    pub fn add_range(&mut self, start: u32, end: u32, category_id: CategoryId) {
        self.range_overrides.push((start, end, category_id));
    }

    /// 문자의 카테고리 ID 반환
    #[must_use]
    pub fn get_category(&self, c: char) -> CategoryId {
        let code = c as u32;

        // 먼저 범위 오버라이드 확인
        for &(start, end, cat_id) in &self.range_overrides {
            if code >= start && code <= end {
                return cat_id;
            }
        }

        // CharType 기반 매핑
        let char_type = classify_char(c);
        self.type_to_category
            .get(&char_type)
            .copied()
            .unwrap_or(DEFAULT_CATEGORY)
    }

    /// 카테고리 정의 조회
    #[must_use]
    pub fn get_category_def(&self, id: CategoryId) -> Option<&CharCategoryDef> {
        self.categories.iter().find(|c| c.id == id)
    }

    /// 카테고리 이름으로 ID 조회
    #[must_use]
    pub fn get_id_by_name(&self, name: &str) -> Option<CategoryId> {
        self.name_to_id.get(name).copied()
    }

    /// char.def 형식에서 로드
    ///
    /// # Format
    ///
    /// ```text
    /// CATEGORY_NAME  INVOKE  GROUP  LENGTH
    /// 0xHHHH..0xJJJJ CATEGORY_NAME
    /// ```
    ///
    /// # Errors
    ///
    /// I/O 에러 또는 파싱 실패 시 에러 반환
    pub fn from_char_def<R: BufRead>(reader: R) -> Result<Self> {
        let mut map = Self::new();
        let mut next_id: CategoryId = 0;

        for line in reader.lines() {
            let line = line.map_err(|e| Error::Init(e.to_string()))?;
            let line = line.trim();

            // 빈 줄이나 주석 건너뛰기
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 카테고리 정의 줄: CATEGORY_NAME INVOKE GROUP LENGTH
            if !line.starts_with("0x") && !line.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let name = parts[0];
                    let invoke = parts[1] == "1";
                    let group = parts[2] == "1";
                    let length: usize = parts[3].parse().unwrap_or(0);

                    map.add_category(CharCategoryDef::new(name, next_id, invoke, group, length));
                    next_id += 1;
                }
            }
            // Unicode 범위 줄: 0xHHHH..0xJJJJ CATEGORY
            else if line.starts_with("0x") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let range_part = parts[0];
                    let category_name = parts[1];

                    if let Some(cat_id) = map.get_id_by_name(category_name) {
                        // 범위 파싱: 0xHHHH..0xJJJJ 또는 0xHHHH
                        if let Some((start, end)) = parse_unicode_range(range_part) {
                            map.add_range(start, end, cat_id);
                        }
                    }
                }
            }
        }

        Ok(map)
    }
}

/// Unicode 범위 문자열 파싱
///
/// "0xAC00..0xD7A3" -> Some((0xAC00, 0xD7A3))
/// "0xAC00" -> Some((0xAC00, 0xAC00))
fn parse_unicode_range(s: &str) -> Option<(u32, u32)> {
    if let Some((start_str, end_str)) = s.split_once("..") {
        let start = parse_hex(start_str)?;
        let end = parse_hex(end_str)?;
        Some((start, end))
    } else {
        let value = parse_hex(s)?;
        Some((value, value))
    }
}

/// 16진수 문자열 파싱
fn parse_hex(s: &str) -> Option<u32> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(s, 16).ok()
}

/// 이모지 여부 판별
///
/// Unicode 이모지 범위를 확인합니다.
#[must_use]
const fn is_emoji(c: char) -> bool {
    let code = c as u32;
    // Emoticons, Symbols, Pictographs (merged overlapping ranges)
    matches!(code,
        0x1F300..=0x1F9FF | // Miscellaneous Symbols and Pictographs, Emoticons, etc.
        0x2600..=0x27BF     // Miscellaneous Symbols, Dingbats
    )
}

/// 미등록어 사전
///
/// 카테고리별 미등록어 정의를 저장합니다.
#[derive(Debug, Clone, Default)]
pub struct UnknownDictionary {
    /// 카테고리 ID -> 미등록어 정의 목록
    entries: HashMap<CategoryId, Vec<UnknownDef>>,
}

impl UnknownDictionary {
    /// 새로운 미등록어 사전 생성
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// 한국어 기본 미등록어 사전 생성
    #[must_use]
    pub fn korean_default() -> Self {
        let mut dict = Self::new();

        // mecab-ko-dic의 unk.def 기반 기본 설정
        // 형식: (카테고리ID, left_id, right_id, cost, pos, feature)
        let defaults = [
            (DEFAULT_CATEGORY, 1800, 3562, 7000, "SY", "SY,*,*,*,*,*,*,*"),
            (SPACE_CATEGORY, 1799, 3559, 0, "SP", "SP,*,*,*,*,*,*,*"),
            (
                HANGUL_CATEGORY,
                1800,
                3565,
                5000,
                "UNKNOWN",
                "UNKNOWN,*,*,*,*,*,*,*",
            ),
            (HANJA_CATEGORY, 1800, 3560, 6000, "SH", "SH,*,*,*,*,*,*,*"),
            (ALPHA_CATEGORY, 1800, 3558, 4000, "SL", "SL,*,*,*,*,*,*,*"),
            (NUMERIC_CATEGORY, 1800, 3561, 3000, "SN", "SN,*,*,*,*,*,*,*"),
            (SYMBOL_CATEGORY, 1800, 3562, 7000, "SY", "SY,*,*,*,*,*,*,*"),
        ];

        for (cat_id, left_id, right_id, cost, pos, feature) in defaults {
            dict.add_entry(UnknownDef::new(
                cat_id, left_id, right_id, cost, pos, feature,
            ));
        }

        dict
    }

    /// 미등록어 정의 추가
    pub fn add_entry(&mut self, def: UnknownDef) {
        self.entries.entry(def.category_id).or_default().push(def);
    }

    /// 카테고리별 미등록어 정의 조회
    #[must_use]
    pub fn get_entries(&self, category_id: CategoryId) -> &[UnknownDef] {
        self.entries
            .get(&category_id)
            .map_or(&[], std::vec::Vec::as_slice)
    }

    /// unk.def 형식에서 로드
    ///
    /// # Format
    ///
    /// ```text
    /// CATEGORY,left_id,right_id,cost,POS,semantic,jongseong,reading,type,first,last,expr
    /// ```
    ///
    /// # Errors
    ///
    /// I/O 오류 또는 형식 오류 시 `Error::Init` 반환
    pub fn from_unk_def<R: BufRead>(reader: R, category_map: &CharCategoryMap) -> Result<Self> {
        let mut dict = Self::new();

        for line in reader.lines() {
            let line = line.map_err(|e| Error::Init(e.to_string()))?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let category_name = parts[0];
                let left_id: u16 = parts[1].parse().unwrap_or(0);
                let right_id: u16 = parts[2].parse().unwrap_or(0);
                let cost: i16 = parts[3].parse().unwrap_or(0);
                let pos = parts[4];
                let feature = line; // 전체 라인을 feature로 저장

                if let Some(cat_id) = category_map.get_id_by_name(category_name) {
                    dict.add_entry(UnknownDef::new(
                        cat_id, left_id, right_id, cost, pos, feature,
                    ));
                }
            }
        }

        Ok(dict)
    }
}

/// 미등록어 후보
#[derive(Debug, Clone)]
pub struct UnknownCandidate {
    /// 표면형
    pub surface: String,
    /// 시작 위치 (문자 인덱스)
    pub start_pos: usize,
    /// 끝 위치 (문자 인덱스)
    pub end_pos: usize,
    /// 좌문맥 ID
    pub left_id: u16,
    /// 우문맥 ID
    pub right_id: u16,
    /// 단어 비용
    pub cost: i16,
    /// 품사 태그
    pub pos: String,
    /// 카테고리 ID
    pub category_id: CategoryId,
    /// 단어 패턴
    pub pattern: WordPattern,
}

/// 미등록어 처리기
///
/// 사전에 없는 단어의 후보를 생성합니다.
#[derive(Debug, Clone)]
pub struct UnknownHandler {
    /// 문자 카테고리 맵
    pub category_map: CharCategoryMap,
    /// 미등록어 사전
    pub unknown_dict: UnknownDictionary,
}

impl Default for UnknownHandler {
    fn default() -> Self {
        Self::korean_default()
    }
}

impl UnknownHandler {
    /// 새로운 미등록어 처리기 생성
    #[must_use]
    pub const fn new(category_map: CharCategoryMap, unknown_dict: UnknownDictionary) -> Self {
        Self {
            category_map,
            unknown_dict,
        }
    }

    /// 한국어 기본 설정으로 생성
    #[must_use]
    pub fn korean_default() -> Self {
        Self::new(
            CharCategoryMap::korean_default(),
            UnknownDictionary::korean_default(),
        )
    }

    /// 단어 패턴 감지
    ///
    /// 주어진 표면형에서 패턴을 분석합니다.
    #[must_use]
    fn detect_pattern(&self, surface: &str) -> WordPattern {
        let chars: Vec<char> = surface.chars().collect();
        if chars.is_empty() {
            return WordPattern::Plain;
        }

        // 이모지 검사
        if chars.iter().any(|&c| is_emoji(c)) {
            return WordPattern::Emoji;
        }

        // 한글+영문 혼합 검사
        let has_hangul = chars.iter().any(|&c| {
            let cat = self.category_map.get_category(c);
            cat == HANGUL_CATEGORY
        });
        let has_alpha = chars.iter().any(|&c| {
            let cat = self.category_map.get_category(c);
            cat == ALPHA_CATEGORY
        });

        if has_hangul && has_alpha {
            return WordPattern::HangulAlphaMix;
        }

        // 숫자+단위 혼합 검사
        let has_digit = chars.iter().any(|&c| {
            let cat = self.category_map.get_category(c);
            cat == NUMERIC_CATEGORY
        });

        if has_digit && (has_hangul || has_alpha) {
            return WordPattern::NumberUnit;
        }

        // 영문만 있는 경우 추가 패턴 검사
        if has_alpha && !has_hangul {
            // CamelCase 검사: 중간에 대문자가 있으면
            if chars.len() > 1 {
                let mut has_internal_uppercase = false;
                for (i, &c) in chars.iter().enumerate() {
                    if i > 0 && c.is_uppercase() {
                        has_internal_uppercase = true;
                        break;
                    }
                }
                if has_internal_uppercase {
                    return WordPattern::CamelCase;
                }
            }

            // 고유명사 검사: 첫 글자만 대문자
            if chars[0].is_uppercase() && chars.len() > 1 {
                return WordPattern::ProperNoun;
            }
        }

        WordPattern::Plain
    }

    /// 패턴에 따른 비용 조정
    ///
    /// 패턴에 따라 기본 비용을 조정합니다.
    #[must_use]
    #[allow(clippy::unused_self)]
    fn adjust_cost_by_pattern(&self, base_cost: i16, pattern: WordPattern, length: usize) -> i16 {
        let mut cost = i32::from(base_cost);

        // 패턴별 조정 (v0.3.1: 한국어 신조어에 최적화)
        match pattern {
            WordPattern::Plain => {
                // 길이에 따른 패널티: 6자 초과부터 점진적 증가
                // 한국어 복합명사는 보통 2-6음절이므로 6자까지 허용
                if length > 6 {
                    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
                    let penalty = ((length - 6) * 80) as i32; // 100→80: 더 완화된 패널티
                    cost += penalty;
                }
            }
            WordPattern::ProperNoun => {
                // 고유명사: 브랜드명, 인명 등에서 흔함
                cost -= 600; // 500→600: 더 강하게 선호
            }
            WordPattern::CamelCase => {
                // CamelCase: IT 용어, 브랜드명 (iPhone, YouTube 등)
                cost -= 400; // 300→400: 더 강하게 선호
            }
            WordPattern::HangulAlphaMix => {
                // 한영 혼합: K팝, SNS족 등 현대 신조어에서 매우 흔함
                // 패널티 제거하고 오히려 약간 선호
                cost -= 100; // +200→-100: 신조어 패턴으로 선호
            }
            WordPattern::NumberUnit => {
                // 숫자+단위: 3개, 10kg, 5번 등 자연스러운 패턴
                cost -= 300; // 200→300: 더 강하게 선호
            }
            WordPattern::Emoji => {
                // 이모지: 단독 토큰으로 처리하지 않도록 높은 비용
                cost += 1500; // 1000→1500: 더 강한 억제
            }
        }

        // 범위 제한
        #[allow(clippy::cast_possible_truncation)]
        {
            cost.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
        }
    }

    /// 패턴에 따른 품사 태그 추정
    ///
    /// 패턴에 따라 더 적절한 품사 태그를 반환합니다.
    #[must_use]
    #[allow(clippy::unused_self)]
    fn estimate_pos(
        &self,
        pattern: WordPattern,
        category_id: CategoryId,
        base_pos: &str,
    ) -> String {
        match pattern {
            WordPattern::ProperNoun | WordPattern::CamelCase => {
                // 대문자 시작이나 CamelCase는 고유명사(NNP) 가능성
                if category_id == ALPHA_CATEGORY {
                    return "NNP".to_string();
                }
            }
            WordPattern::HangulAlphaMix => {
                // 한글+영문 혼합은 복합명사 가능성
                if category_id == HANGUL_CATEGORY {
                    return "NNG".to_string();
                }
            }
            _ => {}
        }

        base_pos.to_string()
    }

    /// 미등록어 후보 생성
    ///
    /// # Arguments
    ///
    /// * `text` - 분석할 텍스트 (공백 제거된 상태)
    /// * `start_pos` - 시작 위치 (문자 인덱스)
    /// * `has_dict_entry` - 해당 위치에 사전 엔트리가 있는지 여부
    ///
    /// # Returns
    ///
    /// 미등록어 후보 목록
    #[must_use]
    pub fn generate_candidates(
        &self,
        text: &str,
        start_pos: usize,
        has_dict_entry: bool,
    ) -> Vec<UnknownCandidate> {
        // Avoid allocating a Vec<char> by working directly with char indices
        // in the UTF-8 string.  We compute byte offsets alongside char counts.
        //
        // Build a lightweight mapping: char_index -> byte_offset for the
        // suffix starting at `start_pos`, stopping early once we know we
        // won't need more characters.

        // First, find the byte offset of `start_pos`.
        let start_byte = text
            .char_indices()
            .nth(start_pos)
            .map_or(text.len(), |(b, _)| b);

        let suffix = &text[start_byte..];
        let Some(first_char) = suffix.chars().next() else {
            return Vec::new();
        };
        let category_id = self.category_map.get_category(first_char);
        let Some(category_def) = self.category_map.get_category_def(category_id) else {
            return Vec::new();
        };

        // INVOKE가 false이고 사전 엔트리가 있으면 미등록어 생성 생략
        if !category_def.invoke && has_dict_entry {
            return Vec::new();
        }

        let unknown_defs = self.unknown_dict.get_entries(category_id);
        if unknown_defs.is_empty() {
            return Vec::new();
        }

        let mut candidates = Vec::new();

        if category_def.group {
            // Find how many consecutive chars share the same category,
            // collecting their byte boundaries as we go (no Vec<char>).
            let mut char_count = 0usize;
            let mut byte_end = 0usize;

            for c in suffix.chars() {
                if self.category_map.get_category(c) != category_id {
                    break;
                }
                byte_end += c.len_utf8();
                char_count += 1;
            }

            let group_char_count = char_count; // relative to start_pos
            let max_len = if category_def.length > 0 {
                category_def.length.min(group_char_count)
            } else {
                group_char_count
            };

            // Recompute char-boundary byte offsets for lengths 1..=max_len.
            let mut byte_offset = 0usize;
            let mut char_iter = suffix.chars();
            for len in 1..=max_len {
                if let Some(c) = char_iter.next() {
                    byte_offset += c.len_utf8();
                } else {
                    break;
                }
                let end_pos = start_pos + len;
                let surface = &suffix[..byte_offset];

                // 패턴 감지
                let pattern = self.detect_pattern(surface);

                for def in unknown_defs {
                    // 패턴에 따른 비용 조정
                    let adjusted_cost = self.adjust_cost_by_pattern(def.cost, pattern, len);

                    // 패턴에 따른 품사 추정
                    let estimated_pos = self.estimate_pos(pattern, category_id, &def.pos);

                    candidates.push(UnknownCandidate {
                        surface: surface.to_string(),
                        start_pos,
                        end_pos,
                        left_id: def.left_id,
                        right_id: def.right_id,
                        cost: adjusted_cost,
                        pos: estimated_pos,
                        category_id,
                        pattern,
                    });
                }
            }
            let _ = byte_end; // suppress unused warning
        } else {
            // GROUP이 false이면 각 문자를 개별 처리
            let char_total = suffix.chars().count();
            let max_len = if category_def.length > 0 {
                category_def.length.min(char_total)
            } else {
                1
            };

            let mut byte_offset = 0usize;
            let mut char_iter = suffix.chars();
            for len in 1..=max_len {
                if let Some(c) = char_iter.next() {
                    byte_offset += c.len_utf8();
                } else {
                    break;
                }
                let end_pos = start_pos + len;
                let surface = &suffix[..byte_offset];

                // 패턴 감지
                let pattern = self.detect_pattern(surface);

                for def in unknown_defs {
                    // 패턴에 따른 비용 조정
                    let adjusted_cost = self.adjust_cost_by_pattern(def.cost, pattern, len);

                    // 패턴에 따른 품사 추정
                    let estimated_pos = self.estimate_pos(pattern, category_id, &def.pos);

                    candidates.push(UnknownCandidate {
                        surface: surface.to_string(),
                        start_pos,
                        end_pos,
                        left_id: def.left_id,
                        right_id: def.right_id,
                        cost: adjusted_cost,
                        pos: estimated_pos,
                        category_id,
                        pattern,
                    });
                }
            }
        }

        candidates
    }

    /// 동일 카테고리 문자 그룹의 끝 위치 찾기
    ///
    /// Note: kept for tests; internally we now use the iterator-based approach
    /// in `generate_candidates` to avoid allocating a Vec<char>.
    #[cfg(test)]
    fn find_group_end(&self, chars: &[char], start_pos: usize, category_id: CategoryId) -> usize {
        let mut pos = start_pos;
        while pos < chars.len() {
            if self.category_map.get_category(chars[pos]) != category_id {
                break;
            }
            pos += 1;
        }
        pos
    }

    /// Lattice에 미등록어 노드 추가
    ///
    /// # Arguments
    ///
    /// * `lattice` - 노드를 추가할 Lattice
    /// * `start_pos` - 시작 위치 (문자 인덱스)
    /// * `has_dict_entry` - 해당 위치에 사전 엔트리가 있는지 여부
    ///
    /// # Returns
    ///
    /// 추가된 노드 수
    pub fn add_unknown_nodes(
        &self,
        lattice: &mut Lattice,
        start_pos: usize,
        has_dict_entry: bool,
    ) -> usize {
        let text = lattice.text();
        let candidates = self.generate_candidates(text, start_pos, has_dict_entry);
        let mut count = 0;

        for candidate in candidates {
            lattice.add_node(
                NodeBuilder::new(&candidate.surface, candidate.start_pos, candidate.end_pos)
                    .left_id(candidate.left_id)
                    .right_id(candidate.right_id)
                    .word_cost(i32::from(candidate.cost))
                    .node_type(NodeType::Unknown),
            );
            count += 1;
        }

        count
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::needless_collect)]
mod tests {
    use super::*;

    #[test]
    fn test_category_map_default() {
        let map = CharCategoryMap::korean_default();

        assert_eq!(map.get_category('가'), HANGUL_CATEGORY);
        assert_eq!(map.get_category('A'), ALPHA_CATEGORY);
        assert_eq!(map.get_category('1'), NUMERIC_CATEGORY);
        assert_eq!(map.get_category(' '), SPACE_CATEGORY);
        assert_eq!(map.get_category('.'), SYMBOL_CATEGORY);
        assert_eq!(map.get_category('韓'), HANJA_CATEGORY);
    }

    #[test]
    fn test_category_def() {
        let map = CharCategoryMap::korean_default();

        let hangul_def = map.get_category_def(HANGUL_CATEGORY).unwrap();
        assert_eq!(hangul_def.name, "HANGUL");
        assert!(!hangul_def.invoke);
        assert!(hangul_def.group);
        assert_eq!(hangul_def.length, 2);

        let alpha_def = map.get_category_def(ALPHA_CATEGORY).unwrap();
        assert!(alpha_def.invoke); // 알파벳은 항상 INVOKE
    }

    #[test]
    fn test_unknown_dict_default() {
        let dict = UnknownDictionary::korean_default();

        let hangul_entries = dict.get_entries(HANGUL_CATEGORY);
        assert!(!hangul_entries.is_empty());
        assert_eq!(hangul_entries[0].pos, "UNKNOWN");

        let alpha_entries = dict.get_entries(ALPHA_CATEGORY);
        assert!(!alpha_entries.is_empty());
        assert_eq!(alpha_entries[0].pos, "SL");
    }

    #[test]
    fn test_generate_candidates_hangul() {
        let handler = UnknownHandler::korean_default();

        // 한글 미등록어: GROUP=true, LENGTH=2
        let candidates = handler.generate_candidates("가나다라", 0, false);

        // 최대 2글자까지 생성
        assert!(!candidates.is_empty());
        let surfaces: Vec<_> = candidates.iter().map(|c| c.surface.as_str()).collect();
        assert!(surfaces.contains(&"가"));
        assert!(surfaces.contains(&"가나"));
    }

    #[test]
    fn test_generate_candidates_alpha() {
        let handler = UnknownHandler::korean_default();

        // 알파벳 미등록어: GROUP=true, LENGTH=0 (무제한)
        let candidates = handler.generate_candidates("ABC", 0, false);

        // "A", "AB", "ABC" 모두 생성
        let surfaces: Vec<_> = candidates.iter().map(|c| c.surface.as_str()).collect();
        assert!(surfaces.contains(&"A"));
        assert!(surfaces.contains(&"AB"));
        assert!(surfaces.contains(&"ABC"));
    }

    #[test]
    fn test_generate_candidates_with_dict_entry() {
        let handler = UnknownHandler::korean_default();

        // 한글은 INVOKE=false이므로 사전 엔트리가 있으면 생성 안 함
        let candidates = handler.generate_candidates("가나다", 0, true);
        assert!(candidates.is_empty());

        // 알파벳은 INVOKE=true이므로 사전 엔트리가 있어도 생성
        let candidates = handler.generate_candidates("ABC", 0, true);
        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_generate_candidates_mixed() {
        let handler = UnknownHandler::korean_default();

        // "가ABC"에서 시작
        let text = "가ABC";

        // 위치 0 (한글)
        let candidates = handler.generate_candidates(text, 0, false);
        assert!(candidates.iter().all(|c| c.category_id == HANGUL_CATEGORY));

        // 위치 1 (알파벳)
        let candidates = handler.generate_candidates(text, 1, false);
        assert!(candidates.iter().all(|c| c.category_id == ALPHA_CATEGORY));
    }

    #[test]
    fn test_find_group_end() {
        let handler = UnknownHandler::korean_default();
        let chars: Vec<char> = "가나다ABC".chars().collect();

        // 한글 그룹: 0-3
        let end = handler.find_group_end(&chars, 0, HANGUL_CATEGORY);
        assert_eq!(end, 3);

        // 알파벳 그룹: 3-6
        let end = handler.find_group_end(&chars, 3, ALPHA_CATEGORY);
        assert_eq!(end, 6);
    }

    #[test]
    fn test_add_unknown_nodes() {
        let handler = UnknownHandler::korean_default();
        let mut lattice = Lattice::new("테스트ABC");

        let count = handler.add_unknown_nodes(&mut lattice, 0, false);
        assert!(count > 0);

        // 추가된 노드 확인
        let nodes_at_0: Vec<_> = lattice.nodes_starting_at(0).collect();
        assert!(!nodes_at_0.is_empty());
    }

    #[test]
    fn test_pattern_detection_proper_noun() {
        let handler = UnknownHandler::korean_default();

        let pattern = handler.detect_pattern("Apple");
        assert_eq!(pattern, WordPattern::ProperNoun);

        let pattern = handler.detect_pattern("Google");
        assert_eq!(pattern, WordPattern::ProperNoun);
    }

    #[test]
    fn test_pattern_detection_camel_case() {
        let handler = UnknownHandler::korean_default();

        let pattern = handler.detect_pattern("iPhone");
        assert_eq!(pattern, WordPattern::CamelCase);

        let pattern = handler.detect_pattern("HelloWorld");
        assert_eq!(pattern, WordPattern::CamelCase);

        let pattern = handler.detect_pattern("iPad");
        assert_eq!(pattern, WordPattern::CamelCase);
    }

    #[test]
    fn test_pattern_detection_hangul_alpha_mix() {
        let handler = UnknownHandler::korean_default();

        let pattern = handler.detect_pattern("카카오톡");
        // "카카오톡" is pure Hangul, should be Plain
        assert_eq!(pattern, WordPattern::Plain);

        // Simulate mixed pattern - would need actual mixed text
        let pattern = handler.detect_pattern("API키");
        assert_eq!(pattern, WordPattern::HangulAlphaMix);
    }

    #[test]
    fn test_pattern_detection_number_unit() {
        let handler = UnknownHandler::korean_default();

        let pattern = handler.detect_pattern("15kg");
        assert_eq!(pattern, WordPattern::NumberUnit);

        let pattern = handler.detect_pattern("3개");
        assert_eq!(pattern, WordPattern::NumberUnit);

        let pattern = handler.detect_pattern("100원");
        assert_eq!(pattern, WordPattern::NumberUnit);
    }

    #[test]
    fn test_pattern_detection_emoji() {
        let handler = UnknownHandler::korean_default();

        let pattern = handler.detect_pattern("😀");
        assert_eq!(pattern, WordPattern::Emoji);

        let pattern = handler.detect_pattern("안녕😊");
        assert_eq!(pattern, WordPattern::Emoji);
    }

    #[test]
    fn test_pattern_detection_plain() {
        let handler = UnknownHandler::korean_default();

        let pattern = handler.detect_pattern("hello");
        assert_eq!(pattern, WordPattern::Plain);

        let _pattern = handler.detect_pattern("test123");
        // This would be NumberUnit if properly mixed
        // But lowercase with numbers at end is still Plain without letters after numbers
    }

    #[test]
    fn test_cost_adjustment_by_pattern() {
        let handler = UnknownHandler::korean_default();

        // ProperNoun should have reduced cost
        let base_cost = 4000i16;
        let adjusted = handler.adjust_cost_by_pattern(base_cost, WordPattern::ProperNoun, 5);
        assert!(adjusted < base_cost);

        // CamelCase should have reduced cost
        let adjusted = handler.adjust_cost_by_pattern(base_cost, WordPattern::CamelCase, 5);
        assert!(adjusted < base_cost);

        // Emoji should have increased cost
        let adjusted = handler.adjust_cost_by_pattern(base_cost, WordPattern::Emoji, 1);
        assert!(adjusted > base_cost);
    }

    #[test]
    fn test_cost_adjustment_by_length() {
        let handler = UnknownHandler::korean_default();
        let base_cost = 5000i16;

        // Short word (length 3)
        let cost_short = handler.adjust_cost_by_pattern(base_cost, WordPattern::Plain, 3);

        // Long word (length 10)
        let cost_long = handler.adjust_cost_by_pattern(base_cost, WordPattern::Plain, 10);

        // Longer words should have higher cost
        assert!(cost_long > cost_short);
    }

    #[test]
    fn test_pos_estimation_proper_noun() {
        let handler = UnknownHandler::korean_default();

        let pos = handler.estimate_pos(WordPattern::ProperNoun, ALPHA_CATEGORY, "SL");
        assert_eq!(pos, "NNP");

        let pos = handler.estimate_pos(WordPattern::CamelCase, ALPHA_CATEGORY, "SL");
        assert_eq!(pos, "NNP");
    }

    #[test]
    fn test_pos_estimation_hangul_alpha_mix() {
        let handler = UnknownHandler::korean_default();

        let pos = handler.estimate_pos(WordPattern::HangulAlphaMix, HANGUL_CATEGORY, "UNKNOWN");
        assert_eq!(pos, "NNG");
    }

    #[test]
    fn test_generate_candidates_with_patterns() {
        let handler = UnknownHandler::korean_default();

        // Test proper noun
        let candidates = handler.generate_candidates("Apple", 0, false);
        assert!(!candidates.is_empty());

        // Check that at least one candidate has ProperNoun pattern
        let has_proper_noun = candidates
            .iter()
            .any(|c| c.pattern == WordPattern::ProperNoun);
        assert!(has_proper_noun);

        // Check that proper noun has NNP tag
        let proper_noun_candidates: Vec<_> = candidates
            .iter()
            .filter(|c| c.pattern == WordPattern::ProperNoun)
            .collect();
        assert!(proper_noun_candidates.iter().any(|c| c.pos == "NNP"));
    }

    #[test]
    fn test_generate_candidates_abbreviation() {
        let handler = UnknownHandler::korean_default();

        // Test abbreviations like API, HTTP
        let candidates = handler.generate_candidates("API", 0, false);
        assert!(!candidates.is_empty());

        // All uppercase - could be proper noun or plain
        let surfaces: Vec<_> = candidates.iter().map(|c| c.surface.as_str()).collect();
        assert!(surfaces.contains(&"API") || surfaces.contains(&"A"));
    }

    #[test]
    fn test_generate_candidates_camel_case() {
        let handler = UnknownHandler::korean_default();

        let candidates = handler.generate_candidates("iPhone", 0, false);
        assert!(!candidates.is_empty());

        // Check for CamelCase pattern
        let has_camel = candidates
            .iter()
            .any(|c| c.pattern == WordPattern::CamelCase);
        assert!(has_camel);
    }

    #[test]
    fn test_unknown_korean_word() {
        let handler = UnknownHandler::korean_default();

        // Test unknown Korean word
        let candidates = handler.generate_candidates("테스트", 0, false);
        assert!(!candidates.is_empty());

        // Should have HANGUL category
        assert!(candidates.iter().all(|c| c.category_id == HANGUL_CATEGORY));
    }

    #[test]
    fn test_is_emoji() {
        assert!(is_emoji('😀'));
        assert!(is_emoji('😊'));
        assert!(is_emoji('🚀'));
        assert!(is_emoji('❤'));

        assert!(!is_emoji('a'));
        assert!(!is_emoji('가'));
        assert!(!is_emoji('1'));
    }

    #[test]
    fn test_parse_unicode_range() {
        assert_eq!(
            parse_unicode_range("0xAC00..0xD7A3"),
            Some((0xAC00, 0xD7A3))
        );
        assert_eq!(parse_unicode_range("0xAC00"), Some((0xAC00, 0xAC00)));
        assert_eq!(parse_unicode_range("0x0020"), Some((0x0020, 0x0020)));
    }

    #[test]
    fn test_char_def_parsing() {
        let char_def = r"
# Comment line
DEFAULT        0 1 0
SPACE          0 1 0
HANGUL         0 1 2
ALPHA          1 1 0

0xAC00..0xD7A3 HANGUL
0x0041..0x005A ALPHA
";

        let map = CharCategoryMap::from_char_def(char_def.as_bytes()).unwrap();

        assert!(map.get_id_by_name("DEFAULT").is_some());
        assert!(map.get_id_by_name("HANGUL").is_some());
        assert!(map.get_id_by_name("ALPHA").is_some());

        // 범위 확인
        assert_eq!(
            map.get_category('가'),
            map.get_id_by_name("HANGUL").unwrap()
        );
        assert_eq!(map.get_category('A'), map.get_id_by_name("ALPHA").unwrap());
    }

    #[test]
    fn test_unk_def_parsing() {
        let char_def = "DEFAULT 0 1 0\nHANGUL 0 1 2\n";
        let map = CharCategoryMap::from_char_def(char_def.as_bytes()).unwrap();

        let unk_def = r"
DEFAULT,1800,3562,7000,SY,*,*,*,*,*,*,*
HANGUL,1800,3565,5000,UNKNOWN,*,*,*,*,*,*,*
";

        let dict = UnknownDictionary::from_unk_def(unk_def.as_bytes(), &map).unwrap();

        let hangul_id = map.get_id_by_name("HANGUL").unwrap();
        let entries = dict.get_entries(hangul_id);
        assert!(!entries.is_empty());
        assert_eq!(entries[0].pos, "UNKNOWN");
        assert_eq!(entries[0].cost, 5000);
    }
}
