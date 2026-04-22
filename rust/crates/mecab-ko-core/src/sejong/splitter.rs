//! 형태소 분리 함수

use std::collections::HashMap;

use super::hangul::{remove_jongseong_bieup, remove_jongseong_nieun, remove_jongseong_rieul};
use super::types::EndingRule;

/// 복합 품사 태그인지 확인 (`+`로 구분된 태그)
#[must_use]
pub fn is_compound_tag(pos: &str) -> bool {
    pos.contains('+')
}

/// 복합 품사 태그를 분리된 태그 목록으로 변환
#[must_use]
pub fn split_compound_tag<S: std::hash::BuildHasher>(
    tag_map: &HashMap<String, Vec<String>, S>,
    pos: &str,
) -> Vec<String> {
    tag_map.get(pos).cloned().unwrap_or_else(|| {
        if pos.contains('+') {
            // 매핑 테이블에 없으면 단순 분리
            pos.split('+').map(String::from).collect()
        } else {
            vec![pos.to_string()]
        }
    })
}

/// 표면형에서 어미를 분리
///
/// # Arguments
/// * `surface` - 표면형 (예: "갔다")
/// * `pos` - 품사 태그 (예: "VV+EF")
///
/// # Returns
/// 분리된 (표면형, 품사) 쌍의 벡터
#[must_use]
pub fn split_morpheme<S: std::hash::BuildHasher>(
    surface: &str,
    pos: &str,
    tag_map: &HashMap<String, Vec<String>, S>,
    ending_rules: &[EndingRule],
) -> Vec<(String, String)> {
    // 복합 태그가 아니면 그대로 반환
    if !is_compound_tag(pos) {
        return vec![(surface.to_string(), pos.to_string())];
    }

    // EP+EP (존칭+과거 복합) 특별 처리
    // "셨" → "시/EP + 었/EP", "셨어" → "시/EP + 었/EP + 어/EF" 아님
    // EP+EP는 단일 형태소 "셨"을 두 개의 EP로 분리
    if pos == "EP+EP" {
        // "셨" = "시(존칭)" + "었(과거)"
        if surface == "셨" {
            return vec![
                ("시".to_string(), "EP".to_string()),
                ("었".to_string(), "EP".to_string()),
            ];
        }
        // "았" 계열도 처리 (필요시)
        // 일반적으로 EP+EP는 위 케이스만 해당
    }

    // 중복 태그 처리: "JKB+JKB" 같은 경우 첫 번째 태그만 사용
    // 이는 사전 버그로 발생하는 패턴 (EP+EP 제외)
    let tags = split_compound_tag(tag_map, pos);
    if tags.len() >= 2 && tags[0] == tags[1] && pos != "EP+EP" {
        return vec![(surface.to_string(), tags[0].clone())];
    }

    // EP+EF (긍정지정사+어미) 특별 처리
    // "입니다" → "이/VCP + 습니다/EF", "입니까" → "이/VCP + 습니까/EF"
    if pos == "EP+EF" {
        if surface == "입니다" {
            return vec![
                ("이".to_string(), "VCP".to_string()),
                ("습니다".to_string(), "EF".to_string()),
            ];
        } else if surface == "입니까" {
            return vec![
                ("이".to_string(), "VCP".to_string()),
                ("습니까".to_string(), "EF".to_string()),
            ];
        }
    }

    // VCP+EF (긍정지정사+종결어미) 특별 처리
    // "입니다" → "이/VCP + 습니다/EF"
    if pos == "VCP+EF" {
        if surface == "입니다" {
            return vec![
                ("이".to_string(), "VCP".to_string()),
                ("습니다".to_string(), "EF".to_string()),
            ];
        } else if surface == "입니까" {
            return vec![
                ("이".to_string(), "VCP".to_string()),
                ("습니까".to_string(), "EF".to_string()),
            ];
        }
    }

    // 143차: VV+EC "는다" 특수 처리 (평서형 종결어미)
    // MeCab이 "는다"를 VV+EC로 태그하지만 실제로는 종결어미
    // "는다/VV+EC" → "는다/EF" (단일 토큰 유지)
    if pos == "VV+EC" && surface == "는다" {
        return vec![("는다".to_string(), "EF".to_string())];
    }

    // 150차: EP+EC "며" 특수 처리
    // MeCab이 "며"를 EP+EC (시/EP + 며/EC)로 분석하지만 실제로는 연결어미만 사용
    // "며/EP+EC" → "며/EC" (시/EP 제거)
    if pos == "EP+EC" && surface == "며" {
        return vec![("며".to_string(), "EC".to_string())];
    }

    // 169차: VV+EC+EP+EF "-야겠다" 패턴 분리
    // MeCab이 분해 정보 없이 복합 품사로 출력하는 경우
    // 예: "자야겠다/VV+EC+EP+EF" → "자/VV 아야겠/EP 다/EF"
    // "자야겠다" = 4글자 ("자" + "야겠다"), "가야겠다" = 4글자
    if pos == "VV+EC+EP+EF" && surface.ends_with("야겠다") && surface.chars().count() >= 4 {
        let stem_len = surface.chars().count() - 3; // "야겠다"에서 "야겠" 제외 (어간 + 다)
        let stem: String = surface.chars().take(stem_len).collect();
        return vec![
            (stem, "VV".to_string()),
            ("아야겠".to_string(), "EP".to_string()),
            ("다".to_string(), "EF".to_string()),
        ];
    }

    // 182차: VV+VX+EF "어지다/아지다" 패턴 특별 처리
    // "만들어지다" → "만들/VV + 어지/VX + 다/EF"
    // "커지다" → "커/VV + 지/VX + 다/EF" (주의: 이 케이스는 다름)
    if pos == "VV+VX+EF" {
        // "어지다" 패턴 (3글자 이상)
        if surface.ends_with("어지다") && surface.chars().count() >= 4 {
            let stem: String = surface.chars().take(surface.chars().count() - 3).collect();
            return vec![
                (stem, "VV".to_string()),
                ("어지".to_string(), "VX".to_string()),
                ("다".to_string(), "EF".to_string()),
            ];
        }
        // "아지다" 패턴 (3글자 이상)
        if surface.ends_with("아지다") && surface.chars().count() >= 4 {
            let stem: String = surface.chars().take(surface.chars().count() - 3).collect();
            return vec![
                (stem, "VV".to_string()),
                ("아지".to_string(), "VX".to_string()),
                ("다".to_string(), "EF".to_string()),
            ];
        }
    }

    // VV+EF 특별 패턴 처리
    if pos == "VV+EF" {
        // 183차: 사동사/피동사는 단일 동사로 처리 (VX로 분리하지 않음)
        // "웃기다" = "웃기/VV + 다/EF" (not "웃/VV + 기/VX + 다/EF")
        // "울리다" = "울리/VV + 다/EF", "높이다" = "높이/VV + 다/EF"
        // 212차: "들리다" 추가 (sample.tsv 기준)
        let causative_verbs = [
            "웃기다",
            "울리다",
            "높이다",
            "낮추다",
            "늘이다",
            "줄이다",
            "살리다",
            "죽이다",
            "알리다",
            "먹이다",
            "재우다",
            "깨우다",
            "들리다",
            "놀리다",
        ];
        if causative_verbs.contains(&surface) {
            let stem_len = surface.chars().count() - 1;
            let stem: String = surface.chars().take(stem_len).collect();
            return vec![
                (stem, "VV".to_string()),
                ("다".to_string(), "EF".to_string()),
            ];
        }

        // "ㅂ니다" 패턴: "합니다" → "하/VV + ㅂ니다/EF"
        if surface.ends_with("니다") && surface.chars().count() >= 3 {
            let chars: Vec<char> = surface.chars().collect();
            let first_char = chars[0];
            if let Some(stem) = remove_jongseong_bieup(first_char) {
                if chars.len() == 3 {
                    return vec![
                        (stem.to_string(), "VV".to_string()),
                        ("ㅂ니다".to_string(), "EF".to_string()),
                    ];
                }
            }
        }
        // "ㅂ니까" 패턴: "합니까" → "하/VV + ㅂ니까/EF"
        if surface.ends_with("니까") && surface.chars().count() >= 3 {
            let chars: Vec<char> = surface.chars().collect();
            let first_char = chars[0];
            if let Some(stem) = remove_jongseong_bieup(first_char) {
                if chars.len() == 3 {
                    return vec![
                        (stem.to_string(), "VV".to_string()),
                        ("ㅂ니까".to_string(), "EF".to_string()),
                    ];
                }
            }
        }

        // "ㄹ게요" 패턴: "할게요" → "하/VV + ㄹ게요/EF", "갈게요" → "가/VV + ㄹ게요/EF"
        if surface.ends_with("게요") && surface.chars().count() >= 3 {
            let chars: Vec<char> = surface.chars().collect();
            let stem_char = chars[chars.len() - 3]; // "할게요"에서 "할"
            if let Some(stem) = remove_jongseong_rieul(stem_char) {
                let prefix: String = chars[..chars.len() - 3].iter().collect();
                let full_stem = format!("{prefix}{stem}");
                return vec![
                    (full_stem, "VV".to_string()),
                    ("ㄹ게요".to_string(), "EF".to_string()),
                ];
            }
        }

        // "ㄹ까요" 패턴: "할까요" → "하/VV + ㄹ까요/EF", "볼까요" → "보/VV + ㄹ까요/EF"
        if surface.ends_with("까요") && surface.chars().count() >= 3 {
            let chars: Vec<char> = surface.chars().collect();
            let stem_char = chars[chars.len() - 3]; // "할까요"에서 "할"
            if let Some(stem) = remove_jongseong_rieul(stem_char) {
                let prefix: String = chars[..chars.len() - 3].iter().collect();
                let full_stem = format!("{prefix}{stem}");
                return vec![
                    (full_stem, "VV".to_string()),
                    ("ㄹ까요".to_string(), "EF".to_string()),
                ];
            }
        }

        // 221차 보정: "ㄹ까" 패턴: "갈까" → "가/VV + ㄹ까/EF", "할까" → "하/VV + ㄹ까/EF"
        // "갈까 하다"의 경우 109차 보정에서 EC로 변환됨
        if surface.ends_with("까") && !surface.ends_with("까요") && surface.chars().count() >= 2
        {
            let chars: Vec<char> = surface.chars().collect();
            let stem_char = chars[chars.len() - 2]; // "갈까"에서 "갈"
            if let Some(stem) = remove_jongseong_rieul(stem_char) {
                let prefix: String = chars[..chars.len() - 2].iter().collect();
                let full_stem = format!("{prefix}{stem}");
                return vec![
                    (full_stem, "VV".to_string()),
                    ("ㄹ까".to_string(), "EF".to_string()),
                ];
            }
        }

        // "ㄹ래요" 패턴: "할래요" → "하/VV + ㄹ래요/EF"
        if surface.ends_with("래요") && surface.chars().count() >= 3 {
            let chars: Vec<char> = surface.chars().collect();
            let stem_char = chars[chars.len() - 3]; // "할래요"에서 "할"
            if let Some(stem) = remove_jongseong_rieul(stem_char) {
                let prefix: String = chars[..chars.len() - 3].iter().collect();
                let full_stem = format!("{prefix}{stem}");
                return vec![
                    (full_stem, "VV".to_string()),
                    ("ㄹ래요".to_string(), "EF".to_string()),
                ];
            }
        }

        // "해요" → "하/VV + 어요/EF" (하+여요 = 해요, 세종 코퍼스 표준)
        if surface == "해요" {
            return vec![
                ("하".to_string(), "VV".to_string()),
                ("어요".to_string(), "EF".to_string()),
            ];
        }

        // "봐요" → "보/VV + 아요/EF" (보+아요 = 봐요)
        if surface == "봐요" {
            return vec![
                ("보".to_string(), "VV".to_string()),
                ("아요".to_string(), "EF".to_string()),
            ];
        }

        // 151차: "와요" → "오/VV + 아요/EF" (오+아요 = 와요)
        if surface == "와요" {
            return vec![
                ("오".to_string(), "VV".to_string()),
                ("아요".to_string(), "EF".to_string()),
            ];
        }

        // "해" → "하/VV + 어/EF" (하+여 = 해, 세종 코퍼스 표준)
        if surface == "해" {
            return vec![
                ("하".to_string(), "VV".to_string()),
                ("어".to_string(), "EF".to_string()),
            ];
        }

        // "돼요" → "되/VV + 어요/EF" (되+어요 = 돼요)
        if surface == "돼요" {
            return vec![
                ("되".to_string(), "VV".to_string()),
                ("어요".to_string(), "EF".to_string()),
            ];
        }

        // "돼" → "되/VV + 어/EF" (되+어 = 돼)
        if surface == "돼" {
            return vec![
                ("되".to_string(), "VV".to_string()),
                ("어".to_string(), "EF".to_string()),
            ];
        }
    }

    // 축약형 처리를 먼저 시도 (해요→하+어요, 했어요→하+았+어요 등)
    // 이 처리가 일반 규칙보다 우선해야 함 (해요가 해+요로 분리되는 것 방지)

    // 3개 태그 축약형 처리 (했어요, 갔어요 등)
    if let Some(result) = try_split_contracted(surface, pos, tag_map) {
        return result;
    }

    // 2개 태그 축약형 처리 (해요, 돼요 등)
    if let Some(result) = try_split_contracted_two_tags(surface, pos, tag_map) {
        return result;
    }

    // 적용 가능한 규칙 찾기
    for rule in ending_rules {
        if rule.pos_pattern == pos {
            // 어미 패턴 매칭 시도
            for ending in &rule.endings {
                if surface.ends_with(ending.as_str()) {
                    let stem_len = surface.chars().count() - ending.chars().count();
                    if stem_len > 0 {
                        let stem: String = surface.chars().take(stem_len).collect();

                        // 분리된 형태소 생성
                        return create_split_morphemes(&stem, ending, &rule.target_tags);
                    }
                }
            }
        }
    }

    // 종성 자음 어미 처리 (VV+ETM, VA+ETM에서 ㄹ, ㄴ 받침)
    // 예: "올/VV+ETM" → "오/VV + ㄹ/ETM", "간/VV+ETM" → "가/VV + ㄴ/ETM"
    if (pos == "VV+ETM" || pos == "VA+ETM") && surface.chars().count() == 1 {
        let ch = surface.chars().next().unwrap();
        // ㄹ 받침 처리
        if let Some(stem_char) = remove_jongseong_rieul(ch) {
            return vec![
                (
                    stem_char.to_string(),
                    if pos.starts_with("VV") {
                        "VV".to_string()
                    } else {
                        "VA".to_string()
                    },
                ),
                ("ㄹ".to_string(), "ETM".to_string()),
            ];
        }
        // ㄴ 받침 처리
        if let Some(stem_char) = remove_jongseong_nieun(ch) {
            return vec![
                (
                    stem_char.to_string(),
                    if pos.starts_with("VV") {
                        "VV".to_string()
                    } else {
                        "VA".to_string()
                    },
                ),
                ("ㄴ".to_string(), "ETM".to_string()),
            ];
        }
    }

    // 규칙이 적용되지 않으면 태그만 분리
    let tags = split_compound_tag(tag_map, pos);
    if tags.len() > 1 {
        // 어미를 분리할 수 없으면 표면형 전체에 첫 번째 태그 부여
        return vec![(surface.to_string(), tags[0].clone())];
    }

    vec![(surface.to_string(), pos.to_string())]
}

