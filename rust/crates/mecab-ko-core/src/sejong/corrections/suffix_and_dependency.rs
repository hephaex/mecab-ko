use crate::sejong::hangul::has_jongseong;
use crate::sejong::types::SejongToken;

/// 167~172차, 68~85차: 접미사·의존명사·파생어·목적 연결어미 보정
pub(super) fn apply_suffix_and_dependency_corrections(tokens: &mut Vec<SejongToken>) {
    // 167차 보정: NNG + "적/XSN" → NNG 병합
    // "성공/NNG + 적/XSN" → "성공적/NNG"
    // "적극/NNG + 적/XSN" → "적극적/NNG"
    let mut jeok_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i].pos == "NNG" && tokens[i + 1].pos == "XSN" && tokens[i + 1].surface == "적" {
            jeok_merge_indices.push(i);
        }
    }

    for idx in jeok_merge_indices.into_iter().rev() {
        let merged = format!("{}적", tokens[idx].surface);
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&merged, "NNG", start, end);
        tokens.remove(idx + 1);
    }

    // 168차 보정: "의/JKB" → "의/JKG" (관형격 조사)
    // MeCab이 "의"를 JKB로 분석하지만 세종 표준은 JKG
    // NNG/NNP/NP/XSN 뒤의 "의"는 관형격 조사
    for i in 0..tokens.len().saturating_sub(1) {
        let prev_pos = &tokens[i].pos;
        if (prev_pos == "NNG" || prev_pos == "NNP" || prev_pos == "NP" || prev_pos == "XSN")
            && tokens[i + 1].pos == "JKB"
            && tokens[i + 1].surface == "의"
        {
            tokens[i + 1].pos = "JKG".to_string();
        }
    }

    // 68차 보정: "시었/EP" → "시/EP + 었/EP" 분리
    // "오셨습니다"에서 "시었"이 하나의 EP로 분석되면 분리
    let mut sieot_split_indices: Vec<usize> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if token.pos == "EP" && token.surface == "시었" {
            sieot_split_indices.push(i);
        }
    }

    for idx in sieot_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        tokens[idx] = SejongToken::new("시", "EP", start, start + 1);
        tokens.insert(idx + 1, SejongToken::new("었", "EP", start + 1, end));
    }

    // 69차 보정: "겠습니다/EP+EF" → "겠/EP + 습니다/EF" 분리
    let mut gyeot_split_indices: Vec<usize> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if token.pos == "EP+EF" && token.surface == "겠습니다" {
            gyeot_split_indices.push(i);
        }
    }

    for idx in gyeot_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        tokens[idx] = SejongToken::new("겠", "EP", start, start + 1);
        tokens.insert(idx + 1, SejongToken::new("습니다", "EF", start + 1, end));
    }

    // 70차 보정: EC + "하/XSV + ㅂ니다/EF" → EC + "합니다/EF"
    // "가야 합니다"에서 보조동사 "합니다"를 단일 종결어미로 병합
    let mut hapnida_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        // EC 다음에 "하/XSV + ㅂ니다/EF" 패턴
        if i > 0
            && tokens[i - 1].pos == "EC"
            && tokens[i].surface == "하"
            && tokens[i].pos == "XSV"
            && tokens[i + 1].surface == "ㅂ니다"
            && tokens[i + 1].pos == "EF"
        {
            hapnida_merge_indices.push(i);
        }
    }

    for idx in hapnida_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new("합니다", "EF", start, end);
        tokens.remove(idx + 1);
    }

    // 71차 보정: 의존명사 "중" 처리
    // "계류 중이다", "분석 중이다" 등에서 "중/NNG + 이/VCP" → "중/NNB + 이/VCP"
    // 앞에 NNG/NNP가 있고 "중"이 오면 의존명사로 처리
    for i in 1..tokens.len() {
        if tokens[i].surface == "중"
            && tokens[i].pos == "NNG"
            && (tokens[i - 1].pos == "NNG" || tokens[i - 1].pos == "NNP")
        {
            // 뒤에 VCP(이다) 또는 JX(조사)가 오는 경우 의존명사
            if i + 1 < tokens.len()
                && (tokens[i + 1].pos == "VCP"
                    || tokens[i + 1].pos == "JX"
                    || tokens[i + 1].pos == "JKS"
                    || tokens[i + 1].pos == "EF")
            {
                tokens[i].pos = "NNB".to_string();
            }
        }
    }

    // 72차 보정: ETM 뒤의 "지" 의존명사 처리
    // "만난 지", "먹은 지" 등에서 ETM + "지/VX" → ETM + "지/NNB"
    for i in 1..tokens.len() {
        if tokens[i].surface == "지"
            && (tokens[i].pos == "VX" || tokens[i].pos == "EC")
            && tokens[i - 1].pos == "ETM"
        {
            tokens[i].pos = "NNB".to_string();
        }
    }

    // 80차 보정: NR 뒤의 시간/단위 의존명사 처리
    // "삼십분", "열시", "백만원" 등에서 NR + "분/XSN" → NR + "분/NNB"
    let time_unit_nouns = ["분", "시", "원", "년", "월", "일", "개", "명", "번"];
    for i in 1..tokens.len() {
        if time_unit_nouns.contains(&tokens[i].surface.as_str())
            && (tokens[i].pos == "XSN" || tokens[i].pos == "NNG")
            && tokens[i - 1].pos == "NR"
        {
            tokens[i].pos = "NNB".to_string();
        }
    }

    // 73차 보정: "것"을 NNB로 처리
    // "것"이 단독으로 오거나 VCP 앞에 오면 의존명사
    for i in 0..tokens.len() {
        if tokens[i].surface == "것" && tokens[i].pos == "NP" {
            // 다음 토큰이 VCP, JKS, JX, NNB 등이면 의존명사
            if i + 1 < tokens.len() {
                let next_pos = &tokens[i + 1].pos;
                if next_pos == "VCP"
                    || next_pos == "JKS"
                    || next_pos == "JX"
                    || next_pos == "JKO"
                    || next_pos == "NNB"
                {
                    tokens[i].pos = "NNB".to_string();
                }
            }
            // 이전 토큰이 ETM이면 의존명사
            if tokens[i].pos == "NP" && i > 0 && tokens[i - 1].pos == "ETM" {
                tokens[i].pos = "NNB".to_string();
            }
            // 단독으로 사용되면 NNB (문장 끝이거나 유일 토큰)
            if tokens[i].pos == "NP" && (tokens.len() == 1 || i == tokens.len() - 1) {
                tokens[i].pos = "NNB".to_string();
            }
        }
    }

    // 74차 보정: 관형형 VV 분리
    // "간/VV", "온/VV", "한/VV" 등이 명사 앞에 오면 VV + ㄴ/ETM으로 분리
    // "갈/VV", "올/VV", "할/VV" 등이 명사 앞에 오면 VV + ㄹ/ETM으로 분리
    let mut adnominal_splits: Vec<(usize, String, String, String)> = Vec::new();
    for i in 0..tokens.len() {
        if tokens[i].pos == "VV" {
            // 다음 토큰이 명사류인지 확인
            let next_is_noun = if i + 1 < tokens.len() {
                let next_pos = &tokens[i + 1].pos;
                next_pos == "NNG" || next_pos == "NNP" || next_pos == "NNB" || next_pos == "NP"
            } else {
                false
            };

            if next_is_noun {
                let surface = &tokens[i].surface;
                // ㄴ 종성 (받침)으로 끝나는 1음절 어휘
                // "간" → "가/VV ㄴ/ETM"
                // "온" → "오/VV ㄴ/ETM"
                // "한" → "하/VV ㄴ/ETM"
                if surface == "간" {
                    adnominal_splits.push((
                        i,
                        "가".to_string(),
                        "VV".to_string(),
                        "ㄴ".to_string(),
                    ));
                } else if surface == "온" {
                    adnominal_splits.push((
                        i,
                        "오".to_string(),
                        "VV".to_string(),
                        "ㄴ".to_string(),
                    ));
                } else if surface == "한" {
                    adnominal_splits.push((
                        i,
                        "하".to_string(),
                        "VV".to_string(),
                        "ㄴ".to_string(),
                    ));
                }
                // ㄹ 종성 (받침)으로 끝나는 1음절 어휘
                // "갈" → "가/VV ㄹ/ETM"
                // "올" → "오/VV ㄹ/ETM"
                // "할" → "하/VV ㄹ/ETM"
                else if surface == "갈" {
                    adnominal_splits.push((
                        i,
                        "가".to_string(),
                        "VV".to_string(),
                        "ㄹ".to_string(),
                    ));
                } else if surface == "올" {
                    adnominal_splits.push((
                        i,
                        "오".to_string(),
                        "VV".to_string(),
                        "ㄹ".to_string(),
                    ));
                } else if surface == "할" {
                    adnominal_splits.push((
                        i,
                        "하".to_string(),
                        "VV".to_string(),
                        "ㄹ".to_string(),
                    ));
                }
            }
        }
    }

    // 역순으로 처리
    for (idx, stem, stem_pos, ending) in adnominal_splits.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        tokens[idx] = SejongToken::new(&stem, &stem_pos, start, end);
        tokens.insert(idx + 1, SejongToken::new(&ending, "ETM", end, end));
    }

    // 75차 보정: ㄹ 탈락 동사 기본형 복원 (VV + 세요/EF 패턴)
    // "드/VV + 세요/EF" → "들/VV + 세요/EF" (들다 → 드세요)
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i].pos == "VV" && tokens[i + 1].surface == "세요" && tokens[i + 1].pos == "EF" {
            // ㄹ 탈락 동사 패턴
            let rieul_verbs = [
                ("드", "들"), // 들다 → 드세요
                ("아", "알"), // 알다 → 아세요
            ];
            for (dropped, original) in rieul_verbs {
                if tokens[i].surface == dropped {
                    tokens[i].surface = original.to_string();
                    break;
                }
            }
        }
    }

    // 76차 보정: 파생명사 → VV + 음/ETN 분리
    // "웃음/NNG", "울음/NNG" 등을 "웃/VV + 음/ETN"으로 분리
    let derived_nouns: std::collections::HashMap<&str, (&str, &str)> = [
        ("웃음", ("웃", "VV")),
        ("울음", ("울", "VV")),
        ("걸음", ("걷", "VV")),
        ("놀이", ("놀", "VV")),
        ("먹이", ("먹", "VV")),
        ("잠", ("자", "VV")),
        ("꿈", ("꾸", "VV")),
    ]
    .into_iter()
    .collect();

    let mut derived_split_indices: Vec<(usize, String, String, String)> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if token.pos == "NNG" {
            if let Some(&(stem, stem_pos)) = derived_nouns.get(token.surface.as_str()) {
                // 어미 결정: 음/ㅁ/이
                let suffix = if token.surface.ends_with("음") {
                    "음"
                } else if token.surface == "잠" || token.surface == "꿈" {
                    "ㅁ"
                } else if token.surface.ends_with("이") {
                    "이"
                } else {
                    continue;
                };
                derived_split_indices.push((
                    i,
                    stem.to_string(),
                    stem_pos.to_string(),
                    suffix.to_string(),
                ));
            }
        }
    }

    for (idx, stem, stem_pos, suffix) in derived_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        let stem_len = stem.chars().count();
        tokens[idx] = SejongToken::new(&stem, &stem_pos, start, start + stem_len);
        tokens.insert(
            idx + 1,
            SejongToken::new(&suffix, "ETN", start + stem_len, end),
        );
    }

    // 77차 보정: 단음절 VV + ㄴ/ㄹ 받침 → VV + ETM 분리
    // "간", "온", "한", "갈", "올", "할" 등의 관형형을 분리
    // 예: "간 사람" → "가/VV ㄴ/ETM 사람/NNG"
    // 예: "간 온" → "가/VV ㄴ/ETM 오/VV ㄴ/ETM"
    let vv_etm_patterns: std::collections::HashMap<&str, (&str, &str)> = [
        // ㄴ/은 관형형 (과거/완료)
        ("간", ("가", "ㄴ")), // 가다
        ("온", ("오", "ㄴ")), // 오다
        ("한", ("하", "ㄴ")), // 하다
        ("본", ("보", "ㄴ")), // 보다
        ("잔", ("자", "ㄴ")), // 자다
        ("산", ("사", "ㄴ")), // 사다
        ("된", ("되", "ㄴ")), // 되다
        ("쓴", ("쓰", "ㄴ")), // 쓰다
        // ㄹ/을 관형형 (미래/추측)
        ("갈", ("가", "ㄹ")), // 가다
        ("올", ("오", "ㄹ")), // 오다
        ("할", ("하", "ㄹ")), // 하다
        ("볼", ("보", "ㄹ")), // 보다
        ("살", ("살", "ㄹ")), // 살다 (ㄹ 불규칙)
        ("알", ("알", "ㄹ")), // 알다 (ㄹ 불규칙)
        ("될", ("되", "ㄹ")), // 되다
    ]
    .into_iter()
    .collect();

    let mut etm_split_indices: Vec<(usize, String, String)> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        // VV/VA 단일 토큰 (단음절)
        if (token.pos == "VV" || token.pos == "VA") && token.surface.chars().count() == 1 {
            if let Some(&(stem, etm)) = vv_etm_patterns.get(token.surface.as_str()) {
                // 조건: 뒤에 명사, 다른 VV, 의존명사, 지시대명사 등이 오는 경우
                // 또는 문장 끝이 아닌 경우 (단독 VV는 관형형으로 분리)
                let should_split = if i + 1 < tokens.len() {
                    let next_pos = &tokens[i + 1].pos;
                    // 명사, 대명사, 다른 동사/형용사 앞에서 분리
                    next_pos.starts_with("NN")
                        || next_pos == "NP"
                        || next_pos == "VV"
                        || next_pos == "VA"
                        || next_pos == "MM"
                        || next_pos == "MAG"
                } else {
                    // 문장 끝에서도 분리 (sample.tsv 기준)
                    true
                };

                if should_split {
                    etm_split_indices.push((i, stem.to_string(), etm.to_string()));
                }
            }
        }
    }

    for (idx, stem, etm) in etm_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        let stem_len = stem.chars().count();
        tokens[idx] = SejongToken::new(&stem, "VV", start, start + stem_len);
        tokens.insert(
            idx + 1,
            SejongToken::new(&etm, "ETM", start + stem_len, end),
        );
    }

    // 78차 보정: XSV 복합 패턴 분리
    // "되었다/XSV" → "되/XSV 었/EP 다/EF"
    // "하였다/XSV" → "하/XSV 었/EP 다/EF"
    let xsv_split_patterns: std::collections::HashMap<&str, (&str, &str, &str)> = [
        ("되었다", ("되", "었", "다")),
        ("하였다", ("하", "었", "다")),
        ("되었어", ("되", "었", "어")),
        ("하였어", ("하", "었", "어")),
        ("되었으면", ("되", "었", "으면")),
        ("하였으면", ("하", "었", "으면")),
    ]
    .into_iter()
    .collect();

    let mut xsv_split_indices: Vec<(usize, String, String, String)> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if token.pos == "XSV" {
            if let Some(&(stem, ep, ef)) = xsv_split_patterns.get(token.surface.as_str()) {
                xsv_split_indices.push((i, stem.to_string(), ep.to_string(), ef.to_string()));
            }
        }
    }

    for (idx, stem, ep, ef) in xsv_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        let stem_len = stem.chars().count();
        let ep_len = ep.chars().count();
        tokens[idx] = SejongToken::new(&stem, "XSV", start, start + stem_len);
        tokens.insert(
            idx + 1,
            SejongToken::new(&ep, "EP", start + stem_len, start + stem_len + ep_len),
        );
        // ef_pos 결정: 다/EF, 어/EF, 으면/EC
        let ef_pos = if ef == "다" || ef == "어" {
            "EF"
        } else {
            "EC"
        };
        tokens.insert(
            idx + 2,
            SejongToken::new(&ef, ef_pos, start + stem_len + ep_len, end),
        );
    }

    // 79차 보정: VV 뒤의 "이/MM" → "이/ETN"
    // 파생명사 패턴: 먹이, 놀이 등에서 MeCab이 "이/MM"으로 잘못 태깅
    for i in 1..tokens.len() {
        if tokens[i].surface == "이" && tokens[i].pos == "MM" && tokens[i - 1].pos == "VV" {
            // 특정 어간 뒤에서만 적용 (명사형어미가 아닌 경우 방지)
            let prev_surface = &tokens[i - 1].surface;
            let etn_triggers = ["먹", "놀", "알", "살", "높", "낮", "깊", "넓", "짧"];
            if etn_triggers.iter().any(|&s| prev_surface == s) {
                tokens[i].pos = "ETN".to_string();
            }
        }
    }

    // 81차 보정: VCP + 시/NNB + 어요/EF → VCP + 세요/EF
    // "누구세요" = "누구/NP 이/VCP 세요/EF"
    // MeCab이 "이/VCP 시/NNB 어요/EF"로 분리하는 경우 병합
    let mut seyo_merge_indices: Vec<usize> = Vec::new();
    for i in 2..tokens.len() {
        if tokens[i - 2].pos == "VCP"
            && tokens[i - 1].surface == "시"
            && (tokens[i - 1].pos == "NNB" || tokens[i - 1].pos == "EP")
            && tokens[i].surface == "어요"
            && tokens[i].pos == "EF"
        {
            seyo_merge_indices.push(i - 1);
        }
    }

    for idx in seyo_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new("세요", "EF", start, end);
        tokens.remove(idx + 1);
    }

    // 82차 보정: "아/EC" → "어/EC" 통일
    // sample.tsv에서는 모음 조화와 관계없이 "어/EC"를 사용
    // 예: "하/VV 아/EC" → "하/VV 어/EC", "위하/VV 아/EC" → "위하/VV 어/EC"
    for token in tokens.iter_mut() {
        if token.pos == "EC" && token.surface == "아" {
            token.surface = "어".to_string();
        }
    }

    // 83차 보정: VV/VA/XSV 뒤의 "어/IC" → "어/EC"
    // "먹어 버렸다"에서 "어"가 IC(감탄사)로 태깅되는 오류 수정
    // 동사/형용사 뒤의 "어"는 연결어미
    for i in 1..tokens.len() {
        if tokens[i].surface == "어"
            && tokens[i].pos == "IC"
            && (tokens[i - 1].pos == "VV"
                || tokens[i - 1].pos == "VA"
                || tokens[i - 1].pos == "XSV")
        {
            tokens[i].pos = "EC".to_string();
        }
    }

    // 84차 보정: 명사 뒤의 "이/MM" → "이/JKS"
    // "성장률이", "의료진이" 등에서 "이"가 관형사(MM)로 태깅되는 오류 수정
    // 받침 있는 명사 뒤의 "이"는 주격 조사
    for i in 1..tokens.len() {
        if tokens[i].surface == "이"
            && tokens[i].pos == "MM"
            && (tokens[i - 1].pos == "NNG"
                || tokens[i - 1].pos == "NNP"
                || tokens[i - 1].pos == "NNB"
                || tokens[i - 1].pos == "NP"
                || tokens[i - 1].pos == "XSN")
        {
            // 이전 토큰의 마지막 글자에 받침이 있는지 확인
            if let Some(last_char) = tokens[i - 1].surface.chars().last() {
                if has_jongseong(last_char) {
                    tokens[i].pos = "JKS".to_string();
                }
            }
        }
    }

    // 264차 보정: NNG/NNP 뒤의 "에/IC" → "에/JKB"
    // "순방길에", "인스타에", "회의에" 등에서 "에"가 IC로 태깅되는 오류 수정
    // 명사 뒤의 "에"는 부사격 조사
    for i in 1..tokens.len() {
        if tokens[i].surface == "에"
            && tokens[i].pos == "IC"
            && (tokens[i - 1].pos == "NNG"
                || tokens[i - 1].pos == "NNP"
                || tokens[i - 1].pos == "NNB")
        {
            tokens[i].pos = "JKB".to_string();
        }
    }

    // 85차 보정: NP 뒤의 "야/IC" → "이/VCP 야/EF" 분리
    // "뭐야", "누구야" 등에서 "야"가 감탄사로 태깅되는 오류 수정
    let mut ya_split_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len() {
        if tokens[i].surface == "야" && tokens[i].pos == "IC" && tokens[i - 1].pos == "NP" {
            // 이전 토큰의 마지막 글자에 받침이 없으면 VCP 분리
            if let Some(last_char) = tokens[i - 1].surface.chars().last() {
                if !has_jongseong(last_char) {
                    ya_split_indices.push(i);
                }
            }
        }
    }

    // 역순으로 처리하여 인덱스 변경 방지
    for idx in ya_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        // "야/IC" → "이/VCP 야/EF"
        tokens[idx] = SejongToken::new("이", "VCP", start, start);
        tokens.insert(idx + 1, SejongToken::new("야", "EF", start, end));
    }

    // 171차 보정: NNG + "이/NP" + "네/XSN" → NNG + "이/VCP" + "네/EF"
    // "노잼이네"가 "노잼/NNG 이/NP 네/XSN"으로 분석될 때
    // → "노잼/NNG 이/VCP 네/EF"로 수정
    for i in 1..tokens.len().saturating_sub(1) {
        if tokens[i].surface == "이"
            && tokens[i].pos == "NP"
            && tokens[i - 1].pos == "NNG"
            && tokens[i + 1].surface == "네"
            && tokens[i + 1].pos == "XSN"
        {
            tokens[i].pos = "VCP".to_string();
            tokens[i + 1].pos = "EF".to_string();
        }
    }

    // 172차 보정: "-러" 목적 연결어미 분리
    // MeCab이 "보러", "놀러" 등을 JKB/NNP로 잘못 분석하는 경우
    // VV+EF 패턴("갈래", "가자", "가요") 앞의 "-러" 표면형을 VV+EC로 분리
    let mut reo_split_indices: Vec<(usize, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let surface = &tokens[i].surface;
        let pos = &tokens[i].pos;
        let next_pos = &tokens[i + 1].pos;

        // "-러"로 끝나고 JKB/NNP이면서 다음이 VV+EF 또는 VV
        if surface.ends_with("러")
            && surface.chars().count() >= 2
            && (pos == "JKB" || pos == "NNP")
            && (next_pos == "VV+EF" || next_pos == "VV")
        {
            let stem: String = surface.chars().take(surface.chars().count() - 1).collect();
            reo_split_indices.push((i, stem));
        }
    }

    for (idx, stem) in reo_split_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        let stem_len = stem.chars().count();
        // "보러/JKB" → "보/VV 러/EC"
        tokens[idx] = SejongToken::new(&stem, "VV", start, start + stem_len);
        tokens.insert(idx + 1, SejongToken::new("러", "EC", start + stem_len, end));
    }
}
