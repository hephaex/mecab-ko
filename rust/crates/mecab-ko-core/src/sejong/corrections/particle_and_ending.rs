use std::collections::HashMap;
use crate::sejong::hangul::extract_vowel;
use crate::sejong::types::SejongToken;

/// 1~23차: 조사 및 어미 보정
///
/// `apply_tag_normalization_corrections` 호출 직후 실행.
///
/// - 1차: 체언 뒤 잘못 태그된 품사 → 조사(`JK*/JX/JC`) 보정 (`particle_map`)
/// - 2차: 동사/형용사 뒤 관형형어미(ETM) 보정
/// - 3차: XSV (파생접미사) 보정
/// - 4차 / 4-2차: EC/EF 표면형 복원 (아서/어서, 아요/어요)
/// - 5차: "하면서" 분리 병합 보정
/// - 6차: (비활성화)
/// - 7차: "합니다" 병합
/// - 8~9차: 종결어미/XSV→VV 보정
/// - 11~13차: VCP 삽입, NNB→EC, 동사기본형 분리
/// - 14~16차: XSV→VV, 기/ETN 분리, JX 삭제
/// - 17~19차: 소유격/존칭 분리·병합
/// - 20차: MAJ→MAG
/// - 21차: EP→VCP 보정
/// - 22차: 시간 표현 분리
/// - 23차: "그렇다면" 분리
pub(super) fn apply_particle_and_ending_corrections(tokens: &mut Vec<SejongToken>) {
    // 조사로 보정해야 할 표면형 -> 품사 매핑
    let particle_map: HashMap<&str, &str> = [
        // 주격조사 (JKS)
        ("이", "JKS"),
        ("가", "JKS"),
        ("께서", "JKS"),
        // 목적격조사 (JKO)
        ("을", "JKO"),
        ("를", "JKO"),
        // 부사격조사 (JKB)
        ("에", "JKB"),
        ("에서", "JKB"),
        ("에게", "JKB"),
        ("로", "JKB"),
        ("으로", "JKB"),
        ("한테", "JKB"),
        ("보다", "JKB"),
        ("처럼", "JKB"),
        ("같이", "JKB"),
        // 관형격조사 (JKG)
        ("의", "JKG"),
        // 호격조사 (JKV) - 191차: "아"는 sample.tsv 기준 JX로 처리
        // ("아", "JKV"), // 191차 수정: JKV → JX
        ("야", "JKV"),
        ("여", "JKV"),
        ("이여", "JKV"),
        // 보조사 (JX)
        ("은", "JX"),
        ("는", "JX"),
        ("도", "JX"),
        ("만", "JX"),
        ("까지", "JX"),
        ("부터", "JX"),
        ("마저", "JX"),
        ("조차", "JX"),
        ("라도", "JX"),
        ("밖에", "JX"),
        ("요", "JX"),
        // 접속조사 (JC)
        ("와", "JC"),
        ("과", "JC"),
        ("이랑", "JC"),
        ("랑", "JC"),
        ("하고", "JC"),
    ]
    .into_iter()
    .collect();

    // 체언 품사 집합
    let noun_poses: std::collections::HashSet<&str> =
        ["NNG", "NNP", "NNB", "NP", "NR"].into_iter().collect();

    // 수정이 필요한 인덱스와 새 품사를 저장
    let mut corrections: Vec<(usize, String)> = Vec::new();

    // 의문대명사 집합 (이 뒤의 VV는 조사가 아님)
    let interrogatives: std::collections::HashSet<&str> = [
        "어디", "언제", "뭐", "무엇", "누구", "어느", "어떤", "왜", "어찌",
    ]
    .into_iter()
    .collect();

    for i in 1..tokens.len() {
        let prev_surface = &tokens[i - 1].surface;
        let prev_pos = &tokens[i - 1].pos;
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // 체언 뒤의 잘못 태그된 품사를 조사로 보정
        // ETN: "을" 등이 명사형어미로 잘못 태그되는 경우
        // EF/EC: "가", "는" 등이 어미로 잘못 태그되는 경우
        // EP: "씨" 등이 선어말어미로 잘못 태그되는 경우
        // JKB: "께서" 등이 부사격조사로 잘못 태그되는 경우 → JKS로 보정
        // NNG: "의" 등이 명사로 잘못 태그되는 경우 → JKG로 보정
        if noun_poses.contains(prev_pos.as_str())
            && (curr_pos == "EF"
                || curr_pos == "EC"
                || curr_pos == "ETN"
                || curr_pos == "EP"
                || curr_pos == "VV"
                || curr_pos == "VA"
                || curr_pos == "JKB"
                || curr_pos == "NNG")
        {
            // 다음 토큰이 EP(선어말어미)인 경우 동사의 일부이므로 조사로 보정하지 않음
            // 예: 학교/NNG 가/VV 았/EP 다/EF -> "가"는 동사 "가다"의 어간
            let next_is_ep = i + 1 < tokens.len() && tokens[i + 1].pos == "EP";

            // 다음 토큰이 EF/EC인 경우 현재 토큰은 동사의 어간이므로 조사로 보정하지 않음
            // 예: 어디/NP 가/VV 니/EF -> "가"는 동사 "가다"의 어간
            let next_is_ending =
                i + 1 < tokens.len() && (tokens[i + 1].pos == "EF" || tokens[i + 1].pos == "EC");

            // 의문대명사 뒤의 VV는 동사로 유지 (조사가 아님)
            // 예: 어디 가니, 뭐 하니
            let prev_is_interrogative = interrogatives.contains(prev_surface.as_str());

            // "께서"는 항상 주격조사 (동사 어간이 될 수 없음)
            let is_definite_particle = curr_surface == "께서";

            if is_definite_particle || (!next_is_ep && !next_is_ending && !prev_is_interrogative) {
                if let Some(&correct_pos) = particle_map.get(curr_surface.as_str()) {
                    corrections.push((i, correct_pos.to_string()));
                }
            }
        }
    }

    // 보정 적용
    for (idx, new_pos) in corrections {
        tokens[idx].pos = new_pos;
    }

    // 2차 보정: 동사/형용사 뒤의 관형형어미(ETM) 보정
    let verb_poses: std::collections::HashSet<&str> = ["VV", "VA", "VX"].into_iter().collect();
    let etm_map: HashMap<&str, &str> = [
        ("는", "ETM"), // 현재 관형형: 가는, 먹는
        ("ㄴ", "ETM"), // 과거 관형형: 간, 먹은
        ("은", "ETM"), // 과거 관형형: 먹은
        ("ㄹ", "ETM"), // 미래 관형형: 갈, 먹을
        ("을", "ETM"), // 미래 관형형: 먹을
        ("던", "ETM"), // 회상 관형형: 가던, 먹던
    ]
    .into_iter()
    .collect();

    let mut etm_corrections: Vec<(usize, String)> = Vec::new();

    for i in 1..tokens.len() {
        let prev_pos = &tokens[i - 1].pos;
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // 동사/형용사 뒤의 JX/EF를 ETM으로 보정
        if verb_poses.contains(prev_pos.as_str())
            && (curr_pos == "JX" || curr_pos == "EF" || curr_pos == "EC")
        {
            if let Some(&correct_pos) = etm_map.get(curr_surface.as_str()) {
                etm_corrections.push((i, correct_pos.to_string()));
            }
        }
    }

    // ETM 보정 적용
    for (idx, new_pos) in etm_corrections {
        tokens[idx].pos = new_pos;
    }

    // 3차 보정: XSV (파생접미사) 보정
    // 일반명사 뒤의 "하다/되다" 계열을 XSV로 보정
    // 패턴: NNG + 하/했/해/되/됐 → NNG + XSV
    // 주의: NP(대명사) 뒤에는 적용하지 않음 (예: "뭐 하니"에서 "하"는 VV)
    let xsv_patterns: HashMap<&str, bool> = [
        ("하", true), // 하다
        ("해", true), // 해요 (하+어)
        ("했", true), // 했다 (하+았)
        ("되", true), // 되다
        ("됐", true), // 됐다 (되+었)
    ]
    .into_iter()
    .collect();

    // XSV 보정 대상: 일반명사만 (대명사 NP 제외)
    let xsv_trigger_poses: std::collections::HashSet<&str> =
        ["NNG", "NNP", "NNB"].into_iter().collect();

    let mut xsv_corrections: Vec<(usize, String)> = Vec::new();

    for i in 1..tokens.len() {
        let prev_pos = &tokens[i - 1].pos;
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // 일반명사 뒤의 VV/EF를 XSV로 보정 (대명사 NP 제외)
        if xsv_trigger_poses.contains(prev_pos.as_str())
            && (curr_pos == "VV" || curr_pos == "EF" || curr_pos == "VA")
            && xsv_patterns.contains_key(curr_surface.as_str())
        {
            xsv_corrections.push((i, "XSV".to_string()));
        }
    }

    // XSV 보정 적용
    for (idx, new_pos) in xsv_corrections {
        tokens[idx].pos = new_pos;
    }

    // 4차 보정: 축약된 연결어미 복원
    // 동사 뒤의 "서"를 "아서/어서"로 복원 (모음 조화)
    // 예: 만나/VV + 서/EC → 만나/VV + 아서/EC
    let mut ec_restorations: Vec<(usize, String)> = Vec::new();

    for i in 1..tokens.len() {
        let prev_surface = &tokens[i - 1].surface;
        let prev_pos = &tokens[i - 1].pos;
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // VV/VA 뒤의 "서"를 복원
        if (prev_pos == "VV" || prev_pos == "VA") && curr_surface == "서" && curr_pos == "EC" {
            // 어간의 마지막 모음에 따라 아서/어서 결정
            // ㅏ, ㅗ → 아서 (양성모음)
            // 그 외 → 어서 (음성모음)
            if let Some(last_char) = prev_surface.chars().last() {
                let vowel = extract_vowel(last_char);
                let restored = if vowel == 'ㅏ' || vowel == 'ㅗ' {
                    "아서"
                } else {
                    "어서"
                };
                ec_restorations.push((i, restored.to_string()));
            }
        }
    }

    // 연결어미 복원 적용
    for (idx, new_surface) in ec_restorations {
        tokens[idx].surface = new_surface;
    }

    // 4-2차 보정: 축약된 종결어미 복원
    // 동사 뒤의 "요"를 "아요/어요"로 복원 (모음 조화)
    // 예: 가/VV + 요/EF → 가/VV + 아요/EF
    let mut ef_restorations: Vec<(usize, String)> = Vec::new();

    for i in 1..tokens.len() {
        let prev_surface = &tokens[i - 1].surface;
        let prev_pos = &tokens[i - 1].pos;
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // VV/VA 뒤의 "요"를 복원
        if (prev_pos == "VV" || prev_pos == "VA") && curr_surface == "요" && curr_pos == "EF" {
            // 어간의 마지막 모음에 따라 아요/어요 결정
            // ㅏ, ㅗ → 아요 (양성모음)
            // 그 외 → 어요 (음성모음)
            if let Some(last_char) = prev_surface.chars().last() {
                let vowel = extract_vowel(last_char);
                let restored = if vowel == 'ㅏ' || vowel == 'ㅗ' {
                    "아요"
                } else {
                    "어요"
                };
                ef_restorations.push((i, restored.to_string()));
            }
        }
    }

    // 종결어미 복원 적용
    for (idx, new_surface) in ef_restorations {
        tokens[idx].surface = new_surface;
    }

    // 5차 보정: "하면/XSV + 서/EC" → "하/XSV + 면서/EC" 변환
    // MeCab이 "하면서"를 "하면" + "서"로 잘못 분리하는 문제 해결
    let mut ec_merge_corrections: Vec<(usize, String, String)> = Vec::new();

    for i in 1..tokens.len() {
        let prev_surface = &tokens[i - 1].surface;
        let prev_pos = &tokens[i - 1].pos;
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // XSV/VV/VA 뒤 "서/EC" 패턴 체크
        if (prev_pos == "XSV" || prev_pos == "VV" || prev_pos == "VA")
            && curr_surface == "서"
            && curr_pos == "EC"
        {
            // "하면" → "하", "서" → "면서"
            if prev_surface.ends_with("면") {
                let new_prev = prev_surface.trim_end_matches("면").to_string();
                ec_merge_corrections.push((i - 1, new_prev, "면서".to_string()));
            }
        }
    }

    // EC 병합 보정 적용
    for (prev_idx, new_prev_surface, new_curr_surface) in ec_merge_corrections {
        if !new_prev_surface.is_empty() {
            tokens[prev_idx].surface = new_prev_surface;
        }
        tokens[prev_idx + 1].surface = new_curr_surface;
    }

    // 6차 보정: (비활성화) JC → JKB 변환은 평가 데이터와 불일치
    // "친구와/JC 만나다" - JC 유지 (평가 데이터 기준)

    // 7차 보정: "합니/VV + 다/EF" → "합니다/EF"
    // MeCab이 "합니다"를 "합니 + 다"로 분리하는 문제 해결
    let mut merge_indices: Vec<usize> = Vec::new();

    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // 합니/VV + 다/EF → 합니다/EF
        if curr_surface == "합니" && curr_pos == "VV" && next_surface == "다" && next_pos == "EF"
        {
            merge_indices.push(i);
        }
    }

    // 역순으로 병합 (인덱스 변화 방지)
    for idx in merge_indices.into_iter().rev() {
        let merged = format!("{}{}", tokens[idx].surface, tokens[idx + 1].surface);
        tokens[idx].surface = merged;
        tokens[idx].pos = "EF".to_string();
        tokens[idx].end_pos = tokens[idx + 1].end_pos;
        tokens.remove(idx + 1);
    }

    // 8차 보정: 문장 끝 종결어미 보정
    // EC로 분석되었지만 문장 끝에 있으면 EF로 보정
    // "하니/VV+니/EC" → "하니/VV+니/EF" (종결어미로 사용될 때)
    // "먹다/VV+다/EC" → "먹다/VV+다/EF" (종결어미로 사용될 때)
    if let Some(last) = tokens.last_mut() {
        if last.pos == "EC" {
            // 문장 끝에서 종결어미로 사용되는 패턴
            let final_endings = ["니", "다", "요", "죠", "지", "나", "자"];
            if final_endings.contains(&last.surface.as_str()) {
                last.pos = "EF".to_string();
            }
        }
    }

    // 9차 보정: "하/XSV + 아야/EC" → "하/VV + 아야/EC"
    // "준비해야" 등에서 "하다"는 VV로 분석
    // 또한 "하/XSV + 세요/EF" → "하/VV + 세요/EF" (말씀하세요 등)
    let mut xsv_to_vv_indices: Vec<usize> = Vec::new();

    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // 하/XSV + 아야/EC → 하/VV + 아야/EC
        if (curr_surface == "하" || curr_surface == "해" || curr_surface == "했")
            && curr_pos == "XSV"
            && next_pos == "EC"
            && (next_surface == "아야" || next_surface == "어야" || next_surface == "야")
        {
            xsv_to_vv_indices.push(i);
        }

        // 하/XSV + 세요/EF → 하/VV + 세요/EF (말씀하세요 등)
        if curr_surface == "하"
            && curr_pos == "XSV"
            && next_pos == "EF"
            && (next_surface == "세요" || next_surface == "시오" || next_surface == "십시오")
        {
            xsv_to_vv_indices.push(i);
        }
    }

    for idx in xsv_to_vv_indices {
        tokens[idx].pos = "VV".to_string();
    }

    // 10차 보정: "고/EC + 나/NP" 다음에 서/EC가 아니면 "고나서" 패턴 아님
    // 일단 단순한 보정: "먹고/EC 나서/EC" → "먹/VV 고나서/EC"
    // TODO: 토큰 병합 패턴으로 분리 검토

    // 11차 보정: NP + 세요/EF → NP + 이/VCP + 세요/EF
    // "누구세요"에서 계사 "이다"가 생략된 경우 복원
    let mut vcp_insert_indices: Vec<usize> = Vec::new();

    for i in 0..tokens.len().saturating_sub(1) {
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // NP + 세요/EF|EC 또는 NP + 에요/EF|EC → NP + 이/VCP + 세요/EF
        // "세요"가 EC로 분석되는 경우도 포함
        if curr_pos == "NP"
            && (next_pos == "EF" || next_pos == "EC")
            && (next_surface == "세요" || next_surface == "에요" || next_surface == "예요")
        {
            vcp_insert_indices.push(i + 1);
        }
    }

    // 역순으로 삽입 (인덱스 변화 방지)
    for idx in vcp_insert_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        // "세요/EC" → "세요/EF"로 변환
        if tokens[idx].pos == "EC" {
            tokens[idx].pos = "EF".to_string();
        }
        tokens.insert(idx, SejongToken::new("이", "VCP", start, start));
    }

    // 12차 보정: "지/NNB" + "않/VX" → "지/EC" + "않/VX"
    // "하지 않아요"에서 "지"는 연결어미(EC)
    let mut nnb_to_ec_indices: Vec<usize> = Vec::new();

    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // 지/NNB + 않/VX → 지/EC + 않/VX
        if curr_surface == "지" && curr_pos == "NNB" && next_surface == "않" && next_pos == "VX" {
            nnb_to_ec_indices.push(i);
        }
    }

    for idx in nnb_to_ec_indices {
        tokens[idx].pos = "EC".to_string();
    }

    // 13차 보정: 동사 기본형 분리 (Xda/VV → X/VV + 다/EF)
    // 가다, 먹다, 오다, 보다, 하다 등 기본형을 분리
    // 주의: 단독 사용 시만 분리 (문장 내에서는 어간+어미로 분석됨)
    let base_verbs: std::collections::HashSet<&str> = [
        "가다", "오다", "보다", "먹다", "되다", "주다", "받다", "쓰다", "읽다", "듣다", "말다",
        "살다", "죽다", "자다", "일다", "앉다", "서다", "놓다", "두다", "치다", "잡다", "놀다",
        "울다",
    ]
    .into_iter()
    .collect();

    // "하다"는 별도 처리 (XSV인 경우만 VV로 변환 후 분리)

    let mut verb_split_indices: Vec<usize> = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        if token.pos == "VV" && base_verbs.contains(token.surface.as_str()) {
            verb_split_indices.push(i);
        }
    }

    // 역순으로 분리 (인덱스 변화 방지)
    for idx in verb_split_indices.into_iter().rev() {
        let surface = &tokens[idx].surface;
        if let Some(stem) = surface.strip_suffix("다") {
            if !stem.is_empty() {
                let start = tokens[idx].start_pos;
                let end = tokens[idx].end_pos;
                let stem_len = stem.chars().count();
                tokens[idx] = SejongToken::new(stem, "VV", start, start + stem_len);
                tokens.insert(idx + 1, SejongToken::new("다", "EF", start + stem_len, end));
            }
        }
    }

    // 14차 보정: "하/XSV + 다/EF" → "하/VV + 다/EF"
    // 단독 "하다"는 VV로 분석
    let mut xsv_da_to_vv_indices: Vec<usize> = Vec::new();

    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // 하/XSV + 다/EF → 하/VV + 다/EF
        if curr_surface == "하" && curr_pos == "XSV" && next_surface == "다" && next_pos == "EF" {
            xsv_da_to_vv_indices.push(i);
        }
    }

    for idx in xsv_da_to_vv_indices {
        tokens[idx].pos = "VV".to_string();
    }

    // 15차 보정: 복합명사+기(NNG) + 전(NNG) → 어간+기(ETN) + 전(NNG)
    // "가기 전에", "먹기 전에" 등 명사형어미 분리
    let mut gi_split_indices: Vec<usize> = Vec::new();

    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;

        // X기/NNG + 전/NNG → X/VV + 기/ETN + 전/NNG
        if curr_pos == "NNG"
            && curr_surface.ends_with("기")
            && curr_surface.chars().count() >= 2
            && (next_surface == "전" || next_surface == "위해" || next_surface == "시작")
        {
            gi_split_indices.push(i);
        }
    }

    // 역순으로 분리 (인덱스 변화 방지)
    for idx in gi_split_indices.into_iter().rev() {
        let surface = &tokens[idx].surface;
        if let Some(stem) = surface.strip_suffix("기") {
            if !stem.is_empty() {
                let start = tokens[idx].start_pos;
                let end = tokens[idx].end_pos;
                let stem_len = stem.chars().count();
                tokens[idx] = SejongToken::new(stem, "VV", start, start + stem_len);
                tokens.insert(
                    idx + 1,
                    SejongToken::new("기", "ETN", start + stem_len, end),
                );
            }
        }
    }

    // 16차 보정: 잘못된 "는/JX + 들/XSN + 이/JKS" 패턴 수정
    // 사전 버그로 인해 "들이"가 "는+들+이"로 분해됨
    // 실제로는 "들+이"여야 함 → "는/JX" 토큰 삭제
    let mut jx_delete_indices: Vec<usize> = Vec::new();

    for i in 0..tokens.len().saturating_sub(2) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;
        let next2_surface = &tokens[i + 2].surface;
        let next2_pos = &tokens[i + 2].pos;

        // "는/JX + 들/XSN + 이/JKS" 패턴 감지
        if curr_surface == "는"
            && curr_pos == "JX"
            && next_surface == "들"
            && next_pos == "XSN"
            && next2_surface == "이"
            && next2_pos == "JKS"
        {
            jx_delete_indices.push(i);
        }
    }

    // 역순으로 삭제 (인덱스 변화 방지)
    for idx in jx_delete_indices.into_iter().rev() {
        tokens.remove(idx);
    }

    // 17차 보정: "X의/NNG" → "X/NP + 의/JKG" 분리
    // "나의", "우리의" 등 대명사+관형격조사 패턴 분리
    let possessive_pronouns: std::collections::HashSet<&str> =
        ["나의", "너의", "우리의", "저의", "그의", "그녀의"]
            .into_iter()
            .collect();

    let mut possessive_split_indices: Vec<usize> = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        if token.pos == "NNG" && possessive_pronouns.contains(token.surface.as_str()) {
            possessive_split_indices.push(i);
        }
    }

    // 역순으로 분리 (인덱스 변화 방지)
    for idx in possessive_split_indices.into_iter().rev() {
        let surface = &tokens[idx].surface;
        if let Some(stem) = surface.strip_suffix("의") {
            if !stem.is_empty() {
                let start = tokens[idx].start_pos;
                let end = tokens[idx].end_pos;
                let stem_len = stem.chars().count();
                tokens[idx] = SejongToken::new(stem, "NP", start, start + stem_len);
                tokens.insert(
                    idx + 1,
                    SejongToken::new("의", "JKG", start + stem_len, end),
                );
            }
        }
    }

    // 18차 보정: NP + "의X/NNG" → NP + "의/JKG" + "X/NNG" 분리
    // "우리의 집" → "우리/NP + 의집/NNG" → "우리/NP + 의/JKG + 집/NNG"
    let mut genitive_split_indices: Vec<usize> = Vec::new();

    for i in 1..tokens.len() {
        let prev_pos = &tokens[i - 1].pos;
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;

        // NP 뒤의 "의X/NNG" 패턴 감지
        if prev_pos == "NP"
            && curr_pos == "NNG"
            && curr_surface.starts_with("의")
            && curr_surface.chars().count() >= 2
        {
            genitive_split_indices.push(i);
        }
    }

    // 역순으로 분리 (인덱스 변화 방지)
    for idx in genitive_split_indices.into_iter().rev() {
        let surface = tokens[idx].surface.clone();
        if let Some(rest) = surface.strip_prefix("의") {
            if !rest.is_empty() {
                let start = tokens[idx].start_pos;
                let end = tokens[idx].end_pos;
                let rest_owned = rest.to_string();
                tokens[idx] = SejongToken::new("의", "JKG", start, start + 1);
                tokens.insert(
                    idx + 1,
                    SejongToken::new(&rest_owned, "NNG", start + 1, end),
                );
            }
        }
    }

    // 19차 보정: "선생/NNG + 님의/NNP" → "선생님/NNG + 의/JKG"
    // 또는 "X/NNG + 님의/NNP" 패턴을 "X님/NNG + 의/JKG"로 병합
    let mut honorific_merge_indices: Vec<usize> = Vec::new();

    for i in 0..tokens.len().saturating_sub(1) {
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // NNG + "님의/NNP" 패턴 감지
        if curr_pos == "NNG" && next_surface == "님의" && (next_pos == "NNP" || next_pos == "NNG")
        {
            honorific_merge_indices.push(i);
        }
    }

    // 역순으로 병합 (인덱스 변화 방지)
    for idx in honorific_merge_indices.into_iter().rev() {
        let merged = format!("{}님", tokens[idx].surface);
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&merged, "NNG", start, end - 1);
        tokens[idx + 1] = SejongToken::new("의", "JKG", end - 1, end);
    }

    // 20차 보정: MAJ → MAG 보정
    // "또한", "따라서" 등 일반부사(MAG)로 분류되어야 하는 단어들
    // 주의: "하지만", "그러나", "그래서", "그리고"는 접속부사(MAJ) 유지
    let maj_to_mag: std::collections::HashSet<&str> =
        ["또한", "따라서", "그러므로"].into_iter().collect();

    for token in tokens.iter_mut() {
        if token.pos == "MAJ" && maj_to_mag.contains(token.surface.as_str()) {
            token.pos = "MAG".to_string();
        }
    }

    // 21차 보정: VCP 삽입 - NNG + "이/EP" → NNG + "이/VCP"
    // 예: "학생입니다" → 학생/NNG + 이/EP + ㅂ니다/EF → 학생/NNG + 이/VCP + 습니다/EF
    // NNG/NNP/NP 다음에 오는 "이/EP"를 "이/VCP"로 보정
    for i in 1..tokens.len() {
        let prev_pos = &tokens[i - 1].pos;
        let curr_pos = &tokens[i].pos;
        let curr_surface = &tokens[i].surface;

        // NNG/NNP/NP 다음에 "이/EP" 패턴 → "이/VCP"로 보정
        if (prev_pos == "NNG" || prev_pos == "NNP" || prev_pos == "NP")
            && curr_pos == "EP"
            && curr_surface == "이"
        {
            tokens[i].pos = "VCP".to_string();
        }
    }

    // 22차 보정: 시간 표현 분리 - "열시/NNG" → "열/NR + 시/NNB"
    // "세시", "열시", "한시" 등의 패턴
    let time_words: std::collections::HashMap<&str, (&str, &str)> = [
        ("열시", ("열", "시")),
        ("세시", ("세", "시")),
        ("한시", ("한", "시")),
        ("두시", ("두", "시")),
        ("네시", ("네", "시")),
        ("다섯시", ("다섯", "시")),
        ("여섯시", ("여섯", "시")),
        ("일곱시", ("일곱", "시")),
        ("여덟시", ("여덟", "시")),
        ("아홉시", ("아홉", "시")),
    ]
    .into_iter()
    .collect();

    let mut time_split_indices: Vec<(usize, String, String)> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if token.pos == "NNG" {
            if let Some(&(num, unit)) = time_words.get(token.surface.as_str()) {
                time_split_indices.push((i, num.to_string(), unit.to_string()));
            }
        }
    }

    for (idx, num, unit) in time_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        let mid = start + num.chars().count();
        tokens[idx] = SejongToken::new(&num, "NR", start, mid);
        tokens.insert(idx + 1, SejongToken::new(&unit, "NNB", mid, end));
    }

    // 23차 보정: "그렇다면/MAJ" → "그렇/VA + 다면/EC"
    let mut maj_split_indices: Vec<usize> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if token.pos == "MAJ" && token.surface == "그렇다면" {
            maj_split_indices.push(i);
        }
    }

    for idx in maj_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        tokens[idx] = SejongToken::new("그렇", "VA", start, start + 2);
        tokens.insert(idx + 1, SejongToken::new("다면", "EC", start + 2, end));
    }

}
