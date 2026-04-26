use crate::sejong::types::SejongToken;

/// 220차, 86차, 87차, 87-2차, 88차, 205차: 활용 보정 후처리
///
/// `apply_conjugation_corrections` 호출 직후, `apply_sentence_final_corrections` 호출 직전에 실행.
///
/// - 220차: "ㄹ/ETM + 지/NNB|VX" → "ㄹ지/EC" 병합
/// - 86차: "ㄴ/ETM + 다/EF" → "ㄴ다/EF", "는/ETM + 다/EF" → "는다/EF" 병합
/// - 87차: EC 뒤의 보조동사 VV → VX
/// - 87-2차: "고/EC" 뒤의 특정 보조동사 VV → VX
/// - 88차: NNG + "되/VV" → NNG + "되/XSV"
/// - 205차: NNP + "되/XSV" → "되/VV"
pub(super) fn apply_post_conjugation_corrections(tokens: &mut Vec<SejongToken>) {
    // 220차 보정: "ㄹ/ETM + 지/NNB|VX" → "ㄹ지/EC" 병합
    // sample.tsv 기준: "진학할지 취업을" → "진학/NNG 하/XSV ㄹ지/EC 취업/NNG"
    // MeCab이 "할지"를 "하/VV ㄹ/ETM + 지/VX"로 분석
    let mut lji_merge_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len() {
        if tokens[i - 1].surface == "ㄹ"
            && tokens[i - 1].pos == "ETM"
            && tokens[i].surface == "지"
            && (tokens[i].pos == "NNB" || tokens[i].pos == "VX")
        {
            lji_merge_indices.push(i - 1);
        }
    }

    for idx in lji_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new("ㄹ지", "EC", start, end);
        tokens.remove(idx + 1);
    }

    // 86차 보정: "ㄴ/ETM + 다/EF" → "ㄴ다/EF", "는/ETM + 다/EF" → "는다/EF" 병합
    // "간다" = "가/VV ㄴ다/EF", "먹는다" = "먹/VV 는다/EF"
    // sample.tsv 형식에 맞춰 현재형 종결어미를 단일 토큰으로 처리
    // 143차: "다/NNG"도 문장 끝이면 EF로 처리 (MeCab이 "다"를 NNG로 분석하는 경우)
    let mut nda_merge_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len() {
        if (tokens[i - 1].surface == "ㄴ" || tokens[i - 1].surface == "는")
            && tokens[i - 1].pos == "ETM"
            && tokens[i].surface == "다"
            && (tokens[i].pos == "EF" || (tokens[i].pos == "NNG" && i == tokens.len() - 1))
        {
            nda_merge_indices.push(i - 1);
        }
    }

    for idx in nda_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        let merged_surface = if tokens[idx].surface == "ㄴ" {
            "ㄴ다"
        } else {
            "는다"
        };
        tokens[idx] = SejongToken::new(merged_surface, "EF", start, end);
        tokens.remove(idx + 1);
    }

    // 87차 보정: EC 뒤의 보조동사 VV → VX
    // "먹어 버렸다" = "먹/VV 어/EC 버리/VX 었/EP 다/EF"
    // "기다려 주세요" = "기다리/VV 어/EC 주/VX 세요/EF"
    let aux_verbs = ["보", "주", "버리", "내", "놓", "두", "가"];
    for i in 1..tokens.len() {
        if tokens[i].pos == "VV"
            && aux_verbs.contains(&tokens[i].surface.as_str())
            && tokens[i - 1].pos == "EC"
        {
            tokens[i].pos = "VX".to_string();
        }
    }

    // 87-2차 보정: "고/EC" 뒤의 특정 보조동사 VV → VX
    // "읽고 싶다" = "읽/VV 고/EC 싶/VX 다/EF"
    // "하지 않다" = "하/VV 지/EC 않/VX 다/EF"
    // 주의: "있/VV"는 본동사로도 쓰이므로 제외
    let go_aux_verbs = ["싶", "않"];
    for i in 1..tokens.len() {
        if tokens[i].pos == "VV"
            && go_aux_verbs.contains(&tokens[i].surface.as_str())
            && tokens[i - 1].pos == "EC"
            && tokens[i - 1].surface == "고"
        {
            tokens[i].pos = "VX".to_string();
        }
    }

    // 88차 보정: NNG + "되/VV" → NNG + "되/XSV"
    // "공개됐다" = "공개/NNG 되/XSV 었/EP 다/EF"
    // "발표될" = "발표/NNG 되/XSV ㄹ/ETM"
    // 주의: "하/VV" → "하/XSV"는 VV 정확도를 떨어뜨리므로 적용하지 않음
    for i in 1..tokens.len() {
        if tokens[i].surface == "되" && tokens[i].pos == "VV" && tokens[i - 1].pos == "NNG" {
            tokens[i].pos = "XSV".to_string();
        }
    }

    // 205차 보정: NNP + "되/XSV" → "되/VV"
    // "크리에이터 되고 싶어" = "크리에이터/NNG 되/VV 고/EC 싶/VX 어/EF"
    // NNP 뒤의 "되"는 본동사 (NNG 뒤의 "되"만 XSV)
    for i in 1..tokens.len() {
        if tokens[i].surface == "되" && tokens[i].pos == "XSV" && tokens[i - 1].pos == "NNP" {
            tokens[i].pos = "VV".to_string();
        }
    }
}
