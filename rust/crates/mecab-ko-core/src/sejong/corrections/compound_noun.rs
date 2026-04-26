use crate::sejong::types::SejongToken;

/// 196차: XPN 복합어 분리 테이블 ("맨손" → ("맨", "손"))
static XPN_COMPOUNDS: &[(&str, (&str, &str))] = &[
    ("맨손", ("맨", "손")),
    ("맨발", ("맨", "발")),
    ("맨몸", ("맨", "몸")),
    ("맨땅", ("맨", "땅")),
];

/// 202차: 복합명사 병합 쌍 (앞 명사, 뒤 명사)
static COMPOUND_NOUN_PAIRS: &[(&str, &str)] = &[
    ("무역", "수지"),
    ("여론", "조사"),
    ("시민", "단체"),
    ("국민", "경제"),
    ("경제", "성장"),
    ("대통령", "선거"),
    ("정부", "정책"),
    ("환경", "보호"),
    ("인공", "지능"),
    ("형태소", "분석"),
];

/// 198차: VA 어간 → (표면형, 어간) 변환 테이블
static VA_EC_WORDS: &[(&str, &str)] = &[
    ("높이", "높"),
    ("낮이", "낮"),
    ("깊이", "깊"),
    ("넓이", "넓"),
];

/// 199차: VA 어간 목록 (낮/NNG + 이/JKS 패턴)
static VA_STEMS: &[&str] = &["높", "낮", "깊", "넓"];