/// 축약형 동사 분리 시도
/// 예: 했어요 → 하 + 았 + 어요, 갔어요 → 가 + 았 + 어요
/// 예: 만났어요 → 만나 + 았 + 어요
fn try_split_contracted(
    surface: &str,
    pos: &str,
    tag_map: &HashMap<String, Vec<String>>,
) -> Option<Vec<(String, String)>> {
    let tags = split_compound_tag(tag_map, pos);
    if tags.len() != 3 {
        return None;
    }

    // 축약형 패턴 정의: (축약된 음절, 원래 어간 끝, 선어말어미)
    // 하다류: 하+았 → 했, 하+았+어 → 했어
    // 가다류: 가+았 → 갔, 오+았 → 왔
    // 보다류: 보+았 → 봤
    // 나다류: 나+았 → 났 (만나다 등)
    let contracted_stems = [
        ("했", "하", "았"),
        ("갔", "가", "았"),
        ("왔", "오", "았"),
        ("봤", "보", "았"),
        ("샀", "사", "았"),
        ("잤", "자", "았"),
        ("됐", "되", "었"),
        ("났", "나", "았"), // 만났다, 났다
        ("랐", "라", "았"), // 불렀다 (부르다+았)
        ("섰", "서", "었"), // 섰다 (서다+었)
    ];

    let chars: Vec<char> = surface.chars().collect();
    if chars.is_empty() {
        return None;
    }

    // 종결어미/연결어미 패턴
    let ef_patterns = ["어요", "어", "다", "지", "니", "나", "습니다", "습니까"];
    let ec_patterns = ["다고", "라고", "냐고", "자고"]; // 간접인용 EC

    // 1. 첫 글자가 축약형 어간인 경우 (했어요, 갔다, 왔다고 등)
    let first_char = chars[0].to_string();
    for (contracted, stem, prefinal) in &contracted_stems {
        if first_char == *contracted {
            let ending: String = chars[1..].iter().collect();
            if !ending.is_empty() {
                // EF 패턴 확인
                for ef in &ef_patterns {
                    if ending == *ef || ending.ends_with(ef) {
                        return Some(vec![
                            ((*stem).to_string(), tags[0].clone()),
                            ((*prefinal).to_string(), tags[1].clone()),
                            (ending, tags[2].clone()),
                        ]);
                    }
                }
                // EC 패턴 확인 (왔다고, 갔다고 등)
                for ec in &ec_patterns {
                    if ending == *ec || ending.ends_with(ec) {
                        return Some(vec![
                            ((*stem).to_string(), tags[0].clone()),
                            ((*prefinal).to_string(), tags[1].clone()),
                            (ending, tags[2].clone()),
                        ]);
                    }
                }
            }
        }
    }

    // 2. 중간에 축약형이 있는 경우 (만났어요 → 만나+았+어요)
    // 패턴: prefix + contracted + suffix
    for i in 1..chars.len() {
        let mid_char = chars[i].to_string();
        for (contracted, stem, prefinal) in &contracted_stems {
            if mid_char == *contracted {
                // prefix + 원래어간끝 = 동사어간
                let prefix: String = chars[..i].iter().collect();
                let full_stem = format!("{prefix}{stem}");

                // suffix = 어미
                let suffix: String = chars[i + 1..].iter().collect();
                if !suffix.is_empty() {
                    // EF 패턴 확인
                    for ef in &ef_patterns {
                        if suffix == *ef || suffix.ends_with(ef) {
                            return Some(vec![
                                (full_stem, tags[0].clone()),
                                ((*prefinal).to_string(), tags[1].clone()),
                                (suffix, tags[2].clone()),
                            ]);
                        }
                    }
                    // EC 패턴 확인
                    for ec in &ec_patterns {
                        if suffix == *ec || suffix.ends_with(ec) {
                            return Some(vec![
                                (full_stem, tags[0].clone()),
                                ((*prefinal).to_string(), tags[1].clone()),
                                (suffix, tags[2].clone()),
                            ]);
                        }
                    }
                }
            }
        }
    }

    None
}

