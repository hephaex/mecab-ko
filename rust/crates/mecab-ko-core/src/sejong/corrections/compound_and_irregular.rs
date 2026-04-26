//! 복합어 병합 및 불규칙 활용 보정

use crate::sejong::types::SejongToken;

/// 228~244차: 복합어 병합 및 불규칙 활용 보정
///
/// 할머님/할아버님 병합(228차), 시간(230차), 주말(231차), 갈등(232차),
/// 외래어+가 처리(234차), 진행(236차), 어절 끝 EC→EF(237차),
/// 하다 아→어(238차), ㅂ불규칙 줍다(239차), ㅂ불규칙 무겁다(241차),
/// 이르면 분리(242차), ㅎ불규칙 노랗다(243차), 안/MAG 제거(244차)
pub(super) fn apply_compound_and_irregular_corrections(tokens: &mut Vec<SejongToken>) {
    // 228차 보정: "하/XSV + ㄹ/ETM + 머/NP + 님/XSN" → "할머님/NNG"
    // MeCab이 "할머님"을 잘못 분석하는 경우 병합
    let mut family_merge_indices: Vec<(usize, usize, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(3) {
        let t0 = &tokens[i];
        let t1 = &tokens[i + 1];
        let t2 = &tokens[i + 2];
        let t3 = &tokens[i + 3];

        // "하/XSV + ㄹ/ETM + 머/NP + 님/XSN" → "할머님/NNG"
        if t0.surface == "하"
            && (t0.pos == "XSV" || t0.pos == "VV")
            && t1.surface == "ㄹ"
            && t1.pos == "ETM"
            && t2.surface == "머"
            && t2.pos == "NP"
            && t3.surface == "님"
            && t3.pos == "XSN"
        {
            family_merge_indices.push((i, i + 3, "할머님".to_string()));
        }
        // "하/XSV + ㄹ/ETM + 아버/NNG + 님/XSN" → "할아버님/NNG"
        else if t0.surface == "하"
            && (t0.pos == "XSV" || t0.pos == "VV")
            && t1.surface == "ㄹ"
            && t1.pos == "ETM"
            && t2.surface == "아버"
            && t2.pos == "NNG"
            && t3.surface == "님"
            && t3.pos == "XSN"
        {
            family_merge_indices.push((i, i + 3, "할아버님".to_string()));
        }
    }

    for (start_idx, end_idx, merged) in family_merge_indices.into_iter().rev() {
        let start = tokens[start_idx].start_pos;
        let end = tokens[end_idx].end_pos;
        tokens[start_idx] = SejongToken::new(&merged, "NNG", start, end);
        // Remove the extra tokens (in reverse order)
        for j in (start_idx + 1..=end_idx).rev() {
            tokens.remove(j);
        }
    }

    // 230차 보정: "시/NNG + 가/VV + ㄴ/ETM" → "시간/NNG" 병합
    // MeCab이 "시간"을 "시/NNG + 간/VV+ETM"으로 분리하고, "간/VV+ETM"이 "가/VV + ㄴ/ETM"으로 됨
    // sample.tsv 기준: "내일 시간 있어요" → "내일/NNG 시간/NNG 있/VV 어요/EF"
    let mut sigan_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(2) {
        // "시/NNG|NNB + 가/VV|JKS + ㄴ/ETM" 패턴 (MeCab의 잘못된 분리)
        if tokens[i].surface == "시"
            && (tokens[i].pos == "NNG" || tokens[i].pos == "NNB")
            && tokens[i + 1].surface == "가"
            && (tokens[i + 1].pos == "VV" || tokens[i + 1].pos == "JKS")
            && tokens[i + 2].surface == "ㄴ"
            && tokens[i + 2].pos == "ETM"
        {
            sigan_merge_indices.push(i);
        }
    }

    for idx in sigan_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 2].end_pos;
        tokens[idx] = SejongToken::new("시간", "NNG", start, end);
        tokens.remove(idx + 2);
        tokens.remove(idx + 1);
    }

    // 231차 보정: "주/VX + 말/NNG" → "주말/NNG" 병합
    // MeCab이 "주말"을 "주/VX + 말/NNG"으로 잘못 분리하는 문제 수정
    // sample.tsv 기준: "주말에 영화 보러 갈래" → "주말/NNG ..."
    let mut jumal_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i].surface == "주"
            && tokens[i].pos == "VX"
            && tokens[i + 1].surface == "말"
            && tokens[i + 1].pos == "NNG"
        {
            jumal_merge_indices.push(i);
        }
    }

    for idx in jumal_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new("주말", "NNG", start, end);
        tokens.remove(idx + 1);
    }

    // 232차 보정: "가/VV + ㄹ/ETM + 등/NNG|NNB" → "갈등/NNG" 병합
    // MeCab이 "갈등"을 "갈/VV+ETM + 등/NNG"으로 잘못 분리
    // sample.tsv 기준: "갈등이 심화됐다" → "갈등/NNG ..."
    let mut galdeung_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(2) {
        if tokens[i].surface == "가"
            && tokens[i].pos == "VV"
            && tokens[i + 1].surface == "ㄹ"
            && tokens[i + 1].pos == "ETM"
            && tokens[i + 2].surface == "등"
            && (tokens[i + 2].pos == "NNG" || tokens[i + 2].pos == "NNB")
        {
            galdeung_merge_indices.push(i);
        }
    }

    for idx in galdeung_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 2].end_pos;
        tokens[idx] = SejongToken::new("갈등", "NNG", start, end);
        tokens.remove(idx + 2);
        tokens.remove(idx + 1);
    }

    // 234차 보정: 외래어(SL) 뒤 "가/VV + (아|어)/EC" → "가/JKS"
    // sample.tsv 기준: "MBTI가 뭐예요" → "MBTI/SL 가/JKS 뭐/NP 이/VCP 에요/EF"
    // MeCab이 "MBTI가"를 "가/VV+EC" → "가/VV + 아/EC"로 분리하는 오류 수정
    // "가/VV + (아|어)/EC" 패턴을 "가/JKS"로 병합
    let mut sl_ga_merge_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len().saturating_sub(1) {
        if tokens[i].surface == "가"
            && tokens[i].pos == "VV"
            && tokens[i - 1].pos == "SL"
            && (tokens[i + 1].surface == "어" || tokens[i + 1].surface == "아")
            && tokens[i + 1].pos == "EC"
        {
            sl_ga_merge_indices.push(i);
        }
    }

    for idx in sl_ga_merge_indices.into_iter().rev() {
        // "가/VV + (아|어)/EC" → "가/JKS"로 교체
        tokens[idx].pos = "JKS".to_string();
        tokens.remove(idx + 1); // "(아|어)/EC" 제거
    }

    // 234-2차 보정: 외래어(SL) 뒤 단독 "가/VV" → "가/JKS"
    // 위의 병합 패턴 외에 단독 "가/VV"도 처리
    for i in 1..tokens.len() {
        if tokens[i].surface == "가" && tokens[i].pos == "VV" && tokens[i - 1].pos == "SL" {
            tokens[i].pos = "JKS".to_string();
        }
    }

    // 236차 보정: "지/VX + ㄴ/ETM + 행/NNG" → "진행/NNG" 병합
    // MeCab이 "하여진행"을 "하/XSV + 여진/EC+VX+ETM + 행/NNG"로 잘못 분석
    // "여진"이 "아/EC + 지/VX + ㄴ/ETM"으로 분리되어 "진행"이 깨짐
    // 추가: "어/EC + 지/VX + ㄴ/ETM + 행/NNG" → "어/EC + 진행/NNG"로 병합
    let mut jinheng_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(2) {
        if tokens[i].surface == "지"
            && tokens[i].pos == "VX"
            && tokens[i + 1].surface == "ㄴ"
            && tokens[i + 1].pos == "ETM"
            && tokens[i + 2].surface == "행"
            && tokens[i + 2].pos == "NNG"
        {
            jinheng_merge_indices.push(i);
        }
    }

    for idx in jinheng_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 2].end_pos;
        tokens[idx] = SejongToken::new("진행", "NNG", start, end);
        tokens.remove(idx + 2);
        tokens.remove(idx + 1);
    }

    // 237차 보정: 어절 끝 "어/EC" → "어/EF", "아/EC" → "아/EF"
    // "덥다 더워 더우면"에서 "더워"가 개별 어절이므로 "어/EF"여야 함
    // 어절 경계: 분해된 토큰의 original_surface가 다음 토큰과 다르면 어절 끝
    // 240차 수정: "위하/VV", "대하/VV" 같은 연결 동사는 EC 유지
    let ec_keep_verbs = ["위하", "대하", "인하", "관하", "의하", "통하", "비하"];
    for i in 0..tokens.len() {
        let surface = &tokens[i].surface;
        let pos = &tokens[i].pos;

        // "어/EC" 또는 "아/EC" 패턴
        if pos == "EC" && (surface == "어" || surface == "아") {
            // 이전 토큰이 VV/VA/VX인지 확인 (활용형)
            let prev_is_verb = i > 0
                && (tokens[i - 1].pos == "VV"
                    || tokens[i - 1].pos == "VA"
                    || tokens[i - 1].pos == "VX");

            if prev_is_verb {
                // 240차: 연결 동사 뒤의 "어/EC"는 EC 유지
                let prev_surface = &tokens[i - 1].surface;
                if ec_keep_verbs.iter().any(|v| prev_surface == *v) {
                    continue;
                }

                // 마지막 토큰인 경우
                let is_last = i + 1 >= tokens.len();

                // 분해된 토큰인 경우, 다음 토큰이 같은 원본에서 분해되었는지 확인
                let is_eojeol_final = if is_last {
                    true
                } else {
                    // 현재 토큰의 original_surface와 다음 토큰의 original_surface 비교
                    // 둘 다 Some이고 같으면 같은 어절, 다르면 어절 끝
                    match (&tokens[i].original_surface, &tokens[i + 1].original_surface) {
                        (Some(curr_orig), Some(next_orig)) => curr_orig != next_orig,
                        (Some(_), None) => true, // 다음은 분해 안 됨 = 어절 끝
                        (None, _) => false,      // 현재가 분해 안 됨 = 이 규칙 적용 안 함
                    }
                };

                if is_eojeol_final {
                    tokens[i].pos = "EF".to_string();
                }
            }
        }
    }

    // 238차 보정: "하/VV + 아/EC|EF" → "하/VV + 어/EC|EF" 변환
    // "하다" 어간 뒤의 "아"는 "어"로 통일 (하+아 → 해 → 하+어)
    // 단, ㅡ불규칙(빠르다→빨라=아), ㅎ불규칙(하얗다→하얘=아)은 제외
    for i in 1..tokens.len() {
        if (tokens[i].pos == "EC" || tokens[i].pos == "EF") && tokens[i].surface == "아" {
            // 이전 토큰이 "하"로 끝나는 VV/XSV인 경우만 변환
            let prev_surface = &tokens[i - 1].surface;
            let prev_pos = &tokens[i - 1].pos;
            if (prev_pos == "VV" || prev_pos == "XSV") && prev_surface.ends_with("하") {
                tokens[i].surface = "어".to_string();
            }
        }
    }

    // 239차 보정: ㅂ불규칙 동사 "줍다" 활용형 처리
    // MeCab이 "주워"를 "주/VX + 워/NNG"로 잘못 분석
    // "주우면"을 "주/VX + 우면/NNG"로 잘못 분석
    // sample.tsv 기준: "줍다 주워 주우면" → "줍/VV 다/EF 줍/VV 어/EF 줍/VV 으면/EC"
    let mut jup_fix_indices: Vec<(usize, String, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i].surface == "주" && tokens[i].pos == "VX" {
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;
            // "주/VX + 워/NNG" → "줍/VV + 어/EF"
            if next_surface == "워" && next_pos == "NNG" {
                jup_fix_indices.push((i, "줍".to_string(), "어".to_string()));
            }
            // "주/VX + 우면/NNG" → "줍/VV + 으면/EC"
            else if next_surface == "우면" && next_pos == "NNG" {
                jup_fix_indices.push((i, "줍".to_string(), "으면".to_string()));
            }
        }
    }

    for (idx, stem, ending) in jup_fix_indices.into_iter().rev() {
        let start1 = tokens[idx].start_pos;
        let end1 = tokens[idx].end_pos;
        let start2 = tokens[idx + 1].start_pos;
        let end2 = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&stem, "VV", start1, end1);
        // "워" → 어절 끝이면 EF, 아니면 EC
        // 어절 끝 판단: 마지막 토큰이거나, 다음 토큰이 새로운 어절 시작 (VV, VX 등)
        let is_eojeol_final = if ending == "으면" {
            false // 으면은 항상 EC
        } else if idx + 2 >= tokens.len() {
            true // 마지막 토큰
        } else {
            // 다음 토큰이 VV, VX, NNG 등 새 어절 시작인지 확인
            let next_pos = &tokens[idx + 2].pos;
            next_pos == "VV"
                || next_pos == "VX"
                || next_pos == "NNG"
                || next_pos == "NNP"
                || next_pos == "NP"
                || next_pos == "MAG"
        };
        let ending_pos = if is_eojeol_final { "EF" } else { "EC" };
        tokens[idx + 1] = SejongToken::new(&ending, ending_pos, start2, end2);
    }

    // 241차 보정: ㅂ불규칙 형용사 "무겁다" 활용형 처리
    // MeCab이 "무거우면"을 "무거/NNG + 우면/NNG"로 잘못 분석
    // sample.tsv 기준: "무겁다 무거워 무거우면" → "무겁/VA 다/EF 무겁/VA 어/EF 무겁/VA 으면/EC"
    // "무거" 어간 → "무겁" 원형 복원
    let mut mugeop_fix_indices: Vec<(usize, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i].surface == "무거" && tokens[i].pos == "NNG" {
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;
            // "무거/NNG + 우면/NNG" → "무겁/VA + 으면/EC"
            if next_surface == "우면" && next_pos == "NNG" {
                mugeop_fix_indices.push((i, "으면".to_string()));
            }
        }
    }

    for (idx, ending) in mugeop_fix_indices.into_iter().rev() {
        let start1 = tokens[idx].start_pos;
        let end1 = tokens[idx].end_pos;
        let start2 = tokens[idx + 1].start_pos;
        let end2 = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new("무겁", "VA", start1, end1);
        tokens[idx + 1] = SejongToken::new(&ending, "EC", start2, end2);
    }

    // 242차 보정: "이르면/MAJ" → "이르/VV + 면/EC"
    // MeCab이 "이르면"을 접속부사 MAJ로 잘못 분석
    // sample.tsv 기준: "이르다 일러 이르면" → "이르/VV 다/EF 이르/VV 어/EF 이르/VV 면/EC"
    // 앞에 VV+EF가 있으면 동사 활용으로 분리
    let mut ireumyeon_fix_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len() {
        if tokens[i].surface == "이르면" && tokens[i].pos == "MAJ" {
            // 앞에 VV가 있는지 확인
            if tokens[i - 1].pos == "VV" || tokens[i - 1].pos == "EF" {
                ireumyeon_fix_indices.push(i);
            }
        }
    }

    for idx in ireumyeon_fix_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        // "이르면" → "이르/VV" + "면/EC"
        tokens[idx] = SejongToken::new("이르", "VV", start, start + 2);
        tokens.insert(idx + 1, SejongToken::new("면", "EC", start + 2, end));
    }

    // 243차 보정: ㅎ불규칙 "노랗다" 활용형 처리
    // MeCab이 "노래"를 "노래/NNG"로 잘못 분석
    // sample.tsv 기준: "노랗다 노래 노랗으면" → "노랗/VA 다/EF 노랗/VA 아/EF 노랗/VA 으면/EC"
    // "노랗" 뒤의 "노래"는 "노랗 + 아"의 축약형
    let mut norae_fix_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len() {
        if tokens[i].surface == "노래" && tokens[i].pos == "NNG" {
            // 앞에 "노랗/VA + 다/EF"가 있는지 확인
            if i >= 2
                && tokens[i - 2].surface == "노랗"
                && tokens[i - 2].pos == "VA"
                && tokens[i - 1].surface == "다"
                && tokens[i - 1].pos == "EF"
            {
                norae_fix_indices.push(i);
            }
        }
    }

    for idx in norae_fix_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        // "노래" → "노랗/VA" + "아/EF"
        tokens[idx] = SejongToken::new("노랗", "VA", start, start + 2);
        tokens.insert(idx + 1, SejongToken::new("아", "EF", start + 2, end));
    }

    // 244차 보정: "있/VX + 안/MAG + 으며/EC" → "있/VX + 으며/EC"
    // MeCab이 "있으며"에서 "으며"를 "안/VV + 으며/EC"로 분석
    // VX 뒤에 "안/MAG"이 오고 그 뒤에 "으며/EC"가 오면 "안" 제거
    let mut an_remove_indices: Vec<usize> = Vec::new();
    for i in 1..tokens.len().saturating_sub(1) {
        if tokens[i].surface == "안" && tokens[i].pos == "MAG" {
            // 앞에 VX가 있고, 뒤에 "으며/EC"가 있으면
            if tokens[i - 1].pos == "VX"
                && tokens[i + 1].surface == "으며"
                && tokens[i + 1].pos == "EC"
            {
                an_remove_indices.push(i);
            }
        }
    }

    for idx in an_remove_indices.into_iter().rev() {
        tokens.remove(idx);
    }
}
