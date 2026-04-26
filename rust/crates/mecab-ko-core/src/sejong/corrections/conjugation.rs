use crate::sejong::types::SejongToken;

/// 219차: ㄷ불규칙 동사 활용형 → 원형 변환 테이블 (활용형, 원형)
static D_IRREGULAR_VERBS: &[(&str, &str)] = &[
    ("걸", "걷"),     // 걷다 → 걸어
    ("들", "듣"),     // 듣다 → 들어
    ("물", "묻"),     // 묻다 → 물어
    ("실", "싣"),     // 싣다 → 실어
    ("깨달", "깨닫"), // 깨닫다 → 깨달아
];

/// 174~219차: 동사/형용사 활용 보정
///
/// - 174차: 형용사적 "하다"의 XSV → XSA 변환
/// - 226차: "목/NNG + 마르/VV" → "목마르/VA" 병합
/// - 215차: 형용사 어근 + 하 → VA 병합
/// - 225차: "NNG + 하/XSV + ㅁ/ETN" → "NNG하/VV + ㅁ/ETN" 병합
/// - 217차: "으면/EF" → "으면/EC" (VA 뒤 연결어미)
/// - 218차: "는데/EF" → "는데/EC" (문장 중간 연결어미)
/// - 219차: ㄷ불규칙 동사 어간 복원 (229차 수정 포함)
pub(super) fn apply_conjugation_corrections(tokens: &mut Vec<SejongToken>) {
    // 174차 보정: 형용사적 "하다"의 XSV → XSA 변환
    // "미안해요" = "미안/NNG 하/XSA 어요/EF" (형용사적)
    // "발표했다" = "발표/NNG 하/XSV 았/EP 다/EF" (동사적)
    // 형용사 어근 목록을 기반으로 XSV를 XSA로 변환
    let adj_roots = [
        "미안",
        "심심",
        "피곤",
        "건강",
        "조용",
        "깨끗",
        "더럽",
        "시끄럽",
        "행복",
        "불행",
        "편안",
        "불편",
        "따뜻",
        "차가움",
        "친절",
        "불친절",
        "정확",
        "부정확",
        "명확",
        "불명확",
        "솔직",
        "불성실",
        "성실",
        "유명",
        "무명",
        "다양",
        "단순",
        "복잡",
        "간단",
        "적합",
        "부적합",
    ];
    for i in 1..tokens.len() {
        if tokens[i].surface == "하"
            && tokens[i].pos == "XSV"
            && adj_roots.contains(&tokens[i - 1].surface.as_str())
        {
            tokens[i].pos = "XSA".to_string();
        }
    }

    // 226차: "목/NNG + 마르/VV" → "목마르/VA" 병합
    // sample.tsv 기준: "목말라요" → "목마르/VA 아요/EF"
    // MeCab이 "목/NNG + 말라요/VV+EC"로 분석하는 경우 병합
    {
        let mut i = 0;
        while i + 1 < tokens.len() {
            if tokens[i].surface == "목"
                && tokens[i].pos == "NNG"
                && tokens[i + 1].surface.starts_with("마르")
                && tokens[i + 1].pos == "VV"
            {
                let merged_surface = format!("목{}", tokens[i + 1].surface);
                let start = tokens[i].start_pos;
                let end = tokens[i + 1].end_pos;
                tokens[i] = SejongToken::new(&merged_surface, "VA", start, end);
                tokens.remove(i + 1);
            }
            i += 1;
        }
    }

    // 215차: 형용사 어근 + 하 → VA 병합
    // sample.tsv 기준: "미안해요" → "미안하/VA 어요/EF"
    // "미안/NNG 하/XSA" → "미안하/VA"로 병합
    let va_merge_roots = ["미안", "심심"];
    let mut i = 0;
    while i + 1 < tokens.len() {
        if tokens[i].pos == "NNG"
            && tokens[i + 1].surface == "하"
            && (tokens[i + 1].pos == "XSA" || tokens[i + 1].pos == "XSV")
            && va_merge_roots.contains(&tokens[i].surface.as_str())
        {
            let merged_surface = format!("{}하", tokens[i].surface);
            let start = tokens[i].start_pos;
            let end = tokens[i + 1].end_pos;
            tokens[i] = SejongToken::new(&merged_surface, "VA", start, end);
            tokens.remove(i + 1);
        }
        i += 1;
    }

    // 225차: "NNG + 하/XSV + ㅁ/ETN" → "NNG하/VV + ㅁ/ETN" 병합
    // sample.tsv 기준: "말함" → "말하/VV ㅁ/ETN"
    // MeCab이 "말/NNG + 함/XSV+ETN"으로 분석하는 경우 병합
    let vv_merge_roots = ["말"];
    let mut i = 0;
    while i + 2 < tokens.len() {
        if tokens[i].pos == "NNG"
            && tokens[i + 1].surface == "하"
            && tokens[i + 1].pos == "XSV"
            && tokens[i + 2].surface == "ㅁ"
            && tokens[i + 2].pos == "ETN"
            && vv_merge_roots.contains(&tokens[i].surface.as_str())
        {
            let merged_surface = format!("{}하", tokens[i].surface);
            let start = tokens[i].start_pos;
            let end = tokens[i + 1].end_pos;
            tokens[i] = SejongToken::new(&merged_surface, "VV", start, end);
            tokens.remove(i + 1);
        }
        i += 1;
    }

    // 217차 보정: "으면/EF" → "으면/EC" (VA 뒤 연결어미)
    // sample.tsv 기준: "하얗으면" → "하얗/VA 으면/EC"
    // MeCab이 "으면"을 EF로 분석하지만 실제로는 연결어미(EC)
    for i in 1..tokens.len() {
        if tokens[i].surface == "으면" && tokens[i].pos == "EF" {
            // 앞에 VA/VV가 있으면 EC로 변환
            if tokens[i - 1].pos == "VA" || tokens[i - 1].pos == "VV" {
                tokens[i].pos = "EC".to_string();
            }
        }
    }

    // 218차 보정: "는데/EF" → "는데/EC" (문장 중간 연결어미)
    // sample.tsv 기준: "나왔는데 막상" → "나오/VV 았/EP 는데/EC 막상/MAG"
    // 문장 끝이 아니면 연결어미로 처리
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i].surface == "는데" && tokens[i].pos == "EF" {
            // 문장 끝이 아니면 EC로 변환
            tokens[i].pos = "EC".to_string();
        }
    }

    // 219차 보정: ㄷ불규칙 동사 어간 복원
    // sample.tsv 기준: "걸어" → "걷/VV 어/EF" (활용형 "걸"을 원형 "걷"으로)
    // MeCab이 "걸/VV"로 분석하지만 원형은 "걷"
    // 주요 ㄷ불규칙 동사: 걷다(→걸), 듣다(→들), 묻다(→물), 싣다(→실), 깨닫다(→깨달)
    // 229차 수정: "들/VV + 세요/시" 패턴은 "드시다" (먹다의 존칭)이므로 변환 제외

    for i in 0..tokens.len() {
        if tokens[i].pos == "VV" {
            if let Some(original) = D_IRREGULAR_VERBS
                .iter()
                .find(|(k, _)| *k == tokens[i].surface.as_str())
                .map(|(_, v)| *v)
            {
                // 229차: "들/VV + 세요" 패턴은 "드시다" (먹다의 존칭)이므로 "듣"으로 변환 안함
                // sample.tsv 기준: "드세요" → "들/VV 세요/EF"
                let is_honorific_pattern = if i + 1 < tokens.len() {
                    let next = &tokens[i + 1].surface;
                    next == "세요" || next == "시" || next.starts_with("시")
                } else {
                    false
                };

                // "들" + 존칭어미는 변환하지 않음 (드시다)
                if tokens[i].surface == "들" && is_honorific_pattern {
                    continue;
                }

                tokens[i].surface = original.to_string();
            }
        }
    }
}
