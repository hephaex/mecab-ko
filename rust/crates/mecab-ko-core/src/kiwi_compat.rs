//! Kiwi 형태소 분석기 호환 레이어
//!
//! 이 모듈은 Kiwi 형태소 분석기와의 상호 운용성을 제공합니다.
//! MeCab-Ko와 Kiwi 간 품사 태그 변환 및 출력 형식 호환 기능을 포함합니다.
//!
//! # Kiwi 소개
//!
//! Kiwi(Korean Intelligent Word Identifier)는 C++로 작성된 고성능 한국어 형태소 분석기입니다.
//! 세종 품사 태그 체계를 기반으로 하며, 일부 확장 태그를 포함합니다.
//!
//! # 품사 태그 매핑
//!
//! MeCab-Ko와 Kiwi는 대부분의 품사 태그를 공유하지만, 일부 차이점이 있습니다:
//!
//! - MeCab-Ko의 `NNBC` (단위 의존 명사)는 Kiwi에서 `NNB`로 통합
//! - MeCab-Ko의 `SSO`/`SSC` (여는/닫는 괄호)는 Kiwi에서 `SS`로 통합
//! - MeCab-Ko의 `SC` (구분자)는 Kiwi에서 `SP`로 매핑
//! - MeCab-Ko의 `SY` (기타 기호)는 Kiwi에서 `SO`로 매핑
//! - Kiwi의 웹 관련 태그 (`W_URL`, `W_EMAIL` 등)는 MeCab-Ko의 `SL`로 매핑
//!
//! # Example
//!
//! ```
//! use mecab_ko_core::kiwi_compat::{KiwiPosTag, to_kiwi_tag, from_kiwi_tag};
//! use mecab_ko_core::pos_tag::PosTag;
//!
//! // MeCab -> Kiwi 변환
//! let kiwi_tag = to_kiwi_tag(PosTag::NNG);
//! assert_eq!(kiwi_tag, KiwiPosTag::NNG);
//!
//! // Kiwi -> MeCab 변환
//! let mecab_tag = from_kiwi_tag(KiwiPosTag::NNG);
//! assert_eq!(mecab_tag, PosTag::NNG);
//!
//! // 문자열 파싱
//! let tag = KiwiPosTag::from_str("NNG").unwrap();
//! assert_eq!(tag.as_str(), "NNG");
//! ```

use crate::pos_tag::PosTag;
use std::fmt;