/// 2개 태그 축약형 동사 분리 시도
/// 예: 해요 → 하 + 어요, 돼요 → 되 + 어요
/// VV+EF, VA+EF 에서 '하다/되다' 축약형 처리
fn try_split_contracted_two_tags(
    surface: &str,
    pos: &str,
    tag_map: &HashMap<String, Vec<String>>,
) -> Option<Vec<(String, String)>> {
    let tags = split_compound_tag(tag_map, pos);
    if tags.len() != 2 {
        return None;
    }

    // 축약형 패턴: (축약된 1음절, 원래 어간, 연결되는 어미 접두사)
    // 해요 = 하+어요, 해 = 하+어, 했다 = 하+았+다 (이건 3태그라 위에서 처리)
    let contracted_patterns = [
        ("해", "하", "어"), // 하+어 → 해
        ("돼", "되", "어"), // 되+어 → 돼
        ("봬", "뵈", "어"), // 뵈+어 → 봬
    ];

    let chars: Vec<char> = surface.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let first_char = chars[0].to_string();
    let rest: String = chars[1..].iter().collect();

    for (contracted, stem, vowel) in &contracted_patterns {
        if first_char == *contracted && !rest.is_empty() {
            // 해요 → 하 + 어요
            // 해 → 하 + 어 (rest가 비어있으면 안됨)
            let ending = format!("{vowel}{rest}");
            return Some(vec![
                ((*stem).to_string(), tags[0].clone()),
                (ending, tags[1].clone()),
            ]);
        }
    }

    // 특수 케이스: 단독 축약형 (해, 돼 만 있는 경우)
    // 이 경우 rest가 비어있으므로 위에서 처리 안됨
    for (contracted, stem, vowel) in &contracted_patterns {
        if surface == *contracted {
            return Some(vec![
                ((*stem).to_string(), tags[0].clone()),
                ((*vowel).to_string(), tags[1].clone()),
            ]);
        }
    }

    // 과거 시제 축약형 처리 (봤다, 갔다, 왔다 등)
    // VV+EF로 분석되지만 실제로는 VV+EP+EF여야 함
    // 봤다 → 보/VV + 았/EP + 다/EF
    let past_contracted_patterns = [
        ("봤", "보", "았"), // 보+았 → 봤
        ("갔", "가", "았"), // 가+았 → 갔
        ("왔", "오", "았"), // 오+았 → 왔
        ("샀", "사", "았"), // 사+았 → 샀
        ("잤", "자", "았"), // 자+았 → 잤
        ("됐", "되", "었"), // 되+었 → 됐
        ("했", "하", "았"), // 하+았 → 했
    ];

    for (contracted, stem, prefinal) in &past_contracted_patterns {
        if first_char == *contracted {
            // 봤다 → 보 + 았 + 다
            // tags[0]=VV, 중간에 EP를 삽입, tags[1]=EF
            return Some(vec![
                ((*stem).to_string(), tags[0].clone()),
                ((*prefinal).to_string(), "EP".to_string()),
                (rest, tags[1].clone()),
            ]);
        }
    }

    None
}

