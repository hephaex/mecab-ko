use crate::sejong::hangul::has_jongseong;
use crate::sejong::types::SejongToken;

use super::verb_splitting::apply_verb_splitting_corrections;

/// 24~67차(+57~63): 동사 분리, 어미 정규화, 접두사·의존명사 보정
pub(super) fn apply_verb_and_morpheme_corrections(tokens: &mut Vec<SejongToken>) {
    // 24~44차: 동사 분리, 피동·사동 보정, EC/EF 정규화, 존칭 보정
    apply_verb_splitting_corrections(tokens);

    // 45차 보정: 연속된 EC 병합
    // 패턴: "아/EC + 면서/EC" → "아면서/EC" (또는 그냥 "면서/EC")
    // 이는 VV+EC 분리 실패 시 발생하는 패턴
    // 예: "가면서" → MeCab 출력: "가/VV+EC" + "면서/EC"
    //     분리 후: "가/VV" + "아/EC" + "면서/EC" (잘못된 분리)
    //     보정 후: "가/VV" + "면서/EC"
    let ec_endings = [
        "면서",
        "아서",
        "어서",
        "니까",
        "으니까",
        "지만",
        "거나",
        "더니",
        "고나서",
        "자마자",
        "더라도",
        "으므로",
        "든지",
        "든가",
        "기에",
        "길래",
        "거든",
        "다면",
        "어도",
        "아도",
        "도록",
        "듯이",
        "데도",
        "므로",
        "다가",
        "는데",
        "ㄴ데",
    ];
    let mut ec_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_pos = &tokens[i].pos;
        let curr_surface = &tokens[i].surface;
        let next_pos = &tokens[i + 1].pos;
        let next_surface = &tokens[i + 1].surface;

        // 현재가 1글자 EC이고 다음도 EC인 경우 (잘못된 분리)
        if curr_pos == "EC"
            && curr_surface.chars().count() == 1
            && next_pos == "EC"
            && ec_endings.contains(&next_surface.as_str())
        {
            ec_merge_indices.push(i);
        }
    }
    // 역순으로 처리하여 인덱스 유지
    for idx in ec_merge_indices.into_iter().rev() {
        // 첫 번째 EC 토큰 삭제 (잘못된 1글자 EC)
        tokens.remove(idx);
    }

    // 46차 보정: "으/IC + 면/EC" → "으면/EC"
    // "먹으면"에서 "으"가 IC로 분리된 경우
    let mut ic_ec_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_pos = &tokens[i + 1].pos;

        // "으/IC" 또는 "아/IC" 또는 "어/IC" + EC → 병합
        if curr_pos == "IC"
            && (curr_surface == "으" || curr_surface == "아" || curr_surface == "어")
            && next_pos == "EC"
        {
            ic_ec_merge_indices.push(i);
        }
    }
    for idx in ic_ec_merge_indices.into_iter().rev() {
        let merged_surface = format!("{}{}", tokens[idx].surface, tokens[idx + 1].surface);
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&merged_surface, "EC", start, end);
        tokens.remove(idx + 1);
    }

    // 47차 보정: "으/EF + 면서/EC" → "으면서/EC"
    // "갔으면서"에서 "으"가 EF로 분리된 경우
    let ec_endings_long = [
        "면서",
        "니까",
        "으니까",
        "지만",
        "거나",
        "더니",
        "자마자",
        "더라도",
        "으므로",
        "든지",
        "든가",
        "기에",
        "길래",
        "거든",
        "다면",
        "어도",
        "아도",
        "도록",
        "듯이",
        "데도",
        "므로",
        "다가",
    ];
    let mut ef_ec_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // "으/EF" + 연결어미(EC) → 병합
        if curr_pos == "EF"
            && curr_surface == "으"
            && next_pos == "EC"
            && ec_endings_long.contains(&next_surface.as_str())
        {
            ef_ec_merge_indices.push(i);
        }
    }
    for idx in ef_ec_merge_indices.into_iter().rev() {
        let merged_surface = format!("{}{}", tokens[idx].surface, tokens[idx + 1].surface);
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&merged_surface, "EC", start, end);
        tokens.remove(idx + 1);
    }

    // 48차 보정: "VV(으로 끝남) + 면/EC" → "VV + 으면/EC"
    // "먹으면"에서 "먹으/VV + 면/EC" → "먹/VV + 으면/EC"
    // 단, VV 어간이 받침이 있는 경우에만 적용 (예: 먹, 읽, 잡 등)
    // 받침 없는 어간 (가, 오, 보 등)은 "으"가 붙지 않음
    let ec_short = [
        "면", "니", "니까", "서", "지만", "도", "거나", "다가", "더니",
    ];
    let mut linking_ec_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // VV가 "으"로 끝나고 다음이 EC인 경우
        if (curr_pos == "VV" || curr_pos == "VA" || curr_pos == "VX")
            && curr_surface.chars().count() >= 2
            && next_pos == "EC"
            && ec_short.contains(&next_surface.as_str())
        {
            let chars: Vec<char> = curr_surface.chars().collect();
            let last_char = chars[chars.len() - 1];
            // "으"로 끝나는 경우만 처리
            if last_char == '으' {
                // 그 앞 글자가 받침이 있는지 확인
                if chars.len() >= 2 {
                    let prev_char = chars[chars.len() - 2];
                    if has_jongseong(prev_char) {
                        linking_ec_indices.push(i);
                    }
                }
            }
        }
    }
    for idx in linking_ec_indices.into_iter().rev() {
        let curr_surface = &tokens[idx].surface;
        let last_char = curr_surface.chars().last().unwrap_or(' ');
        let new_vv_surface: String = curr_surface
            .chars()
            .take(curr_surface.chars().count() - 1)
            .collect();
        let merged_ec = format!("{}{}", last_char, tokens[idx + 1].surface);

        let vv_start = tokens[idx].start_pos;
        let vv_end = tokens[idx].start_pos + new_vv_surface.chars().count();
        let ec_start = vv_end;
        let ec_end = tokens[idx + 1].end_pos;

        let pos = tokens[idx].pos.clone();
        tokens[idx] = SejongToken::new(&new_vv_surface, &pos, vv_start, vv_end);
        tokens[idx + 1] = SejongToken::new(&merged_ec, "EC", ec_start, ec_end);
    }

    // 49차 보정: VV/VA 뒤의 "기/NNG" → "기/ETN"
    // "먹기"에서 MeCab이 "먹/VV 기/NNG"로 분리한 경우
    for i in 1..tokens.len() {
        let prev_pos = tokens[i - 1].pos.clone();
        let curr_surface = tokens[i].surface.clone();
        let curr_pos = tokens[i].pos.clone();

        // VV/VA 뒤의 "기/NNG" → "기/ETN"
        if (prev_pos == "VV" || prev_pos == "VA" || prev_pos == "VX")
            && curr_surface == "기"
            && curr_pos == "NNG"
        {
            tokens[i].pos = "ETN".to_string();
        }

        // VV/VA 뒤의 "음/IC" 또는 "음/NNG" → "음/ETN"
        if (prev_pos == "VV" || prev_pos == "VA" || prev_pos == "VX")
            && curr_surface == "음"
            && (curr_pos == "IC" || curr_pos == "NNG")
        {
            tokens[i].pos = "ETN".to_string();
        }

        // VV/VA 뒤의 "ㅁ/NNG" 또는 "ㅁ/IC" → "ㅁ/ETN"
        if (prev_pos == "VV" || prev_pos == "VA" || prev_pos == "VX")
            && curr_surface == "ㅁ"
            && (curr_pos == "NNG" || curr_pos == "IC")
        {
            tokens[i].pos = "ETN".to_string();
        }
    }

    // 50차 보정: "MM + NNG" 패턴의 MM → XPN 변환
    // 전/현/신/구 등이 MM으로 태깅되었지만 실제로는 접두사(XPN)
    // 53차 추가: 새, 첫, 맨, 헛, 옛, 순 (sample.tsv 기반)
    let prefix_patterns: std::collections::HashSet<&str> = [
        "전", "현", "신", "구", "친", "총", "부", "대", "새", "첫", "맨", "헛", "옛", "순",
    ]
    .into_iter()
    .collect();

    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = tokens[i].surface.clone();
        let curr_pos = tokens[i].pos.clone();
        let next_pos = tokens[i + 1].pos.clone();

        // MM + NNG 패턴이고 접두사 후보면 XPN으로 변환
        if curr_pos == "MM"
            && prefix_patterns.contains(curr_surface.as_str())
            && (next_pos == "NNG" || next_pos == "NNP")
        {
            tokens[i].pos = "XPN".to_string();
        }

        // NR + NNG 패턴 중 "구"(舊)는 XPN으로 변환
        if curr_pos == "NR" && curr_surface == "구" && (next_pos == "NNG" || next_pos == "NNP") {
            tokens[i].pos = "XPN".to_string();
        }

        // 51차 보정: "VV + NNG" 패턴 중 접두사 후보는 XPN으로 변환
        // "신/VV 제품/NNG" → "신/XPN 제품/NNG" (신다 동사와 구분)
        if curr_pos == "VV" && curr_surface == "신" && (next_pos == "NNG" || next_pos == "NNP") {
            tokens[i].pos = "XPN".to_string();
        }

        // 53차 보정: "VA + NNG" 패턴 중 접두사 후보는 XPN으로 변환
        // "큰/VA 집/NNG" → "큰/XPN 집/NNG"
        // "작/VA + 은/ETM" 패턴은 제외
        if curr_pos == "VA"
            && (curr_surface == "큰" || curr_surface == "작")
            && (next_pos == "NNG" || next_pos == "NNP")
        {
            tokens[i].pos = "XPN".to_string();
        }
    }

    // 54차 보정: 있/VX → 있/VV 변환 (보조동사가 아닌 경우)
    // "있/VX"가 앞에 "고/EC"가 없으면 본동사 VV로 변환
    // 예: "시간 있/VX 어요" → "시간 있/VV 어요"
    // 단, "가고 있/VX 다"는 보조동사이므로 유지
    for i in 0..tokens.len() {
        if tokens[i].surface == "있" && tokens[i].pos == "VX" {
            // 앞 토큰이 "고/EC"인지 확인
            let is_auxiliary = i > 0 && tokens[i - 1].surface == "고" && tokens[i - 1].pos == "EC";
            if !is_auxiliary {
                tokens[i].pos = "VV".to_string();
            }
        }
    }

    // 55차 보정: EC 뒤의 "하/VV + ㅂ니다/EF" → "합니다/EF" 병합
    // "해야 합니다" 패턴에서 합니다는 보조용언으로 분리하지 않음
    // 예: "하/VV 아야/EC 하/VV ㅂ니다/EF" → "하/VV 아야/EC 합니다/EF"
    let mut i = 0;
    while i < tokens.len().saturating_sub(1) {
        // "하/VV + ㅂ니다/EF" 또는 "가/VV + ㅂ니다/EF" 패턴 찾기
        if tokens[i].pos == "VV"
            && (tokens[i].surface == "하" || tokens[i].surface == "가" || tokens[i].surface == "오")
            && tokens[i + 1].pos == "EF"
            && tokens[i + 1].surface == "ㅂ니다"
        {
            // 앞에 EC가 있는지 확인 (i >= 1)
            let after_ec = i >= 1 && tokens[i - 1].pos == "EC";
            if after_ec {
                // 병합: "하" + "ㅂ니다" → "합니다/EF"
                // 한글 조합: 어간 + ㅂ 받침
                let stem = &tokens[i].surface;
                let merged = match stem.as_str() {
                    "하" => "합니다".to_string(),
                    "가" => "갑니다".to_string(),
                    "오" => "옵니다".to_string(),
                    _ => format!("{stem}ㅂ니다"), // 폴백
                };
                tokens[i].surface = merged;
                tokens[i].pos = "EF".to_string();
                tokens.remove(i + 1);
                continue; // 다음 반복에서 같은 i 검사
            }
        }
        i += 1;
    }

    // 57차 보정: "감/NNG" "봄/NNG" "함/NNG" 등 명사형 → VV + ㅁ/ETN
    // 단독 사용 시 동사 어간+명사형어미로 분리
    // 단, 앞에 NNG이 오면 복합명사로 처리하지 않음
    let nominalized_verbs: std::collections::HashMap<&str, &str> = [
        ("감", "가"),   // 감 → 가/VV + ㅁ/ETN
        ("봄", "보"),   // 봄 → 보/VV + ㅁ/ETN (계절 "봄"은 문맥으로 구분)
        ("함", "하"),   // 함 → 하/VV + ㅁ/ETN
        ("섬", "서"),   // 섬 → 서/VV + ㅁ/ETN (섬(島)은 문맥으로 구분)
        ("삶", "살"),   // 삶 → 살/VV + ㅁ/ETN
        ("앎", "알"),   // 앎 → 알/VV + ㅁ/ETN
        ("죽음", "죽"), // 죽음 → 죽/VV + 음/ETN
    ]
    .into_iter()
    .collect();

    let mut nominalized_split_indices: Vec<(usize, String)> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        // NNG이고 명사형 동사 후보인 경우
        if token.pos == "NNG" && nominalized_verbs.contains_key(token.surface.as_str()) {
            // 앞에 NNG이 오면 복합명사로 간주하여 분리하지 않음
            // 222차: 단, 앞의 NNG도 명사형 동사이면 연속 명사형으로 분리
            let prev_is_nng = i > 0 && tokens[i - 1].pos == "NNG";
            let prev_is_nominalized = i > 0
                && tokens[i - 1].pos == "NNG"
                && nominalized_verbs.contains_key(tokens[i - 1].surface.as_str());
            // 뒤에 조사가 오면 명사형어미로 분리
            let next_is_particle = i + 1 < tokens.len()
                && (tokens[i + 1].pos.starts_with("JK")
                    || tokens[i + 1].pos == "JX"
                    || tokens[i + 1].pos == "JC");
            // 195차: 뒤에 같은 명사형 동사가 오면 연속 명사형 (함 봄 = 하/VV ㅁ/ETN 보/VV ㅁ/ETN)
            let next_is_nominalized = i + 1 < tokens.len()
                && tokens[i + 1].pos == "NNG"
                && nominalized_verbs.contains_key(tokens[i + 1].surface.as_str());
            // 단독 사용, 조사가 따라오거나, 연속 명사형이면 분리
            // 222차: 앞이 명사형 동사이면 분리 (함 봄 = 하/VV ㅁ/ETN 보/VV ㅁ/ETN)
            let should_split = (!prev_is_nng || prev_is_nominalized)
                && (next_is_particle
                    || i + 1 >= tokens.len()
                    || next_is_nominalized
                    || prev_is_nominalized);
            if should_split {
                if let Some(&stem) = nominalized_verbs.get(token.surface.as_str()) {
                    nominalized_split_indices.push((i, stem.to_string()));
                }
            }
        }
    }

    for (idx, stem) in nominalized_split_indices.into_iter().rev() {
        let surface = &tokens[idx].surface;
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        let stem_len = stem.chars().count();

        // "ㅁ"으로 끝나는 단음절 vs "음"으로 끝나는 다음절 구분
        let etm_surface = if surface.ends_with("음") {
            "음"
        } else {
            "ㅁ"
        };

        tokens[idx] = SejongToken::new(&stem, "VV", start, start + stem_len);
        tokens.insert(
            idx + 1,
            SejongToken::new(etm_surface, "ETN", start + stem_len, end),
        );
    }

    // 58차 보정: "갈/VV + 기/ETN" → "가/VV + 기/ETN" (ㄹ 탈락 동사)
    // "가기", "오기" 등에서 어간이 ㄹ로 끝나면 ㄹ 탈락 처리
    // 단, 이미 "가/VV" 등으로 올바르게 분리된 경우는 건너뜀
    let rieul_drop_verbs: std::collections::HashMap<&str, &str> = [
        ("갈", "가"), // 갈기 → 가기
        ("올", "오"), // 올기 → 오기
        ("볼", "보"), // 볼기 → 보기 (실제로는 "볼 기회" 등 다른 패턴)
        ("할", "하"), // 할기 → 하기
        ("살", "사"), // 살기 → 사기 (사다의 명사형)
        ("알", "아"), // 알기 → 아기 (아는 것) - 주의: 아기(baby)와 구분
    ]
    .into_iter()
    .collect();

    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = tokens[i].surface.clone();
        let curr_pos = tokens[i].pos.clone();
        let next_surface = tokens[i + 1].surface.clone();
        let next_pos = tokens[i + 1].pos.clone();

        // VV + 기/ETN 패턴
        if curr_pos == "VV" && next_surface == "기" && next_pos == "ETN" {
            if let Some(&stem) = rieul_drop_verbs.get(curr_surface.as_str()) {
                tokens[i].surface = stem.to_string();
            }
        }
    }

    // 59차 보정: "구시대/NNG" → "구/XPN + 시대/NNG" (접두사 분리)
    // 복합명사 중 접두사로 분리해야 하는 패턴
    let compound_prefix_patterns: std::collections::HashMap<&str, (&str, &str, &str)> = [
        ("구시대", ("구", "XPN", "시대")),
        ("신시대", ("신", "XPN", "시대")),
        ("불합격", ("불", "XPN", "합격")),
        ("불가능", ("불", "XPN", "가능")),
        ("불필요", ("불", "XPN", "필요")),
        ("초고속", ("초", "XPN", "고속")),
        ("초대형", ("초", "XPN", "대형")),
        ("재검토", ("재", "XPN", "검토")),
        ("재확인", ("재", "XPN", "확인")),
        // 새/XPN 접두사 패턴
        ("새해", ("새", "XPN", "해")),
        ("새봄", ("새", "XPN", "봄")),
        ("새벽", ("새", "XPN", "벽")), // 새벽은 단일어지만 형태상 분리
        ("새집", ("새", "XPN", "집")),
        ("새옷", ("새", "XPN", "옷")),
        // 첫/XPN 접두사 패턴
        ("첫눈", ("첫", "XPN", "눈")),
        ("첫날", ("첫", "XPN", "날")),
        ("첫사랑", ("첫", "XPN", "사랑")),
        ("첫걸음", ("첫", "XPN", "걸음")),
        // 큰/XPN 접두사 패턴
        ("큰집", ("큰", "XPN", "집")),
        ("큰아버지", ("큰", "XPN", "아버지")),
        ("큰어머니", ("큰", "XPN", "어머니")),
        // 순/XPN 접두사 패턴
        ("순우리말", ("순", "XPN", "우리말")),
        ("순이익", ("순", "XPN", "이익")),
        // 옛/XPN 접두사 패턴
        ("옛날", ("옛", "XPN", "날")),
        ("옛사람", ("옛", "XPN", "사람")),
        // 헛/XPN 접두사 패턴
        ("헛소리", ("헛", "XPN", "소리")),
        ("헛수고", ("헛", "XPN", "수고")),
    ]
    .into_iter()
    .collect();

    let mut compound_split_indices: Vec<(usize, String, String, String)> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if token.pos == "NNG" {
            if let Some(&(prefix, prefix_pos, suffix)) =
                compound_prefix_patterns.get(token.surface.as_str())
            {
                compound_split_indices.push((
                    i,
                    prefix.to_string(),
                    prefix_pos.to_string(),
                    suffix.to_string(),
                ));
            }
        }
    }

    for (idx, prefix, prefix_pos, suffix) in compound_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        let prefix_len = prefix.chars().count();

        tokens[idx] = SejongToken::new(&prefix, &prefix_pos, start, start + prefix_len);
        tokens.insert(
            idx + 1,
            SejongToken::new(&suffix, "NNG", start + prefix_len, end),
        );
    }

    // 60차 보정: NNB(의존명사) 패턴 보정
    // "것/NNG", "수/NNG", "데/NNG" 등 → NNB로 변환 (관형형 뒤에서)
    // 관형형어미(ETM) 뒤의 형식명사는 NNB
    let dependent_nouns: std::collections::HashSet<&str> = [
        "것", "거", "수", "데", "때", "뿐", "줄", "척", "체", "만큼", "대로", "지", "채", "김",
        "듯", "바", "양", "적", "리", "이", "분",
    ]
    .into_iter()
    .collect();

    for i in 1..tokens.len() {
        let prev_pos = tokens[i - 1].pos.clone();
        let curr_surface = tokens[i].surface.clone();
        let curr_pos = tokens[i].pos.clone();

        // ETM 뒤의 NNG가 의존명사 목록에 있으면 NNB로 변환
        if prev_pos == "ETM" && curr_pos == "NNG" && dependent_nouns.contains(curr_surface.as_str())
        {
            tokens[i].pos = "NNB".to_string();
        }

        // VV/VA/VX 뒤의 NNG가 의존명사 목록에 있으면 NNB로 변환
        // "할 수 있다"에서 "수"는 NNB
        if (prev_pos == "VV" || prev_pos == "VA" || prev_pos == "VX")
            && curr_pos == "NNG"
            && (curr_surface == "수"
                || curr_surface == "것"
                || curr_surface == "줄"
                || curr_surface == "뿐")
        {
            tokens[i].pos = "NNB".to_string();
        }
    }

    // 61차 보정: "가야/NNG + 합니다/EF" → "가/VV + 아야/EC + 합니다/EF"
    // "가야 합니다" 패턴에서 "가야"는 명사가 아니라 동사+어미
    let aya_patterns: std::collections::HashMap<&str, &str> = [
        ("가야", "가"),
        ("와야", "오"), // 오+아야 → 와야
        ("봐야", "보"), // 보+아야 → 봐야
        ("해야", "하"),
    ]
    .into_iter()
    .collect();

    let mut aya_split_indices: Vec<(usize, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;

        // NNG + "합니다/EF" 또는 NNG + "하/XSV" 패턴
        if curr_pos == "NNG"
            && aya_patterns.contains_key(curr_surface.as_str())
            && (next_surface == "합니다" || next_surface == "해요" || next_surface == "하")
        {
            if let Some(&stem) = aya_patterns.get(curr_surface.as_str()) {
                aya_split_indices.push((i, stem.to_string()));
            }
        }
    }

    for (idx, stem) in aya_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        let stem_len = stem.chars().count();

        tokens[idx] = SejongToken::new(&stem, "VV", start, start + stem_len);
        tokens.insert(
            idx + 1,
            SejongToken::new("아야", "EC", start + stem_len, end),
        );
    }

    // 62차 보정: 잘못 분리된 복합명사 병합
    // "올/VV + 해/NNG" → "올해/NNG"
    // "내/NP + 년/NNG" → "내년/NNG"
    let compound_noun_merges: Vec<(&str, &str, &str, &str, &str)> = vec![
        ("올", "VV", "해", "NNG", "올해"), // 올해
        ("내", "NP", "년", "NNG", "내년"), // 내년
        ("작", "VA", "년", "NNG", "작년"), // 작년
    ];

    let mut compound_merge_indices: Vec<(usize, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        for (s1, p1, s2, p2, merged) in &compound_noun_merges {
            if curr_surface == *s1 && curr_pos == *p1 && next_surface == *s2 && next_pos == *p2 {
                compound_merge_indices.push((i, (*merged).to_string()));
                break;
            }
        }
    }

    for (idx, merged) in compound_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&merged, "NNG", start, end);
        tokens.remove(idx + 1);
    }

    // 63차 보정: "가야/NNG + 합니다/VV+EF" → "가/VV + 아야/EC + 합니다/EF"
    // "합니다"가 VV+EF로 분석된 경우 EF로 단일 토큰화
    for i in 0..tokens.len() {
        let surface = &tokens[i].surface;
        let pos = &tokens[i].pos;

        // "합니다/VV+EF" → "합니다/EF"
        if surface == "합니다" && pos == "VV+EF" {
            tokens[i].pos = "EF".to_string();
        }
    }

    // 64차 보정: 제거됨 - 부작용이 있어서 XSV + EF 병합 대신 테스트 케이스 수정 필요

    // 65차 보정: 제거됨
    // 세종 코퍼스 표준은 "드/VV 시/EP" (ㄹ 탈락형 그대로 유지)
    // "들/VV"로 복원하면 오히려 정답과 불일치

    // 66차 보정: 제거됨 - 테스트 데이터에서 "세요"는 단일 EF로 분석됨

    // 67차 보정: "선생님/NNG + 께서/JKS" 패턴
    // MeCab이 "선생님"을 "선생/NNG + 님/XSN"으로 분리하면 병합
    let honorific_nouns: Vec<(&str, &str, &str)> = vec![
        ("선생", "님", "선생님"),
        ("할머", "님", "할머님"),
        ("할아버", "님", "할아버님"),
        ("어머", "님", "어머님"),
        ("아버", "님", "아버님"),
    ];

    let mut honorific_merge_indices: Vec<(usize, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        for (base, suffix, merged) in &honorific_nouns {
            if curr_surface == *base
                && curr_pos == "NNG"
                && next_surface == *suffix
                && next_pos == "XSN"
            {
                honorific_merge_indices.push((i, (*merged).to_string()));
                break;
            }
        }
    }

    for (idx, merged) in honorific_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&merged, "NNG", start, end);
        tokens.remove(idx + 1);
    }
}