/// Kiwi 품사 태그
///
/// Kiwi 형태소 분석기에서 사용하는 품사 태그 체계
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KiwiPosTag {
    // ============================================
    // 체언 (Nominals)
    // ============================================
    /// 일반 명사 (General Noun)
    NNG,
    /// 고유 명사 (Proper Noun)
    NNP,
    /// 의존 명사 (Dependent Noun)
    NNB,
    /// 수사 (Numeral)
    NR,
    /// 대명사 (Pronoun)
    NP,

    // ============================================
    // 용언 (Predicates)
    // ============================================
    /// 동사 (Verb)
    VV,
    /// 형용사 (Adjective)
    VA,
    /// 보조 용언 (Auxiliary Predicate)
    VX,
    /// 긍정 지정사 (Positive Copula)
    VCP,
    /// 부정 지정사 (Negative Copula)
    VCN,

    // ============================================
    // 수식언 (Modifiers)
    // ============================================
    /// 관형사 (Determiner)
    MM,
    /// 일반 부사 (General Adverb)
    MAG,
    /// 접속 부사 (Conjunctive Adverb)
    MAJ,

    // ============================================
    // 독립언 (Interjection)
    // ============================================
    /// 감탄사 (Interjection)
    IC,

    // ============================================
    // 관계언 - 조사 (Particles)
    // ============================================
    /// 주격 조사 (Subjective Case Marker)
    JKS,
    /// 보격 조사 (Complement Case Marker)
    JKC,
    /// 관형격 조사 (Genitive Case Marker)
    JKG,
    /// 목적격 조사 (Objective Case Marker)
    JKO,
    /// 부사격 조사 (Adverbial Case Marker)
    JKB,
    /// 호격 조사 (Vocative Case Marker)
    JKV,
    /// 인용격 조사 (Quotative Case Marker)
    JKQ,
    /// 보조사 (Auxiliary Particle)
    JX,
    /// 접속 조사 (Conjunctive Particle)
    JC,

    // ============================================
    // 어미 (Endings)
    // ============================================
    /// 선어말 어미 (Pre-final Ending)
    EP,
    /// 종결 어미 (Final Ending)
    EF,
    /// 연결 어미 (Connective Ending)
    EC,
    /// 명사형 전성 어미 (Nominal Transformative Ending)
    ETN,
    /// 관형형 전성 어미 (Adnominal Transformative Ending)
    ETM,

    // ============================================
    // 접사 (Affixes)
    // ============================================
    /// 체언 접두사 (Noun Prefix)
    XPN,
    /// 명사파생 접미사 (Noun Derivational Suffix)
    XSN,
    /// 동사파생 접미사 (Verb Derivational Suffix)
    XSV,
    /// 형용사파생 접미사 (Adjective Derivational Suffix)
    XSA,
    /// 어근 (Root)
    XR,

    // ============================================
    // 기호 (Symbols)
    // ============================================
    /// 마침 부호 (Terminal Punctuation)
    SF,
    /// 쉼표, 가운뎃점, 콜론, 빗금 (Separator)
    SP,
    /// 따옴표, 괄호 등 (Quote/Bracket)
    SS,
    /// 줄임표 (Ellipsis)
    SE,
    /// 그 외 기호 (Other Symbol)
    SO,
    /// 붙임표(물결,숨김,빠짐) (Wave dash)
    SW,

    /// 외국어 (Foreign Language)
    SL,
    /// 한자 (Hanja/Chinese Character)
    SH,
    /// 숫자 (Number)
    SN,

    // ============================================
    // 웹 관련 (Web-related)
    // ============================================
    /// URL
    #[allow(non_camel_case_types)]
    W_URL,
    /// 이메일
    #[allow(non_camel_case_types)]
    W_EMAIL,
    /// 해시태그
    #[allow(non_camel_case_types)]
    W_HASHTAG,
    /// 멘션
    #[allow(non_camel_case_types)]
    W_MENTION,
    /// 이모티콘
    #[allow(non_camel_case_types)]
    W_EMOJI,
    /// 기타 웹 관련
    #[allow(non_camel_case_types)]
    W_OTHER,

    // ============================================
    // 특수 (Special)
    // ============================================
    /// 미등록어 (Unknown Word)
    Unknown,
}

impl KiwiPosTag {
    /// 문자열에서 Kiwi 품사 태그 파싱
    ///
    /// # Example
    /// ```
    /// use mecab_ko_core::kiwi_compat::KiwiPosTag;
    ///
    /// assert_eq!(KiwiPosTag::from_str("NNG"), Some(KiwiPosTag::NNG));
    /// assert_eq!(KiwiPosTag::from_str("W_URL"), Some(KiwiPosTag::W_URL));
    /// assert_eq!(KiwiPosTag::from_str("INVALID"), None);
    /// ```
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            // 체언
            "NNG" => Some(Self::NNG),
            "NNP" => Some(Self::NNP),
            "NNB" => Some(Self::NNB),
            "NR" => Some(Self::NR),
            "NP" => Some(Self::NP),
            // 용언
            "VV" => Some(Self::VV),
            "VA" => Some(Self::VA),
            "VX" => Some(Self::VX),
            "VCP" => Some(Self::VCP),
            "VCN" => Some(Self::VCN),
            // 수식언
            "MM" => Some(Self::MM),
            "MAG" => Some(Self::MAG),
            "MAJ" => Some(Self::MAJ),
            // 독립언
            "IC" => Some(Self::IC),
            // 조사
            "JKS" => Some(Self::JKS),
            "JKC" => Some(Self::JKC),
            "JKG" => Some(Self::JKG),
            "JKO" => Some(Self::JKO),
            "JKB" => Some(Self::JKB),
            "JKV" => Some(Self::JKV),
            "JKQ" => Some(Self::JKQ),
            "JX" => Some(Self::JX),
            "JC" => Some(Self::JC),
            // 어미
            "EP" => Some(Self::EP),
            "EF" => Some(Self::EF),
            "EC" => Some(Self::EC),
            "ETN" => Some(Self::ETN),
            "ETM" => Some(Self::ETM),
            // 접사
            "XPN" => Some(Self::XPN),
            "XSN" => Some(Self::XSN),
            "XSV" => Some(Self::XSV),
            "XSA" => Some(Self::XSA),
            "XR" => Some(Self::XR),
            // 기호
            "SF" => Some(Self::SF),
            "SP" => Some(Self::SP),
            "SS" => Some(Self::SS),
            "SE" => Some(Self::SE),
            "SO" => Some(Self::SO),
            "SW" => Some(Self::SW),
            "SL" => Some(Self::SL),
            "SH" => Some(Self::SH),
            "SN" => Some(Self::SN),
            // 웹 관련
            "W_URL" => Some(Self::W_URL),
            "W_EMAIL" => Some(Self::W_EMAIL),
            "W_HASHTAG" => Some(Self::W_HASHTAG),
            "W_MENTION" => Some(Self::W_MENTION),
            "W_EMOJI" => Some(Self::W_EMOJI),
            "W_OTHER" => Some(Self::W_OTHER),
            // 특수
            "UNKNOWN" | "UNK" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// Kiwi 품사 태그 문자열 반환
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            // 체언
            Self::NNG => "NNG",
            Self::NNP => "NNP",
            Self::NNB => "NNB",
            Self::NR => "NR",
            Self::NP => "NP",
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
            Self::SS => "SS",
            Self::SE => "SE",
            Self::SO => "SO",
            Self::SW => "SW",
            Self::SL => "SL",
            Self::SH => "SH",
            Self::SN => "SN",
            // 웹 관련
            Self::W_URL => "W_URL",
            Self::W_EMAIL => "W_EMAIL",
            Self::W_HASHTAG => "W_HASHTAG",
            Self::W_MENTION => "W_MENTION",
            Self::W_EMOJI => "W_EMOJI",
            Self::W_OTHER => "W_OTHER",
            // 특수
            Self::Unknown => "UNKNOWN",
        }
    }
}