/// 분리된 형태소 생성 (어간 + 어미들)
fn create_split_morphemes(stem: &str, ending: &str, tags: &[String]) -> Vec<(String, String)> {
    let mut result = Vec::new();

    if tags.len() == 2 {
        // 어간 + 어미 (예: VV + EF)
        result.push((stem.to_string(), tags[0].clone()));
        result.push((ending.to_string(), tags[1].clone()));
    } else if tags.len() == 3 {
        // VV + VX + EF (피동/사동) 특수 처리
        // 예: "이다" → "이/VX + 다/EF"
        if tags[1] == "VX" && tags[2] == "EF" {
            let (vx_part, ef_part) = split_causative_ending(ending);
            result.push((stem.to_string(), tags[0].clone()));
            result.push((vx_part, tags[1].clone()));
            result.push((ef_part, tags[2].clone()));
        } else {
            // 어간 + 선어말어미 + 종결어미 (예: VV + EP + EF)
            // 어미 부분에서 선어말어미와 종결어미 분리 시도
            let (prefinal, final_ending) = split_prefinal_ending(ending);
            result.push((stem.to_string(), tags[0].clone()));
            result.push((prefinal, tags[1].clone()));
            result.push((final_ending, tags[2].clone()));
        }
    } else if tags.len() == 4 {
        // VV + EP + EP + EF (존칭+과거 복합형)
        // 예: "셨습니다" → "시/EP + 었/EP + 습니다/EF"
        let (ep1, ep2, ef) = split_honorific_past_ending(ending);
        result.push((stem.to_string(), tags[0].clone()));
        result.push((ep1, tags[1].clone()));
        result.push((ep2, tags[2].clone()));
        result.push((ef, tags[3].clone()));
    } else {
        // 기타 경우
        result.push((stem.to_string(), tags[0].clone()));
        if tags.len() > 1 {
            result.push((ending.to_string(), tags[tags.len() - 1].clone()));
        }
    }

    result
}

