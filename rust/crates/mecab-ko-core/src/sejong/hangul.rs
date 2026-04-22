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
        ('ᆨ', 'ㄱ'), ('ᆩ', 'ㄲ'), ('ᆪ', 'ㄳ'), ('ᆫ', 'ㄴ'), ('ᆬ', 'ㄵ'),
        ('ᆭ', 'ㄶ'), ('ᆮ', 'ㄷ'), ('ᆯ', 'ㄹ'), ('ᆰ', 'ㄺ'), ('ᆱ', 'ㄻ'),
        ('ᆲ', 'ㄼ'), ('ᆳ', 'ㄽ'), ('ᆴ', 'ㄾ'), ('ᆵ', 'ㄿ'), ('ᆶ', 'ㅀ'),
        ('ᆷ', 'ㅁ'), ('ᆸ', 'ㅂ'), ('ᆹ', 'ㅄ'), ('ᆺ', 'ㅅ'), ('ᆻ', 'ㅆ'),
        ('ᆼ', 'ㅇ'), ('ᆽ', 'ㅈ'), ('ᆾ', 'ㅊ'), ('ᆿ', 'ㅋ'), ('ᇀ', 'ㅌ'),
        ('ᇁ', 'ㅍ'), ('ᇂ', 'ㅎ'),
    ];
    let map: std::collections::HashMap<char, char> = jongseong_to_compat.into_iter().collect();
    text.chars().map(|c| *map.get(&c).unwrap_or(&c)).collect()
}
