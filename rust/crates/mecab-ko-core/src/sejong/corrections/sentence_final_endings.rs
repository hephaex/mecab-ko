use crate::sejong::types::SejongToken;

/// 149~259차: 문장 종결·EC/EF 변환 보정 (후반부)
///
/// 의문대명사 NP 변환, EP 표면형 정규화, 합성 형용사,
/// 수사 병합, 종결어미 정규화, 의문문 EF 변환 등
/// 149~259차 보정 패스를 포함합니다.
pub(super) fn apply_sentence_final_endings_corrections(tokens: &mut Vec<SejongToken>) {
    // 149차 보정: "VCP + 시/EP + 라고/EC" → "VCP + 라고/EC" (잘못된 EP 제거)
    // "학생이라고" = "학생/NNG 이/VCP 라고/EC" (시/EP 불필요)
    let mut remove_si_ep_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(2) {
        if tokens[i].pos == "VCP"
            && tokens[i + 1].surface == "시"
            && tokens[i + 1].pos == "EP"
            && tokens[i + 2].surface == "라고"
            && tokens[i + 2].pos == "EC"
        {
            remove_si_ep_indices.push(i + 1);
        }
    }

    for idx in remove_si_ep_indices.into_iter().rev() {
        tokens.remove(idx);
    }

    // 149차 보정: "ㄴ/ETM + 다/NNG" (문장 중간) → "ㄴ다/EF"
    // "간다 온다" = "가/VV ㄴ다/EF 오/VV ㄴ다/EF"
    // 조건: 다음 토큰이 VV인 경우 (문장 중간)
    let mut nda_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(2) {
        if (tokens[i].surface == "ㄴ" || tokens[i].surface == "는")
            && tokens[i].pos == "ETM"
            && tokens[i + 1].surface == "다"
            && tokens[i + 1].pos == "NNG"
            && i + 2 < tokens.len()
            && (tokens[i + 2].pos == "VV" || tokens[i + 2].pos == "VA")
        {
            nda_merge_indices.push(i);
        }
    }

    for idx in nda_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        let merged_surface = format!("{}다", tokens[idx].surface);
        tokens[idx] = SejongToken::new(&merged_surface, "EF", start, end);
        tokens.remove(idx + 1);
    }

    // 152차: "큰/VA+ETM + 집/NNG" → "큰/XPN + 집/NNG"
    // 관형사형 어미가 붙은 형용사가 접두사처럼 사용될 때
    let xpn_prefixes: std::collections::HashSet<&str> = ["큰", "작은", "새", "헌", "젊은", "늙은"]
        .into_iter()
        .collect();

    for i in 0..tokens.len().saturating_sub(1) {
        if xpn_prefixes.contains(tokens[i].surface.as_str())
            && (tokens[i].pos == "VA" || tokens[i].pos == "ETM")
            && tokens[i + 1].pos == "NNG"
        {
            // "큰/VA" 또는 "ㄴ/ETM" 이후 "집/NNG" → XPN + NNG
            // VA+ETM 분리된 경우 (크/VA + ㄴ/ETM) → 큰/XPN으로 병합 필요
        }
    }

    // VA+ETM 분리 후 재병합이 필요한 패턴: "크/VA + ㄴ/ETM + 집/NNG" → "큰/XPN + 집/NNG"
    let mut xpn_merge_indices: Vec<(usize, String)> = Vec::new();
    let xpn_stem_map: std::collections::HashMap<&str, &str> =
        [("크", "큰"), ("작", "작은")].into_iter().collect();

    for i in 0..tokens.len().saturating_sub(2) {
        if tokens[i].pos == "VA"
            && tokens[i + 1].surface == "ㄴ"
            && tokens[i + 1].pos == "ETM"
            && tokens[i + 2].pos == "NNG"
        {
            if let Some(merged) = xpn_stem_map.get(tokens[i].surface.as_str()) {
                xpn_merge_indices.push((i, (*merged).to_string()));
            }
        }
    }

    for (idx, merged) in xpn_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&merged, "XPN", start, end);
        tokens.remove(idx + 1);
    }

    // 153차 보정: 의존명사 NNB 패턴 수정
    // "채/VV + 어/EC" → "채/NNB" (눈을 감은 채로, 그 채로)
    // "대/NNG + 로/JKB" → "대로/NNB" (있는 대로, 원하는 대로)
    // "따르/VV + 어/EC" → "따라/NNB" (결과에 따라, 상황에 따라)
    let mut nnb_fix_indices: Vec<(usize, &str)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        // "채/VV + 어/EC" → "채/NNB"
        if tokens[i].surface == "채"
            && tokens[i].pos == "VV"
            && tokens[i + 1].surface == "어"
            && tokens[i + 1].pos == "EC"
        {
            nnb_fix_indices.push((i, "채"));
        }
        // "대/NNG + 로/JKB" → "대로/NNB"
        if tokens[i].surface == "대"
            && tokens[i].pos == "NNG"
            && tokens[i + 1].surface == "로"
            && tokens[i + 1].pos == "JKB"
        {
            nnb_fix_indices.push((i, "대로"));
        }
        // "따르/VV + 어/EC" → "따라/NNB"
        if tokens[i].surface == "따르"
            && tokens[i].pos == "VV"
            && tokens[i + 1].surface == "어"
            && tokens[i + 1].pos == "EC"
        {
            // 문맥 확인: 앞에 ETM이 있으면 의존명사
            if i > 0 && tokens[i - 1].pos == "ETM" {
                nnb_fix_indices.push((i, "따라"));
            }
        }
    }

    for (idx, surface) in nnb_fix_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(surface, "NNB", start, end);
        tokens.remove(idx + 1);
    }

    // 154차 보정: 문장 끝 "다/NNG" → "다/EF"
    // "먹다", "가다" 등 동사 원형의 "다"가 NNG로 분석될 때
    // 문장 끝이거나 앞에 VV/VA/VX가 있으면 EF로 변환
    if let Some(last) = tokens.last_mut() {
        if last.surface == "다" && last.pos == "NNG" {
            // 문장 마지막 "다" → EF
            last.pos = "EF".to_string();
        }
    }

    // 문장 중간의 "VV/VA/VX + 다/NNG" → "VV/VA/VX + 다/EF"
    for i in 1..tokens.len() {
        if tokens[i].surface == "다"
            && tokens[i].pos == "NNG"
            && (tokens[i - 1].pos == "VV" || tokens[i - 1].pos == "VA" || tokens[i - 1].pos == "VX")
        {
            tokens[i].pos = "EF".to_string();
        }
    }

    // 155차 보정: XSV/EP 뒤의 "다/NNG" → "다/EF"
    // "발표했다", "개선됐다" 등 XSV+EP+다 패턴
    for i in 1..tokens.len() {
        if tokens[i].surface == "다"
            && tokens[i].pos == "NNG"
            && (tokens[i - 1].pos == "XSV" || tokens[i - 1].pos == "EP")
        {
            tokens[i].pos = "EF".to_string();
        }
    }

    // VCP 뒤의 "다/NNG" → "다/EF" (이다)
    for i in 1..tokens.len() {
        if tokens[i].surface == "다" && tokens[i].pos == "NNG" && tokens[i - 1].pos == "VCP" {
            tokens[i].pos = "EF".to_string();
        }
    }

    // 156차 보정: 의문대명사 NP 변환
    // "얼마/NNG + 이/VCP" → "얼마/NP + 이/VCP"
    // "뭐", "무엇", "누구", "어디", "언제", "어느" 등
    let question_pronouns = [
        "얼마",
        "뭐",
        "무엇",
        "누구",
        "어디",
        "언제",
        "어느",
        "왜",
        "어떻게",
    ];
    for token in tokens.iter_mut() {
        if token.pos == "NNG" && question_pronouns.contains(&token.surface.as_str()) {
            token.pos = "NP".to_string();
        }
    }

    // 157차 보정: EP 표면형 정규화
    // "ㅓㅆ/EP" → "었/EP", "ㅏㅆ/EP" → "았/EP"
    for token in tokens.iter_mut() {
        if token.pos == "EP" {
            if token.surface == "ㅓㅆ" {
                token.surface = "었".to_string();
            } else if token.surface == "ㅏㅆ" {
                token.surface = "았".to_string();
            }
        }
    }

    // 158차 보정: 합성 형용사 VA 병합
    // "NNG + 있/VV" → "NNG있/VA", "NNG + 없/VX" → "NNG없/VA"
    // 세종 태깅: "재미있다" = "재미있/VA + 다/EF"
    let compound_va_nouns = [
        "재미", "맛", "멋", "값", "뜻", "힘", // 기본
        "흥미", "의미", "가치", "효과", "보람", // 추가
        "관심", "정", "맥", "볼", // 추가 (관심있다, 정없다 등)
    ];
    let mut va_merge_indices: Vec<(usize, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        if compound_va_nouns.contains(&tokens[i].surface.as_str())
            && tokens[i].pos == "NNG"
            && (tokens[i + 1].surface == "있" || tokens[i + 1].surface == "없")
            && (tokens[i + 1].pos == "VV" || tokens[i + 1].pos == "VX")
        {
            let merged = format!("{}{}", tokens[i].surface, tokens[i + 1].surface);
            va_merge_indices.push((i, merged));
        }
    }

    for (idx, merged) in va_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&merged, "VA", start, end);
        tokens.remove(idx + 1);
    }

    // 159차 보정: "VA + ㄹ/ETM + EF" 잘못된 ETM 제거
    // "힘들어요" = "힘들/VA 어요/EF" (not "힘들/VA ㄹ/ETM 어요/EF")
    // VA+ETM 분해에서 잘못 삽입된 ETM 제거
    let mut spurious_etm_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len().saturating_sub(1) {
        if tokens[i].surface == "ㄹ"
            && tokens[i].pos == "ETM"
            && tokens[i - 1].pos == "VA"
            && tokens[i + 1].pos == "EF"
        {
            // VA 다음에 바로 EF가 오면 ㄹ/ETM은 잘못 삽입된 것
            spurious_etm_indices.push(i);
        }
    }

    for idx in spurious_etm_indices.into_iter().rev() {
        tokens.remove(idx);
    }

    // 160차 보정: "VV + 어디/NP + 서/JKB" → "VV + 어서/EC"
    // MeCab이 "어서"를 "어디+서"로 잘못 분석하는 버그
    let mut eoseo_fix_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(2) {
        if tokens[i].pos == "VV"
            && tokens[i + 1].surface == "어디"
            && tokens[i + 1].pos == "NP"
            && tokens[i + 2].surface == "서"
            && tokens[i + 2].pos == "JKB"
        {
            eoseo_fix_indices.push(i + 1);
        }
    }

    for idx in eoseo_fix_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new("어서", "EC", start, end);
        tokens.remove(idx + 1);
    }

    // 161차 보정: 문장 끝 "ㅏ/EC" → "아/EF", "ㅓ/EC" → "어/EF"
    // VX 뒤의 축약 모음을 정규화하고 EC → EF 변환
    // 203차: "어/EC", "아/EC"도 EF로 변환
    // 206차: "어/IC", "아/IC"도 EF로 변환
    if let Some(last) = tokens.last_mut() {
        if last.pos == "EC" || last.pos == "IC" {
            if last.surface == "ㅏ" || last.surface == "아" {
                last.surface = "아".to_string();
                last.pos = "EF".to_string();
            } else if last.surface == "ㅓ" || last.surface == "어" {
                last.surface = "어".to_string();
                last.pos = "EF".to_string();
            }
        }
    }

    // 265차 보정: VV/VA 뒤의 문장 끝 "네/IC" → "네/EF"
    // "킹받네" = "킹받/VV 네/EF" (종결어미)
    // 동사/형용사 뒤의 "네"는 종결어미
    if tokens.len() >= 2 {
        let last_idx = tokens.len() - 1;
        if tokens[last_idx].surface == "네"
            && tokens[last_idx].pos == "IC"
            && (tokens[last_idx - 1].pos == "VV"
                || tokens[last_idx - 1].pos == "VA"
                || tokens[last_idx - 1].pos == "VX")
        {
            tokens[last_idx].pos = "EF".to_string();
        }
    }

    // 163차 보정: EF 축약 모음 정규화
    // "ㅔ요/EF" → "에요/EF", "ㅐ요/EF" → "애요/EF"
    for token in tokens.iter_mut() {
        if token.pos == "EF" {
            match token.surface.as_str() {
                "ㅔ요" => token.surface = "에요".to_string(),
                "ㅐ요" => token.surface = "애요".to_string(),
                "ㅔ" => token.surface = "에".to_string(),
                "ㅐ" => token.surface = "애".to_string(),
                _ => {}
            }
        }
    }

    // 204차 보정: 문장 끝 "ㄴ데요/EC" → "ㄴ데요/EF"
    // "TMI인데요" = "TMI/NNG 이/VCP ㄴ데요/EF"
    // 문장 마지막 "ㄴ데요", "ᆫ데요" 는 종결어미
    if let Some(last) = tokens.last_mut() {
        if last.pos == "EC" {
            if last.surface == "ᆫ데요" || last.surface == "ㄴ데요" {
                last.surface = "ㄴ데요".to_string();
                last.pos = "EF".to_string();
            } else if last.surface == "ᆫ데" || last.surface == "ㄴ데" {
                last.surface = "ㄴ데".to_string();
                last.pos = "EF".to_string();
            } else if last.surface == "네" {
                // "킹받네" = "킹받/VV 네/EF"
                last.pos = "EF".to_string();
            }
        }
    }

    // 164차 보정: NR 수사 병합
    // "삼/NR + 십/NR" → "삼십/NR", "이/NR + 백/NR" → "이백/NR"
    // 십/백/천/만 앞의 수사를 병합
    let mut idx = 0;
    while idx + 1 < tokens.len() {
        if tokens[idx].pos == "NR" && tokens[idx + 1].pos == "NR" {
            let second = tokens[idx + 1].surface.as_str();
            // 십, 백, 천, 만 뒤에 올 수 있는 1자리 수사
            if ["십", "백", "천", "만"].contains(&second) {
                let first = tokens[idx].surface.clone();
                // 일, 이, 삼, 사, 오, 육, 칠, 팔, 구 등 1자리 수사
                if ["일", "이", "삼", "사", "오", "육", "칠", "팔", "구"].contains(&first.as_str())
                {
                    // 병합
                    tokens[idx].surface = format!("{first}{second}");
                    tokens.remove(idx + 1);
                    continue;
                }
            }
        }
        idx += 1;
    }

    // 189차: 한자 숫자 NR → SN 변환 (164차 병합 이후 실행!)
    // "일 이 삼" = "일/SN 이/SN 삼/SN"
    // 한자 숫자는 SN(숫자), 아라비아 숫자도 SN
    // 주의: 병합된 "삼십/NR"은 NR 유지 (sample.tsv 기준)
    let single_sino_numerals: std::collections::HashSet<&str> = [
        "일", "이", "삼", "사", "오", "육", "칠", "팔", "구", "영", "공",
    ]
    .into_iter()
    .collect();

    for token in tokens.iter_mut() {
        // 단일 글자 한자 숫자만 SN으로 변환
        // "삼십", "이백" 등 합성 수사는 NR 유지
        if token.pos == "NR"
            && token.surface.chars().count() == 1
            && single_sino_numerals.contains(token.surface.as_str())
        {
            token.pos = "SN".to_string();
        }
    }

    // 255차: "어/EF + 요/JX" → "어요/EF" 병합 (8차 EC→EF 변환 후 실행)
    // "추워요" = "춥/VA 어요/EF"
    // MeCab이 "어/EC + 요/JX"로 분리하는 경우 병합
    let mut i = 0;
    while i + 1 < tokens.len() {
        if tokens[i].surface == "어"
            && tokens[i].pos == "EF"
            && tokens[i + 1].surface == "요"
            && tokens[i + 1].pos == "JX"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i + 1].end_pos;
            tokens[i] = SejongToken::new("어요", "EF", start, end);
            tokens.remove(i + 1);
            continue;
        }
        i += 1;
    }

    // 258차: 삭제됨 - sample.tsv는 "말씀/NNG 하/VV 세요/EF" (하/VV 유지)

    // 259차: "채/VV + 아/EF" (문장 끝) → "채/NNB"
    // "만큼 뿐 채" 등에서 "채"는 의존명사
    // MeCab이 "채/VV 아/EF"로 분석하는 경우 수정
    let len = tokens.len();
    if len >= 2
        && tokens[len - 2].surface == "채"
        && tokens[len - 2].pos == "VV"
        && (tokens[len - 1].surface == "아" || tokens[len - 1].surface == "ㅏ")
        && tokens[len - 1].pos == "EF"
    {
        // "채/VV 아/EF" → "채/NNB"
        tokens[len - 2].pos = "NNB".to_string();
        // "아/EF" 제거
        tokens.remove(len - 1);
    }
}
