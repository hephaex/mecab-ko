//! Kiwi 호환 레이어 데모
//!
//! MeCab-Ko와 Kiwi 품사 태그 간 변환 예제

use mecab_ko_core::{from_kiwi_tag, to_kiwi_tag, KiwiPosTag, KiwiToken, PosTag};

fn main() {
    println!("=== Kiwi 호환 레이어 데모 ===\n");

    // 1. MeCab -> Kiwi 변환
    println!("1. MeCab -> Kiwi 변환");
    println!("{}", "-".repeat(50));
    let mecab_tags = [
        PosTag::NNG,
        PosTag::VV,
        PosTag::JKS,
        PosTag::NNBC, // 단위명사
        PosTag::SSO,  // 여는괄호
    ];

    for mecab_tag in mecab_tags {
        let kiwi_tag = to_kiwi_tag(mecab_tag);
        println!("  MeCab: {:6} -> Kiwi: {}", mecab_tag.as_str(), kiwi_tag.as_str());
    }

    // 2. Kiwi -> MeCab 변환
    println!("\n2. Kiwi -> MeCab 변환");
    println!("{}", "-".repeat(50));
    let kiwi_tags = [
        KiwiPosTag::NNG,
        KiwiPosTag::VV,
        KiwiPosTag::W_URL,   // 웹 URL
        KiwiPosTag::W_EMAIL, // 이메일
        KiwiPosTag::SS,      // 괄호 (통합)
    ];

    for kiwi_tag in kiwi_tags {
        let mecab_tag = from_kiwi_tag(kiwi_tag);
        println!("  Kiwi: {:10} -> MeCab: {}", kiwi_tag.as_str(), mecab_tag.as_str());
    }

    // 3. 정보 손실 예제
    println!("\n3. 정보 손실 변환 예제");
    println!("{}", "-".repeat(50));
    let lossy_examples = [
        (PosTag::NNBC, "단위 의존 명사 -> 의존 명사"),
        (PosTag::SSC, "닫는 괄호 -> 괄호 (통합)"),
        (PosTag::SC, "구분자 -> 쉼표 (통합)"),
    ];

    for (tag, desc) in lossy_examples {
        let kiwi_tag = to_kiwi_tag(tag);
        let back = from_kiwi_tag(kiwi_tag);
        println!(
            "  {} -> {} -> {} ({})",
            tag.as_str(),
            kiwi_tag.as_str(),
            back.as_str(),
            if tag == back { "완전 복원" } else { "정보 손실" }
        );
        println!("    설명: {desc}");
    }

    // 4. KiwiToken 사용 예제
    println!("\n4. KiwiToken 사용 예제");
    println!("{}", "-".repeat(50));
    let tokens = vec![
        KiwiToken::new("안녕", KiwiPosTag::NNG, 0, 6, -10.5),
        KiwiToken::new("하", KiwiPosTag::VV, 6, 3, -5.2),
        KiwiToken::new("세요", KiwiPosTag::EP, 9, 9, -8.1),
    ];

    for token in &tokens {
        println!(
            "  형태소: {:6} | 품사: {:4} | 위치: {:2}-{:2} | 점수: {:6.2}",
            token.form,
            token.tag.as_str(),
            token.start,
            token.end(),
            token.score
        );
    }

    // 5. Display trait 예제
    println!("\n5. 형태소 출력 (Display trait)");
    println!("{}", "-".repeat(50));
    for token in &tokens {
        println!("  {token}");
    }

    // 6. 웹 관련 태그 예제
    println!("\n6. 웹 관련 태그 (Kiwi 전용)");
    println!("{}", "-".repeat(50));
    let web_tokens = vec![
        KiwiToken::new("https://example.com", KiwiPosTag::W_URL, 0, 19, -15.0),
        KiwiToken::new("user@example.com", KiwiPosTag::W_EMAIL, 20, 16, -12.0),
        KiwiToken::new("#한국어", KiwiPosTag::W_HASHTAG, 37, 12, -8.5),
        KiwiToken::new("@username", KiwiPosTag::W_MENTION, 50, 9, -7.2),
    ];

    for token in web_tokens {
        let mecab_tag = token.to_mecab_tag();
        println!(
            "  {:<20} | Kiwi: {:12} -> MeCab: {}",
            token.form,
            token.tag.as_str(),
            mecab_tag.as_str()
        );
    }

    println!("\n=== 데모 완료 ===");
}
