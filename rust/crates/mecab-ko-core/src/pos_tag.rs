//! 품사 태그 정의 (세종 품사 태그 체계 기반)
//!
//! 이 모듈은 21세기 세종계획의 품사 태그 체계를 기반으로
//! mecab-ko-dic 확장 태그를 포함한 품사 태그를 정의합니다.

use std::{fmt, str::FromStr};

/// 품사 태그 (Part-of-Speech Tag)
///
/// 21세기 세종계획 품사 태그 체계 + mecab-ko-dic 확장
///
/// # Example
/// ```
/// use mecab_ko_core::pos_tag::PosTag;
/// use std::str::FromStr;
///
/// let tag = PosTag::from_str("NNG").unwrap();
/// assert_eq!(tag.as_str(), "NNG");
/// assert!(tag.is_content_word());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PosTag {
    // ============================================
    // 체언 (Nominals) - 0~9
    // ============================================
    /// 일반 명사 (General Noun)
    /// 예: 사과, 컴퓨터, 사랑, 학교
    NNG = 0,

    /// 고유 명사 (Proper Noun)
    /// 예: 서울, 세종대왕, 삼성, 한강
    NNP = 1,

    /// 의존 명사 (Dependent Noun)
    /// 예: 것, 수, 줄, 뿐, 데, 바
    NNB = 2,

    /// 단위 의존 명사 (Counter/Unit Noun)
    /// mecab-ko-dic 확장 (세종 NNB에서 분리)
    /// 예: 개, 마리, 권, 명, 원
    NNBC = 3,

    /// 대명사 (Pronoun)
    /// 예: 나, 너, 우리, 이것, 저기
    NP = 4,

    /// 수사 (Numeral)
    /// 예: 하나, 둘, 첫째, 셋째
    NR = 5,

    // ============================================
    // 용언 (Predicates) - 10~19
    // ============================================
    /// 동사 (Verb)
    /// 예: 먹다, 가다, 하다, 되다
    VV = 10,

    /// 형용사 (Adjective)
    /// 예: 예쁘다, 크다, 좋다, 빠르다
    VA = 11,

    /// 보조 용언 (Auxiliary Predicate)
    /// 예: -아/어 있다, -고 싶다, -아/어 보다
    VX = 12,

    /// 긍정 지정사 (Positive Copula)
    /// 예: 이다
    VCP = 13,

    /// 부정 지정사 (Negative Copula)
    /// 예: 아니다
    VCN = 14,

    // ============================================
    // 수식언 (Modifiers) - 20~29
    // ============================================
    /// 관형사 (Determiner)
    /// 예: 새, 헌, 이, 그, 저, 모든
    MM = 20,

    /// 일반 부사 (General Adverb)
    /// 예: 매우, 아주, 빨리, 천천히
    MAG = 21,

    /// 접속 부사 (Conjunctive Adverb)
    /// 예: 그러나, 그리고, 따라서, 하지만
    MAJ = 22,

    // ============================================
    // 독립언 (Interjection) - 30~39
    // ============================================
    /// 감탄사 (Interjection)
    /// 예: 아, 와, 어머나, 네, 예
    IC = 30,

    // ============================================
    // 관계언 - 조사 (Particles) - 40~49
    // ============================================
    /// 주격 조사 (Subjective Case Marker)
    /// 예: 이, 가
    JKS = 40,

    /// 보격 조사 (Complement Case Marker)
    /// 예: 이, 가 (되다/아니다 앞)
    JKC = 41,

    /// 관형격 조사 (Genitive Case Marker)
    /// 예: 의
    JKG = 42,

    /// 목적격 조사 (Objective Case Marker)
    /// 예: 을, 를
    JKO = 43,

    /// 부사격 조사 (Adverbial Case Marker)
    /// 예: 에, 에서, 로, 으로, 에게
    JKB = 44,

    /// 호격 조사 (Vocative Case Marker)
    /// 예: 아, 야, 이여
    JKV = 45,

    /// 인용격 조사 (Quotative Case Marker)
    /// 예: 라고, 고
    JKQ = 46,

    /// 보조사 (Auxiliary Particle)
    /// 예: 은, 는, 도, 만, 까지, 조차
    JX = 47,

    /// 접속 조사 (Conjunctive Particle)
    /// 예: 와, 과, 하고
    JC = 48,

    // ============================================
    // 어미 (Endings) - 50~59
    // ============================================
    /// 선어말 어미 (Pre-final Ending)
    /// 예: -시-, -었-, -겠-
    EP = 50,

    /// 종결 어미 (Final Ending)
    /// 예: -다, -ㄴ다, -니, -라
    EF = 51,

    /// 연결 어미 (Connective Ending)
    /// 예: -고, -면, -어서, -니까
    EC = 52,

    /// 명사형 전성 어미 (Nominal Transformative Ending)
    /// 예: -ㅁ, -기
    ETN = 53,

    /// 관형형 전성 어미 (Adnominal Transformative Ending)
    /// 예: -ㄴ, -는, -ㄹ
    ETM = 54,

    // ============================================
    // 접사 (Affixes) - 60~69
    // ============================================
    /// 체언 접두사 (Noun Prefix)
    /// 예: 풋-, 맨-, 헛-, 순-
    XPN = 60,

    /// 명사파생 접미사 (Noun Derivational Suffix)
    /// 예: -님, -적, -화, -성
    XSN = 61,

    /// 동사파생 접미사 (Verb Derivational Suffix)
    /// 예: -하다, -되다, -시키다
    XSV = 62,

    /// 형용사파생 접미사 (Adjective Derivational Suffix)
    /// 예: -스럽다, -롭다, -답다
    XSA = 63,

    /// 어근 (Root)
    /// 예: 꼼꼼, 달콤, 시큼
    XR = 64,

    // ============================================
    // 기호 (Symbols) - 70~89
    // ============================================
    /// 마침 부호 (Terminal Punctuation)
    /// 예: . ? !
    SF = 70,

    /// 공백 (Space)
    SP = 71,

    /// 여는 괄호 (Opening Bracket)
    /// mecab-ko-dic 확장 (세종 SS에서 분리)
    /// 예: ( [ {
    SSO = 72,

    /// 닫는 괄호 (Closing Bracket)
    /// mecab-ko-dic 확장 (세종 SS에서 분리)
    /// 예: ) ] }
    SSC = 73,

    /// 구분자 (Separator)
    /// mecab-ko-dic 확장
    /// 예: , ; :
    SC = 74,

    /// 줄임표 (Ellipsis)
    /// 예: ...
    SE = 75,

    /// 기타 기호 (Other Symbol)
    /// 예: @ # $ % & *
    SY = 76,

    /// 외국어 (Foreign Language)
    /// 예: hello, computer, ABC
    SL = 77,

    /// 한자 (Hanja/Chinese Character)
    /// 예: 韓國, 大學, 文化
    SH = 78,

    /// 숫자 (Number)
    /// 예: 123, 45.6, 2024
    SN = 79,

    // ============================================
    // 특수 (Special) - 90~99
    // ============================================
    /// 미등록어 (Unknown Word)
    /// 사전에 없는 단어
    Unknown = 99,
}

