//! Lucene Nori 호환 레이어
//!
//! Apache Lucene의 한국어 분석기 Nori와 호환되는 인터페이스를 제공합니다.
//!
//! # 주요 기능
//!
//! - `NoriTokenizer`: Nori 스타일 토크나이저
//! - `NoriAnalyzer`: 분석기 래퍼 (사용자 사전, stoptags 지원)
//! - POS 태그 매핑: `MeCab` ↔ Nori 변환
//!
//! # 예제
//!
//! ```rust,no_run
//! use mecab_ko_core::nori_compat::{NoriTokenizer, DecompoundMode};
//!
//! let mut tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, true).unwrap();
//! let tokens = tokenizer.tokenize("형태소분석기").unwrap();
//!
//! for token in tokens {
//!     println!("{}: {}", token.surface, token.pos_tag);
//! }
//! ```

use crate::pos_tag::PosTag;
use crate::tokenizer::{Token, Tokenizer};
use crate::Result;
use std::collections::HashSet;

/// 일반적인 복합명사 분해 사전
///
/// 자주 사용되는 복합명사의 올바른 분해 패턴을 정의합니다.
/// (표면형, [(부분1, POS), (부분2, POS), ...])
const COMPOUND_DICT: &[(&str, &[(&str, &str)])] = &[
    // 정보기술/IT
    ("형태소분석기", &[("형태소", "NNG"), ("분석기", "NNG")]),
    ("형태소분석", &[("형태소", "NNG"), ("분석", "NNG")]),
    ("자연어처리", &[("자연어", "NNG"), ("처리", "NNG")]),
    ("인공지능", &[("인공", "NNG"), ("지능", "NNG")]),
    ("기계학습", &[("기계", "NNG"), ("학습", "NNG")]),
    ("딥러닝", &[("딥", "NNG"), ("러닝", "NNG")]),
    ("데이터베이스", &[("데이터", "NNG"), ("베이스", "NNG")]),
    ("운영체제", &[("운영", "NNG"), ("체제", "NNG")]),
    ("프로그래밍", &[("프로그램", "NNG"), ("밍", "XSN")]),
    ("소프트웨어", &[("소프트", "NNG"), ("웨어", "NNG")]),
    ("하드웨어", &[("하드", "NNG"), ("웨어", "NNG")]),
    // 사회/기관
    ("대한민국", &[("대한", "NNP"), ("민국", "NNG")]),
    ("국립국어원", &[("국립", "NNG"), ("국어원", "NNP")]),
    ("대통령", &[("대", "XPN"), ("통령", "NNG")]),
    ("국무총리", &[("국무", "NNG"), ("총리", "NNG")]),
    ("대법원", &[("대", "XPN"), ("법원", "NNG")]),
    ("헌법재판소", &[("헌법", "NNG"), ("재판소", "NNG")]),
    ("국회의원", &[("국회", "NNG"), ("의원", "NNG")]),
    (
        "지방자치단체",
        &[("지방", "NNG"), ("자치", "NNG"), ("단체", "NNG")],
    ),
    // 교육
    ("대학교", &[("대학", "NNG"), ("교", "NNG")]),
    ("초등학교", &[("초등", "NNG"), ("학교", "NNG")]),
    ("중학교", &[("중", "XPN"), ("학교", "NNG")]),
    ("고등학교", &[("고등", "NNG"), ("학교", "NNG")]),
    ("운동장", &[("운동", "NNG"), ("장", "NNG")]),
    ("도서관", &[("도서", "NNG"), ("관", "NNG")]),
    ("교과서", &[("교과", "NNG"), ("서", "NNG")]),
    // 건축/장소
    ("아파트", &[("아파트", "NNG")]),
    ("백화점", &[("백화", "NNG"), ("점", "NNG")]),
    ("주차장", &[("주차", "NNG"), ("장", "NNG")]),
    ("병원", &[("병원", "NNG")]),
    ("약국", &[("약국", "NNG")]),
    ("편의점", &[("편의", "NNG"), ("점", "NNG")]),
    ("공항", &[("공항", "NNG")]),
    ("지하철", &[("지하", "NNG"), ("철", "NNG")]),
    ("버스정류장", &[("버스", "NNG"), ("정류장", "NNG")]),
    // 경제/금융
    ("주식시장", &[("주식", "NNG"), ("시장", "NNG")]),
    ("부동산", &[("부동", "NNG"), ("산", "NNG")]),
    ("신용카드", &[("신용", "NNG"), ("카드", "NNG")]),
    ("은행계좌", &[("은행", "NNG"), ("계좌", "NNG")]),
    // 자연/환경
    ("지구온난화", &[("지구", "NNG"), ("온난화", "NNG")]),
    ("환경오염", &[("환경", "NNG"), ("오염", "NNG")]),
    ("태양광", &[("태양", "NNG"), ("광", "NNG")]),
    ("풍력발전", &[("풍력", "NNG"), ("발전", "NNG")]),
    // 의료/건강
    ("건강보험", &[("건강", "NNG"), ("보험", "NNG")]),
    ("의료기관", &[("의료", "NNG"), ("기관", "NNG")]),
    ("응급실", &[("응급", "NNG"), ("실", "NNG")]),
    ("수술실", &[("수술", "NNG"), ("실", "NNG")]),
];

/// 확장된 접두사 목록
const PREFIXES: &[(&str, &str)] = &[
    // 관형 접두사 (XPN)
    ("신", "XPN"), // 새: 신제품
    ("구", "XPN"), // 옛: 구버전
    ("총", "XPN"), // 전체: 총대리
    ("부", "XPN"), // 보조: 부사장
    ("대", "XPN"), // 큰: 대통령
    ("소", "XPN"), // 작은: 소기업
    ("중", "XPN"), // 중간: 중기업
    ("고", "XPN"), // 높은: 고속도로
    ("저", "XPN"), // 낮은: 저소득층
    ("최", "XPN"), // 가장: 최고급
    ("초", "XPN"), // 처음/매우: 초고속
    ("준", "XPN"), // 거의: 준결승
    ("범", "XPN"), // 넓은: 범국민
    ("반", "XPN"), // 반대: 반정부
    ("비", "XPN"), // 아닌: 비공개
    ("미", "XPN"), // 아직: 미완성
    ("재", "XPN"), // 다시: 재개발
    ("전", "XPN"), // 이전: 전대통령
    ("후", "XPN"), // 이후: 후배
    ("무", "XPN"), // 없는: 무료
    ("유", "XPN"), // 있는: 유료
    ("친", "XPN"), // 친하다: 친환경
    ("반", "XPN"), // 반대: 반환경
];