impl fmt::Display for KiwiPosTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// MeCab-Ko 품사 태그를 Kiwi 품사 태그로 변환
///
/// # Example
/// ```
/// use mecab_ko_core::kiwi_compat::{to_kiwi_tag, KiwiPosTag};
/// use mecab_ko_core::pos_tag::PosTag;
///
/// assert_eq!(to_kiwi_tag(PosTag::NNG), KiwiPosTag::NNG);
/// assert_eq!(to_kiwi_tag(PosTag::NNBC), KiwiPosTag::NNB); // 단위명사 -> 의존명사
/// assert_eq!(to_kiwi_tag(PosTag::SSO), KiwiPosTag::SS); // 여는괄호 -> 따옴표/괄호
/// assert_eq!(to_kiwi_tag(PosTag::SC), KiwiPosTag::SP); // 구분자 -> 쉼표
/// ```
#[must_use]
pub const fn to_kiwi_tag(mecab_tag: PosTag) -> KiwiPosTag {
    match mecab_tag {
        // 체언 - 대부분 1:1 매핑
        PosTag::NNG => KiwiPosTag::NNG,
        PosTag::NNP => KiwiPosTag::NNP,
        PosTag::NNB | PosTag::NNBC => KiwiPosTag::NNB, // 단위명사 -> 의존명사 통합
        PosTag::NP => KiwiPosTag::NP,
        PosTag::NR => KiwiPosTag::NR,

        // 용언 - 1:1 매핑
        PosTag::VV => KiwiPosTag::VV,
        PosTag::VA => KiwiPosTag::VA,
        PosTag::VX => KiwiPosTag::VX,
        PosTag::VCP => KiwiPosTag::VCP,
        PosTag::VCN => KiwiPosTag::VCN,

        // 수식언 - 1:1 매핑
        PosTag::MM => KiwiPosTag::MM,
        PosTag::MAG => KiwiPosTag::MAG,
        PosTag::MAJ => KiwiPosTag::MAJ,

        // 독립언 - 1:1 매핑
        PosTag::IC => KiwiPosTag::IC,

        // 조사 - 1:1 매핑
        PosTag::JKS => KiwiPosTag::JKS,
        PosTag::JKC => KiwiPosTag::JKC,
        PosTag::JKG => KiwiPosTag::JKG,
        PosTag::JKO => KiwiPosTag::JKO,
        PosTag::JKB => KiwiPosTag::JKB,
        PosTag::JKV => KiwiPosTag::JKV,
        PosTag::JKQ => KiwiPosTag::JKQ,
        PosTag::JX => KiwiPosTag::JX,
        PosTag::JC => KiwiPosTag::JC,

        // 어미 - 1:1 매핑
        PosTag::EP => KiwiPosTag::EP,
        PosTag::EF => KiwiPosTag::EF,
        PosTag::EC => KiwiPosTag::EC,
        PosTag::ETN => KiwiPosTag::ETN,
        PosTag::ETM => KiwiPosTag::ETM,

        // 접사 - 1:1 매핑
        PosTag::XPN => KiwiPosTag::XPN,
        PosTag::XSN => KiwiPosTag::XSN,
        PosTag::XSV => KiwiPosTag::XSV,
        PosTag::XSA => KiwiPosTag::XSA,
        PosTag::XR => KiwiPosTag::XR,

        // 기호 - 일부 통합
        PosTag::SF => KiwiPosTag::SF,
        PosTag::SP | PosTag::SC => KiwiPosTag::SP, // 구분자 -> 쉼표 통합
        PosTag::SSO | PosTag::SSC => KiwiPosTag::SS, // 여는/닫는괄호 -> 따옴표/괄호 통합
        PosTag::SE => KiwiPosTag::SE,
        PosTag::SY => KiwiPosTag::SO, // 기타기호 -> 그외기호
        PosTag::SL => KiwiPosTag::SL,
        PosTag::SH => KiwiPosTag::SH,
        PosTag::SN => KiwiPosTag::SN,

        // 특수
        PosTag::Unknown => KiwiPosTag::Unknown,
    }
}

