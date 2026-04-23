//! 한글 처리 유틸리티 함수

/// 한글 음절에 종성(받침)이 있는지 확인
#[must_use]
pub fn has_jongseong(ch: char) -> bool {
    let code = ch as u32;
    if (0xAC00..=0xD7A3).contains(&code) {
        (code - 0xAC00) % 28 != 0
    } else {
        false
    }
}

/// 한글 음절에서 ㄹ 받침을 제거
/// 예: 할 → 하, 갈 → 가, 볼 → 보
#[must_use]
pub fn remove_jongseong_rieul(ch: char) -> Option<char> {
    let code = ch as u32;
    if (0xAC00..=0xD7A3).contains(&code) {
        let jongseong = (code - 0xAC00) % 28;
        if jongseong == 8 {
            let new_code = code - 8;
            char::from_u32(new_code)
        } else {
            None
        }
    } else {
        None
    }
}

/// 한글 음절에서 ㄴ 받침을 제거
/// 예: 간 → 가, 산 → 사, 온 → 오
#[must_use]
pub fn remove_jongseong_nieun(ch: char) -> Option<char> {
    let code = ch as u32;
    if (0xAC00..=0xD7A3).contains(&code) {
        let jongseong = (code - 0xAC00) % 28;
        if jongseong == 4 {
            let new_code = code - 4;
            char::from_u32(new_code)
        } else {
            None
        }
    } else {
        None
    }
}

/// 한글 음절에서 ㅂ 받침을 제거
/// 예: 합 → 하, 갑 → 가, 옵 → 오
#[must_use]
pub fn remove_jongseong_bieup(ch: char) -> Option<char> {
    let code = ch as u32;
    if (0xAC00..=0xD7A3).contains(&code) {
        let jongseong = (code - 0xAC00) % 28;
        if jongseong == 17 {
            let new_code = code - 17;
            char::from_u32(new_code)
        } else {
            None
        }
    } else {
        None
    }
}

/// 한글 음절에서 모음 추출
#[must_use]
pub fn extract_vowel(ch: char) -> char {
    let code = ch as u32;
    if (0xAC00..=0xD7A3).contains(&code) {
        let vowel_idx = ((code - 0xAC00) / 28) % 21;
        let vowels = [
            'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ',
            'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
        ];
        vowels[vowel_idx as usize]
    } else {
        ch
    }
}

/// 한글 자모 정규화: 종성 자모(U+11xx)를 호환 자모(U+31xx)로 변환
/// 124차 보정: `MeCab` 출력의 종성 자모를 세종 코퍼스 형식(호환 자모)으로 통일
#[must_use]
pub fn normalize_jamo(text: &str) -> String {
    let jongseong_to_compat: [(char, char); 27] = [
        ('ᆨ', 'ㄱ'),
        ('ᆩ', 'ㄲ'),
        ('ᆪ', 'ㄳ'),
        ('ᆫ', 'ㄴ'),
        ('ᆬ', 'ㄵ'),
        ('ᆭ', 'ㄶ'),
        ('ᆮ', 'ㄷ'),
        ('ᆯ', 'ㄹ'),
        ('ᆰ', 'ㄺ'),
        ('ᆱ', 'ㄻ'),
        ('ᆲ', 'ㄼ'),
        ('ᆳ', 'ㄽ'),
        ('ᆴ', 'ㄾ'),
        ('ᆵ', 'ㄿ'),
        ('ᆶ', 'ㅀ'),
        ('ᆷ', 'ㅁ'),
        ('ᆸ', 'ㅂ'),
        ('ᆹ', 'ㅄ'),
        ('ᆺ', 'ㅅ'),
        ('ᆻ', 'ㅆ'),
        ('ᆼ', 'ㅇ'),
        ('ᆽ', 'ㅈ'),
        ('ᆾ', 'ㅊ'),
        ('ᆿ', 'ㅋ'),
        ('ᇀ', 'ㅌ'),
        ('ᇁ', 'ㅍ'),
        ('ᇂ', 'ㅎ'),
    ];
    let map: std::collections::HashMap<char, char> = jongseong_to_compat.into_iter().collect();
    text.chars().map(|c| *map.get(&c).unwrap_or(&c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_jongseong_with_batchim() {
        // 받침 있는 글자: 밥(ㅂ), 닭(ㄱ), 한(ㄴ)
        assert!(has_jongseong('밥'));
        assert!(has_jongseong('닭'));
        assert!(has_jongseong('한'));
    }

    #[test]
    fn test_has_jongseong_without_batchim() {
        // 받침 없는 글자: 가, 나, 보
        assert!(!has_jongseong('가'));
        assert!(!has_jongseong('나'));
        assert!(!has_jongseong('보'));
        // ASCII 문자는 false
        assert!(!has_jongseong('a'));
    }

    #[test]
    fn test_remove_jongseong_rieul_removes_rieul() {
        // 할(하+ㄹ) → 하, 갈(가+ㄹ) → 가, 볼(보+ㄹ) → 보
        assert_eq!(remove_jongseong_rieul('할'), Some('하'));
        assert_eq!(remove_jongseong_rieul('갈'), Some('가'));
        assert_eq!(remove_jongseong_rieul('볼'), Some('보'));
    }

    #[test]
    fn test_remove_jongseong_rieul_returns_none_for_other_jongseong() {
        // ㄹ이 아닌 받침이나 받침 없는 글자는 None
        assert_eq!(remove_jongseong_rieul('가'), None);
        assert_eq!(remove_jongseong_rieul('밥'), None); // ㅂ 받침
        assert_eq!(remove_jongseong_rieul('한'), None); // ㄴ 받침
    }

    #[test]
    fn test_normalize_jamo_converts_jongseong_to_compat() {
        // 종성 자모 U+11AF(ᆯ) → 호환 자모 U+3139(ㄹ)
        let jongseong_rieul = '\u{11AF}';
        assert_eq!(normalize_jamo(&jongseong_rieul.to_string()), "ㄹ");
        // 종성 ᆫ(U+11AB) → 호환 ㄴ(U+3134)
        let jongseong_nieun = '\u{11AB}';
        assert_eq!(normalize_jamo(&jongseong_nieun.to_string()), "ㄴ");
    }

    #[test]
    fn test_normalize_jamo_passes_through_regular_syllables() {
        // 일반 한글 음절과 ASCII는 변환 없이 통과
        assert_eq!(normalize_jamo("가나다"), "가나다");
        assert_eq!(normalize_jamo("hello"), "hello");
    }
}