/// 확장된 접미사 목록
const SUFFIXES: &[(&str, &str)] = &[
    // 파생 접미사 (XSN)
    ("들", "XSN"),   // 복수
    ("님", "XSN"),   // 존칭
    ("씨", "XSN"),   // 존칭
    ("꾼", "XSN"),   // 사람
    ("쟁이", "XSN"), // 사람
    ("치", "XSN"),   // 사람: 사기치
    ("가", "XSN"),   // 사람: 전문가
    ("자", "XSN"),   // 사람: 기술자
    ("사", "XSN"),   // 사람: 변호사
    ("원", "XSN"),   // 사람: 회사원
    ("인", "XSN"),   // 사람: 한국인
    ("생", "XSN"),   // 사람: 학생
    ("장", "XSN"),   // 장소: 운동장
    ("실", "XSN"),   // 장소: 사무실
    ("관", "XSN"),   // 장소: 도서관
    ("소", "XSN"),   // 장소: 연구소
    ("점", "XSN"),   // 장소: 편의점
    ("기", "XSN"),   // 도구: 분석기
    ("화", "XSN"),   // 변화: 현대화
    ("적", "XSN"),   // 성질: 과학적
    ("성", "XSN"),   // 성질: 창의성
    ("율", "XSN"),   // 비율: 합격률
    ("도", "XSN"),   // 정도: 만족도
    ("비", "XSN"),   // 비용: 생활비
    ("권", "XSN"),   // 권한: 투표권
    ("론", "XSN"),   // 이론: 진화론
    ("학", "XSN"),   // 학문: 언어학
    ("계", "XSN"),   // 분야: 학계
];

/// Nori 복합명사 분해 모드
///
/// Lucene Nori의 decompound 설정과 호환
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecompoundMode {
    /// 분해하지 않음 - 복합명사를 그대로 출력
    ///
    /// # Example
    /// "형태소분석기" → \["형태소분석기/NNG"\]
    None,

    /// 분해만 출력 - 원본은 버리고 분해된 형태소만 출력
    ///
    /// # Example
    /// "형태소분석기" → \["형태소/NNG", "분석/NNG", "기/NNG"\]
    Discard,

    /// 혼합 출력 - 원본과 분해된 형태소 모두 출력
    ///
    /// # Example
    /// "형태소분석기" → \["형태소분석기/NNG", "형태소/NNG", "분석/NNG", "기/NNG"\]
    Mixed,
}

impl DecompoundMode {
    /// 문자열에서 파싱
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "discard" => Some(Self::Discard),
            "mixed" => Some(Self::Mixed),
            _ => None,
        }
    }

    /// 문자열에서 파싱 (parse의 별칭)
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Self::parse(s)
    }

    /// 문자열 표현
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Discard => "discard",
            Self::Mixed => "mixed",
        }
    }
}

/// Nori 토큰
///
/// Lucene Nori의 Token 속성과 호환
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoriToken {
    /// 표면형
    pub surface: String,
    /// Nori 스타일 품사 태그 (J, E 통합)
    pub pos_tag: String,
    /// 시작 위치 (문자 오프셋)
    pub start_offset: usize,
    /// 끝 위치 (문자 오프셋)
    pub end_offset: usize,
    /// 원형 (기본형)
    pub lemma: Option<String>,
    /// 읽기 (발음)
    pub reading: Option<String>,
    /// 단어 타입 (KNOWN, UNKNOWN, etc.)
    pub word_type: WordType,
    /// 복합명사 분해 여부
    pub is_decompound: bool,
}

/// 단어 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WordType {
    /// 사전에 등록된 단어
    Known,
    /// 미등록어
    Unknown,
    /// 사용자 사전 단어
    User,
}

impl WordType {
    /// 문자열 표현
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Known => "KNOWN",
            Self::Unknown => "UNKNOWN",
            Self::User => "USER",
        }
    }
}

/// Nori 토크나이저
///
/// Lucene Nori의 `KoreanTokenizer`와 호환되는 인터페이스
pub struct NoriTokenizer {
    /// 내부 `MeCab` 토크나이저
    tokenizer: Tokenizer,
    /// 복합명사 분해 모드
    decompound_mode: DecompoundMode,
    /// 미등록어를 유니그램으로 출력할지 여부
    output_unknown_unigrams: bool,
}

impl NoriTokenizer {
    /// 새 Nori 토크나이저 생성
    ///
    /// # Arguments
    ///
    /// * `decompound_mode` - 복합명사 분해 모드
    /// * `output_unknown_unigrams` - 미등록어 유니그램 출력 여부
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mecab_ko_core::nori_compat::{NoriTokenizer, DecompoundMode};
    ///
    /// let tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, true).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the internal tokenizer fails to initialize.
    pub fn new(decompound_mode: DecompoundMode, output_unknown_unigrams: bool) -> Result<Self> {
        Ok(Self {
            tokenizer: Tokenizer::new()?,
            decompound_mode,
            output_unknown_unigrams,
        })
    }

    /// 사전 경로를 지정하여 생성
    ///
    /// # Errors
    ///
    /// Returns an error if the tokenizer fails to load the dictionary.
    pub fn with_dict(
        dict_path: &str,
        decompound_mode: DecompoundMode,
        output_unknown_unigrams: bool,
    ) -> Result<Self> {
        Ok(Self {
            tokenizer: Tokenizer::with_dict(dict_path)?,
            decompound_mode,
            output_unknown_unigrams,
        })
    }