/// Kiwi 품사 태그를 MeCab-Ko 품사 태그로 변환
///
/// Kiwi의 웹 관련 태그는 MeCab-Ko의 SL (외국어)로 매핑됩니다.
///
/// # Example
/// ```
/// use mecab_ko_core::kiwi_compat::{from_kiwi_tag, KiwiPosTag};
/// use mecab_ko_core::pos_tag::PosTag;
///
/// assert_eq!(from_kiwi_tag(KiwiPosTag::NNG), PosTag::NNG);
/// assert_eq!(from_kiwi_tag(KiwiPosTag::SS), PosTag::SSO); // 괄호 -> 여는괄호 (기본값)
/// assert_eq!(from_kiwi_tag(KiwiPosTag::W_URL), PosTag::SL); // 웹 태그 -> 외국어
/// ```
#[must_use]
pub const fn from_kiwi_tag(kiwi_tag: KiwiPosTag) -> PosTag {
    match kiwi_tag {
        // 체언 - 1:1 매핑
        KiwiPosTag::NNG => PosTag::NNG,
        KiwiPosTag::NNP => PosTag::NNP,
        KiwiPosTag::NNB => PosTag::NNB, // Kiwi NNB -> MeCab NNB (NNBC는 손실)
        KiwiPosTag::NP => PosTag::NP,
        KiwiPosTag::NR => PosTag::NR,

        // 용언 - 1:1 매핑
        KiwiPosTag::VV => PosTag::VV,
        KiwiPosTag::VA => PosTag::VA,
        KiwiPosTag::VX => PosTag::VX,
        KiwiPosTag::VCP => PosTag::VCP,
        KiwiPosTag::VCN => PosTag::VCN,

        // 수식언 - 1:1 매핑
        KiwiPosTag::MM => PosTag::MM,
        KiwiPosTag::MAG => PosTag::MAG,
        KiwiPosTag::MAJ => PosTag::MAJ,

        // 독립언 - 1:1 매핑
        KiwiPosTag::IC => PosTag::IC,

        // 조사 - 1:1 매핑
        KiwiPosTag::JKS => PosTag::JKS,
        KiwiPosTag::JKC => PosTag::JKC,
        KiwiPosTag::JKG => PosTag::JKG,
        KiwiPosTag::JKO => PosTag::JKO,
        KiwiPosTag::JKB => PosTag::JKB,
        KiwiPosTag::JKV => PosTag::JKV,
        KiwiPosTag::JKQ => PosTag::JKQ,
        KiwiPosTag::JX => PosTag::JX,
        KiwiPosTag::JC => PosTag::JC,

        // 어미 - 1:1 매핑
        KiwiPosTag::EP => PosTag::EP,
        KiwiPosTag::EF => PosTag::EF,
        KiwiPosTag::EC => PosTag::EC,
        KiwiPosTag::ETN => PosTag::ETN,
        KiwiPosTag::ETM => PosTag::ETM,

        // 접사 - 1:1 매핑
        KiwiPosTag::XPN => PosTag::XPN,
        KiwiPosTag::XSN => PosTag::XSN,
        KiwiPosTag::XSV => PosTag::XSV,
        KiwiPosTag::XSA => PosTag::XSA,
        KiwiPosTag::XR => PosTag::XR,

        // 기호 - 역변환 시 정보 손실 가능
        KiwiPosTag::SF => PosTag::SF,
        KiwiPosTag::SP => PosTag::SP, // SP는 SC도 포함할 수 있음 (손실)
        KiwiPosTag::SS => PosTag::SSO, // SS -> SSO (기본값, SSC는 손실)
        KiwiPosTag::SE => PosTag::SE,
        KiwiPosTag::SO | KiwiPosTag::SW => PosTag::SY, // SO, SW -> SY (붙임표를 기타기호로)
        KiwiPosTag::SL
        | KiwiPosTag::W_URL
        | KiwiPosTag::W_EMAIL
        | KiwiPosTag::W_HASHTAG
        | KiwiPosTag::W_MENTION
        | KiwiPosTag::W_EMOJI
        | KiwiPosTag::W_OTHER => PosTag::SL, // 웹 관련 - SL (외국어)로 통합
        KiwiPosTag::SH => PosTag::SH,
        KiwiPosTag::SN => PosTag::SN,

        // 특수
        KiwiPosTag::Unknown => PosTag::Unknown,
    }
}

