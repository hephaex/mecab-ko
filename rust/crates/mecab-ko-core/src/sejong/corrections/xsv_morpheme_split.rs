use crate::sejong::hangul::has_jongseong;
use crate::sejong::types::SejongToken;

/// 113~134차(+128, 235): 복합 EC 병합, EF 정규화, NNP 분리, 청유형/의문형 분리
pub(super) fn apply_xsv_morpheme_split_corrections(tokens: &mut Vec<SejongToken>) {
    // 113차 보정: 복합 연결어미 병합
    // "고/EC + 나/NP + 서/JKB" → "고나서/EC"
    // "면/EC + 서/JKB" → "면서/EC"
    let mut compound_ec_merges: Vec<(usize, String)> = Vec::new();
    if tokens.len() >= 3 {
        for i in 0..tokens.len() - 2 {
            let t0 = &tokens[i];
            let t1 = &tokens[i + 1];
            let t2 = &tokens[i + 2];

            // "고/EC + 나/NP + 서/JKB" → "고나서/EC"
            if t0.surface == "고"
                && t0.pos == "EC"
                && t1.surface == "나"
                && (t1.pos == "NP" || t1.pos == "VV")
                && t2.surface == "서"
                && (t2.pos == "JKB" || t2.pos == "EC")
            {
                compound_ec_merges.push((i, "고나서".to_string()));
            }
        }
    }

    for (idx, merged) in compound_ec_merges.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 2].end_pos;
        tokens[idx] = SejongToken::new(&merged, "EC", start, end);
        tokens.remove(idx + 2);
        tokens.remove(idx + 1);
    }

    // 114차 보정: "으면서/EF" → "으면서/EC" (문장 중간 연결어미)
    // "먹으면서 갔다"에서 "먹/VV 으면서/EC"
    // 128차 확장: "아서/EF", "어서/EF" 등 추가
    let mut ef_to_ec_final: Vec<usize> = Vec::new();
    for i in 0..tokens.len() {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // "면서", "아서", "어서" 계열이 EF로 분석된 경우 EC로 변환
        if curr_pos == "EF"
            && (curr_surface == "으면서"
                || curr_surface == "면서"
                || curr_surface == "으며"
                || curr_surface == "며"
                || curr_surface == "아서"  // 128차 추가
                || curr_surface == "어서"  // 128차 추가
                || curr_surface == "니까"  // 128차 추가
                || curr_surface == "으니까"  // 128차 추가
                || curr_surface == "니"  // 128차 추가
                || curr_surface == "으니")
        // 128차 추가
        {
            // 마지막 토큰이 아닌 경우만 EC로 변환
            if i + 1 < tokens.len() {
                ef_to_ec_final.push(i);
            }
        }
    }
    for idx in ef_to_ec_final {
        tokens[idx].pos = "EC".to_string();
    }

    // 126차 보정: "어/EF" + VX → "어/EC" + VX
    // "먹어 버렸다"에서 "먹/VV 어/EF 버리/VX" → "먹/VV 어/EC 버리/VX"
    // 보조용언(VX) 앞의 연결어미는 EC여야 함
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_pos = &tokens[i + 1].pos;

        // "어/아/여" + VX 패턴
        if curr_pos == "EF"
            && (curr_surface == "어" || curr_surface == "아" || curr_surface == "여")
            && next_pos == "VX"
        {
            tokens[i].pos = "EC".to_string();
        }
    }

    // 115차 보정: "보이/NNG + 이/VCP + 고/EC" → "보이/VV + 고/EC"
    // MeCab이 "보이고"를 NNG+VCP+EC로 분석하는 문제 수정
    // "보이고 있다" = "보이/VV 고/EC 있/VX"
    // 패턴: NNG + "이/VCP" + "고/EC" → NNG를 VV로 변환하고 VCP 삭제
    // 동사 어간 "보이", "들이", "붙이" 등
    let verb_stems_with_i: std::collections::HashSet<&str> = [
        "보이", "들이", "붙이", "놓이", "쓰이", "덮이", "갈이", "꺾이",
    ]
    .into_iter()
    .collect();

    // VV 변환 및 VCP 삭제할 인덱스 수집
    let mut vv_convert_and_vcp_delete: Vec<(usize, usize)> = Vec::new(); // (NNG idx, VCP idx)
    for i in 0..tokens.len().saturating_sub(2) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;
        let next2_pos = &tokens[i + 2].pos;

        // NNG + "이/VCP" + EC 패턴
        if curr_pos == "NNG"
            && verb_stems_with_i.contains(curr_surface.as_str())
            && next_surface == "이"
            && next_pos == "VCP"
            && next2_pos == "EC"
        {
            vv_convert_and_vcp_delete.push((i, i + 1));
        }
    }

    // 역순으로 처리 (인덱스 변화 방지)
    for (nng_idx, vcp_idx) in vv_convert_and_vcp_delete.into_iter().rev() {
        tokens[nng_idx].pos = "VV".to_string();
        tokens.remove(vcp_idx);
    }

    // 116차 보정: "준/VV+ETM" → "주/VX + ㄴ/ETM"
    // "해 준 식당"에서 "준/VV+ETM"을 "주/VX + ㄴ/ETM"으로 분리
    // 앞에 "어/EC" 또는 "아/EC"가 있는 경우만
    let mut jun_split_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len() {
        let prev_pos = &tokens[i - 1].pos;
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // EC 뒤의 "준"이 VV+ETM 또는 VV로 분석된 경우 VX+ETM으로 변환
        if prev_pos == "EC"
            && curr_surface == "준"
            && (curr_pos == "VV" || curr_pos.starts_with("VV+"))
        {
            jun_split_indices.push(i);
        }
    }
    for idx in jun_split_indices.into_iter().rev() {
        // "준/VV+ETM" → "주/VX" + "ㄴ/ETM"
        tokens[idx].surface = "주".to_string();
        tokens[idx].pos = "VX".to_string();
        // ETM은 이미 분리되어 있거나 다음 토큰으로 처리됨
    }

    // 119차 보정: NNG + "다/NNG" → NNG + "이/VCP + 다/EF"
    // "레게노다"에서 "레게노/NNG 다/NNG" → "레게노/NNG 이/VCP 다/EF"
    // 신조어+이다 패턴 보정
    let mut vcp_insert_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len() {
        let prev_pos = &tokens[i - 1].pos;
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // NNG + "다/NNG" 패턴 → VCP 삽입 필요
        if (prev_pos == "NNG" || prev_pos == "NNP") && curr_surface == "다" && curr_pos == "NNG" {
            // 이전 토큰의 마지막 글자 확인 (받침 있으면 이/VCP, 없으면 다/EF만)
            let prev_surface = &tokens[i - 1].surface;
            if let Some(last_char) = prev_surface.chars().last() {
                // 받침 없는 경우만 VCP 삽입 (레게노, 존맛탱 등)
                // 받침 있는 경우는 다른 패턴일 수 있음
                if !has_jongseong(last_char) {
                    vcp_insert_indices.push(i);
                }
            }
        }
    }
    for idx in vcp_insert_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        // "다/NNG" → "이/VCP" + "다/EF"
        tokens[idx] = SejongToken::new("다", "EF", start, start);
        tokens.insert(idx, SejongToken::new("이", "VCP", start, start));
    }

    // 120차 보정: NNG + "이/VCP + 오/EC" → VV + "아요/EF" (문장 끝에서)
    // "만나요"에서 "만나/NNG 이/VCP 오/EC" → "만나/VV 아요/EF"
    // MeCab이 "요/VCP+EC"를 분리하면 "이/VCP + 오/EC"가 됨
    // 받침 없는 NNG + 이/VCP + 오/EC 패턴
    if tokens.len() >= 3 {
        let last_idx = tokens.len() - 1;
        let mid_idx = tokens.len() - 2;
        let nng_idx = tokens.len() - 3;

        // 마지막이 "오/EC", 중간이 "이/VCP", 첫번째가 NNG인 경우
        if tokens[last_idx].surface == "오"
            && tokens[last_idx].pos == "EC"
            && tokens[mid_idx].surface == "이"
            && tokens[mid_idx].pos == "VCP"
            && tokens[nng_idx].pos == "NNG"
        {
            // 213차: 명사+보조사 패턴 예외 처리
            // "진짜/NNG + 요" → "진짜/NNG 요/JX" (VV로 변환하지 않음)
            let nng_exceptions = ["진짜", "정말", "별로"];
            if nng_exceptions.contains(&tokens[nng_idx].surface.as_str()) {
                // NNG 유지, VCP+EC → JX 변환
                let start = tokens[mid_idx].start_pos;
                let end = tokens[last_idx].end_pos;
                tokens[mid_idx] = SejongToken::new("요", "JX", start, end);
                tokens.remove(last_idx);
            }
            // NNG의 마지막 글자 확인
            else if let Some(last_char) = tokens[nng_idx].surface.chars().last() {
                // 받침 없는 경우 (만나, 보다 등)
                if !has_jongseong(last_char) {
                    // NNG → VV, 이/VCP + 오/EC → 아요/EF
                    tokens[nng_idx].pos = "VV".to_string();
                    let start = tokens[mid_idx].start_pos;
                    let end = tokens[last_idx].end_pos;
                    tokens[mid_idx] = SejongToken::new("아요", "EF", start, end);
                    tokens.remove(last_idx);
                }
            }
        }
    }

    // 235차 보정: NP + "예/VV" + "아요/EF" → NP + "이/VCP" + "에요/EF"
    // sample.tsv 기준: "뭐예요" → "뭐/NP 이/VCP 에요/EF"
    // MeCab이 "예/NNG + 요/VCP+EC"로 분석하여 "예/VV 아요/EF"로 분리되는 오류 수정
    let mut np_yeyo_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len().saturating_sub(1) {
        if tokens[i - 1].pos == "NP"
            && tokens[i].surface == "예"
            && tokens[i].pos == "VV"
            && tokens[i + 1].surface == "아요"
            && tokens[i + 1].pos == "EF"
        {
            np_yeyo_indices.push(i);
        }
    }

    for idx in np_yeyo_indices.into_iter().rev() {
        // "예/VV + 아요/EF" → "이/VCP + 에요/EF"로 교체
        let start1 = tokens[idx].start_pos;
        let end1 = tokens[idx].end_pos;
        tokens[idx] = SejongToken::new("이", "VCP", start1, end1);

        let start2 = tokens[idx + 1].start_pos;
        let end2 = tokens[idx + 1].end_pos;
        tokens[idx + 1] = SejongToken::new("에요", "EF", start2, end2);
    }

    // 122차 보정: 문장 끝 "ᆯ래요/EC" → "ᆯ래요/EF"
    // "할래요" = "하/VV ᆯ래요/EF" (not EC)
    // "갈래요" = "가/VV ᆯ래요/EF"
    // Note: ᆯ (U+11AF) is jongseong rieul, vs ㄹ (U+3139)
    if let Some(last) = tokens.last_mut() {
        if (last.surface == "ᆯ래요" || last.surface == "ㄹ래요") && last.pos == "EC" {
            last.pos = "EF".to_string();
        }
    }

    // 125차 보정: 문장 끝 "ㄹ까요/EC" → "ㄹ까요/EF"
    // "올까요" = "오/VV ㄹ까요/EF" (not EC)
    // "할까요" = "하/VV ㄹ까요/EF"
    if let Some(last) = tokens.last_mut() {
        if (last.surface == "ᆯ까요" || last.surface == "ㄹ까요") && last.pos == "EC" {
            last.pos = "EF".to_string();
        }
    }

    // 127차 보정: 형용사 기본형 NNP → VA + 다/EF
    // "크다/NNP" → "크/VA 다/EF"
    // "예쁘다/NNP" → "예쁘/VA 다/EF"
    // MeCab이 형용사 기본형을 NNP로 잘못 분석하는 경우
    let adjective_stems: std::collections::HashSet<&str> = [
        "크",
        "작",
        "예쁘",
        "귀엽",
        "멋지",
        "아름답",
        "못생기",
        "높",
        "낮",
        "길",
        "짧",
        "넓",
        "좁",
        "두껍",
        "얇",
        "무겁",
        "가볍",
        "빠르",
        "느리",
        "밝",
        "어두",
        "덥",
        "춥",
        "시원하",
        "따뜻하",
        "뜨겁",
        "차갑",
    ]
    .into_iter()
    .collect();

    let mut nnp_to_va_split: Vec<usize> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if token.pos == "NNP" && token.surface.ends_with("다") {
            // "다"를 제거하고 어간 확인
            let stem: String = token
                .surface
                .chars()
                .take(token.surface.chars().count() - 1)
                .collect();
            if adjective_stems.contains(stem.as_str()) {
                nnp_to_va_split.push(i);
            }
        }
    }

    for idx in nnp_to_va_split.into_iter().rev() {
        let surface = &tokens[idx].surface;
        let stem: String = surface.chars().take(surface.chars().count() - 1).collect();
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;

        // NNP → VA + 다/EF
        tokens[idx].surface = stem;
        tokens[idx].pos = "VA".to_string();
        tokens.insert(idx + 1, SejongToken::new("다", "EF", start, end));
    }

    // 129차 보정: 일반명사가 NNP로 분석된 경우 NNG로 변환
    // "인공지능/NNP" → "인공지능/NNG"
    // MeCab이 일반명사를 고유명사로 잘못 분석하는 경우
    let common_nouns: std::collections::HashSet<&str> = [
        "인공지능",
        "머신러닝",
        "딥러닝",
        "빅데이터",
        "클라우드",
        "인터넷",
        "컴퓨터",
        "소프트웨어",
        "하드웨어",
        "프로그램",
        "알고리즘",
        "데이터베이스",
        "네트워크",
        "서버",
        "클라이언트",
    ]
    .into_iter()
    .collect();

    for token in tokens.iter_mut() {
        if token.pos == "NNP" && common_nouns.contains(token.surface.as_str()) {
            token.pos = "NNG".to_string();
        }
    }

    // 130차 보정: 의문사 + 가/JKS + 니/NP → 의문사 + VV + 니/EF
    // "어디 가니"에서 "어디/NP 가/JKS 니/NP" → "어디/NP 가/VV 니/EF"
    // MeCab이 의문형 종결어미 "니"를 NP로 잘못 분석하는 경우
    let question_words: std::collections::HashSet<&str> =
        ["어디", "뭐", "언제", "누구", "왜", "어떻게", "무엇", "어느"]
            .into_iter()
            .collect();

    if tokens.len() >= 3 {
        for i in 0..tokens.len() - 2 {
            let first_surface = &tokens[i].surface;
            let first_pos = &tokens[i].pos;
            let second_surface = &tokens[i + 1].surface;
            let second_pos = &tokens[i + 1].pos;
            let third_surface = &tokens[i + 2].surface;
            let third_pos = &tokens[i + 2].pos;

            // 의문사 + 가/JKS + 니/NP 패턴
            if first_pos == "NP"
                && question_words.contains(first_surface.as_str())
                && second_surface == "가"
                && second_pos == "JKS"
                && third_surface == "니"
                && third_pos == "NP"
            {
                tokens[i + 1].pos = "VV".to_string();
                tokens[i + 2].pos = "EF".to_string();
            }
        }
    }

    // 130차 보정 추가: 의문사 + 동사니/NNG → 의문사 + VV + 니/EF
    // "언제 오니"에서 "언제/NP 오니/NNG" → "언제/NP 오/VV 니/EF"
    let mut split_indices: Vec<(usize, String)> = Vec::new();
    if tokens.len() >= 2 {
        for i in 0..tokens.len() - 1 {
            let first_surface = &tokens[i].surface;
            let first_pos = &tokens[i].pos;
            let second_surface = &tokens[i + 1].surface;
            let second_pos = &tokens[i + 1].pos;

            // 의문사 뒤에 "~니" 형태의 NNG가 오는 경우
            if first_pos == "NP"
                && question_words.contains(first_surface.as_str())
                && (second_pos == "NNG" || second_pos == "NNP")
                && second_surface.ends_with("니")
                && second_surface.chars().count() >= 2
            {
                let stem: String = second_surface
                    .chars()
                    .take(second_surface.chars().count() - 1)
                    .collect();
                split_indices.push((i + 1, stem));
            }
        }
    }

    for (idx, stem) in split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        tokens[idx].surface = stem;
        tokens[idx].pos = "VV".to_string();
        tokens.insert(idx + 1, SejongToken::new("니", "EF", start, end));
    }

    // 131차 보정: 의문사 포함 NNP 분리
    // "뭐하니/NNP" → "뭐/NP 하/VV 니/EF"
    // MeCab이 띄어쓰기 무시하고 붙여서 NNP로 분석한 경우
    let mut split_compound_indices: Vec<(usize, String, String, String)> = Vec::new();
    for i in 0..tokens.len() {
        let surface = &tokens[i].surface;
        let pos = &tokens[i].pos;

        if pos == "NNP" && surface.ends_with("니") && surface.chars().count() >= 3 {
            // 의문사로 시작하는지 확인
            for qword in &["뭐", "뭘", "언제", "어디", "누가", "누구"] {
                if surface.starts_with(qword) {
                    // qword.chars().count()로 글자 수 기준으로 skip
                    let qword_char_count = qword.chars().count();
                    let rest: String = surface.chars().skip(qword_char_count).collect();
                    if rest.ends_with("니") && rest.chars().count() >= 2 {
                        let verb_stem: String =
                            rest.chars().take(rest.chars().count() - 1).collect();
                        split_compound_indices.push((
                            i,
                            (*qword).to_string(),
                            verb_stem,
                            "니".to_string(),
                        ));
                        break;
                    }
                }
            }
        }
    }

    for (idx, qword, verb_stem, ending) in split_compound_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        // 원본 토큰을 의문사로 변경
        tokens[idx].surface = qword;
        tokens[idx].pos = "NP".to_string();
        // 동사 어간 삽입
        tokens.insert(idx + 1, SejongToken::new(&verb_stem, "VV", start, end));
        // 어미 삽입
        tokens.insert(idx + 2, SejongToken::new(&ending, "EF", start, end));
    }

    // 132차 보정: 청유형 "~자" NNG 분리
    // "가자/NNG" → "가/VV + 자/EF"
    // 2글자 NNG가 "자"로 끝나고 앞 글자가 동사 어간인 경우
    // 134차: 받침 있는 동사 어간 추가 (먹, 읽, 잡, 걷, 듣, 짓 등)
    let imperative_verbs: std::collections::HashSet<&str> = [
        // 받침 없는 동사 어간
        "가", "오", "보", "사", "자", "두", "주", "타", "서", "나", "하",
        // 받침 있는 동사 어간 (134차)
        "먹", "읽", "잡", "걷", "듣", "짓", "넣", "씻", "입", "웃",
    ]
    .into_iter()
    .collect();

    let mut split_imperative_indices: Vec<(usize, String)> = Vec::new();
    for i in 0..tokens.len() {
        let surface = &tokens[i].surface;
        let pos = &tokens[i].pos;

        if pos == "NNG" && surface.ends_with("자") && surface.chars().count() == 2 {
            let stem: String = surface.chars().take(1).collect();
            if imperative_verbs.contains(stem.as_str()) {
                split_imperative_indices.push((i, stem));
            }
        }
    }

    for (idx, stem) in split_imperative_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        tokens[idx].surface = stem;
        tokens[idx].pos = "VV".to_string();
        tokens.insert(idx + 1, SejongToken::new("자", "EF", start, end));
    }
}