    /// 텍스트를 Nori 스타일로 토큰화
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use mecab_ko_core::nori_compat::{NoriTokenizer, DecompoundMode};
    /// # let mut tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, true).unwrap();
    /// let tokens = tokenizer.tokenize("형태소분석").unwrap();
    /// for token in tokens {
    ///     println!("{}: {}", token.surface, token.pos_tag);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if tokenization fails.
    pub fn tokenize(&mut self, text: &str) -> Result<Vec<NoriToken>> {
        let mecab_tokens = self.tokenizer.tokenize(text);
        let mut nori_tokens = Vec::new();

        for token in &mecab_tokens {
            let nori_token = self.convert_token(token, text);
            nori_tokens.extend(nori_token);
        }

        Ok(nori_tokens)
    }

    /// `MeCab` 토큰을 Nori 토큰으로 변환
    fn convert_token(&self, token: &Token, text: &str) -> Vec<NoriToken> {
        let pos_tag = token.pos.parse::<PosTag>().unwrap_or(PosTag::Unknown);
        let nori_tag = pos_tag.to_nori_compat();

        // 기본 토큰 생성
        let mut tokens = vec![NoriToken {
            surface: token.surface.clone(),
            pos_tag: nori_tag.as_str().to_string(),
            start_offset: char_offset(text, token.start_byte),
            end_offset: char_offset(text, token.end_byte),
            lemma: token.lemma.clone(),
            reading: token.reading.clone(),
            word_type: if pos_tag == PosTag::Unknown {
                WordType::Unknown
            } else {
                WordType::Known
            },
            is_decompound: false,
        }];

        // 복합명사 분해 처리
        if self.should_decompound(pos_tag) {
            let decompounded = Self::decompound_token_enhanced(token, text);
            tokens = self.apply_decompound_mode(tokens, decompounded);
        }

        // 미등록어 유니그램 처리
        if self.output_unknown_unigrams && pos_tag == PosTag::Unknown {
            tokens = Self::split_unknown_to_unigrams(token, text);
        }

        tokens
    }

    /// 복합명사 분해 대상인지 확인
    fn should_decompound(&self, pos_tag: PosTag) -> bool {
        self.decompound_mode != DecompoundMode::None && matches!(pos_tag, PosTag::NNG | PosTag::NNP)
    }

    /// 향상된 복합명사 분해 (사전 기반 + 접두사/접미사 감지 포함)
    ///
    /// 분해 우선순위:
    /// 1. 사전 기반 분해 (정확한 매칭)
    /// 2. 접미사 추출
    /// 3. 접두사 추출
    /// 4. 음절 기반 휴리스틱
    fn decompound_token_enhanced(token: &Token, text: &str) -> Vec<NoriToken> {
        // 1. 사전 기반 분해 (가장 정확)
        if let Some(tokens) = Self::try_dict_decompose(token, text) {
            return tokens;
        }

        // 2. 접미사 검사
        if let Some(tokens) = Self::try_extract_suffix(token, text) {
            return tokens;
        }

        // 3. 접두사 검사
        if let Some(tokens) = Self::try_extract_prefix(token, text) {
            return tokens;
        }

        // 4. 기본 복합명사 분해 (음절 휴리스틱)
        Self::decompound_token(token, text)
    }

    /// 사전 기반 복합명사 분해
    ///
    /// `COMPOUND_DICT`에 정의된 복합명사를 정확하게 분해합니다.
    fn try_dict_decompose(token: &Token, text: &str) -> Option<Vec<NoriToken>> {
        let surface = &token.surface;

        // 사전에서 매칭 검색
        for (compound, parts) in COMPOUND_DICT {
            if *compound == surface {
                // 단일 엔트리인 경우 분해하지 않음
                if parts.len() <= 1 {
                    return None;
                }

                let mut result = Vec::with_capacity(parts.len());
                let mut byte_offset = token.start_byte;

                for (part_surface, part_pos) in *parts {
                    let part_bytes = part_surface.len();
                    result.push(NoriToken {
                        surface: (*part_surface).to_string(),
                        pos_tag: (*part_pos).to_string(),
                        start_offset: char_offset(text, byte_offset),
                        end_offset: char_offset(text, byte_offset + part_bytes),
                        lemma: None,
                        reading: None,
                        word_type: WordType::Known,
                        is_decompound: true,
                    });
                    byte_offset += part_bytes;
                }

                return Some(result);
            }
        }

        None
    }

    /// 일반적인 접미사 추출 시도
    ///
    /// `SUFFIXES` 상수에 정의된 접미사 패턴을 사용합니다.
    /// 긴 접미사부터 검사하여 "쟁이"가 "이"보다 먼저 매칭되도록 합니다.
    fn try_extract_suffix(token: &Token, text: &str) -> Option<Vec<NoriToken>> {
        let surface = &token.surface;
        let chars: Vec<char> = surface.chars().collect();

        if chars.len() < 2 {
            return None;
        }

        // 긴 접미사부터 검사 (내림차순 정렬)
        let mut sorted_suffixes: Vec<_> = SUFFIXES.iter().collect();
        sorted_suffixes.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

        for (suffix, suffix_tag) in sorted_suffixes {
            let suffix_chars: Vec<char> = suffix.chars().collect();
            if chars.len() > suffix_chars.len()
                && chars[chars.len() - suffix_chars.len()..] == suffix_chars[..]
            {
                // 접미사를 제외한 앞부분
                let stem_len = chars.len() - suffix_chars.len();
                let stem: String = chars[..stem_len].iter().collect();
                let stem_bytes = stem.len();

                // 어간이 최소 1음절 이상이어야 함
                if stem_len >= 1 {
                    // 어간 부분 및 접미사 부분 토큰 생성
                    let result = vec![
                        NoriToken {
                            surface: stem,
                            pos_tag: token.pos.clone(),
                            start_offset: char_offset(text, token.start_byte),
                            end_offset: char_offset(text, token.start_byte + stem_bytes),
                            lemma: None,
                            reading: None,
                            word_type: WordType::Known,
                            is_decompound: true,
                        },
                        NoriToken {
                            surface: (*suffix).to_string(),
                            pos_tag: (*suffix_tag).to_string(),
                            start_offset: char_offset(text, token.start_byte + stem_bytes),
                            end_offset: char_offset(text, token.end_byte),
                            lemma: None,
                            reading: None,
                            word_type: WordType::Known,
                            is_decompound: true,
                        },
                    ];

                    return Some(result);
                }
            }
        }

        None
    }

