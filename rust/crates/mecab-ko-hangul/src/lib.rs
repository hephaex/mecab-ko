//! # mecab-ko-hangul
//!
//! 한글 자소(Jamo) 처리를 위한 유틸리티 라이브러리입니다.
//!
//! ## Features
//!
//! - 한글 자모 분리/결합
//! - 초/중/종성 추출
//! - 한글 판별 함수
//! - 종성 유무 판별
//!
//! ## Example
//!
//! ```rust
//! use mecab_ko_hangul::{decompose, compose, is_hangul, has_jongseong};
//!
//! // 자모 분리
//! let (cho, jung, jong) = decompose('한').unwrap();
//! assert_eq!(cho, 'ㅎ');
//! assert_eq!(jung, 'ㅏ');
//! assert_eq!(jong, Some('ㄴ'));
//!
//! // 자모 결합
//! let c = compose('ㅎ', 'ㅏ', Some('ㄴ')).unwrap();
//! assert_eq!(c, '한');
//!
//! // 한글 판별
//! assert!(is_hangul('가'));
//! assert!(!is_hangul('a'));
//!
//! // 종성 판별
//! assert_eq!(has_jongseong('한'), Some(true));
//! assert_eq!(has_jongseong('하'), Some(false));
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

/// 한글 음절의 시작 코드포인트 (가)
const HANGUL_BASE: u32 = 0xAC00;

/// 한글 음절의 끝 코드포인트 (힣)
const HANGUL_END: u32 = 0xD7A3;

/// 초성 개수 (19개)
#[allow(dead_code)]
const CHOSEONG_COUNT: u32 = 19;

/// 중성 개수 (21개)
const JUNGSEONG_COUNT: u32 = 21;

/// 종성 개수 (28개, 종성 없음 포함)
const JONGSEONG_COUNT: u32 = 28;

/// 초성 목록 (19개)
const CHOSEONG_LIST: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
];

/// 중성 목록 (21개)
const JUNGSEONG_LIST: [char; 21] = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ',
    'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];

/// 종성 목록 (28개, 첫 번째는 종성 없음)
const JONGSEONG_LIST: [Option<char>; 28] = [
    None,
    Some('ㄱ'),
    Some('ㄲ'),
    Some('ㄳ'),
    Some('ㄴ'),
    Some('ㄵ'),
    Some('ㄶ'),
    Some('ㄷ'),
    Some('ㄹ'),
    Some('ㄺ'),
    Some('ㄻ'),
    Some('ㄼ'),
    Some('ㄽ'),
    Some('ㄾ'),
    Some('ㄿ'),
    Some('ㅀ'),
    Some('ㅁ'),
    Some('ㅂ'),
    Some('ㅄ'),
    Some('ㅅ'),
    Some('ㅆ'),
    Some('ㅇ'),
    Some('ㅈ'),
    Some('ㅊ'),
    Some('ㅋ'),
    Some('ㅌ'),
    Some('ㅍ'),
    Some('ㅎ'),
];

/// 주어진 문자가 한글 음절인지 확인합니다.
///
/// # Arguments
///
/// * `c` - 확인할 문자
///
/// # Returns
///
/// 한글 음절이면 `true`, 아니면 `false`
///
/// # Example
///
/// ```rust
/// use mecab_ko_hangul::is_hangul_syllable;
///
/// assert!(is_hangul_syllable('가'));
/// assert!(is_hangul_syllable('힣'));
/// assert!(!is_hangul_syllable('ㄱ')); // 자모는 false
/// assert!(!is_hangul_syllable('a'));
/// ```
#[inline]
#[must_use]
pub fn is_hangul_syllable(c: char) -> bool {
    let code = c as u32;
    (HANGUL_BASE..=HANGUL_END).contains(&code)
}

/// 주어진 문자가 한글(음절 또는 자모)인지 확인합니다.
///
/// # Arguments
///
/// * `c` - 확인할 문자
///
/// # Returns
///
/// 한글이면 `true`, 아니면 `false`
#[inline]
#[must_use]
pub fn is_hangul(c: char) -> bool {
    is_hangul_syllable(c) || is_jamo(c)
}

