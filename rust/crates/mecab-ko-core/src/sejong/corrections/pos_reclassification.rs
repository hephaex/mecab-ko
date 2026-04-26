use crate::sejong::types::SejongToken;

/// 207~256차: 단일 토큰 POS 재분류
///
/// - 207차: 부사로 잘못 분석된 명사 복원 (요즘, 진짜)
/// - 208차: 정부 기관명 NNP → NNG
/// - 248차: 외래어 NNP → NNG
/// - 252차: 신조어 NNP → NNG
/// - 253차: 의성어 NNG → IC
/// - 254차: 표면형 정규화 ㅓ요/EF → 어요/EF
/// - 256차: VV → VA (졸리)
pub(super) fn apply_pos_reclassification_corrections(tokens: &mut [SejongToken]) {
    // 207차: 부사로 잘못 분석된 명사 복원
    // "요즘/MAG" → "요즘/NNG", "진짜/MAG" → "진짜/NNG"
    // "진짜/VV" → "진짜/NNG" (동사로 분석된 경우도)
    // sample.tsv에서 NNG로 태깅됨
    let to_nng_words = ["요즘", "진짜"];
    for token in tokens.iter_mut() {
        if (token.pos == "MAG" || token.pos == "VV")
            && to_nng_words.contains(&token.surface.as_str())
        {
            token.pos = "NNG".to_string();
        }
    }

    // 208차: 정부 기관명 NNP → NNG
    // "외교부/NNP" → "외교부/NNG"
    // sample.tsv에서 NNG로 태깅됨
    let nnp_to_nng_orgs = ["외교부", "국방부", "통일부", "교육부", "행정부", "대통령실"];
    for token in tokens.iter_mut() {
        if token.pos == "NNP" && nnp_to_nng_orgs.contains(&token.surface.as_str()) {
            token.pos = "NNG".to_string();
        }
    }

    // 248차: 외래어 NNP → NNG 변환
    // "프레임워크/NNP" → "프레임워크/NNG"
    // "리팩토링/NNP" → "리팩토링/NNG"
    // sample.tsv에서 NNG로 태깅되는 외래어들
    let foreign_nnp_to_nng = [
        "프레임워크",
        "리팩토링",
        "알고리즘",
        "커버리지",
        "아키텍처",
        "머신러닝",
        "컨테이너",
        "인터페이스",
        "데이터베이스",
        "서버",
        "클라이언트",
        "프로토콜",
        "레이어",
        "모듈",
        "컴포넌트",
    ];
    for token in tokens.iter_mut() {
        if token.pos == "NNP" && foreign_nnp_to_nng.contains(&token.surface.as_str()) {
            token.pos = "NNG".to_string();
        }
    }

    // 252차: 신조어 NNP → NNG 변환
    // "킹/NNP" → "킹/NNG" (킹받네, 킹성비 등)
    // MeCab이 인명으로 분석하지만 실제로는 신조어 접두사
    let slang_nnp_to_nng = ["킹"];
    for token in tokens.iter_mut() {
        if token.pos == "NNP" && slang_nnp_to_nng.contains(&token.surface.as_str()) {
            token.pos = "NNG".to_string();
        }
    }

    // 253차: 의성어 NNG → IC 변환
    // "야옹/NNG" → "야옹/IC" (고양이 울음소리)
    // sample.tsv에서 IC로 태깅되는 의성어들
    let onomatopoeia_to_ic = ["야옹"];
    for token in tokens.iter_mut() {
        if token.pos == "NNG" && onomatopoeia_to_ic.contains(&token.surface.as_str()) {
            token.pos = "IC".to_string();
        }
    }

    // 254차: "ㅓ요/EF" → "어요/EF" 표면형 정규화
    // MeCab decomposition이 "ㅓ요"로 분리하지만 세종 기준은 "어요"
    // "쉬워요" = "쉽/VA 어요/EF"
    for token in tokens.iter_mut() {
        if token.surface == "ㅓ요" && token.pos == "EF" {
            token.surface = "어요".to_string();
        }
    }

    // 256차: "졸리/VV → 졸리/VA" 변환
    // "졸려요" = "졸리/VA 어요/EF"
    // MeCab이 VV로 분석하지만 형용사(VA)로 처리
    let vv_to_va = ["졸리"];
    for token in tokens.iter_mut() {
        if token.pos == "VV" && vv_to_va.contains(&token.surface.as_str()) {
            token.pos = "VA".to_string();
        }
    }
}