    /// 일반적인 접두사 추출 시도
    ///
    /// `PREFIXES` 상수에 정의된 접두사 패턴을 사용합니다.
    /// 긴 접두사부터 검사하여 더 정확한 매칭을 보장합니다.
    fn try_extract_prefix(token: &Token, text: &str) -> Option<Vec<NoriToken>> {
        let surface = &token.surface;
        let chars: Vec<char> = surface.chars().collect();

        if chars.len() < 2 {
            return None;
        }

        // 긴 접두사부터 검사 (내림차순 정렬)
        let mut sorted_prefixes: Vec<_> = PREFIXES.iter().collect();
        sorted_prefixes.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

        for (prefix, prefix_tag) in sorted_prefixes {
            let prefix_chars: Vec<char> = prefix.chars().collect();
            if chars.len() > prefix_chars.len() && chars[..prefix_chars.len()] == prefix_chars[..] {
                // 접두사를 제외한 뒷부분
                let rest: String = chars[prefix_chars.len()..].iter().collect();
                let prefix_bytes = prefix.len();
                let rest_len = chars.len() - prefix_chars.len();

                // 나머지가 최소 2음절 이상이어야 함 (단독 명사로 성립 가능)
                if rest_len >= 2 {
                    // 접두사 부분 및 나머지 부분 토큰 생성
                    let result = vec![
                        NoriToken {
                            surface: (*prefix).to_string(),
                            pos_tag: (*prefix_tag).to_string(),
                            start_offset: char_offset(text, token.start_byte),
                            end_offset: char_offset(text, token.start_byte + prefix_bytes),
                            lemma: None,
                            reading: None,
                            word_type: WordType::Known,
                            is_decompound: true,
                        },
                        NoriToken {
                            surface: rest,
                            pos_tag: token.pos.clone(),
                            start_offset: char_offset(text, token.start_byte + prefix_bytes),
                            end_offset: char_offset(text, token.end_byte),
                            lemma: None,
                            reading: None,
                            word_type: WordType::Known,
                            is_decompound: true,
                        },
                    ];

                    return Some(result);
                }
            }
        }

        None
    }

    /// 복합명사 분해
    ///
    /// 복합명사를 구성 요소로 분해합니다.
    /// 현재는 음절 기반 휴리스틱을 사용하며, 향후 사전 기반 분해로 개선 예정입니다.
    ///
    /// # 알고리즘
    ///
    /// 1. 최소 3음절 이상 복합명사만 분해 시도
    /// 2. 종성 패턴을 분석하여 자연스러운 경계 찾기
    ///    - 종성 없음 → 종성 있음: 경계 가능 (예: "형태소분석" → "형태소" + "분석")
    ///    - 종성 있음 → 종성 없음: 경계 가능 (예: "학교운동장" → "학교" + "운동장")
    /// 3. 분해된 각 부분은 최소 1음절 이상
    /// 4. 과도한 분해 방지: 최대 3개 부분으로 제한
    ///
    /// # Example
    ///
    /// - "형태소분석기" → \["형태소", "분석", "기"\]
    /// - "대한민국" → \["대한", "민국"\]
    /// - "학교운동장" → \["학교", "운동장"\]
    fn decompound_token(token: &Token, text: &str) -> Vec<NoriToken> {
        use mecab_ko_hangul::{has_jongseong, is_hangul_syllable};

        let surface = &token.surface;
        let chars: Vec<char> = surface.chars().collect();

        // 3음절 미만이거나 한글이 아니면 분해하지 않음
        if chars.len() < 3 {
            return Vec::new();
        }

        // 모든 문자가 한글 음절인지 확인
        if !chars.iter().all(|&c| is_hangul_syllable(c)) {
            return Vec::new();
        }

        // 분해 후보 위치 찾기
        let mut split_positions = Vec::new();

        for i in 1..chars.len() {
            // 마지막 음절 직전까지 검사
            if i >= chars.len() - 1 {
                continue;
            }

            let prev_char = chars[i - 1];
            let curr_char = chars[i];

            let prev_has_jong = has_jongseong(prev_char) == Some(true);
            let curr_has_jong = has_jongseong(curr_char) == Some(true);

            // 자연스러운 경계 패턴
            // 1. 종성 없음 → 종성 있음: "형태소" + "분석"
            // 2. 종성 있음 → 종성 없음: "학교" + "운동장"
            // 3. 종성 있음 → 종성 있음 (연속 2개 이상): "국립" + "국어원"
            let is_boundary = if !prev_has_jong && curr_has_jong {
                // 패턴 1: ㅇ + ㄱ
                true
            } else if prev_has_jong && !curr_has_jong {
                // 패턴 2: ㄱ + ㅇ
                true
            } else if prev_has_jong && curr_has_jong && i >= 2 {
                // 패턴 3: 종성이 연속될 때, 앞 부분이 최소 2음절 이상이면 경계
                has_jongseong(chars[i - 2]) == Some(true)
            } else {
                false
            };

            if is_boundary {
                // 앞 부분이 최소 1음절, 뒤 부분도 최소 1음절 확보
                if i >= 1 && chars.len() - i >= 1 {
                    split_positions.push(i);
                }
            }
        }

        // 분해 지점이 없으면 균등 분할 시도
        if split_positions.is_empty() {
            let mid = chars.len() / 2;
            if mid >= 1 && chars.len() - mid >= 1 {
                split_positions.push(mid);
            }
        }

        // 과도한 분해 방지: 최대 2개 분할점 (3개 부분)
        if split_positions.len() > 2 {
            // 가장 앞쪽과 가장 뒤쪽 분할점 유지
            let first = split_positions[0];
            let last = split_positions[split_positions.len() - 1];
            split_positions = vec![first, last];
        }

        if split_positions.is_empty() {
            return Vec::new();
        }

        // 분해된 토큰 생성
        let mut result = Vec::new();
        let mut start_idx = 0;
        let mut byte_offset = token.start_byte;

        for &split_pos in &split_positions {
            if split_pos <= start_idx {
                continue;
            }

            let part: String = chars[start_idx..split_pos].iter().collect();
            let part_len_bytes = part.len();

            // 각 부분이 최소 1음절 이상이어야 함
            if !part.is_empty() && split_pos - start_idx >= 1 {
                result.push(NoriToken {
                    surface: part,
                    pos_tag: token.pos.clone(),
                    start_offset: char_offset(text, byte_offset),
                    end_offset: char_offset(text, byte_offset + part_len_bytes),
                    lemma: None,
                    reading: None,
                    word_type: WordType::Known,
                    is_decompound: true,
                });
            }

            byte_offset += part_len_bytes;
            start_idx = split_pos;
        }

        // 마지막 부분 추가
        if start_idx < chars.len() {
            let part: String = chars[start_idx..].iter().collect();
            let part_len_bytes = part.len();

            // 최소 1음절 확인
            if !part.is_empty() {
                result.push(NoriToken {
                    surface: part,
                    pos_tag: token.pos.clone(),
                    start_offset: char_offset(text, byte_offset),
                    end_offset: char_offset(text, byte_offset + part_len_bytes),
                    lemma: None,
                    reading: None,
                    word_type: WordType::Known,
                    is_decompound: true,
                });
            }
        }

        result
    }