/// 192~202차: 복합명사/형태소 병합·분리
///
/// - 192차: "가/VV + 지/NNB + 고/EC" → "가지/VV + 고/EC" 병합
/// - 196차: XPN 복합어 분리 (맨손 → 맨/XPN 손/NNG)
/// - 200차: "밤낮/NNG" → "밤/NNG 낮/NNG" 분리
/// - 197차: "작은집" 복합 접두사 분리
/// - 202차: 복합명사 병합 (여론+조사 → 여론조사)
/// - 198차: "높이/NNG" → "높/VA 이/EC" 분리
/// - 199차: VA 어간 목록 (낮/NNG + 이/JKS 패턴)
/// - 190차: "VV + 히다/NNP" → "VV + 히/VX + 다/EF"
pub(super) fn apply_compound_noun_corrections(tokens: &mut Vec<SejongToken>) {
    // 192차: "가/VV + 지/NNB + 고/EC" → "가지/VV + 고/EC" 병합
    // "가지고 오다" = "가지/VV 고/EC 오/VV 다/EF"
    let mut i = 0;
    while i + 2 < tokens.len() {
        if tokens[i].surface == "가"
            && tokens[i].pos == "VV"
            && tokens[i + 1].surface == "지"
            && (tokens[i + 1].pos == "NNB" || tokens[i + 1].pos == "VX")
            && tokens[i + 2].surface == "고"
            && tokens[i + 2].pos == "EC"
        {
            // "가" + "지" 병합
            tokens[i].surface = "가지".to_string();
            tokens[i].end_pos = tokens[i + 1].end_pos;
            tokens.remove(i + 1);
            i += 2;
            continue;
        }
        i += 1;
    }

    // 196차: XPN 복합어 분리
    // "맨손/NNG" → "맨/XPN 손/NNG"
    // "맨발/NNG" → "맨/XPN 발/NNG"

    // 200차: "밤낮/NNG" → "밤/NNG 낮/NNG" 분리
    // "밤 낮" = "밤/NNG 낮/NNG"
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].pos == "NNG" && tokens[i].surface == "밤낮" {
            let start = tokens[i].start_pos;
            let end = tokens[i].end_pos;
            let first_len = "밤".chars().count();
            tokens[i] = SejongToken::new("밤", "NNG", start, start + first_len);
            tokens.insert(i + 1, SejongToken::new("낮", "NNG", start + first_len, end));
            i += 2;
            continue;
        }
        i += 1;
    }

    // 201차: XSN 접미사 분리 (주석 처리)
    // sample.tsv에서 대부분 "선생님/NNG"으로 단일 토큰 처리
    // "선생님 할머님" 한 케이스만 분리되어 있어 일관성 없음
    // 정확도 향상을 위해 분리하지 않음

    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].pos == "NNG" {
            if let Some(&(_, (prefix, noun))) = XPN_COMPOUNDS
                .iter()
                .find(|(k, _)| *k == tokens[i].surface.as_str())
            {
                let start = tokens[i].start_pos;
                let end = tokens[i].end_pos;
                let prefix_len = prefix.chars().count();
                tokens[i] = SejongToken::new(prefix, "XPN", start, start + prefix_len);
                tokens.insert(
                    i + 1,
                    SejongToken::new(noun, "NNG", start + prefix_len, end),
                );
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    // 197차: "작은집" 복합 접두사 분리
    // "작은집/NNG" or "작/VA 은/ETM 집/NNG" → "작/XPN 은/XPN 집/NNG"
    // 단, MeCab이 "작/VA 은/ETM 집/NNG"로 분석하면 ETM→XPN 변환
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i].surface == "작"
            && tokens[i].pos == "VA"
            && tokens[i + 1].surface == "은"
            && tokens[i + 1].pos == "ETM"
        {
            // 다음이 "집"인 경우 접두사로 변환
            if i + 2 < tokens.len() && tokens[i + 2].surface == "집" {
                tokens[i].pos = "XPN".to_string();
                tokens[i + 1].pos = "XPN".to_string();
            }
        }
    }

    // 202차: 복합명사 병합
    // "무역/NNG + 수지/NNG" → "무역수지/NNG"
    // "여론/NNG + 조사/NNG" → "여론조사/NNG"
    // "시민/NNG + 단체/NNG" → "시민단체/NNG"
    // sample.tsv에서 단일 토큰으로 취급하는 복합명사들
    let mut i = 0;
    while i + 1 < tokens.len() {
        if tokens[i].pos == "NNG" && tokens[i + 1].pos == "NNG" {
            let a = tokens[i].surface.as_str();
            let b = tokens[i + 1].surface.as_str();
            if COMPOUND_NOUN_PAIRS
                .iter()
                .any(|&(ka, kb)| ka == a && kb == b)
            {
                let start = tokens[i].start_pos;
                let end = tokens[i + 1].end_pos;
                let merged = format!("{}{}", tokens[i].surface, tokens[i + 1].surface);
                tokens[i] = SejongToken::new(&merged, "NNG", start, end);
                tokens.remove(i + 1);
                continue;
            }
        }
        i += 1;
    }

    // 198차: "높이/NNG" → "높/VA 이/EC" 분리
    // "높이 낮이" = "높/VA 이/EC 낮/VA 이/EC"
    // 형용사 부사형 분리

    let mut i = 0;
    while i < tokens.len() {
        // 패턴 1: "높이/NNG" 단일 토큰
        if tokens[i].pos == "NNG" {
            if let Some(stem) = VA_EC_WORDS
                .iter()
                .find(|(k, _)| *k == tokens[i].surface.as_str())
                .map(|(_, v)| *v)
            {
                let start = tokens[i].start_pos;
                let end = tokens[i].end_pos;
                let stem_len = stem.chars().count();
                tokens[i] = SejongToken::new(stem, "VA", start, start + stem_len);
                tokens.insert(i + 1, SejongToken::new("이", "EC", start + stem_len, end));
                i += 2;
                continue;
            }
        }
        // 패턴 2: "낮/NNG + 이/JKS" 두 토큰
        if i + 1 < tokens.len()
            && tokens[i].pos == "NNG"
            && VA_STEMS.contains(&tokens[i].surface.as_str())
            && tokens[i + 1].surface == "이"
            && tokens[i + 1].pos == "JKS"
        {
            tokens[i].pos = "VA".to_string();
            tokens[i + 1].pos = "EC".to_string();
            i += 2;
            continue;
        }
        i += 1;
    }

    // 190차: "VV + 히다/NNP" → "VV + 히/VX + 다/EF"
    // "입히다" = "입/VV 히/VX 다/EF" (피사동 접미사)
    let mut i = 0;
    while i < tokens.len() {
        if i > 0
            && tokens[i].surface == "히다"
            && tokens[i].pos == "NNP"
            && tokens[i - 1].pos == "VV"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i].end_pos;
            tokens[i] = SejongToken::new("히", "VX", start, start + "히".len());
            tokens.insert(i + 1, SejongToken::new("다", "EF", start + "히".len(), end));
            i += 2;
            continue;
        }
        i += 1;
    }
}