/// Error type for `PosTag` parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePosTagError {
    invalid_tag: String,
}

impl fmt::Display for ParsePosTagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid POS tag: {}", self.invalid_tag)
    }
}

impl std::error::Error for ParsePosTagError {}

impl FromStr for PosTag {
    type Err = ParsePosTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // 체언
            "NNG" => Ok(Self::NNG),
            "NNP" => Ok(Self::NNP),
            "NNB" => Ok(Self::NNB),
            "NNBC" => Ok(Self::NNBC),
            "NP" => Ok(Self::NP),
            "NR" => Ok(Self::NR),
            // 용언
            "VV" => Ok(Self::VV),
            "VA" => Ok(Self::VA),
            "VX" => Ok(Self::VX),
            "VCP" => Ok(Self::VCP),
            "VCN" => Ok(Self::VCN),
            // 수식언
            "MM" => Ok(Self::MM),
            "MAG" => Ok(Self::MAG),
            "MAJ" => Ok(Self::MAJ),
            // 독립언
            "IC" => Ok(Self::IC),
            // 조사
            "JKS" => Ok(Self::JKS),
            "JKC" => Ok(Self::JKC),
            "JKG" => Ok(Self::JKG),
            "JKO" => Ok(Self::JKO),
            "JKB" => Ok(Self::JKB),
            "JKV" => Ok(Self::JKV),
            "JKQ" => Ok(Self::JKQ),
            "JX" => Ok(Self::JX),
            "JC" => Ok(Self::JC),
            // 어미
            "EP" => Ok(Self::EP),
            "EF" => Ok(Self::EF),
            "EC" => Ok(Self::EC),
            "ETN" => Ok(Self::ETN),
            "ETM" => Ok(Self::ETM),
            // 접사
            "XPN" => Ok(Self::XPN),
            "XSN" => Ok(Self::XSN),
            "XSV" => Ok(Self::XSV),
            "XSA" => Ok(Self::XSA),
            "XR" => Ok(Self::XR),
            // 기호
            "SF" => Ok(Self::SF),
            "SP" => Ok(Self::SP),
            "SSO" => Ok(Self::SSO),
            "SSC" => Ok(Self::SSC),
            "SC" => Ok(Self::SC),
            "SE" => Ok(Self::SE),
            "SY" => Ok(Self::SY),
            "SL" => Ok(Self::SL),
            "SH" => Ok(Self::SH),
            "SN" => Ok(Self::SN),
            // 특수
            "UNKNOWN" | "UN" => Ok(Self::Unknown),
            _ => Err(ParsePosTagError {
                invalid_tag: s.to_string(),
            }),
        }
    }
}