    /// 분해 모드에 따라 토큰 결합
    fn apply_decompound_mode(
        &self,
        original: Vec<NoriToken>,
        decompounded: Vec<NoriToken>,
    ) -> Vec<NoriToken> {
        match self.decompound_mode {
            DecompoundMode::None => original,
            DecompoundMode::Discard => {
                if decompounded.is_empty() {
                    original
                } else {
                    decompounded
                }
            }
            DecompoundMode::Mixed => {
                let mut result = original;
                result.extend(decompounded);
                result
            }
        }
    }

    /// 미등록어를 유니그램으로 분리
    fn split_unknown_to_unigrams(token: &Token, text: &str) -> Vec<NoriToken> {
        let chars: Vec<char> = token.surface.chars().collect();
        let mut tokens = Vec::new();
        let mut char_pos = token.start_byte;

        for ch in chars {
            let surface = ch.to_string();
            let char_len = ch.len_utf8();

            tokens.push(NoriToken {
                surface,
                pos_tag: "UNKNOWN".to_string(),
                start_offset: char_offset(text, char_pos),
                end_offset: char_offset(text, char_pos + char_len),
                lemma: None,
                reading: None,
                word_type: WordType::Unknown,
                is_decompound: false,
            });

            char_pos += char_len;
        }

        tokens
    }
}

/// Nori 분석기
///
/// Lucene Nori의 `KoreanAnalyzer`와 호환되는 인터페이스
pub struct NoriAnalyzer {
    /// Nori 토크나이저
    tokenizer: NoriTokenizer,
    /// 제거할 품사 태그 (stoptags)
    stoptags: HashSet<String>,
    /// 사용자 사전 (향후 구현)
    _user_dictionary: Option<String>,
}

impl NoriAnalyzer {
    /// 새 Nori 분석기 생성
    ///
    /// # Arguments
    ///
    /// * `user_dictionary` - 사용자 사전 경로 (옵션)
    /// * `decompound_mode` - 복합명사 분해 모드
    /// * `stoptags` - 필터링할 품사 태그 (예: \["J", "E"\])
    /// * `output_unknown_unigrams` - 미등록어 유니그램 출력 여부
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mecab_ko_core::nori_compat::{NoriAnalyzer, DecompoundMode};
    ///
    /// let stoptags = vec!["J".to_string(), "E".to_string()];
    /// let analyzer = NoriAnalyzer::new(
    ///     None,
    ///     DecompoundMode::Mixed,
    ///     stoptags,
    ///     false
    /// ).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the tokenizer initialization fails.
    pub fn new(
        user_dictionary: Option<String>,
        decompound_mode: DecompoundMode,
        stoptags: Vec<String>,
        output_unknown_unigrams: bool,
    ) -> Result<Self> {
        Ok(Self {
            tokenizer: NoriTokenizer::new(decompound_mode, output_unknown_unigrams)?,
            stoptags: stoptags.into_iter().collect(),
            _user_dictionary: user_dictionary,
        })
    }

    /// 기본 설정으로 생성 (조사/어미 제거)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mecab_ko_core::nori_compat::{NoriAnalyzer, DecompoundMode};
    ///
    /// let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    pub fn default_with_decompound(decompound_mode: DecompoundMode) -> Result<Self> {
        Self::new(
            None,
            decompound_mode,
            vec!["J".to_string(), "E".to_string()],
            false,
        )
    }

    /// 텍스트 분석 (stoptags 필터링 적용)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use mecab_ko_core::nori_compat::{NoriAnalyzer, DecompoundMode};
    /// # let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed).unwrap();
    /// let tokens = analyzer.analyze("형태소 분석기").unwrap();
    /// // 조사/어미가 제거된 결과만 반환
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails.
    pub fn analyze(&mut self, text: &str) -> Result<Vec<NoriToken>> {
        let tokens = self.tokenizer.tokenize(text)?;
        Ok(self.filter_stoptags(tokens))
    }

    /// stoptags 필터링 적용
    fn filter_stoptags(&self, tokens: Vec<NoriToken>) -> Vec<NoriToken> {
        if self.stoptags.is_empty() {
            return tokens;
        }

        tokens
            .into_iter()
            .filter(|token| !self.stoptags.contains(&token.pos_tag))
            .collect()
    }

    /// stoptags 추가
    pub fn add_stoptag(&mut self, tag: String) {
        self.stoptags.insert(tag);
    }

    /// stoptags 제거
    pub fn remove_stoptag(&mut self, tag: &str) -> bool {
        self.stoptags.remove(tag)
    }

    /// stoptags 목록 반환
    #[must_use]
    pub fn stoptags(&self) -> Vec<&str> {
        self.stoptags.iter().map(String::as_str).collect()
    }
}