/// 피동/사동 접미사와 종결어미 분리
/// 예: "이다" → ("이", "다"), "히다" → ("히", "다")
fn split_causative_ending(ending: &str) -> (String, String) {
    let causative_patterns = ["이", "히", "리", "기"];

    for pattern in &causative_patterns {
        if ending.starts_with(pattern) {
            let vx_part = (*pattern).to_string();
            let ef_part: String = ending.chars().skip(pattern.chars().count()).collect();
            if !ef_part.is_empty() {
                return (vx_part, ef_part);
            }
        }
    }

    // 분리 불가능하면 첫 글자를 VX로, 나머지를 EF로
    let chars: Vec<char> = ending.chars().collect();
    if chars.len() >= 2 {
        (chars[0].to_string(), chars[1..].iter().collect())
    } else {
        (ending.to_string(), String::new())
    }
}

/// 선어말어미와 종결어미 분리
pub(crate) fn split_prefinal_ending(ending: &str) -> (String, String) {
    // 복합 선어말어미 패턴 (긴 것부터 먼저 매칭)
    // 시제 + 높임: 으셨, 셨, 으시었, 시었
    let compound_prefinal_patterns = [
        "으셨",
        "셨",
        "으시었",
        "시었", // 높임 + 과거
        "으시겠",
        "시겠", // 높임 + 추측
    ];

    for pattern in &compound_prefinal_patterns {
        if ending.starts_with(pattern) {
            let prefinal = (*pattern).to_string();
            let final_part: String = ending.chars().skip(pattern.chars().count()).collect();
            if !final_part.is_empty() {
                return (prefinal, final_part);
            }
        }
    }

    // 단순 선어말어미 패턴: 었, 았, 였, 겠, 시, 으시, 더
    let prefinal_patterns = ["었", "았", "였", "겠", "으시", "시", "더"];

    for pattern in &prefinal_patterns {
        if ending.starts_with(pattern) {
            let prefinal = (*pattern).to_string();
            let final_part: String = ending.chars().skip(pattern.chars().count()).collect();
            if !final_part.is_empty() {
                return (prefinal, final_part);
            }
        }
    }

    // 분리 불가능하면 전체를 EP로
    (ending.to_string(), String::new())
}

/// 존칭+과거 복합 어미 분리
/// 예: "셨습니다" → ("시", "었", "습니다")
fn split_honorific_past_ending(ending: &str) -> (String, String, String) {
    // "셨습니다" → "시" + "었" + "습니다"
    // "셨어요" → "시" + "었" + "어요"
    let patterns = [
        ("셨습니다", "시", "었", "습니다"),
        ("셨습니까", "시", "었", "습니까"),
        ("셨어요", "시", "었", "어요"),
        ("셨어", "시", "었", "어"),
        ("셨다", "시", "었", "다"),
    ];

    for (pattern, ep1, ep2, ef) in &patterns {
        if ending == *pattern {
            return ((*ep1).to_string(), (*ep2).to_string(), (*ef).to_string());
        }
    }

    // 기본 분리: 첫 글자 + 둘째 글자 + 나머지
    let chars: Vec<char> = ending.chars().collect();
    if chars.len() >= 3 {
        (
            chars[0].to_string(),
            chars[1].to_string(),
            chars[2..].iter().collect(),
        )
    } else {
        (ending.to_string(), String::new(), String::new())
    }
}