impl PosTag {
    /// 품사 태그 문자열 반환
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            // 체언
            Self::NNG => "NNG",
            Self::NNP => "NNP",
            Self::NNB => "NNB",
            Self::NNBC => "NNBC",
            Self::NP => "NP",
            Self::NR => "NR",
            // 용언
            Self::VV => "VV",
            Self::VA => "VA",
            Self::VX => "VX",
            Self::VCP => "VCP",
            Self::VCN => "VCN",
            // 수식언
            Self::MM => "MM",
            Self::MAG => "MAG",
            Self::MAJ => "MAJ",
            // 독립언
            Self::IC => "IC",
            // 조사
            Self::JKS => "JKS",
            Self::JKC => "JKC",
            Self::JKG => "JKG",
            Self::JKO => "JKO",
            Self::JKB => "JKB",
            Self::JKV => "JKV",
            Self::JKQ => "JKQ",
            Self::JX => "JX",
            Self::JC => "JC",
            // 어미
            Self::EP => "EP",
            Self::EF => "EF",
            Self::EC => "EC",
            Self::ETN => "ETN",
            Self::ETM => "ETM",
            // 접사
            Self::XPN => "XPN",
            Self::XSN => "XSN",
            Self::XSV => "XSV",
            Self::XSA => "XSA",
            Self::XR => "XR",
            // 기호
            Self::SF => "SF",
            Self::SP => "SP",
            Self::SSO => "SSO",
            Self::SSC => "SSC",
            Self::SC => "SC",
            Self::SE => "SE",
            Self::SY => "SY",
            Self::SL => "SL",
            Self::SH => "SH",
            Self::SN => "SN",
            // 특수
            Self::Unknown => "UNKNOWN",
        }
    }

    /// 한글 명칭 반환
    #[must_use]
    pub const fn korean_name(&self) -> &'static str {
        match self {
            // 체언
            Self::NNG => "일반 명사",
            Self::NNP => "고유 명사",
            Self::NNB => "의존 명사",
            Self::NNBC => "단위 의존 명사",
            Self::NP => "대명사",
            Self::NR => "수사",
            // 용언
            Self::VV => "동사",
            Self::VA => "형용사",
            Self::VX => "보조 용언",
            Self::VCP => "긍정 지정사",
            Self::VCN => "부정 지정사",
            // 수식언
            Self::MM => "관형사",
            Self::MAG => "일반 부사",
            Self::MAJ => "접속 부사",
            // 독립언
            Self::IC => "감탄사",
            // 조사
            Self::JKS => "주격 조사",
            Self::JKC => "보격 조사",
            Self::JKG => "관형격 조사",
            Self::JKO => "목적격 조사",
            Self::JKB => "부사격 조사",
            Self::JKV => "호격 조사",
            Self::JKQ => "인용격 조사",
            Self::JX => "보조사",
            Self::JC => "접속 조사",
            // 어미
            Self::EP => "선어말 어미",
            Self::EF => "종결 어미",
            Self::EC => "연결 어미",
            Self::ETN => "명사형 전성 어미",
            Self::ETM => "관형형 전성 어미",
            // 접사
            Self::XPN => "체언 접두사",
            Self::XSN => "명사파생 접미사",
            Self::XSV => "동사파생 접미사",
            Self::XSA => "형용사파생 접미사",
            Self::XR => "어근",
            // 기호
            Self::SF => "마침 부호",
            Self::SP => "공백",
            Self::SSO => "여는 괄호",
            Self::SSC => "닫는 괄호",
            Self::SC => "구분자",
            Self::SE => "줄임표",
            Self::SY => "기타 기호",
            Self::SL => "외국어",
            Self::SH => "한자",
            Self::SN => "숫자",
            // 특수
            Self::Unknown => "미등록어",
        }
    }

    /// 품사 대분류 반환
    #[must_use]
    pub const fn category(&self) -> PosCategory {
        match self {
            Self::NNG | Self::NNP | Self::NNB | Self::NNBC | Self::NP | Self::NR => {
                PosCategory::Nominal
            }
            Self::VV | Self::VA | Self::VX | Self::VCP | Self::VCN => PosCategory::Predicate,
            Self::MM | Self::MAG | Self::MAJ => PosCategory::Modifier,
            Self::IC => PosCategory::Interjection,
            Self::JKS
            | Self::JKC
            | Self::JKG
            | Self::JKO
            | Self::JKB
            | Self::JKV
            | Self::JKQ
            | Self::JX
            | Self::JC => PosCategory::Particle,
            Self::EP | Self::EF | Self::EC | Self::ETN | Self::ETM => PosCategory::Ending,
            Self::XPN | Self::XSN | Self::XSV | Self::XSA | Self::XR => PosCategory::Affix,
            Self::SF
            | Self::SP
            | Self::SSO
            | Self::SSC
            | Self::SC
            | Self::SE
            | Self::SY
            | Self::SL
            | Self::SH
            | Self::SN => PosCategory::Symbol,
            Self::Unknown => PosCategory::Unknown,
        }
    }

    /// 내용어 여부 (검색 인덱싱에 유용)
    ///
    /// 명사, 동사, 형용사, 어근 등 의미를 담고 있는 형태소
    #[must_use]
    pub const fn is_content_word(&self) -> bool {
        matches!(
            self,
            Self::NNG | Self::NNP | Self::NNB | Self::NNBC | Self::VV | Self::VA | Self::XR
        )
    }

    /// 기능어 여부
    ///
    /// 조사, 어미, 접사 등 문법적 기능을 담당하는 형태소
    #[must_use]
    pub const fn is_function_word(&self) -> bool {
        matches!(self.category(), PosCategory::Particle | PosCategory::Ending)
    }

    /// 명사 여부
    #[must_use]
    pub const fn is_noun(&self) -> bool {
        matches!(
            self,
            Self::NNG | Self::NNP | Self::NNB | Self::NNBC | Self::NP | Self::NR
        )
    }

    /// 용언 여부 (동사/형용사)
    #[must_use]
    pub const fn is_predicate(&self) -> bool {
        matches!(self, Self::VV | Self::VA | Self::VX | Self::VCP | Self::VCN)
    }

    /// 조사 여부
    #[must_use]
    pub const fn is_particle(&self) -> bool {
        matches!(
            self,
            Self::JKS
                | Self::JKC
                | Self::JKG
                | Self::JKO
                | Self::JKB
                | Self::JKV
                | Self::JKQ
                | Self::JX
                | Self::JC
        )
    }

    /// 어미 여부
    #[must_use]
    pub const fn is_ending(&self) -> bool {
        matches!(self, Self::EP | Self::EF | Self::EC | Self::ETN | Self::ETM)
    }

    /// 기호 여부
    #[must_use]
    pub const fn is_symbol(&self) -> bool {
        matches!(
            self,
            Self::SF
                | Self::SP
                | Self::SSO
                | Self::SSC
                | Self::SC
                | Self::SE
                | Self::SY
                | Self::SL
                | Self::SH
                | Self::SN
        )
    }

    /// Nori 호환 태그로 변환 (조사/어미 통합)
    #[must_use]
    pub const fn to_nori_compat(&self) -> NoriTag {
        match self {
            // 조사 통합 → J
            Self::JKS
            | Self::JKC
            | Self::JKG
            | Self::JKO
            | Self::JKB
            | Self::JKV
            | Self::JKQ
            | Self::JX
            | Self::JC => NoriTag::J,
            // 어미 통합 → E
            Self::EP | Self::EF | Self::EC | Self::ETN | Self::ETM => NoriTag::E,
            // 나머지는 동일
            _ => NoriTag::Same(*self),
        }
    }

    /// 세종 기본 태그로 정규화
    #[must_use]
    pub const fn to_sejong_base(&self) -> Self {
        match self {
            // NNBC → NNB (단위명사 → 의존명사)
            Self::NNBC => Self::NNB,
            // SSO/SSC → 그대로 (세종에서는 SS이지만 분석 정확도를 위해 유지)
            _ => *self,
        }
    }

    /// 모든 태그 목록 반환
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            // 체언
            Self::NNG,
            Self::NNP,
            Self::NNB,
            Self::NNBC,
            Self::NP,
            Self::NR,
            // 용언
            Self::VV,
            Self::VA,
            Self::VX,
            Self::VCP,
            Self::VCN,
            // 수식언
            Self::MM,
            Self::MAG,
            Self::MAJ,
            // 독립언
            Self::IC,
            // 조사
            Self::JKS,
            Self::JKC,
            Self::JKG,
            Self::JKO,
            Self::JKB,
            Self::JKV,
            Self::JKQ,
            Self::JX,
            Self::JC,
            // 어미
            Self::EP,
            Self::EF,
            Self::EC,
            Self::ETN,
            Self::ETM,
            // 접사
            Self::XPN,
            Self::XSN,
            Self::XSV,
            Self::XSA,
            Self::XR,
            // 기호
            Self::SF,
            Self::SP,
            Self::SSO,
            Self::SSC,
            Self::SC,
            Self::SE,
            Self::SY,
            Self::SL,
            Self::SH,
            Self::SN,
            // 특수
            Self::Unknown,
        ]
    }
}