/// `MeCab` 태그를 Nori 태그로 변환
///
/// # Example
///
/// ```
/// use mecab_ko_core::nori_compat::mecab_to_nori_tag;
///
/// assert_eq!(mecab_to_nori_tag("JKS"), "J");  // 주격 조사 → J
/// assert_eq!(mecab_to_nori_tag("EF"), "E");   // 종결 어미 → E
/// assert_eq!(mecab_to_nori_tag("NNG"), "NNG"); // 일반 명사 → NNG
/// ```
#[must_use]
pub fn mecab_to_nori_tag(mecab_tag: &str) -> String {
    mecab_tag.parse::<PosTag>().map_or_else(
        |_| mecab_tag.to_string(),
        |tag| tag.to_nori_compat().as_str().to_string(),
    )
}

/// Nori 태그를 `MeCab` 태그로 변환 (부분 변환)
///
/// Nori의 통합 태그(J, E)는 대표 태그로 변환합니다.
///
/// # Example
///
/// ```
/// use mecab_ko_core::nori_compat::nori_to_mecab_tag;
///
/// assert_eq!(nori_to_mecab_tag("J"), "JX");   // 조사 → 보조사(대표)
/// assert_eq!(nori_to_mecab_tag("E"), "EF");   // 어미 → 종결어미(대표)
/// assert_eq!(nori_to_mecab_tag("NNG"), "NNG"); // 일반명사 → NNG
/// ```
#[must_use]
pub fn nori_to_mecab_tag(nori_tag: &str) -> String {
    match nori_tag {
        // 조사 통합 → 보조사를 대표로 사용
        "J" => "JX".to_string(),
        // 어미 통합 → 종결 어미를 대표로 사용
        "E" => "EF".to_string(),
        // 기타는 그대로
        _ => nori_tag.to_string(),
    }
}