/// Kiwi 호환 토큰 구조체
///
/// Kiwi 형태소 분석 결과와 호환되는 출력 형식
///
/// # Example
/// ```
/// use mecab_ko_core::kiwi_compat::{KiwiToken, KiwiPosTag};
///
/// let token = KiwiToken {
///     form: "안녕".to_string(),
///     tag: KiwiPosTag::NNG,
///     start: 0,
///     length: 6, // UTF-8 바이트 길이
///     score: -10.5,
/// };
///
/// assert_eq!(token.form, "안녕");
/// assert_eq!(token.tag, KiwiPosTag::NNG);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct KiwiToken {
    /// 형태소 표면형
    pub form: String,
    /// 품사 태그
    pub tag: KiwiPosTag,
    /// 시작 위치 (바이트 오프셋)
    pub start: usize,
    /// 길이 (바이트)
    pub length: usize,
    /// 분석 점수 (로그 확률)
    pub score: f64,
}

impl KiwiToken {
    /// 새 토큰 생성
    ///
    /// # Example
    /// ```
    /// use mecab_ko_core::kiwi_compat::{KiwiToken, KiwiPosTag};
    ///
    /// let token = KiwiToken::new("하다", KiwiPosTag::VV, 0, 6, -5.2);
    /// assert_eq!(token.form, "하다");
    /// ```
    pub fn new(
        form: impl Into<String>,
        tag: KiwiPosTag,
        start: usize,
        length: usize,
        score: f64,
    ) -> Self {
        Self {
            form: form.into(),
            tag,
            start,
            length,
            score,
        }
    }

    /// 끝 위치 계산 (start + length)
    #[must_use]
    pub const fn end(&self) -> usize {
        self.start + self.length
    }

    /// MeCab-Ko 품사 태그로 변환
    #[must_use]
    pub const fn to_mecab_tag(&self) -> PosTag {
        from_kiwi_tag(self.tag)
    }
}