impl fmt::Display for PosTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 품사 대분류
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PosCategory {
    /// 체언 (명사, 대명사, 수사)
    Nominal,
    /// 용언 (동사, 형용사, 보조용언, 지정사)
    Predicate,
    /// 수식언 (관형사, 부사)
    Modifier,
    /// 독립언 (감탄사)
    Interjection,
    /// 관계언 (조사)
    Particle,
    /// 어미
    Ending,
    /// 접사
    Affix,
    /// 기호
    Symbol,
    /// 미상
    Unknown,
}

impl PosCategory {
    /// 대분류 한글 명칭
    #[must_use]
    pub const fn korean_name(&self) -> &'static str {
        match self {
            Self::Nominal => "체언",
            Self::Predicate => "용언",
            Self::Modifier => "수식언",
            Self::Interjection => "독립언",
            Self::Particle => "관계언",
            Self::Ending => "어미",
            Self::Affix => "접사",
            Self::Symbol => "기호",
            Self::Unknown => "미상",
        }
    }
}

/// Nori 호환 태그 (Elasticsearch)
///
/// Nori는 조사와 어미를 각각 J, E로 통합
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoriTag {
    /// 조사 통합 태그
    J,
    /// 어미 통합 태그
    E,
    /// 기존 태그와 동일
    Same(PosTag),
}