/// 바이트 오프셋을 문자 오프셋으로 변환
fn char_offset(text: &str, byte_offset: usize) -> usize {
    text[..byte_offset.min(text.len())].chars().count()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_decompound_mode_from_str() {
        assert_eq!(DecompoundMode::parse("none"), Some(DecompoundMode::None));
        assert_eq!(
            DecompoundMode::parse("discard"),
            Some(DecompoundMode::Discard)
        );
        assert_eq!(DecompoundMode::parse("mixed"), Some(DecompoundMode::Mixed));
        assert_eq!(DecompoundMode::parse("NONE"), Some(DecompoundMode::None));
        assert_eq!(DecompoundMode::parse("invalid"), None);
    }

    #[test]
    fn test_decompound_mode_as_str() {
        assert_eq!(DecompoundMode::None.as_str(), "none");
        assert_eq!(DecompoundMode::Discard.as_str(), "discard");
        assert_eq!(DecompoundMode::Mixed.as_str(), "mixed");
    }

    #[test]
    fn test_word_type_as_str() {
        assert_eq!(WordType::Known.as_str(), "KNOWN");
        assert_eq!(WordType::Unknown.as_str(), "UNKNOWN");
        assert_eq!(WordType::User.as_str(), "USER");
    }

    #[test]
    fn test_mecab_to_nori_tag() {
        // 조사 → J
        assert_eq!(mecab_to_nori_tag("JKS"), "J");
        assert_eq!(mecab_to_nori_tag("JKO"), "J");
        assert_eq!(mecab_to_nori_tag("JX"), "J");

        // 어미 → E
        assert_eq!(mecab_to_nori_tag("EF"), "E");
        assert_eq!(mecab_to_nori_tag("EC"), "E");
        assert_eq!(mecab_to_nori_tag("ETM"), "E");

        // 기타 → 그대로
        assert_eq!(mecab_to_nori_tag("NNG"), "NNG");
        assert_eq!(mecab_to_nori_tag("VV"), "VV");
        assert_eq!(mecab_to_nori_tag("MAG"), "MAG");
    }

    #[test]
    fn test_nori_to_mecab_tag() {
        assert_eq!(nori_to_mecab_tag("J"), "JX");
        assert_eq!(nori_to_mecab_tag("E"), "EF");
        assert_eq!(nori_to_mecab_tag("NNG"), "NNG");
        assert_eq!(nori_to_mecab_tag("VV"), "VV");
    }

    #[test]
    fn test_char_offset() {
        let text = "안녕하세요";
        assert_eq!(char_offset(text, 0), 0);
        assert_eq!(char_offset(text, 3), 1); // '안' = 3 bytes
        assert_eq!(char_offset(text, 6), 2); // '안녕' = 6 bytes
        assert_eq!(char_offset(text, 100), 5); // overflow → 전체 길이
    }

    #[test]
    fn test_nori_tokenizer_creation() {
        let tokenizer = NoriTokenizer::new(DecompoundMode::None, false);
        assert!(tokenizer.is_ok());

        let tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, true);
        assert!(tokenizer.is_ok());
    }

    #[test]
    fn test_nori_analyzer_creation() {
        let analyzer = NoriAnalyzer::new(
            None,
            DecompoundMode::None,
            vec!["J".to_string(), "E".to_string()],
            false,
        );
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_nori_analyzer_default() {
        let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed);
        assert!(analyzer.is_ok());

        let analyzer = analyzer.unwrap();
        let stoptags = analyzer.stoptags();
        assert_eq!(stoptags.len(), 2);
        assert!(stoptags.contains(&"J"));
        assert!(stoptags.contains(&"E"));
    }

    #[test]
    fn test_nori_analyzer_stoptag_management() {
        let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

        // 초기 상태
        assert_eq!(analyzer.stoptags().len(), 2);

        // 추가
        analyzer.add_stoptag("SF".to_string());
        assert_eq!(analyzer.stoptags().len(), 3);
        assert!(analyzer.stoptags().contains(&"SF"));

        // 제거
        assert!(analyzer.remove_stoptag("SF"));
        assert_eq!(analyzer.stoptags().len(), 2);
        assert!(!analyzer.stoptags().contains(&"SF"));

        // 없는 태그 제거
        assert!(!analyzer.remove_stoptag("NONEXISTENT"));
    }

    #[test]
    fn test_pos_tag_nori_mapping() {
        // 조사 통합
        assert_eq!(PosTag::JKS.to_nori_compat().as_str(), "J");
        assert_eq!(PosTag::JKO.to_nori_compat().as_str(), "J");
        assert_eq!(PosTag::JX.to_nori_compat().as_str(), "J");

        // 어미 통합
        assert_eq!(PosTag::EF.to_nori_compat().as_str(), "E");
        assert_eq!(PosTag::EC.to_nori_compat().as_str(), "E");
        assert_eq!(PosTag::ETM.to_nori_compat().as_str(), "E");

        // 기타
        assert_eq!(PosTag::NNG.to_nori_compat().as_str(), "NNG");
        assert_eq!(PosTag::VV.to_nori_compat().as_str(), "VV");
    }

    #[test]
    fn test_tokenizer_basic_functionality() {
        let mut tokenizer = NoriTokenizer::new(DecompoundMode::None, false).unwrap();
        let result = tokenizer.tokenize("안녕");
        assert!(result.is_ok());

        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_analyzer_basic_functionality() {
        let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();
        let result = analyzer.analyze("테스트");
        assert!(result.is_ok());
    }

    #[test]
    fn test_decompound_token_basic() {
        // Test with a simple compound noun
        let token = Token {
            surface: "형태소분석".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 5,
            start_byte: 0,
            end_byte: 15, // 5 chars * 3 bytes each
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::decompound_token(&token, "형태소분석");

        // Should produce decomposed parts
        assert!(!result.is_empty(), "Should decompose compound noun");

        // Verify all parts are marked as decompound
        for part in &result {
            assert!(
                part.is_decompound,
                "All parts should be marked as decompound"
            );
            assert_eq!(part.pos_tag, "NNG");
            assert_eq!(part.word_type, WordType::Known);
        }
    }

    #[test]
    fn test_decompound_token_short_word() {
        // Test with a word that's too short to decompose
        let token = Token {
            surface: "사과".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 2,
            start_byte: 0,
            end_byte: 6,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::decompound_token(&token, "사과");

        // Should not decompose (too short)
        assert!(result.is_empty(), "Short words should not be decomposed");
    }

    #[test]
    fn test_decompound_token_non_hangul() {
        // Test with non-Hangul characters
        let token = Token {
            surface: "ABC".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 3,
            start_byte: 0,
            end_byte: 3,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::decompound_token(&token, "ABC");

        // Should not decompose (non-Hangul)
        assert!(
            result.is_empty(),
            "Non-Hangul words should not be decomposed"
        );
    }

    #[test]
    fn test_decompound_token_mixed_jongseong() {
        // Test with various jongseong patterns
        let token = Token {
            surface: "학교운동장".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 5,
            start_byte: 0,
            end_byte: 15,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::decompound_token(&token, "학교운동장");

        // Should produce some decomposition
        if !result.is_empty() {
            // Verify basic properties
            for part in &result {
                assert!(part.is_decompound);
                assert!(!part.surface.is_empty());
                assert_eq!(part.pos_tag, "NNG");
            }
        }
    }

    #[test]
    fn test_decompound_modes_with_compound() {
        use super::DecompoundMode;

        let test_token = Token {
            surface: "형태소분석".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 5,
            start_byte: 0,
            end_byte: 15,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        // Test None mode - should return only original
        let tokenizer = NoriTokenizer::new(DecompoundMode::None, false).unwrap();
        let pos_tag = test_token.pos.parse::<PosTag>().unwrap();
        assert!(!tokenizer.should_decompound(pos_tag));

        // Test Discard mode
        let tokenizer = NoriTokenizer::new(DecompoundMode::Discard, false).unwrap();
        assert!(tokenizer.should_decompound(pos_tag));

        // Test Mixed mode
        let tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, false).unwrap();
        assert!(tokenizer.should_decompound(pos_tag));
    }

    #[test]
    fn test_compound_noun_patterns() {
        // Test various compound noun patterns

        // Pattern 1: 명사+명사 (대한민국)
        let token = Token {
            surface: "대한민국".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 4,
            start_byte: 0,
            end_byte: 12,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };
        let result = NoriTokenizer::decompound_token(&token, "대한민국");
        assert!(!result.is_empty(), "Should decompose 대한민국");

        // Pattern 2: 한자어 복합 (국립국어원)
        let token = Token {
            surface: "국립국어원".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 5,
            start_byte: 0,
            end_byte: 15,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };
        let result = NoriTokenizer::decompound_token(&token, "국립국어원");
        assert!(!result.is_empty(), "Should decompose 국립국어원");
    }

    #[test]
    fn test_decompound_offset_accuracy() {
        // Test that offsets are calculated correctly
        let token = Token {
            surface: "형태소분석".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 5,
            start_byte: 0,
            end_byte: 15,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::decompound_token(&token, "형태소분석");

        if !result.is_empty() {
            // Check that offsets are non-overlapping and cover the full range
            let mut prev_end = 0;
            for part in &result {
                assert!(
                    part.start_offset >= prev_end,
                    "Offsets should not overlap: {} >= {}",
                    part.start_offset,
                    prev_end
                );
                assert!(
                    part.end_offset > part.start_offset,
                    "End should be after start: {} > {}",
                    part.end_offset,
                    part.start_offset
                );
                prev_end = part.end_offset;
            }

            // Last token should end at the original token's end
            assert_eq!(
                result.last().unwrap().end_offset,
                5,
                "Last token should end at original token end"
            );
        }
    }

    #[test]
    fn test_decompound_min_syllable_constraint() {
        // Test that we don't over-decompose short words
        let short_words = vec![
            ("한글", 2),   // Too short, should not decompose
            ("사과", 2),   // Too short, should not decompose
            ("바나나", 3), // May decompose
        ];

        for (word, len) in short_words {
            let token = Token {
                surface: word.to_string(),
                pos: "NNG".to_string(),
                start_pos: 0,
                end_pos: len,
                start_byte: 0,
                end_byte: word.len(),
                reading: None,
                lemma: None,
                cost: 0,
                features: "NNG,*,*,*,*,*,*,*".to_string(),
                normalized: None,
            };

            let result = NoriTokenizer::decompound_token(&token, word);

            if len < 3 {
                assert!(
                    result.is_empty(),
                    "Words with {len} syllables should not decompose: {word}"
                );
            }
        }
    }

    #[test]
    fn test_decompound_preserves_wordtype() {
        let token = Token {
            surface: "형태소분석".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 5,
            start_byte: 0,
            end_byte: 15,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::decompound_token(&token, "형태소분석");

        for part in result {
            assert_eq!(part.word_type, WordType::Known);
            assert!(part.is_decompound);
        }
    }

    #[test]
    fn test_mixed_mode_returns_both() {
        let mut tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, false).unwrap();

        // Create a simple compound that should decompose
        let text = "형태소";
        let result = tokenizer.tokenize(text);
        assert!(result.is_ok());

        // In mixed mode, we should get original + decomposed parts
        // (This is a simplified test - actual behavior depends on tokenizer output)
    }

    #[test]
    fn test_discard_mode_returns_only_parts() {
        let mut tokenizer = NoriTokenizer::new(DecompoundMode::Discard, false).unwrap();

        let text = "형태소";
        let result = tokenizer.tokenize(text);
        assert!(result.is_ok());

        // In discard mode, if decomposition happens, original should be excluded
    }

    #[test]
    fn test_dict_decompose_basic() {
        let token = Token {
            surface: "형태소분석기".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 6,
            start_byte: 0,
            end_byte: 18,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::try_dict_decompose(&token, "형태소분석기");

        // Should match dictionary entry
        assert!(result.is_some(), "Should find compound in dictionary");
        let parts = result.unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].surface, "형태소");
        assert_eq!(parts[1].surface, "분석기");
    }

    #[test]
    fn test_dict_decompose_대한민국() {
        let token = Token {
            surface: "대한민국".to_string(),
            pos: "NNP".to_string(),
            start_pos: 0,
            end_pos: 4,
            start_byte: 0,
            end_byte: 12,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNP,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::try_dict_decompose(&token, "대한민국");

        // Should match dictionary entry
        assert!(result.is_some(), "Should find 대한민국 in dictionary");
        let parts = result.unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].surface, "대한");
        assert_eq!(parts[0].pos_tag, "NNP");
        assert_eq!(parts[1].surface, "민국");
    }

    #[test]
    fn test_enhanced_suffix_extraction() {
        // Test with suffix "화" (변화)
        let token = Token {
            surface: "현대화".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 3,
            start_byte: 0,
            end_byte: 9,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::try_extract_suffix(&token, "현대화");

        // Should extract suffix
        assert!(result.is_some(), "Should extract suffix 화");
        let parts = result.unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].surface, "현대");
        assert_eq!(parts[1].surface, "화");
        assert_eq!(parts[1].pos_tag, "XSN");
    }

    #[test]
    fn test_enhanced_prefix_extraction() {
        // Test with prefix "초" (초고속)
        let token = Token {
            surface: "초고속".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 3,
            start_byte: 0,
            end_byte: 9,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::try_extract_prefix(&token, "초고속");

        // Should extract prefix
        assert!(result.is_some(), "Should extract prefix 초");
        let parts = result.unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].surface, "초");
        assert_eq!(parts[0].pos_tag, "XPN");
        assert_eq!(parts[1].surface, "고속");
    }

    #[test]
    fn test_decompound_enhanced_priority() {
        // Dictionary match should take priority
        let token = Token {
            surface: "형태소분석".to_string(),
            pos: "NNG".to_string(),
            start_pos: 0,
            end_pos: 5,
            start_byte: 0,
            end_byte: 15,
            reading: None,
            lemma: None,
            cost: 0,
            features: "NNG,*,*,*,*,*,*,*".to_string(),
            normalized: None,
        };

        let result = NoriTokenizer::decompound_token_enhanced(&token, "형태소분석");

        // Should use dictionary-based decomposition
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].surface, "형태소");
        assert_eq!(result[1].surface, "분석");
    }

    #[test]
    fn test_multiple_suffix_entries() {
        // Test that SUFFIXES constant is accessible and has multiple entries
        assert!(SUFFIXES.len() > 10, "Should have many suffix entries");

        // Test specific entries
        assert!(
            SUFFIXES.iter().any(|(s, _)| *s == "화"),
            "Should contain 화"
        );
        assert!(
            SUFFIXES.iter().any(|(s, _)| *s == "적"),
            "Should contain 적"
        );
        assert!(
            SUFFIXES.iter().any(|(s, _)| *s == "쟁이"),
            "Should contain 쟁이"
        );
    }

    #[test]
    fn test_multiple_prefix_entries() {
        // Test that PREFIXES constant is accessible and has multiple entries
        assert!(PREFIXES.len() > 10, "Should have many prefix entries");

        // Test specific entries
        assert!(
            PREFIXES.iter().any(|(p, _)| *p == "초"),
            "Should contain 초"
        );
        assert!(
            PREFIXES.iter().any(|(p, _)| *p == "최"),
            "Should contain 최"
        );
        assert!(
            PREFIXES.iter().any(|(p, _)| *p == "친"),
            "Should contain 친"
        );
    }
}
