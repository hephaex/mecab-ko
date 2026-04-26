use super::sentence_final_endings::apply_sentence_final_endings_corrections;
use super::xsv_and_ec_ef::apply_xsv_and_ec_ef_corrections;
use super::xsv_morpheme_split::apply_xsv_morpheme_split_corrections;
use crate::sejong::types::SejongToken;

/// 133차 보정에 사용되는 동사 기본형 목록 (NNG → VV + 다/EF 변환 대상)
const VERB_BASE_FORMS: &[&str] = &[
    "하다", "가다", "오다", "보다", "사다", "주다", "타다", "서다", "나다",
];

/// 89~148차: 문장 종결·EC/EF 변환 보정 (전반부)
///
/// XSV/XSA 변환, EC↔EF 변환, 보조동사 VX 패턴,
/// 동사 기본형 NNG→VV, 부정 부사 분리, 인용문 패턴 등
/// 89~148차 보정 패스를 포함합니다.
pub(super) fn apply_sentence_final_corrections(tokens: &mut Vec<SejongToken>) {
    // 89~112차: XSV 변환, EC↔EF 변환, 보조동사 VX 패턴
    apply_xsv_and_ec_ef_corrections(tokens);

    // 113~134차: 복합 EC 병합, EF 정규화, NNP 분리, 청유형/의문형 분리
    apply_xsv_morpheme_split_corrections(tokens);

    // 133차 보정: 동사 기본형 NNG → VV + 다/EF
    // "하다/NNG" → "하/VV + 다/EF"
    // 단독으로 나오는 동사 기본형 (주의: 명사 "하다"와 구분 필요)
    let mut split_verb_base_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len() {
        let surface = &tokens[i].surface;
        let pos = &tokens[i].pos;

        // NNG로 분석된 동사 기본형이면서 단독으로 쓰인 경우
        // (다음 토큰이 없거나 다른 동사 기본형이 이어지는 경우)
        if pos == "NNG" && VERB_BASE_FORMS.contains(&surface.as_str()) {
            let is_standalone = if i + 1 < tokens.len() {
                // 다음 토큰이 동사/형용사 관련 태그가 아닌 경우
                let next_pos = &tokens[i + 1].pos;
                !next_pos.starts_with("VV")
                    && !next_pos.starts_with("VA")
                    && !next_pos.starts_with("EC")
                    && !next_pos.starts_with("EF")
                    && !next_pos.starts_with("EP")
            } else {
                true
            };

            if is_standalone {
                split_verb_base_indices.push(i);
            }
        }
    }

    for idx in split_verb_base_indices.into_iter().rev() {
        let surface = tokens[idx].surface.clone();
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;
        // 어간 추출 ("~다"에서 "다" 제거)
        let stem: String = surface.chars().take(surface.chars().count() - 1).collect();
        tokens[idx].surface = stem;
        tokens[idx].pos = "VV".to_string();
        tokens.insert(idx + 1, SejongToken::new("다", "EF", start, end));
    }

    // 135차 보정: 동사 기본형 뒤 XSV → VV
    // "하다 했다"에서 두 번째 "하/XSV" → "하/VV"
    // 다/EF 바로 뒤에 오는 하/XSV는 독립 동사로 변환
    for i in 1..tokens.len() {
        if i >= 2 {
            let prev_pos = &tokens[i - 1].pos;
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;

            // 이전 토큰이 다/EF이고 현재 토큰이 하/XSV인 경우
            if prev_pos == "EF" && curr_surface == "하" && curr_pos == "XSV" {
                tokens[i].pos = "VV".to_string();
            }
        }
    }

    // 136차 보정: "~세/VV + 아요/EF" → "VV + 세요/EF"
    // "오세요"가 세종 변환 후 "오세/VV + 아요/EF"로 되는 경우
    // 동사 어간 + 세 → 동사 어간 + 세요/EF
    let mut fix_seyo_indices: Vec<(usize, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // ~세/VV + 아요/EF 패턴 (세종 변환 후)
        if curr_pos == "VV"
            && curr_surface.ends_with("세")
            && curr_surface.chars().count() >= 2
            && next_surface == "아요"
            && next_pos == "EF"
        {
            // 동사 어간 추출 (세 제거)
            let stem: String = curr_surface
                .chars()
                .take(curr_surface.chars().count() - 1)
                .collect();
            fix_seyo_indices.push((i, stem));
        }
    }

    for (idx, stem) in fix_seyo_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = if idx + 1 < tokens.len() {
            tokens[idx + 1].end_pos
        } else {
            tokens[idx].end_pos
        };
        // 동사 어간으로 변경
        tokens[idx].surface = stem;
        // 아요를 세요로 변경
        tokens[idx + 1].surface = "세요".to_string();
        tokens[idx + 1].start_pos = start;
        tokens[idx + 1].end_pos = end;
    }

    // 137차 보정: "안가/VV", "못가/VV" → "안/MAG + 가/VV", "못/MAG + 가/VV"
    // MeCab이 "안 가요", "못 가요"를 "안가/VV + 아요/EF"로 분석하는 경우
    let neg_adverbs: std::collections::HashSet<&str> = ["안", "못"].into_iter().collect();
    let mut split_neg_indices: Vec<(usize, String, String)> = Vec::new();

    for i in 0..tokens.len() {
        let surface = &tokens[i].surface;
        let pos = &tokens[i].pos;

        // 2글자 VV가 "안" 또는 "못"으로 시작하는 경우
        if pos == "VV" && surface.chars().count() == 2 {
            let first_char: String = surface.chars().take(1).collect();
            if neg_adverbs.contains(first_char.as_str()) {
                let verb_stem: String = surface.chars().skip(1).collect();
                split_neg_indices.push((i, first_char, verb_stem));
            }
        }
    }

    // 역순으로 처리 (인덱스 변경 방지)
    for (idx, adv, stem) in split_neg_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx].end_pos;

        // 원래 토큰을 부사로 변경
        tokens[idx].surface = adv;
        tokens[idx].pos = "MAG".to_string();
        tokens[idx].end_pos = start; // 부사는 첫 글자만

        // 동사 어간 토큰 삽입
        tokens.insert(
            idx + 1,
            SejongToken {
                surface: stem.clone(),
                pos: "VV".to_string(),
                start_pos: start,
                end_pos: end,
                original_surface: Some(stem),
                original_pos: Some("VV".to_string()),
            },
        );
    }

    // 138차 보정: 인용문 패턴 수정
    // 패턴 1: "~자/EF + 이/VCP + 고/EC" → "~자고/EC" (VCP 제거, EF→EC, 고 병합)
    // 패턴 2: "~다/EF + 고하/VV" → "~다고/EC + 하/VV" (고하 분리)
    let mut quote_fix_indices: Vec<(usize, String, bool)> = Vec::new(); // (idx, new_surface, remove_next)

    for i in 0..tokens.len().saturating_sub(2) {
        let curr_surface = tokens[i].surface.clone();
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;
        let next_pos = &tokens[i + 1].pos;

        // 패턴 1: ~자/EF + 이/VCP + 고/EC → 자고/EC
        if curr_pos == "EF"
            && curr_surface == "자"
            && next_surface == "이"
            && next_pos == "VCP"
            && i + 2 < tokens.len()
            && tokens[i + 2].surface == "고"
            && tokens[i + 2].pos == "EC"
        {
            quote_fix_indices.push((i, "자고".to_string(), true));
        }

        // 패턴 2: ~다/EF + 고하/VV → 다고/EC + 하/VV
        if curr_pos == "EF" && curr_surface == "다" && next_surface == "고하" && next_pos == "VV"
        {
            // 이 경우 다/EF를 다고/EC로 변경하고, 고하/VV를 하/VV로 변경
            // 별도 처리 필요
        }
    }

    // 패턴 1 적용: 역순 처리
    for (idx, new_surface, remove_next) in quote_fix_indices.into_iter().rev() {
        tokens[idx].surface = new_surface;
        tokens[idx].pos = "EC".to_string();

        if remove_next {
            // 이/VCP와 고/EC 제거 (2개)
            if idx + 2 < tokens.len() {
                tokens.remove(idx + 2); // 고/EC 제거
            }
            if idx + 1 < tokens.len() {
                tokens.remove(idx + 1); // 이/VCP 제거
            }
        }
    }

    // 패턴 3: 고하/VV → 고/EC + 하/VV 분리 (세종 변환 후)
    let mut split_goha_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len() {
        if tokens[i].surface == "고하" && tokens[i].pos == "VV" {
            // 앞에 다/EF가 있는지 확인
            if i > 0 && tokens[i - 1].surface == "다" && tokens[i - 1].pos == "EF" {
                split_goha_indices.push(i);
            }
        }
    }

    for idx in split_goha_indices.into_iter().rev() {
        // 앞의 다/EF를 다고/EC로 변경
        if idx > 0 {
            tokens[idx - 1].surface = "다고".to_string();
            tokens[idx - 1].pos = "EC".to_string();
        }
        // 고하/VV를 하/VV로 변경
        tokens[idx].surface = "하".to_string();
    }

    // 139차 보정: 독립 VX → VV 변환
    // "하/VV + 니까/EC + 보/VX" 패턴에서 보/VX → 보/VV
    // 조건: VX 앞에 EC가 있고, VX가 1글자 동사인 경우
    // 145차: 단, "어/EC" 또는 "아/EC" 뒤의 보조동사는 유지
    // "해 보았다" = "하/VV 어/EC 보/VX 았/EP 다/EF"
    let independent_vx: std::collections::HashSet<&str> =
        ["보", "하", "가", "오"].into_iter().collect();

    for i in 0..tokens.len() {
        let surface = &tokens[i].surface;
        let pos = &tokens[i].pos;

        // VX가 1글자이고 앞에 EC가 있는 경우
        if pos == "VX"
            && independent_vx.contains(surface.as_str())
            && i > 0
            && tokens[i - 1].pos == "EC"
        {
            // "어/EC" 또는 "아/EC" 뒤의 보조동사는 VX 유지 (보조 용언 구문)
            let prev_surface = &tokens[i - 1].surface;
            if prev_surface != "어" && prev_surface != "아" {
                tokens[i].pos = "VV".to_string();
            }
        }
    }

    // 140차 보정: 시/EP 제거 (잘못 분리된 경우)
    // "니까", "면" 분리 시 "시/EP"가 삽입되는 경우 제거
    let ec_after_si: std::collections::HashSet<&str> =
        ["니까", "면", "니", "으니까", "으면", "으니"]
            .into_iter()
            .collect();

    let mut remove_si_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        let curr_surface = &tokens[i].surface;
        let curr_pos = &tokens[i].pos;
        let next_surface = &tokens[i + 1].surface;

        // 시/EP + EC(니까, 면 등) 패턴 → EC만 유지
        if curr_surface == "시" && curr_pos == "EP" && ec_after_si.contains(next_surface.as_str())
        {
            remove_si_indices.push(i);
        }
    }

    for idx in remove_si_indices.into_iter().rev() {
        tokens.remove(idx);
    }

    // 146차 보정: "또/MAG + 하/VV + ㄴ/ETM" → "또한/MAG" 병합
    // MeCab이 "또한"을 "또/MAG + 한/VV+ETM"으로 분리
    let mut ddohan_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(2) {
        if tokens[i].surface == "또"
            && tokens[i].pos == "MAG"
            && tokens[i + 1].surface == "하"
            && tokens[i + 1].pos == "VV"
            && tokens[i + 2].surface == "ㄴ"
            && tokens[i + 2].pos == "ETM"
        {
            ddohan_indices.push(i);
        }
    }

    for idx in ddohan_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 2].end_pos;
        tokens[idx] = SejongToken::new("또한", "MAG", start, end);
        tokens.remove(idx + 2);
        tokens.remove(idx + 1);
    }

    // 146차 보정: "한/VV + ㄴ/ETM + 국.../NNG" → "한국/NNP + ..." 복원
    // MeCab이 "한국"을 "한/VV+ETM + 국/NNG"으로 분리
    let hanguk_patterns: [(&str, &str); 4] = [
        ("국", "NNG"),
        ("국의", "NNG"),
        ("국어", "NNG"),
        ("국인", "NNG"),
    ];

    let mut hanguk_merge_indices: Vec<(usize, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(2) {
        if tokens[i].surface == "하"
            && tokens[i].pos == "VV"
            && tokens[i + 1].surface == "ㄴ"
            && tokens[i + 1].pos == "ETM"
        {
            for (suffix, pos) in &hanguk_patterns {
                if tokens[i + 2].surface == *suffix && tokens[i + 2].pos == *pos {
                    // "국의" → "한국" + "의", "국" → "한국"
                    if *suffix == "국의" {
                        hanguk_merge_indices.push((i, "국의".to_string()));
                    } else {
                        hanguk_merge_indices.push((i, (*suffix).to_string()));
                    }
                    break;
                }
            }
        }
    }

    for (idx, suffix) in hanguk_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 2].end_pos;
        if suffix == "국의" {
            // "한국" + "의/JKG"로 분리
            tokens[idx] = SejongToken::new("한국", "NNP", start, start + 2);
            tokens[idx + 1] = SejongToken::new("의", "JKG", start + 2, end);
            tokens.remove(idx + 2);
        } else {
            // "한국" 복원
            let merged_surface = format!("한{suffix}");
            tokens[idx] = SejongToken::new(&merged_surface, "NNP", start, end);
            tokens.remove(idx + 2);
            tokens.remove(idx + 1);
        }
    }

    // 146차 보정: "VV + 자/NNG" (문장 끝) → "VV + 자/EF"
    // 청유형 종결어미: "먹자", "가자" 등
    let n = tokens.len();
    if n >= 2 {
        let is_sentence_end = true; // 단독 문장으로 가정
        if is_sentence_end
            && tokens[n - 1].surface == "자"
            && tokens[n - 1].pos == "NNG"
            && tokens[n - 2].pos == "VV"
        {
            tokens[n - 1].pos = "EF".to_string();
        }
    }

    // 147차 보정: "아버/NNP + 지/VX" → "아버지/NNG"
    // "어머/... + 니/..." → "어머니/NNG"
    // MeCab이 "아버지"를 잘못 분리하는 경우
    let mut family_merge_indices: Vec<(usize, String)> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        // "아버 + 지" 패턴
        if tokens[i].surface == "아버"
            && tokens[i].pos == "NNP"
            && tokens[i + 1].surface == "지"
            && tokens[i + 1].pos == "VX"
        {
            family_merge_indices.push((i, "아버지".to_string()));
        }
    }

    for (idx, merged) in family_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new(&merged, "NNG", start, end);
        tokens.remove(idx + 1);
    }

    // 147차 보정: "어머/IC + 나/NP" → "어머나/IC"
    // MeCab이 "어머나"를 잘못 분리하는 경우
    let mut ic_merge_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i].surface == "어머"
            && tokens[i].pos == "IC"
            && tokens[i + 1].surface == "나"
            && tokens[i + 1].pos == "NP"
        {
            ic_merge_indices.push(i);
        }
    }

    for idx in ic_merge_indices.into_iter().rev() {
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        tokens[idx] = SejongToken::new("어머나", "IC", start, end);
        tokens.remove(idx + 1);
    }

    // 148차 보정: "EF + 고/NNG + 하/XSV" → "EC + 하/VV" 인용문 패턴
    // "가자고 한다" = "가/VV 자고/EC 하/VV ㄴ다/EF"
    // "예쁘다고 한다" = "예쁘/VA 다고/EC 하/VV ㄴ다/EF"
    // 패턴: (EF + 고/NNG + 하/XSV) → (EC 병합 + 하/VV)
    let mut quote_fix_indices: Vec<usize> = Vec::new();
    for i in 0..tokens.len().saturating_sub(2) {
        if tokens[i].pos == "EF"
            && tokens[i + 1].surface == "고"
            && tokens[i + 1].pos == "NNG"
            && tokens[i + 2].surface == "하"
            && tokens[i + 2].pos == "XSV"
        {
            quote_fix_indices.push(i);
        }
    }

    for idx in quote_fix_indices.into_iter().rev() {
        // EF + 고 → EC 병합
        let start = tokens[idx].start_pos;
        let end = tokens[idx + 1].end_pos;
        let merged_surface = format!("{}고", tokens[idx].surface);
        tokens[idx] = SejongToken::new(&merged_surface, "EC", start, end);
        tokens.remove(idx + 1);
        // 다음 하/XSV → 하/VV (인덱스 조정 후)
        if idx + 1 < tokens.len() && tokens[idx + 1].surface == "하" && tokens[idx + 1].pos == "XSV"
        {
            tokens[idx + 1].pos = "VV".to_string();
        }
    }

    // 149~259차: 의문대명사, 수사, 종결어미 등 후반부 보정
    apply_sentence_final_endings_corrections(tokens);
}
