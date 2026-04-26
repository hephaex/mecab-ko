//! 태그 정규화: 빈 POS, XR 태그, 미분류 태그 보정

use crate::sejong::types::SejongToken;

/// 209~223차: 빈 POS 및 XR 태그 정규화
///
/// - 209차: 빈 POS → SL (영문자) 또는 NNG (한글)
/// - 223차: XR(어근) → NNG 변환
pub(super) fn apply_tag_normalization_corrections(tokens: &mut [SejongToken]) {
    // 209차: 빈 POS → SL (외래어/영문자)
    // MeCab이 영문자를 빈 품사로 분석하는 경우 SL 태그 부여
    // "MBTI/" → "MBTI/SL"
    for token in tokens.iter_mut() {
        if token.pos.is_empty() {
            // 영문자로만 구성된 경우 SL 태그 부여
            let is_alpha = token.surface.chars().all(|c| c.is_ascii_alphabetic());
            if is_alpha && !token.surface.is_empty() {
                token.pos = "SL".to_string();
            } else {
                // 223차: 한글 의성어/의태어가 빈 POS인 경우 NNG 부여
                // "왈왈/" → "왈왈/NNG"
                let is_korean = token.surface.chars().all(|c| {
                    let code = c as u32;
                    (0xAC00..=0xD7A3).contains(&code) || (0x3131..=0x318E).contains(&code)
                });
                if is_korean && !token.surface.is_empty() {
                    token.pos = "NNG".to_string();
                }
            }
        }
    }

    // 223차: XR(어근) → NNG 변환
    // 의성어/의태어가 XR로 분석되는 경우 NNG로 처리
    // "멍멍/XR" → "멍멍/NNG"
    for token in tokens.iter_mut() {
        if token.pos == "XR" {
            token.pos = "NNG".to_string();
        }
    }
}