/// 주어진 문자가 한글 자모인지 확인합니다.
///
/// 호환용 자모(ㄱ-ㅎ, ㅏ-ㅣ) 범위를 확인합니다.
#[inline]
#[must_use]
pub fn is_jamo(c: char) -> bool {
    let code = c as u32;
    // 호환용 자모: ㄱ(0x3131) ~ ㅣ(0x3163)
    (0x3131..=0x3163).contains(&code)
}

/// 주어진 문자가 초성 자모인지 확인합니다.
#[inline]
#[must_use]
pub fn is_choseong(c: char) -> bool {
    CHOSEONG_LIST.contains(&c)
}

/// 주어진 문자가 중성 자모인지 확인합니다.
#[inline]
#[must_use]
pub fn is_jungseong(c: char) -> bool {
    JUNGSEONG_LIST.contains(&c)
}

/// 주어진 한글 음절에 종성이 있는지 확인합니다.
///
/// # Arguments
///
/// * `c` - 확인할 한글 음절
///
/// # Returns
///
/// - `Some(true)`: 종성이 있음
/// - `Some(false)`: 종성이 없음
/// - `None`: 한글 음절이 아님
///
/// # Example
///
/// ```rust
/// use mecab_ko_hangul::has_jongseong;
///
/// assert_eq!(has_jongseong('한'), Some(true));
/// assert_eq!(has_jongseong('하'), Some(false));
/// assert_eq!(has_jongseong('a'), None);
/// ```
#[inline]
#[must_use]
pub fn has_jongseong(c: char) -> Option<bool> {
    if !is_hangul_syllable(c) {
        return None;
    }
    let code = c as u32 - HANGUL_BASE;
    Some(code % JONGSEONG_COUNT != 0)
}

/// 한글 음절을 초성, 중성, 종성으로 분해합니다.
///
/// # Arguments
///
/// * `c` - 분해할 한글 음절
///
/// # Returns
///
/// - `Some((초성, 중성, Option<종성>))`: 분해 성공
/// - `None`: 한글 음절이 아님
///
/// # Example
///
/// ```rust
/// use mecab_ko_hangul::decompose;
///
/// let result = decompose('한');
/// assert_eq!(result, Some(('ㅎ', 'ㅏ', Some('ㄴ'))));
///
/// let result = decompose('가');
/// assert_eq!(result, Some(('ㄱ', 'ㅏ', None)));
/// ```
#[must_use]
#[allow(clippy::similar_names)]
pub fn decompose(c: char) -> Option<(char, char, Option<char>)> {
    if !is_hangul_syllable(c) {
        return None;
    }

    let code = c as u32 - HANGUL_BASE;

    let jongseong_idx = code % JONGSEONG_COUNT;
    let jungseong_idx = ((code - jongseong_idx) / JONGSEONG_COUNT) % JUNGSEONG_COUNT;
    let choseong_idx = ((code - jongseong_idx) / JONGSEONG_COUNT) / JUNGSEONG_COUNT;

    let cho = CHOSEONG_LIST[choseong_idx as usize];
    let jung = JUNGSEONG_LIST[jungseong_idx as usize];
    let jong = JONGSEONG_LIST[jongseong_idx as usize];

    Some((cho, jung, jong))
}

/// 초성, 중성, 종성을 결합하여 한글 음절을 만듭니다.
///
/// # Arguments
///
/// * `cho` - 초성 자모
/// * `jung` - 중성 자모
/// * `jong` - 종성 자모 (없으면 `None`)
///
/// # Returns
///
/// - `Some(한글 음절)`: 결합 성공
/// - `None`: 잘못된 자모
///
/// # Example
///
/// ```rust
/// use mecab_ko_hangul::compose;
///
/// let c = compose('ㅎ', 'ㅏ', Some('ㄴ'));
/// assert_eq!(c, Some('한'));
///
/// let c = compose('ㄱ', 'ㅏ', None);
/// assert_eq!(c, Some('가'));
/// ```
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::similar_names)]
pub fn compose(choseong: char, jungseong: char, jongseong: Option<char>) -> Option<char> {
    let choseong_idx = CHOSEONG_LIST.iter().position(|&c| c == choseong)? as u32;
    let jungseong_idx = JUNGSEONG_LIST.iter().position(|&c| c == jungseong)? as u32;
    let jongseong_idx = match jongseong {
        None => 0,
        Some(j) => JONGSEONG_LIST.iter().position(|&c| c == Some(j))? as u32,
    };

    let code = HANGUL_BASE
        + (choseong_idx * JUNGSEONG_COUNT + jungseong_idx) * JONGSEONG_COUNT
        + jongseong_idx;

    char::from_u32(code)
}

