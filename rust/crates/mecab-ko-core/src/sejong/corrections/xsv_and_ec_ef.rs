//! XSV 변환 및 EC↔EF 교차 보정: 보조동사 VX 패턴

use crate::sejong::types::SejongToken;

/// 89~112차(+9, 176, 179, 213, 220, 235): XSV 변환, EC↔EF 변환, 보조동사 VX 패턴
pub(super) fn apply_xsv_and_ec_ef_corrections(tokens: &mut Vec<SejongToken>) {
    // 89차 보정: 문장 끝 "어요/EC" → "어요/EF"
    // "고마워요" = "고맙/VA 어요/EF"
    // "미안해요" = "미안/NNG 하/XSV 어요/EF"
    // 문장의 마지막 토큰이 "어요/EC" 또는 "아요/EC"이면 EF로 변환
    if let Some(last) = tokens.last_mut() {
        if (last.surface == "어요" || last.surface == "아요") && last.pos == "EC" {
            last.pos = "EF".to_string();
        }
    }

    // 90차 보정: NNG + "하/VV" + "세요/EF" → NNG + "하/XSV" + "세요/EF"
    // 177차 수정: sample.tsv 정답에 따라 "세요/EF" 패턴은 VV 유지
    // "말씀하세요" = "말씀/NNG 하/VV 세요/EF" (not XSV)
    // "확인하세요" = "확인/NNG 하/VV 세요/EF" (not XSV)
    // 이 규칙은 더 이상 적용하지 않음

    // 91차 보정: "는다/EC" → "는다/EF"
    // "먹는다" = "먹/VV 는다/EF"
    // "한다" = "하/VV ㄴ다/EF"
    // 121차: ᆫ다 (U+11AB jongseong) 추가
    // 123차: 모든 ㄴ다/는다 적용 (문장 끝뿐만 아니라 문장 중간도)
    //        ㄴ다는 항상 평서형 종결어미이므로 EC가 아닌 EF
    for token in tokens.iter_mut() {
        if (token.surface == "는다" || token.surface == "ㄴ다" || token.surface == "ᆫ다")
            && token.pos == "EC"
        {
            token.pos = "EF".to_string();
        }
    }

    // 92차 보정: NNG + "하/VV" + "어요/EF" → NNG + "하/XSV" + "어요/EF"
    // "축하해요" = "축하/NNG 하/XSV 어요/EF"
    // "사랑해요" = "사랑/NNG 하/XSV 어요/EF"
    // "생각해요" = "생각/NNG 하/XSV 어요/EF"
    if tokens.len() >= 3 {
        for i in 1..tokens.len() - 1 {
            if tokens[i].surface == "하"
                && tokens[i].pos == "VV"
                && tokens[i - 1].pos == "NNG"
                && (tokens[i + 1].surface == "어요" || tokens[i + 1].surface == "아요")
                && tokens[i + 1].pos == "EF"
            {
                tokens[i].pos = "XSV".to_string();
            }
        }
    }

    // 93차 보정: NNG + "하/VV" + "고/EC" + VX → NNG + "하/XSV" + "고/EC" + VX
    // "투자하고 있다" = "투자/NNG 하/XSV 고/EC 있/VX 다/EF"
    // "생각하고 있다" = "생각/NNG 하/XSV 고/EC 있/VX 다/EF"
    if tokens.len() >= 4 {
        for i in 1..tokens.len() - 2 {
            if tokens[i].surface == "하"
                && tokens[i].pos == "VV"
                && tokens[i - 1].pos == "NNG"
                && tokens[i + 1].pos == "EC"
                && tokens[i + 2].pos == "VX"
            {
                tokens[i].pos = "XSV".to_string();
            }
        }
    }

    // 94차 보정: NNG + "이/NP" + "이/VCP" → NNG + "이/VCP" (중복 이 제거)
    // "꿀잼이야"가 "꿀잼/NNG 이/NP 이/VCP 야/EF"로 분석될 때
    // → "꿀잼/NNG 이/VCP 야/EF"로 수정
    let mut remove_indices: Vec<usize> = Vec::new();
    if tokens.len() >= 3 {
        for i in 1..tokens.len() - 1 {
            if tokens[i].surface == "이"
                && tokens[i].pos == "NP"
                && tokens[i + 1].surface == "이"
                && tokens[i + 1].pos == "VCP"
                && tokens[i - 1].pos == "NNG"
            {
                remove_indices.push(i);
            }
        }
    }
    // 역순으로 제거
    for idx in remove_indices.into_iter().rev() {
        tokens.remove(idx);
    }

    // 95차 보정: NNG + "하/VV" + EC 패턴 → "하/XSV"
    // "진행하고" = "진행/NNG 하/XSV 고/EC"
    // "분석하여" = "분석/NNG 하/XSV 어/EC"
    // "발표하면" = "발표/NNG 하/XSV 면/EC"
    // 단, "아야/어야/야" EC 패턴은 제외 (9차 보정에서 VV로 유지)
    // "준비해야" = "준비/NNG 하/VV 아야/EC" (not XSV)
    if tokens.len() >= 3 {
        for i in 1..tokens.len() - 1 {
            if tokens[i].surface == "하"
                && tokens[i].pos == "VV"
                && tokens[i - 1].pos == "NNG"
                && tokens[i + 1].pos == "EC"
            {
                // 176차: "아야", "어야", "야" EC는 VV 유지 (준비해야 합니다)
                // 179차: 단, 뒤에 "하/VV"가 오면 XSV로 변환 (최적화해야 한다)
                let ec_surface = &tokens[i + 1].surface;
                if ec_surface == "아야" || ec_surface == "어야" || ec_surface == "야" {
                    // "아야/어야 하다" 구문 체크 (i+2 위치에 하/VV가 있는지)
                    if i + 2 < tokens.len()
                        && tokens[i + 2].surface == "하"
                        && tokens[i + 2].pos == "VV"
                    {
                        tokens[i].pos = "XSV".to_string();
                    }
                    // 그 외에는 VV 유지
                } else {
                    tokens[i].pos = "XSV".to_string();
                }
            }
        }
    }

    // 96차 보정: NNG + "되/VV" + EC 패턴 → "되/XSV"
    // "완료되면" = "완료/NNG 되/XSV 면/EC"
    // "진행되고" = "진행/NNG 되/XSV 고/EC"
    if tokens.len() >= 3 {
        for i in 1..tokens.len() - 1 {
            if tokens[i].surface == "되"
                && tokens[i].pos == "VV"
                && tokens[i - 1].pos == "NNG"
                && tokens[i + 1].pos == "EC"
            {
                tokens[i].pos = "XSV".to_string();
            }
        }
    }

    // 97차 보정: NNG + "하/VV" + EP 패턴 → "하/XSV"
    // "발표했다" = "발표/NNG 하/XSV 았/EP 다/EF"
    // "공부했으면" = "공부/NNG 하/XSV 았/EP 으면/EC"
    if tokens.len() >= 3 {
        for i in 1..tokens.len() - 1 {
            if tokens[i].surface == "하"
                && tokens[i].pos == "VV"
                && tokens[i - 1].pos == "NNG"
                && tokens[i + 1].pos == "EP"
            {
                tokens[i].pos = "XSV".to_string();
            }
        }
    }

    // 98차 보정: 문장 끝 XSV + "어요/아요" EC → EF
    // "미안해요" = "미안/NNG 하/XSV 어요/EF" (not EC)
    // "심심해요" = "심심/NNG 하/XSV 어요/EF" (not EC)
    if tokens.len() >= 2 {
        let last_idx = tokens.len() - 1;
        let prev_idx = tokens.len() - 2;
        let should_change = (tokens[last_idx].surface == "어요"
            || tokens[last_idx].surface == "아요")
            && tokens[last_idx].pos == "EC"
            && tokens[prev_idx].pos == "XSV";
        if should_change {
            tokens[last_idx].pos = "EF".to_string();
        }
    }

    // 99차 보정: 문장 끝 VV/VA + "어요/아요" → EF
    // "재미있어요" = "재미/NNG 있/VV 어요/EF"
    // "맛없어요" = "맛/NNG 없/VA 어요/EF"
    // "만나요" = "만나/VV 아요/EF"
    if tokens.len() >= 2 {
        let last_idx = tokens.len() - 1;
        let prev_idx = tokens.len() - 2;
        let should_change = (tokens[last_idx].surface == "어요"
            || tokens[last_idx].surface == "아요")
            && tokens[last_idx].pos == "EC"
            && (tokens[prev_idx].pos == "VV" || tokens[prev_idx].pos == "VA");
        if should_change {
            tokens[last_idx].pos = "EF".to_string();
        }
    }

    // 100차 보정: "ㄴ다/EC" 또는 "는다/EC" + "하다" 패턴 → EF
    // "가자고 한다" = "가/VV 자/EC 고/EC 하/VV ㄴ다/EF"
    // 문장 끝에서 "ㄴ다" "는다" 앞에 "하/VV"가 오면 종결어미로
    if tokens.len() >= 2 {
        let last_idx = tokens.len() - 1;
        if (tokens[last_idx].surface == "ㄴ다" || tokens[last_idx].surface == "는다")
            && tokens[last_idx].pos == "EC"
        {
            // 앞에 "하/VV"가 있으면 EF로 변환
            if tokens[last_idx - 1].surface == "하" && tokens[last_idx - 1].pos == "VV" {
                tokens[last_idx].pos = "EF".to_string();
            }
        }
    }

    // 101차 보정: 인용형 "다고/EC" 패턴
    // "가자고 한다" → "자고/EC" (not "자/EF")
    // "예쁘다고 한다" → "다고/EC"
    // 문장 끝이 아닌 중간에 "자고", "다고" 등이 나오면 EC 유지
    // (이미 89차에서 문장 끝 어요/EC → EF 처리됨)

    // 102차 보정: NNG + "하/VV" + "고/EC + 있/VV" → NNG + "하/XSV" + "고/EC + 있/VX"
    // "진행하고 있다" = "진행/NNG 하/XSV 고/EC 있/VX"
    if tokens.len() >= 4 {
        for i in 1..tokens.len() - 2 {
            if tokens[i].surface == "하"
                && tokens[i].pos == "VV"
                && tokens[i - 1].pos == "NNG"
                && tokens[i + 1].surface == "고"
                && tokens[i + 1].pos == "EC"
                && tokens[i + 2].surface == "있"
                && tokens[i + 2].pos == "VV"
            {
                tokens[i].pos = "XSV".to_string();
                tokens[i + 2].pos = "VX".to_string();
            }
        }
    }

    // 103차 보정: 명사 + "되/VV" + "고/EC + 있/VV" → "되/XSV" + "있/VX"
    // "완료되고 있다" = "완료/NNG 되/XSV 고/EC 있/VX"
    if tokens.len() >= 4 {
        for i in 1..tokens.len() - 2 {
            if tokens[i].surface == "되"
                && tokens[i].pos == "VV"
                && tokens[i - 1].pos == "NNG"
                && tokens[i + 1].surface == "고"
                && tokens[i + 1].pos == "EC"
                && tokens[i + 2].surface == "있"
                && tokens[i + 2].pos == "VV"
            {
                tokens[i].pos = "XSV".to_string();
                tokens[i + 2].pos = "VX".to_string();
            }
        }
    }

    // 104차 보정: VV + 어/아/EC|EF + 보조동사VV → VV + EC + VX
    // 보조동사: 주다, 보다, 버리다, 내다, 두다, 놓다
    // "추천해 준" = VV + 어/EC + 주/VX
    // "해 보았다" = VV + 어/EC + 보/VX
    // "먹어 버렸다" = VV + 어/EC + 버리/VX
    // 126차 확장: EF도 EC로 변환 (MeCab이 잘못 EF로 분석하는 경우)
    let auxiliary_verbs: std::collections::HashSet<&str> = [
        "주", "보", "버리", "내", "두", "놓", "오", "가", "드리", "달", "빠지", "치우", "대",
    ]
    .into_iter()
    .collect();

    if tokens.len() >= 3 {
        for i in 0..tokens.len() - 2 {
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;
            let aux_surface = &tokens[i + 2].surface;
            let aux_pos = &tokens[i + 2].pos;

            // VV/VA/XSV + 어/아/EC|EF + 보조동사VV → EC + VX로 변환
            if (curr_pos == "VV" || curr_pos == "VA" || curr_pos == "XSV")
                && (next_pos == "EC" || next_pos == "EF")
                && (next_surface == "어" || next_surface == "아" || next_surface == "여")
                && aux_pos == "VV"
                && auxiliary_verbs.contains(aux_surface.as_str())
            {
                tokens[i + 1].pos = "EC".to_string(); // 126차: EF→EC
                tokens[i + 2].pos = "VX".to_string();
            }
        }
    }

    // 105차 보정: "고/EC + 있/VV" 패턴에서 있/VV → 있/VX
    // "하고 있다", "가고 있다" 등 진행형 표현
    if tokens.len() >= 2 {
        for i in 0..tokens.len() - 1 {
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;

            if curr_surface == "고" && curr_pos == "EC" && next_surface == "있" && next_pos == "VV"
            {
                tokens[i + 1].pos = "VX".to_string();
            }
        }
    }

    // 106차 보정: "지/EC + 않/VV" → "지/EC + 않/VX"
    // "하지 않다" = 지/EC + 않/VX
    if tokens.len() >= 2 {
        for i in 0..tokens.len() - 1 {
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;

            if curr_surface == "지" && curr_pos == "EC" && next_surface == "않" && next_pos == "VV"
            {
                tokens[i + 1].pos = "VX".to_string();
            }
        }
    }

    // 107차 보정: "고/EC + 싶/VV" → "고/EC + 싶/VX"
    // "보고 싶다" = 고/EC + 싶/VX
    if tokens.len() >= 2 {
        for i in 0..tokens.len() - 1 {
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;

            if curr_surface == "고"
                && curr_pos == "EC"
                && next_surface == "싶"
                && (next_pos == "VV" || next_pos == "VA")
            {
                tokens[i + 1].pos = "VX".to_string();
            }
        }
    }

    // 108차 보정: 인용형 연결어미 분리
    // "가자고 한다" = 가/VV 자고/EC 하/VV ㄴ다/EF
    // "예쁘다고 한다" = 예쁘/VA 다고/EC 하/VV ㄴ다/EF
    // "학생이라고 한다" = 학생/NNG 이/VCP 라고/EC 하/VV ㄴ다/EF
    // 패턴: "자/EF + 고/EC" → "자고/EC" 병합
    let mut quote_ec_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // "자/EF + 고/EC" → "자고/EC" (청유형 인용)
        // "다/EF + 고/EC" → "다고/EC" (평서형 인용)
        // "라/EF + 고/EC" → "라고/EC" (명령형/서술형 인용)
        if (curr_surface == "자"
            || curr_surface == "다"
            || curr_surface == "라"
            || curr_surface == "냐"
            || curr_surface == "냐고")
            && curr_pos == "EF"
            && next_surface == "고"
            && next_pos == "EC"
        {
            quote_ec_merge_indices.push(i);
        }
    }

    for idx in quote_ec_merge_indices.into_iter().rev() {
        let merged_surface = format!("{}{}", tokens[idx].surface, tokens[idx + 1].surface);
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&merged_surface, "EC", start, end);
        tokens.remove(idx + 1);
    }

    // 109차 보정: "ㄹ까/EF" → "ㄹ까/EC" when followed by "하다"
    // "갈까 하다" = 가/VV ㄹ까/EC 하/VV 다/EF
    let mut ef_to_ec_indices: Vec<usize> = Vec::new();
    if tokens.len() >= 2 {
        for i in 0..tokens.len() - 1 {
            let curr_surface = tokens[i].surface.clone();
            let curr_pos = tokens[i].pos.clone();
            let next_surface = tokens[i + 1].surface.clone();

            // "ㄹ까/EF + 하/VV" → "ㄹ까/EC"
            if curr_surface == "ㄹ까" && curr_pos == "EF" && next_surface == "하" {
                ef_to_ec_indices.push(i);
            }

            // "ㄹ지/EF + 모르/VV" → "ㄹ지/EC"
            // 220차: "ㄹ지/EF"가 문장 중간이면 EC (진학할지 취업을)
            if curr_surface == "ㄹ지" && curr_pos == "EF" {
                ef_to_ec_indices.push(i);
            }
        }
    }
    for idx in ef_to_ec_indices {
        tokens[idx].pos = "EC".to_string();
    }

    // 110차 보정: "게/NNB" → "게/EC" (동사 뒤에서 연결어미)
    // "오게" = 오/VV 게/EC
    // "가게" = 가/VV 게/EC (단, "가게/NNG"도 있으므로 주의)
    let mut ge_ec_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len() {
        let prev_pos = tokens[i - 1].pos.clone();
        let curr_surface = tokens[i].surface.clone();
        let curr_pos = tokens[i].pos.clone();

        // VV/VA 뒤의 "게/NNB"는 EC
        if (prev_pos == "VV" || prev_pos == "VA" || prev_pos == "VX")
            && curr_surface == "게"
            && curr_pos == "NNB"
        {
            ge_ec_indices.push(i);
        }
    }
    for idx in ge_ec_indices {
        tokens[idx].pos = "EC".to_string();
    }

    // 111차 보정: NNG로 오분석된 "가고/보고/하고" 등을 VV+EC로 분리
    // "가고 있다" = "가고/NNG 있/VA" → "가/VV 고/EC 있/VX"
    let vv_ec_patterns: std::collections::HashMap<&str, (&str, &str)> = [
        // "VV어간 + 고" 패턴
        ("가고", ("가", "VV")),
        ("오고", ("오", "VV")),
        ("보고", ("보", "VV")),
        ("하고", ("하", "VV")),
        ("되고", ("되", "VV")),
        ("먹고", ("먹", "VV")),
        ("읽고", ("읽", "VV")),
        ("쓰고", ("쓰", "VV")),
        ("주고", ("주", "VV")),
        ("사고", ("사", "VV")),
        ("자고", ("자", "VV")),
        ("나고", ("나", "VV")),
        ("서고", ("서", "VV")),
        ("살고", ("살", "VV")),
        ("알고", ("알", "VV")),
        ("듣고", ("듣", "VV")),
        ("걷고", ("걷", "VV")),
        ("만나고", ("만나", "VV")),
        ("기다리고", ("기다리", "VV")),
    ]
    .into_iter()
    .collect();

    let mut vv_ec_splits: Vec<(usize, String, String)> = Vec::new();
    for i in 0..tokens.len() {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // NNG로 분석된 경우에만 VV+EC로 분리
        if curr_pos == "NNG" {
            if let Some((stem, pos)) = vv_ec_patterns.get(curr_surface.as_str()) {
                // 다음 토큰이 VX 또는 VV+EF인 경우만 분리 (있다, 싶다 등)
                if i + 1 < tokens.len() {
                    let next_surface = &tokens[i + 1].surface;
                    let next_pos = &tokens[i + 1].pos;
                    if next_surface == "있"
                        || next_surface == "싶"
                        || next_surface == "없"
                        || next_pos == "VX"
                        || next_pos == "VA"
                        || next_pos == "VV"
                        || next_pos == "NNP"
                    // "싶어요/NNP" 같은 오분석
                    {
                        vv_ec_splits.push((i, (*stem).to_string(), (*pos).to_string()));
                    }
                }
            }
        }
    }

    // 뒤에서부터 처리하여 인덱스 밀림 방지
    for (idx, stem, pos) in vv_ec_splits.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        let mid = start + stem.len();

        // 원래 토큰을 VV 어간으로 변환
        tokens[idx] = SejongToken::new(&stem, &pos, start, mid);
        // "고/EC" 토큰 삽입
        tokens.insert(idx + 1, SejongToken::new("고", "EC", mid, end));

        // 다음 토큰의 "있/VA" → "있/VX" 변환
        if idx + 2 < tokens.len() && tokens[idx + 2].surface == "있" && tokens[idx + 2].pos == "VA"
        {
            tokens[idx + 2].pos = "VX".to_string();
        }
    }

    // 112차 보정: "보고 싶어요" 패턴에서 "싶어요/NNP" 분리
    // "보고/NNG 싶어요/NNP" → "보/VV 고/EC 싶/VX 어요/EF"
    let mut nnp_vx_ef_splits: Vec<usize> = Vec::new();
    for i in 0..tokens.len() {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // "싶어요/NNP"를 VX+EF로 분리
        if curr_pos == "NNP" && curr_surface == "싶어요" {
            nnp_vx_ef_splits.push(i);
        }
    }

    for idx in nnp_vx_ef_splits.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        let mid = start + "싶".len();

        tokens[idx] = SejongToken::new("싶", "VX", start, mid);
        tokens.insert(idx + 1, SejongToken::new("어요", "EF", mid, end));
    }
}