impl NoriTag {
    /// 문자열 표현
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::J => "J",
            Self::E => "E",
            Self::Same(tag) => tag.as_str(),
        }
    }
}

impl fmt::Display for NoriTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(PosTag::from_str("NNG"), Ok(PosTag::NNG));
        assert_eq!(PosTag::from_str("VV"), Ok(PosTag::VV));
        assert_eq!(PosTag::from_str("JKS"), Ok(PosTag::JKS));
        assert_eq!(PosTag::from_str("UNKNOWN"), Ok(PosTag::Unknown));
        assert!(PosTag::from_str("INVALID").is_err());
    }

    #[test]
    fn test_as_str() {
        assert_eq!(PosTag::NNG.as_str(), "NNG");
        assert_eq!(PosTag::VV.as_str(), "VV");
        assert_eq!(PosTag::Unknown.as_str(), "UNKNOWN");
    }

    #[test]
    fn test_category() {
        assert_eq!(PosTag::NNG.category(), PosCategory::Nominal);
        assert_eq!(PosTag::VV.category(), PosCategory::Predicate);
        assert_eq!(PosTag::JKS.category(), PosCategory::Particle);
        assert_eq!(PosTag::EF.category(), PosCategory::Ending);
    }

    #[test]
    fn test_is_content_word() {
        assert!(PosTag::NNG.is_content_word());
        assert!(PosTag::VV.is_content_word());
        assert!(!PosTag::JKS.is_content_word());
        assert!(!PosTag::EF.is_content_word());
    }

    #[test]
    fn test_nori_compat() {
        assert_eq!(PosTag::JKS.to_nori_compat(), NoriTag::J);
        assert_eq!(PosTag::JKO.to_nori_compat(), NoriTag::J);
        assert_eq!(PosTag::EF.to_nori_compat(), NoriTag::E);
        assert_eq!(PosTag::EC.to_nori_compat(), NoriTag::E);
        assert!(matches!(PosTag::NNG.to_nori_compat(), NoriTag::Same(_)));
    }

    #[test]
    fn test_all_tags_count() {
        // 체언(6) + 용언(5) + 수식언(3) + 독립언(1) + 조사(9) + 어미(5) + 접사(5) + 기호(10) + 특수(1) = 45
        assert_eq!(PosTag::all().len(), 45);
    }
}