/// 문자열의 모든 한글 음절을 자모로 분해합니다.
///
/// # Arguments
///
/// * `s` - 입력 문자열
///
/// # Returns
///
/// 자모로 분해된 문자열. 한글이 아닌 문자는 그대로 유지됩니다.
///
/// # Example
///
/// ```rust
/// use mecab_ko_hangul::decompose_str;
///
/// assert_eq!(decompose_str("한글"), "ㅎㅏㄴㄱㅡㄹ");
/// assert_eq!(decompose_str("Hello 한글"), "Hello ㅎㅏㄴㄱㅡㄹ");
/// ```
#[must_use]
pub fn decompose_str(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);

    for c in s.chars() {
        if let Some((cho, jung, jong)) = decompose(c) {
            result.push(cho);
            result.push(jung);
            if let Some(j) = jong {
                result.push(j);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// 자모 문자열을 한글 음절로 결합합니다.
///
/// # Arguments
///
/// * `s` - 자모 문자열
///
/// # Returns
///
/// 결합된 문자열. 결합이 불가능한 자모는 그대로 유지됩니다.
///
/// # Example
///
/// ```rust
/// use mecab_ko_hangul::compose_str;
///
/// assert_eq!(compose_str("ㅎㅏㄴㄱㅡㄹ"), "한글");
/// ```
#[must_use]
pub fn compose_str(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::with_capacity(s.len());
    let mut i = 0;

    while i < chars.len() {
        // 초성 + 중성 + (종성) 패턴 시도
        if i + 1 < chars.len() && is_choseong(chars[i]) && is_jungseong(chars[i + 1]) {
            let cho = chars[i];
            let jung = chars[i + 1];

            // 다음 문자가 종성이 될 수 있는지 확인
            // 단, 그 다음에 중성이 오면 종성이 아님
            let jongseong = if i + 2 < chars.len() {
                let potential_jongseong = chars[i + 2];
                let is_potential_jongseong = JONGSEONG_LIST.contains(&Some(potential_jongseong));

                if is_potential_jongseong {
                    // 다음 다음 문자가 중성이면, 현재 문자는 다음 음절의 초성
                    if i + 3 < chars.len() && is_jungseong(chars[i + 3]) {
                        None
                    } else {
                        Some(potential_jongseong)
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(c) = compose(cho, jung, jongseong) {
                result.push(c);
                i += if jongseong.is_some() { 3 } else { 2 };
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// 문자의 종류를 나타내는 열거형
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharType {
    /// 한글 음절
    HangulSyllable,
    /// 한글 자모
    HangulJamo,
    /// 한자 (CJK Unified Ideographs)
    Hanja,
    /// 가타카나
    Katakana,
    /// 히라가나
    Hiragana,
    /// ASCII 알파벳
    Alphabet,
    /// 숫자
    Digit,
    /// 공백 문자
    Whitespace,
    /// 구두점
    Punctuation,
    /// 기타
    Other,
}

/// 문자의 종류를 판별합니다.
///
/// # Arguments
///
/// * `c` - 판별할 문자
///
/// # Returns
///
/// 문자의 종류를 나타내는 `CharType`
#[must_use]
pub fn classify_char(c: char) -> CharType {
    let code = c as u32;

    if is_hangul_syllable(c) {
        CharType::HangulSyllable
    } else if is_jamo(c) {
        CharType::HangulJamo
    } else if (0x4E00..=0x9FFF).contains(&code) || (0x3400..=0x4DBF).contains(&code) {
        CharType::Hanja
    } else if (0x30A0..=0x30FF).contains(&code) {
        CharType::Katakana
    } else if (0x3040..=0x309F).contains(&code) {
        CharType::Hiragana
    } else if c.is_ascii_alphabetic() {
        CharType::Alphabet
    } else if c.is_ascii_digit() {
        CharType::Digit
    } else if c.is_whitespace() {
        CharType::Whitespace
    } else if c.is_ascii_punctuation() {
        CharType::Punctuation
    } else {
        CharType::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hangul_syllable() {
        assert!(is_hangul_syllable('가'));
        assert!(is_hangul_syllable('힣'));
        assert!(is_hangul_syllable('한'));
        assert!(!is_hangul_syllable('ㄱ'));
        assert!(!is_hangul_syllable('a'));
        assert!(!is_hangul_syllable('あ'));
    }

    #[test]
    fn test_is_hangul() {
        assert!(is_hangul('가'));
        assert!(is_hangul('ㄱ'));
        assert!(is_hangul('ㅏ'));
        assert!(!is_hangul('a'));
    }

    #[test]
    fn test_has_jongseong() {
        assert_eq!(has_jongseong('한'), Some(true));
        assert_eq!(has_jongseong('하'), Some(false));
        assert_eq!(has_jongseong('글'), Some(true));
        assert_eq!(has_jongseong('가'), Some(false));
        assert_eq!(has_jongseong('a'), None);
    }

    #[test]
    fn test_decompose() {
        assert_eq!(decompose('가'), Some(('ㄱ', 'ㅏ', None)));
        assert_eq!(decompose('한'), Some(('ㅎ', 'ㅏ', Some('ㄴ'))));
        assert_eq!(decompose('글'), Some(('ㄱ', 'ㅡ', Some('ㄹ'))));
        assert_eq!(decompose('힣'), Some(('ㅎ', 'ㅣ', Some('ㅎ'))));
        assert_eq!(decompose('a'), None);
    }

    #[test]
    fn test_compose() {
        assert_eq!(compose('ㄱ', 'ㅏ', None), Some('가'));
        assert_eq!(compose('ㅎ', 'ㅏ', Some('ㄴ')), Some('한'));
        assert_eq!(compose('ㄱ', 'ㅡ', Some('ㄹ')), Some('글'));
        assert_eq!(compose('ㅎ', 'ㅣ', Some('ㅎ')), Some('힣'));
    }

    #[test]
    fn test_decompose_compose_roundtrip() {
        let test_chars = ['가', '나', '다', '한', '글', '힣', '뷁'];
        for c in test_chars {
            let (cho, jung, jong) = decompose(c).unwrap();
            let result = compose(cho, jung, jong).unwrap();
            assert_eq!(c, result, "Roundtrip failed for '{}'", c);
        }
    }

    #[test]
    fn test_decompose_str() {
        assert_eq!(decompose_str("한글"), "ㅎㅏㄴㄱㅡㄹ");
        assert_eq!(decompose_str("가나다"), "ㄱㅏㄴㅏㄷㅏ");
        assert_eq!(decompose_str("Hello 한글"), "Hello ㅎㅏㄴㄱㅡㄹ");
    }

    #[test]
    fn test_compose_str() {
        assert_eq!(compose_str("ㅎㅏㄴㄱㅡㄹ"), "한글");
        assert_eq!(compose_str("ㄱㅏㄴㅏㄷㅏ"), "가나다");
    }

    #[test]
    fn test_classify_char() {
        assert_eq!(classify_char('한'), CharType::HangulSyllable);
        assert_eq!(classify_char('ㄱ'), CharType::HangulJamo);
        assert_eq!(classify_char('韓'), CharType::Hanja);
        assert_eq!(classify_char('ア'), CharType::Katakana);
        assert_eq!(classify_char('あ'), CharType::Hiragana);
        assert_eq!(classify_char('a'), CharType::Alphabet);
        assert_eq!(classify_char('1'), CharType::Digit);
        assert_eq!(classify_char(' '), CharType::Whitespace);
        assert_eq!(classify_char('.'), CharType::Punctuation);
    }

    #[test]
    fn test_is_jamo() {
        // Test choseong jamo
        assert!(is_jamo('ㄱ'));
        assert!(is_jamo('ㄲ'));
        assert!(is_jamo('ㅎ'));

        // Test jungseong jamo
        assert!(is_jamo('ㅏ'));
        assert!(is_jamo('ㅐ'));
        assert!(is_jamo('ㅣ'));

        // Test complex jongseong jamo
        assert!(is_jamo('ㄳ'));
        assert!(is_jamo('ㄵ'));
        assert!(is_jamo('ㅄ'));

        // Non-jamo characters
        assert!(!is_jamo('가'));
        assert!(!is_jamo('a'));
        assert!(!is_jamo('1'));
        assert!(!is_jamo(' '));
    }

    #[test]
    fn test_is_choseong() {
        // All valid choseong
        assert!(is_choseong('ㄱ'));
        assert!(is_choseong('ㄲ'));
        assert!(is_choseong('ㄴ'));
        assert!(is_choseong('ㄷ'));
        assert!(is_choseong('ㄸ'));
        assert!(is_choseong('ㅎ'));

        // Not choseong
        assert!(!is_choseong('ㅏ')); // jungseong
        assert!(!is_choseong('ㄳ')); // complex jongseong
        assert!(!is_choseong('가')); // syllable
        assert!(!is_choseong('a'));
    }

    #[test]
    fn test_is_jungseong() {
        // All valid jungseong
        assert!(is_jungseong('ㅏ'));
        assert!(is_jungseong('ㅐ'));
        assert!(is_jungseong('ㅘ'));
        assert!(is_jungseong('ㅣ'));

        // Not jungseong
        assert!(!is_jungseong('ㄱ')); // choseong
        assert!(!is_jungseong('ㄳ')); // jongseong
        assert!(!is_jungseong('가')); // syllable
        assert!(!is_jungseong('a'));
    }

    #[test]
    fn test_boundary_hangul_syllables() {
        // First hangul syllable
        assert!(is_hangul_syllable('가'));
        assert_eq!(decompose('가'), Some(('ㄱ', 'ㅏ', None)));

        // Last hangul syllable
        assert!(is_hangul_syllable('힣'));
        assert_eq!(decompose('힣'), Some(('ㅎ', 'ㅣ', Some('ㅎ'))));

        // One before first (should not be hangul syllable)
        let before_first = char::from_u32(0xAC00 - 1).unwrap();
        assert!(!is_hangul_syllable(before_first));

        // One after last (should not be hangul syllable)
        let after_last = char::from_u32(0xD7A3 + 1).unwrap();
        assert!(!is_hangul_syllable(after_last));
    }

    #[test]
    fn test_compose_invalid_jamo() {
        // Invalid choseong
        assert_eq!(compose('ㅏ', 'ㅏ', None), None);

        // Invalid jungseong
        assert_eq!(compose('ㄱ', 'ㄱ', None), None);

        // Invalid jongseong
        assert_eq!(compose('ㄱ', 'ㅏ', Some('ㅏ')), None);

        // Non-jamo characters
        assert_eq!(compose('a', 'ㅏ', None), None);
        assert_eq!(compose('ㄱ', 'b', None), None);
        assert_eq!(compose('ㄱ', 'ㅏ', Some('c')), None);
    }

    #[test]
    fn test_decompose_all_choseong() {
        let expected_choseong = [
            'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ',
            'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
        ];

        for &cho in &expected_choseong {
            let syllable = compose(cho, 'ㅏ', None).unwrap();
            let (decomposed_cho, _, _) = decompose(syllable).unwrap();
            assert_eq!(cho, decomposed_cho, "Choseong mismatch for {syllable}");
        }
    }

    #[test]
    fn test_decompose_all_jungseong() {
        let expected_jungseong = [
            'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ',
            'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
        ];

        for &jung in &expected_jungseong {
            let syllable = compose('ㄱ', jung, None).unwrap();
            let (_, decomposed_jung, _) = decompose(syllable).unwrap();
            assert_eq!(jung, decomposed_jung, "Jungseong mismatch for {syllable}");
        }
    }

    #[test]
    fn test_decompose_all_jongseong() {
        let expected_jongseong = [
            Some('ㄱ'),
            Some('ㄲ'),
            Some('ㄳ'),
            Some('ㄴ'),
            Some('ㄵ'),
            Some('ㄶ'),
            Some('ㄷ'),
            Some('ㄹ'),
            Some('ㄺ'),
            Some('ㄻ'),
            Some('ㄼ'),
            Some('ㄽ'),
            Some('ㄾ'),
            Some('ㄿ'),
            Some('ㅀ'),
            Some('ㅁ'),
            Some('ㅂ'),
            Some('ㅄ'),
            Some('ㅅ'),
            Some('ㅆ'),
            Some('ㅇ'),
            Some('ㅈ'),
            Some('ㅊ'),
            Some('ㅋ'),
            Some('ㅌ'),
            Some('ㅍ'),
            Some('ㅎ'),
        ];

        for &jong in &expected_jongseong {
            let syllable = compose('ㄱ', 'ㅏ', jong).unwrap();
            let (_, _, decomposed_jong) = decompose(syllable).unwrap();
            assert_eq!(jong, decomposed_jong, "Jongseong mismatch for {syllable}");
        }
    }

    #[test]
    fn test_complex_jongseong() {
        // Test double consonant jongseong
        assert_eq!(compose('ㄱ', 'ㅏ', Some('ㄳ')), Some('갃'));
        assert_eq!(decompose('갃'), Some(('ㄱ', 'ㅏ', Some('ㄳ'))));

        assert_eq!(compose('ㄱ', 'ㅏ', Some('ㄵ')), Some('갅'));
        assert_eq!(decompose('갅'), Some(('ㄱ', 'ㅏ', Some('ㄵ'))));

        assert_eq!(compose('ㄱ', 'ㅏ', Some('ㅄ')), Some('값'));
        assert_eq!(decompose('값'), Some(('ㄱ', 'ㅏ', Some('ㅄ'))));
    }

    #[test]
    fn test_decompose_str_empty() {
        assert_eq!(decompose_str(""), "");
    }

    #[test]
    fn test_decompose_str_no_hangul() {
        assert_eq!(decompose_str("Hello World"), "Hello World");
        assert_eq!(decompose_str("123"), "123");
        assert_eq!(decompose_str("!@#$%"), "!@#$%");
    }

    #[test]
    fn test_decompose_str_mixed() {
        assert_eq!(decompose_str("안녕하세요"), "ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ");
        assert_eq!(decompose_str("Hello 안녕"), "Hello ㅇㅏㄴㄴㅕㅇ");
        assert_eq!(decompose_str("한글123"), "ㅎㅏㄴㄱㅡㄹ123");
    }

    #[test]
    fn test_compose_str_empty() {
        assert_eq!(compose_str(""), "");
    }

    #[test]
    fn test_compose_str_no_jamo() {
        assert_eq!(compose_str("Hello"), "Hello");
        assert_eq!(compose_str("123"), "123");
    }

    #[test]
    fn test_compose_str_incomplete_jamo() {
        // Isolated choseong without jungseong
        assert_eq!(compose_str("ㄱ"), "ㄱ");

        // Isolated jungseong
        assert_eq!(compose_str("ㅏ"), "ㅏ");

        // Choseong + choseong (can't compose)
        assert_eq!(compose_str("ㄱㄴ"), "ㄱㄴ");
    }

    #[test]
    fn test_compose_str_with_jongseong_ambiguity() {
        // ㄱㅏㄴㄴㅕㅇ should become 간녕 (not 간영)
        // The ㄴ should be jongseong of first syllable, not choseong of second
        assert_eq!(compose_str("ㄱㅏㄴㄴㅕㅇ"), "간녕");

        // ㄱㅏㄹㄱㅏ should become 갈가 (not 가ㄹ가)
        assert_eq!(compose_str("ㄱㅏㄹㄱㅏ"), "갈가");
    }

    #[test]
    fn test_decompose_compose_str_roundtrip() {
        let test_strings = ["한글", "안녕하세요", "대한민국", "가나다라마바사"];

        for &s in &test_strings {
            let decomposed = decompose_str(s);
            let recomposed = compose_str(&decomposed);
            assert_eq!(s, recomposed, "Roundtrip failed for '{s}'");
        }
    }

    #[test]
    fn test_has_jongseong_all_combinations() {
        // Test syllable with no jongseong
        assert_eq!(has_jongseong('가'), Some(false));
        assert_eq!(has_jongseong('나'), Some(false));
        assert_eq!(has_jongseong('다'), Some(false));

        // Test syllable with jongseong
        assert_eq!(has_jongseong('각'), Some(true));
        assert_eq!(has_jongseong('난'), Some(true));
        assert_eq!(has_jongseong('닭'), Some(true));

        // Test with complex jongseong
        assert_eq!(has_jongseong('갃'), Some(true)); // ㄳ
        assert_eq!(has_jongseong('닭'), Some(true)); // ㄹ
    }

    #[test]
    fn test_classify_char_edge_cases() {
        // CJK Extension A
        assert_eq!(classify_char('\u{3400}'), CharType::Hanja);
        assert_eq!(classify_char('\u{4DBF}'), CharType::Hanja);

        // CJK Unified Ideographs
        assert_eq!(classify_char('\u{4E00}'), CharType::Hanja);
        assert_eq!(classify_char('\u{9FFF}'), CharType::Hanja);

        // Katakana range
        assert_eq!(classify_char('\u{30A0}'), CharType::Katakana);
        assert_eq!(classify_char('\u{30FF}'), CharType::Katakana);

        // Hiragana range
        assert_eq!(classify_char('\u{3040}'), CharType::Hiragana);
        assert_eq!(classify_char('\u{309F}'), CharType::Hiragana);

        // Other punctuation
        assert_eq!(classify_char('!'), CharType::Punctuation);
        assert_eq!(classify_char(','), CharType::Punctuation);
        assert_eq!(classify_char(';'), CharType::Punctuation);

        // Various whitespace
        assert_eq!(classify_char('\t'), CharType::Whitespace);
        assert_eq!(classify_char('\n'), CharType::Whitespace);

        // Other characters
        assert_eq!(classify_char('™'), CharType::Other);
        assert_eq!(classify_char('€'), CharType::Other);
    }

    #[test]
    fn test_all_hangul_syllables_decompose_compose() {
        // Test a sample of syllables across the entire range
        let sample_codes: Vec<u32> = vec![
            0xAC00, // 가 (first)
            0xAC01, // 각 (second)
            0xB098, // 나
            0xB2E4, // 다
            0xB77C, // 라
            0xB9C8, // 마
            0xBC14, // 바
            0xC0AC, // 사
            0xC544, // 아
            0xC790, // 자
            0xCC28, // 차
            0xCE74, // 카
            0xD0C0, // 타
            0xD30C, // 파
            0xD558, // 하
            0xD7A3, // 힣 (last)
        ];

        for code in sample_codes {
            let c = char::from_u32(code).unwrap();
            let (cho, jung, jong) = decompose(c).unwrap();
            let recomposed = compose(cho, jung, jong).unwrap();
            assert_eq!(c, recomposed, "Failed for U+{code:04X} ({c})");
        }
    }

    #[test]
    fn test_compose_str_real_words() {
        assert_eq!(compose_str("ㅎㅏㄴㄱㅡㄹ"), "한글");
        assert_eq!(compose_str("ㅇㅏㄴㄴㅕㅇㅎㅏㅅㅔㅇㅛ"), "안녕하세요");
        assert_eq!(compose_str("ㄷㅐㅎㅏㄴㅁㅣㄴㄱㅜㄱ"), "대한민국");
    }
}