impl fmt::Display for KiwiToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.form, self.tag)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_kiwi_tag_from_str() {
        assert_eq!(KiwiPosTag::from_str("NNG"), Some(KiwiPosTag::NNG));
        assert_eq!(KiwiPosTag::from_str("VV"), Some(KiwiPosTag::VV));
        assert_eq!(KiwiPosTag::from_str("W_URL"), Some(KiwiPosTag::W_URL));
        assert_eq!(KiwiPosTag::from_str("UNKNOWN"), Some(KiwiPosTag::Unknown));
        assert_eq!(KiwiPosTag::from_str("INVALID"), None);
    }

    #[test]
    fn test_kiwi_tag_as_str() {
        assert_eq!(KiwiPosTag::NNG.as_str(), "NNG");
        assert_eq!(KiwiPosTag::W_URL.as_str(), "W_URL");
        assert_eq!(KiwiPosTag::Unknown.as_str(), "UNKNOWN");
    }

    #[test]
    fn test_to_kiwi_tag_nominals() {
        // 체언
        assert_eq!(to_kiwi_tag(PosTag::NNG), KiwiPosTag::NNG);
        assert_eq!(to_kiwi_tag(PosTag::NNP), KiwiPosTag::NNP);
        assert_eq!(to_kiwi_tag(PosTag::NNB), KiwiPosTag::NNB);
        assert_eq!(to_kiwi_tag(PosTag::NNBC), KiwiPosTag::NNB); // 단위명사 통합
        assert_eq!(to_kiwi_tag(PosTag::NP), KiwiPosTag::NP);
        assert_eq!(to_kiwi_tag(PosTag::NR), KiwiPosTag::NR);
    }

    #[test]
    fn test_to_kiwi_tag_predicates() {
        // 용언
        assert_eq!(to_kiwi_tag(PosTag::VV), KiwiPosTag::VV);
        assert_eq!(to_kiwi_tag(PosTag::VA), KiwiPosTag::VA);
        assert_eq!(to_kiwi_tag(PosTag::VX), KiwiPosTag::VX);
        assert_eq!(to_kiwi_tag(PosTag::VCP), KiwiPosTag::VCP);
        assert_eq!(to_kiwi_tag(PosTag::VCN), KiwiPosTag::VCN);
    }

    #[test]
    fn test_to_kiwi_tag_particles() {
        // 조사
        assert_eq!(to_kiwi_tag(PosTag::JKS), KiwiPosTag::JKS);
        assert_eq!(to_kiwi_tag(PosTag::JKO), KiwiPosTag::JKO);
        assert_eq!(to_kiwi_tag(PosTag::JX), KiwiPosTag::JX);
    }

    #[test]
    fn test_to_kiwi_tag_symbols() {
        // 기호 - 통합 확인
        assert_eq!(to_kiwi_tag(PosTag::SSO), KiwiPosTag::SS); // 여는괄호 -> SS
        assert_eq!(to_kiwi_tag(PosTag::SSC), KiwiPosTag::SS); // 닫는괄호 -> SS
        assert_eq!(to_kiwi_tag(PosTag::SC), KiwiPosTag::SP); // 구분자 -> SP
        assert_eq!(to_kiwi_tag(PosTag::SY), KiwiPosTag::SO); // 기타기호 -> SO
    }

    #[test]
    fn test_from_kiwi_tag_nominals() {
        // 체언
        assert_eq!(from_kiwi_tag(KiwiPosTag::NNG), PosTag::NNG);
        assert_eq!(from_kiwi_tag(KiwiPosTag::NNP), PosTag::NNP);
        assert_eq!(from_kiwi_tag(KiwiPosTag::NNB), PosTag::NNB); // NNBC 정보 손실
    }

    #[test]
    fn test_from_kiwi_tag_symbols() {
        // 기호 - 역변환 확인
        assert_eq!(from_kiwi_tag(KiwiPosTag::SS), PosTag::SSO); // SS -> SSO (기본값)
        assert_eq!(from_kiwi_tag(KiwiPosTag::SO), PosTag::SY); // SO -> SY
        assert_eq!(from_kiwi_tag(KiwiPosTag::SW), PosTag::SY); // SW -> SY
    }

    #[test]
    fn test_from_kiwi_tag_web() {
        // 웹 관련 태그 -> SL
        assert_eq!(from_kiwi_tag(KiwiPosTag::W_URL), PosTag::SL);
        assert_eq!(from_kiwi_tag(KiwiPosTag::W_EMAIL), PosTag::SL);
        assert_eq!(from_kiwi_tag(KiwiPosTag::W_HASHTAG), PosTag::SL);
        assert_eq!(from_kiwi_tag(KiwiPosTag::W_MENTION), PosTag::SL);
        assert_eq!(from_kiwi_tag(KiwiPosTag::W_EMOJI), PosTag::SL);
        assert_eq!(from_kiwi_tag(KiwiPosTag::W_OTHER), PosTag::SL);
    }

    #[test]
    fn test_roundtrip_conversion() {
        // 대부분의 태그는 왕복 변환 가능
        let tags = [
            PosTag::NNG,
            PosTag::VV,
            PosTag::JKS,
            PosTag::EP,
            PosTag::XPN,
            PosTag::SF,
        ];

        for tag in tags {
            let kiwi_tag = to_kiwi_tag(tag);
            let back = from_kiwi_tag(kiwi_tag);
            assert_eq!(tag, back, "Roundtrip failed for {tag:?}");
        }
    }

    #[test]
    fn test_lossy_conversion() {
        // 정보 손실이 있는 변환
        // NNBC -> NNB -> NNB (NNBC 손실)
        assert_eq!(from_kiwi_tag(to_kiwi_tag(PosTag::NNBC)), PosTag::NNB);

        // SSC -> SS -> SSO (SSC 손실)
        assert_eq!(from_kiwi_tag(to_kiwi_tag(PosTag::SSC)), PosTag::SSO);

        // SC -> SP -> SP (SC 손실)
        assert_eq!(from_kiwi_tag(to_kiwi_tag(PosTag::SC)), PosTag::SP);
    }

    #[test]
    fn test_kiwi_token_creation() {
        let token = KiwiToken::new("안녕", KiwiPosTag::NNG, 0, 6, -10.5);
        assert_eq!(token.form, "안녕");
        assert_eq!(token.tag, KiwiPosTag::NNG);
        assert_eq!(token.start, 0);
        assert_eq!(token.length, 6);
        assert_eq!(token.score, -10.5);
        assert_eq!(token.end(), 6);
    }

    #[test]
    fn test_kiwi_token_display() {
        let token = KiwiToken::new("하다", KiwiPosTag::VV, 0, 6, -5.0);
        assert_eq!(token.to_string(), "하다/VV");
    }

    #[test]
    fn test_kiwi_token_to_mecab() {
        let token = KiwiToken::new("것", KiwiPosTag::NNB, 0, 3, -8.2);
        assert_eq!(token.to_mecab_tag(), PosTag::NNB);

        let url_token = KiwiToken::new("http://example.com", KiwiPosTag::W_URL, 0, 18, -15.0);
        assert_eq!(url_token.to_mecab_tag(), PosTag::SL);
    }

    #[test]
    fn test_all_kiwi_tags_covered() {
        // 모든 Kiwi 태그가 MeCab 태그로 변환 가능한지 확인
        let kiwi_tags = [
            KiwiPosTag::NNG,
            KiwiPosTag::NNP,
            KiwiPosTag::NNB,
            KiwiPosTag::NR,
            KiwiPosTag::NP,
            KiwiPosTag::VV,
            KiwiPosTag::VA,
            KiwiPosTag::VX,
            KiwiPosTag::VCP,
            KiwiPosTag::VCN,
            KiwiPosTag::MM,
            KiwiPosTag::MAG,
            KiwiPosTag::MAJ,
            KiwiPosTag::IC,
            KiwiPosTag::JKS,
            KiwiPosTag::JKC,
            KiwiPosTag::JKG,
            KiwiPosTag::JKO,
            KiwiPosTag::JKB,
            KiwiPosTag::JKV,
            KiwiPosTag::JKQ,
            KiwiPosTag::JX,
            KiwiPosTag::JC,
            KiwiPosTag::EP,
            KiwiPosTag::EF,
            KiwiPosTag::EC,
            KiwiPosTag::ETN,
            KiwiPosTag::ETM,
            KiwiPosTag::XPN,
            KiwiPosTag::XSN,
            KiwiPosTag::XSV,
            KiwiPosTag::XSA,
            KiwiPosTag::XR,
            KiwiPosTag::SF,
            KiwiPosTag::SP,
            KiwiPosTag::SS,
            KiwiPosTag::SE,
            KiwiPosTag::SO,
            KiwiPosTag::SW,
            KiwiPosTag::SL,
            KiwiPosTag::SH,
            KiwiPosTag::SN,
            KiwiPosTag::W_URL,
            KiwiPosTag::W_EMAIL,
            KiwiPosTag::W_HASHTAG,
            KiwiPosTag::W_MENTION,
            KiwiPosTag::W_EMOJI,
            KiwiPosTag::W_OTHER,
            KiwiPosTag::Unknown,
        ];

        for tag in kiwi_tags {
            let mecab_tag = from_kiwi_tag(tag);
            // 변환 결과가 유효한지만 확인 (구체적인 매핑은 위 테스트에서 검증)
            assert_ne!(mecab_tag.as_str(), "", "Conversion failed for {tag:?}");
        }
    }

    #[test]
    fn test_all_mecab_tags_covered() {
        // 모든 MeCab 태그가 Kiwi 태그로 변환 가능한지 확인
        for tag in PosTag::all() {
            let kiwi_tag = to_kiwi_tag(*tag);
            // 변환 결과가 유효한지만 확인
            assert_ne!(kiwi_tag.as_str(), "", "Conversion failed for {tag:?}");
        }
    }
}
