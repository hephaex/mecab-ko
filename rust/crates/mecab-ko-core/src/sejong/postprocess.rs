//! 후처리 함수: VV 분리, 토큰 병합, 분해 보정

use super::hangul::remove_jongseong_rieul;
use super::types::SejongToken;

/// VV "세요" 패턴 분리
///
/// `MeCab`에서 "가세요", "오세요", "하세요" 등이 VV 단일 토큰으로 분석되면
/// VV + 세요/EF로 분리합니다. (sample.tsv 형식 준수)
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub(super) fn apply_vv_seyo_splits(tokens: Vec<SejongToken>) -> Vec<SejongToken> {
    let mut result = Vec::with_capacity(tokens.len() + 10);
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        // "가세요", "오세요" 등 VV 단일 토큰 분리
        // sample.tsv 기준: 가/VV + 세요/EF (시/EP + 어요/EF가 아닌 세요/EF 사용)
        if token.pos == "VV"
            && token.surface.ends_with("세요")
            && token.surface.chars().count() >= 2
        {
            let surface = &token.surface;
            let stem = surface.trim_end_matches("세요");
            if !stem.is_empty() {
                let stem_len = stem.chars().count();
                result.push(SejongToken::new(
                    stem,
                    "VV",
                    token.start_pos,
                    token.start_pos + stem_len,
                ));
                result.push(SejongToken::new(
                    "세요",
                    "EF",
                    token.start_pos + stem_len,
                    token.end_pos,
                ));
                i += 1;
                continue;
            }
        }

        // "할게요", "갈게요" 패턴: MeCab이 "할게/VV + 어요/EF"로 분석한 경우
        // → "하/VV + ㄹ게요/EF"로 변환
        // 할게 = 하(어간) + ㄹ게(어미의 시작), 어요 = 어요(어미의 나머지)
        if token.pos == "VV" && token.surface.ends_with("게") && token.surface.chars().count() >= 2
        {
            let surface = &token.surface;
            // "할게" → 첫 글자 "할"에서 어간 "하" 추출 필요
            // 일단 "게"를 제거하고 ㄹ 받침이 있는 글자에서 어간 추출
            let chars: Vec<char> = surface.chars().collect();
            if chars.len() >= 2 && chars[chars.len() - 1] == '게' {
                let stem_char = chars[chars.len() - 2];
                // "할"에서 "하" 추출 (ㄹ 받침 제거)
                if let Some(stem) = remove_jongseong_rieul(stem_char) {
                    // 다음 토큰이 "어요/EF"인지 확인
                    if i + 1 < tokens.len()
                        && tokens[i + 1].surface == "어요"
                        && tokens[i + 1].pos == "EF"
                    {
                        let prefix: String = chars[..chars.len() - 2].iter().collect();
                        let full_stem = format!("{prefix}{stem}");
                        result.push(SejongToken::new(
                            &full_stem,
                            "VV",
                            token.start_pos,
                            token.start_pos + full_stem.chars().count(),
                        ));
                        result.push(SejongToken::new(
                            "ㄹ게요",
                            "EF",
                            token.start_pos + full_stem.chars().count(),
                            tokens[i + 1].end_pos,
                        ));
                        i += 2; // 다음 토큰도 처리됨
                        continue;
                    }
                }
            }
        }

        // "할까요", "볼까요" 패턴: MeCab이 "할까/VV + 아요/EF"로 분석한 경우
        // → "하/VV + ㄹ까요/EF"로 변환
        if token.pos == "VV" && token.surface.ends_with("까") && token.surface.chars().count() >= 2
        {
            let surface = &token.surface;
            let chars: Vec<char> = surface.chars().collect();
            if chars.len() >= 2 && chars[chars.len() - 1] == '까' {
                let stem_char = chars[chars.len() - 2];
                // "할"에서 "하" 추출 (ㄹ 받침 제거)
                if let Some(stem) = remove_jongseong_rieul(stem_char) {
                    // 다음 토큰이 "아요/EF"인지 확인
                    if i + 1 < tokens.len()
                        && tokens[i + 1].surface == "아요"
                        && tokens[i + 1].pos == "EF"
                    {
                        let prefix: String = chars[..chars.len() - 2].iter().collect();
                        let full_stem = format!("{prefix}{stem}");
                        result.push(SejongToken::new(
                            &full_stem,
                            "VV",
                            token.start_pos,
                            token.start_pos + full_stem.chars().count(),
                        ));
                        result.push(SejongToken::new(
                            "ㄹ까요",
                            "EF",
                            token.start_pos + full_stem.chars().count(),
                            tokens[i + 1].end_pos,
                        ));
                        i += 2; // 다음 토큰도 처리됨
                        continue;
                    }
                }
            }
        }

        result.push(token.clone());
        i += 1;
    }

    result
}

/// 잘못 분해된 토큰 병합
///
/// 사전의 Viterbi 경로 선택 문제로 잘못 분해된 토큰들을 병합합니다.
/// 예: "친/VV ᆫ/ETM 구와/NNG" → "친구/NNG 와/JC"
/// 예: "날/NNG 씨/EP" → "날씨/NNG"
#[allow(clippy::useless_let_if_seq, clippy::too_many_lines)]
pub(super) fn apply_token_merges(tokens: &mut Vec<SejongToken>) {
    // 병합 규칙: (패턴, 결과)
    // 패턴: [(surface, pos), ...] - 매칭할 토큰 시퀀스
    // 결과: [(surface, pos), ...] - 병합 결과
    //
    // NOTE: 병합 규칙은 보수적으로 적용 (오탐 방지)
    // 특정 표면형과 품사의 조합만 병합

    let mut i = 0;
    while i < tokens.len() {
        let mut merged = false;

        // 우선 패턴 A: "오/VV + ㄹ/ETM + 하/VV + 아|어/EC" → "올해/NNG" (시간 명사 병합)
        // MeCab이 "올해"를 동사 활용형으로 잘못 분리하는 문제 수정
        // "아/EC" 또는 "어/EC" 모두 처리 (하+아 = 해)
        if i + 3 < tokens.len()
            && tokens[i].surface == "오"
            && tokens[i].pos == "VV"
            && tokens[i + 1].surface == "ㄹ"
            && tokens[i + 1].pos == "ETM"
            && tokens[i + 2].surface == "하"
            && tokens[i + 2].pos == "VV"
            && (tokens[i + 3].surface == "어" || tokens[i + 3].surface == "아")
            && tokens[i + 3].pos == "EC"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i + 3].end_pos;
            tokens[i] = SejongToken::new("올해", "NNG", start, end);
            tokens.remove(i + 3);
            tokens.remove(i + 2);
            tokens.remove(i + 1);
            merged = true;
        }

        // 우선 패턴 B: "사/VV + ㄴ/ETM + 책/NNG" → "산책/NNG" (명사 병합)
        // MeCab이 "산책"을 동사 활용형+명사로 잘못 분리하는 문제 수정
        if !merged
            && i + 2 < tokens.len()
            && tokens[i].surface == "사"
            && tokens[i].pos == "VV"
            && tokens[i + 1].surface == "ㄴ"
            && tokens[i + 1].pos == "ETM"
            && tokens[i + 2].surface == "책"
            && tokens[i + 2].pos == "NNG"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i + 2].end_pos;
            tokens[i] = SejongToken::new("산책", "NNG", start, end);
            tokens.remove(i + 2);
            tokens.remove(i + 1);
            merged = true;
        }

        // 우선 패턴 C: "순/NNG + 우리/NP + 말/NNG" → "순/XPN + 우리말/NNG"
        // MeCab이 "순우리말"을 잘못 분리하는 문제 수정
        if !merged
            && i + 2 < tokens.len()
            && tokens[i].surface == "순"
            && tokens[i].pos == "NNG"
            && tokens[i + 1].surface == "우리"
            && tokens[i + 1].pos == "NP"
            && tokens[i + 2].surface == "말"
            && tokens[i + 2].pos == "NNG"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i + 2].end_pos;
            tokens[i] = SejongToken::new("순", "XPN", start, start + 1);
            tokens[i + 1] = SejongToken::new("우리말", "NNG", start + 1, end);
            tokens.remove(i + 2);
            merged = true;
        }

        // 144차: "주/VX + 가가/NNG" → "주가/NNG + 가/JKS"
        // (주가가가 주/VX+가가/NNG로 분석되는 문제 수정)
        if !merged
            && i + 1 < tokens.len()
            && tokens[i].surface == "주"
            && tokens[i].pos == "VX"
            && tokens[i + 1].surface == "가가"
            && tokens[i + 1].pos == "NNG"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i + 1].end_pos;
            tokens[i] = SejongToken::new("주가", "NNG", start, start + 2);
            tokens[i + 1] = SejongToken::new("가", "JKS", start + 2, end);
            merged = true;
        }

        // 패턴 1: "친/VV + ᆫ/ETM + 구와/NNG" → "친구/NNG + 와/JC"
        // (친구와가 치/VV+ᆫ/ETM+구와/NNG로 분석되는 문제 수정)
        if i + 2 < tokens.len()
            && tokens[i].surface == "치"
            && tokens[i].pos == "VV"
            && tokens[i + 1].surface == "ᆫ"
            && tokens[i + 1].pos == "ETM"
            && tokens[i + 2].surface == "구와"
            && tokens[i + 2].pos == "NNG"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i + 2].end_pos;

            tokens[i] = SejongToken::new("친구", "NNG", start, start + 2);
            tokens[i + 1] = SejongToken::new("와", "JC", start + 2, end);
            tokens.remove(i + 2);
            merged = true;
        }

        // 패턴 2: "날/NNG + 씨/EP + 가/EF" → "날씨/NNG + 가/JKS"
        // (날씨가가 날/NNG+씨/EP+가/EF로 분석되는 문제 수정)
        if !merged
            && i + 2 < tokens.len()
            && tokens[i].surface == "날"
            && tokens[i].pos == "NNG"
            && tokens[i + 1].surface == "씨"
            && tokens[i + 1].pos == "EP"
            && tokens[i + 2].surface == "가"
            && tokens[i + 2].pos == "EF"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i + 2].end_pos;

            tokens[i] = SejongToken::new("날씨", "NNG", start, start + 2);
            tokens[i + 1] = SejongToken::new("가", "JKS", start + 2, end);
            tokens.remove(i + 2);
            merged = true;
        }

        // 패턴 3: "대한/NNG + 민국/NNG + ᆯ/ETM" → "대한민국/NNP"
        // (대한민국이 대한/NNG+민국/NNG+ᆯ/ETM로 분석되는 문제 수정)
        if !merged
            && i + 2 < tokens.len()
            && tokens[i].surface == "대한"
            && tokens[i].pos == "NNG"
            && tokens[i + 1].surface == "민국"
            && tokens[i + 1].pos == "NNG"
            && tokens[i + 2].surface == "ᆯ"
            && tokens[i + 2].pos == "ETM"
        {
            let start = tokens[i].start_pos;

            tokens[i] = SejongToken::new("대한민국", "NNP", start, start + 4);
            tokens.remove(i + 2);
            tokens.remove(i + 1);
            merged = true;
        }

        // 패턴 4: "먹/NNG + 었/EF" → "먹/VV + 었/EP"
        // (먹었이 먹/NNG+었/EF로 분석되는 문제 수정)
        if !merged
            && i + 1 < tokens.len()
            && tokens[i].surface == "먹"
            && tokens[i].pos == "NNG"
            && tokens[i + 1].surface == "었"
            && tokens[i + 1].pos == "EF"
        {
            tokens[i].pos = "VV".to_string();
            tokens[i + 1].pos = "EP".to_string();
            merged = true;
        }

        // 패턴 5: "읽/VA + 고/EF" → "읽/VV + 고/EC"
        // (읽고가 읽/VA+고/EF로 분석되는 문제 수정)
        if !merged
            && i + 1 < tokens.len()
            && tokens[i].surface == "읽"
            && tokens[i].pos == "VA"
            && tokens[i + 1].surface == "고"
            && tokens[i + 1].pos == "EF"
        {
            tokens[i].pos = "VV".to_string();
            tokens[i + 1].pos = "EC".to_string();
            merged = true;
        }

        // 패턴 6: "있/EP + 어요/EF" → "있/VX + 어요/EF"
        // (보조용언 "있다"가 EP로 분석되는 문제 수정)
        if !merged
            && i + 1 < tokens.len()
            && tokens[i].surface == "있"
            && tokens[i].pos == "EP"
            && tokens[i + 1].pos == "EF"
        {
            tokens[i].pos = "VX".to_string();
            merged = true;
        }

        // 패턴 7: "수/NNB + 도/JX" (after NNP) → "수도/NNG"
        // ("대한민국 수도"에서 "수도"가 분리되는 문제 수정)
        if !merged
            && i > 0
            && i + 1 < tokens.len()
            && tokens[i - 1].pos == "NNP"
            && tokens[i].surface == "수"
            && tokens[i].pos == "NNB"
            && tokens[i + 1].surface == "도"
            && tokens[i + 1].pos == "JX"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i + 1].end_pos;

            tokens[i] = SejongToken::new("수도", "NNG", start, end);
            tokens.remove(i + 1);
            merged = true;
        }

        // 패턴 8: "가다/NNG" → "가다/VV" (동사 기본형)
        // (동사 기본형이 NNG으로 분석되는 문제 수정)
        if !merged && tokens[i].surface == "가다" && tokens[i].pos == "NNG" {
            tokens[i].pos = "VV".to_string();
            merged = true;
        }

        // 패턴 9: "보다/JKB" → "보다/VV" (동사 기본형)
        if !merged && tokens[i].surface == "보다" && tokens[i].pos == "JKB" {
            tokens[i].pos = "VV".to_string();
            merged = true;
        }

        // 패턴 10: "오다/NNG" → "오다/VV" (동사 기본형)
        if !merged && tokens[i].surface == "오다" && tokens[i].pos == "NNG" {
            tokens[i].pos = "VV".to_string();
            merged = true;
        }

        // 패턴 11: "먹/NNG + 다/EF" → "먹다/VV" (동사 기본형 병합)
        if !merged
            && i + 1 < tokens.len()
            && tokens[i].surface == "먹"
            && tokens[i].pos == "NNG"
            && tokens[i + 1].surface == "다"
            && tokens[i + 1].pos == "EF"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i + 1].end_pos;
            tokens[i] = SejongToken::new("먹다", "VV", start, end);
            tokens.remove(i + 1);
            merged = true;
        }

        // 패턴 12: "하/IC" → "하다/VV" 앞에 오는 경우 보정
        // "하다 했다"에서 "하/IC 다하/VV"로 분석되는 문제
        if !merged
            && i + 1 < tokens.len()
            && tokens[i].surface == "하"
            && tokens[i].pos == "IC"
            && tokens[i + 1].surface.starts_with("다")
        {
            let start = tokens[i].start_pos;
            tokens[i] = SejongToken::new("하다", "VV", start, start + 2);
            // 다음 토큰의 "다" 부분 제거
            if tokens[i + 1].surface == "다하" {
                tokens[i + 1].surface = "하".to_string();
                tokens[i + 1].start_pos += 1;
            }
            merged = true;
        }

        // 패턴 13: "가/EF" 앞에 VV가 오면 EC로 보정 (가다 = 연결어미)
        if !merged
            && i > 0
            && tokens[i].surface == "가"
            && tokens[i].pos == "EF"
            && tokens[i - 1].pos == "VV"
        {
            tokens[i].pos = "EC".to_string();
            merged = true;
        }

        // 패턴 14: "고/EF" → "고/EC" (연결어미)
        if !merged
            && i > 0
            && tokens[i].surface == "고"
            && tokens[i].pos == "EF"
            && (tokens[i - 1].pos == "VV" || tokens[i - 1].pos == "VA")
        {
            tokens[i].pos = "EC".to_string();
            merged = true;
        }

        // 패턴 15: "서/EF" → "서/EC" (연결어미 - ~해서)
        if !merged
            && i > 0
            && tokens[i].surface == "서"
            && tokens[i].pos == "EF"
            && (tokens[i - 1].pos == "VV" || tokens[i - 1].pos == "VA")
        {
            tokens[i].pos = "EC".to_string();
            merged = true;
        }

        // 패턴 16: "면/EF" → "면/EC" (조건 연결어미)
        if !merged
            && i > 0
            && tokens[i].surface == "면"
            && tokens[i].pos == "EF"
            && (tokens[i - 1].pos == "VV" || tokens[i - 1].pos == "VA")
        {
            tokens[i].pos = "EC".to_string();
            merged = true;
        }

        // 패턴 17: "니/EF" → "니/EC" (이유 연결어미)
        if !merged
            && i > 0
            && tokens[i].surface == "니"
            && tokens[i].pos == "EF"
            && (tokens[i - 1].pos == "VV" || tokens[i - 1].pos == "VA")
        {
            tokens[i].pos = "EC".to_string();
            merged = true;
        }

        // 패턴 18: "게/EF" → "게/EC" (방법 연결어미)
        if !merged
            && i > 0
            && tokens[i].surface == "게"
            && tokens[i].pos == "EF"
            && (tokens[i - 1].pos == "VV" || tokens[i - 1].pos == "VA")
        {
            tokens[i].pos = "EC".to_string();
            merged = true;
        }

        // 패턴 19: "는/EF" + "데/EC" → "는데/EC" (병합)
        if !merged
            && i + 1 < tokens.len()
            && tokens[i].surface == "는"
            && tokens[i].pos == "EF"
            && tokens[i + 1].surface == "데"
        {
            let start = tokens[i].start_pos;
            let end = tokens[i + 1].end_pos;
            tokens[i] = SejongToken::new("는데", "EC", start, end);
            tokens.remove(i + 1);
            merged = true;
        }

        // 패턴 20: "그래요/IC" → "그러/VV + 어요/EF" (분리)
        // "그러다"의 활용형 보정
        if !merged && tokens[i].surface == "그래요" && tokens[i].pos == "IC" {
            let start = tokens[i].start_pos;
            let end = tokens[i].end_pos;
            tokens[i] = SejongToken::new("그러", "VV", start, start + 2);
            tokens.insert(i + 1, SejongToken::new("어요", "EF", start + 2, end));
            merged = true;
        }

        // 패턴 21: "이래요/IC" → "이러/VV + 어요/EF" (분리)
        if !merged && tokens[i].surface == "이래요" && tokens[i].pos == "IC" {
            let start = tokens[i].start_pos;
            let end = tokens[i].end_pos;
            tokens[i] = SejongToken::new("이러", "VV", start, start + 2);
            tokens.insert(i + 1, SejongToken::new("어요", "EF", start + 2, end));
            merged = true;
        }

        // 패턴 22: "저래요/IC" → "저러/VV + 어요/EF" (분리)
        if !merged && tokens[i].surface == "저래요" && tokens[i].pos == "IC" {
            let start = tokens[i].start_pos;
            let end = tokens[i].end_pos;
            tokens[i] = SejongToken::new("저러", "VV", start, start + 2);
            tokens.insert(i + 1, SejongToken::new("어요", "EF", start + 2, end));
            merged = true;
        }

        // 패턴 23: "X세/NNG + 요/EF|JX" → "X/VV + 세요/EF" (동사 + 존칭 종결어미)
        // 오세요, 가세요, 하세요, 보세요 등
        if !merged
            && i + 1 < tokens.len()
            && tokens[i].pos == "NNG"
            && tokens[i].surface.ends_with("세")
            && tokens[i + 1].surface == "요"
            && (tokens[i + 1].pos == "JX" || tokens[i + 1].pos == "EF")
        {
            let surface = &tokens[i].surface;
            // 어간 추출: "오세" → "오", "가세" → "가"
            if let Some(stem) = surface.strip_suffix("세") {
                if !stem.is_empty() {
                    let start = tokens[i].start_pos;
                    let end = tokens[i + 1].end_pos;
                    let stem_len = stem.chars().count();
                    tokens[i] = SejongToken::new(stem, "VV", start, start + stem_len);
                    tokens[i + 1] = SejongToken::new("세요", "EF", start + stem_len, end);
                    merged = true;
                }
            }
        }

        // 패턴 24: "X지/VV" → "X/VV + 지/EC" (부정 연결어미 분리)
        // 하지, 먹지, 가지 등
        // 233차 수정: "-지다" 동사는 분리하지 않음 (떨어지다, 커지다 등)
        if !merged && tokens[i].pos == "VV" && tokens[i].surface.ends_with("지") {
            let surface = &tokens[i].surface;
            // "-지다" 형태의 동사는 분리하지 않음 (233차)
            let jida_verbs = [
                "떨어지",
                "커지",
                "작아지",
                "나아지",
                "없어지",
                "생기",
                "죽이",
                "붙이",
                "늘이",
                "줄이",
                "높이",
                "낮추",
                "밝히",
                "넓히",
                "깊이",
                "걸리",
                "팔리",
                "열리",
                "닫히",
                "막히",
                "뚫리",
                "풀리",
                "묶이",
                "잡히",
                "쫓기",
                "밀리",
                "끌리",
                "불리",
                "실리",
                "읽히",
                "안기",
            ];
            let is_jida_verb = jida_verbs.contains(&surface.as_str());

            if !is_jida_verb {
                if let Some(stem) = surface.strip_suffix("지") {
                    if !stem.is_empty() {
                        let start = tokens[i].start_pos;
                        let end = tokens[i].end_pos;
                        let stem_len = stem.chars().count();
                        tokens[i] = SejongToken::new(stem, "VV", start, start + stem_len);
                        tokens.insert(i + 1, SejongToken::new("지", "EC", start + stem_len, end));
                        merged = true;
                    }
                }
            }
        }

        // 패턴 25: "X기/NNG" + "전/NNG" → "X/VV + 기/ETN" (명사형어미 분리)
        // "가기 전에", "먹기 전에", "오기 전에" 등
        if !merged
            && i + 1 < tokens.len()
            && tokens[i].pos == "NNG"
            && tokens[i].surface.ends_with("기")
            && tokens[i + 1].surface == "전"
        {
            let surface = &tokens[i].surface;
            if let Some(stem) = surface.strip_suffix("기") {
                if !stem.is_empty() {
                    let start = tokens[i].start_pos;
                    let end = tokens[i].end_pos;
                    let stem_len = stem.chars().count();
                    tokens[i] = SejongToken::new(stem, "VV", start, start + stem_len);
                    tokens.insert(i + 1, SejongToken::new("기", "ETN", start + stem_len, end));
                    merged = true;
                }
            }
        }

        // 패턴 26: "고/EC + 나/NP + ..." → "고나서/EC" (연결어미 병합)
        // "먹고나서", "하고나서" 등
        if !merged
            && i + 1 < tokens.len()
            && tokens[i].surface == "고"
            && tokens[i].pos == "EC"
            && tokens[i + 1].surface == "나"
            && tokens[i + 1].pos == "NP"
        {
            // "나서"가 따라오는지 확인
            if i + 2 < tokens.len() && tokens[i + 2].surface.starts_with("서") {
                let start = tokens[i].start_pos;
                let end = tokens[i + 1].end_pos + 1; // "나" + "서" 일부
                tokens[i] = SejongToken::new("고나서", "EC", start, end);
                tokens.remove(i + 1);
                // i+2가 "서"로 시작하면 처리
                if i + 1 < tokens.len() && tokens[i + 1].surface.starts_with("서") {
                    let remaining = tokens[i + 1].surface.strip_prefix("서").unwrap_or("");
                    if remaining.is_empty() {
                        tokens.remove(i + 1);
                    } else {
                        tokens[i + 1].surface = remaining.to_string();
                    }
                }
                merged = true;
            }
        }

        // 패턴 27: "Xㄹ/VV + 까요/EF" → "X/VV + ㄹ까요/EF" (ㄹ 이동)
        // "올까요" → "오/VV + ㄹ까요/EF", "볼까요" → "보/VV + ㄹ까요/EF" 등
        // ㄹ-final verb stems where ㄹ should be part of the ending
        // 246차 수정: "까", "게", "래" 등 ㄹ로 시작해야 하는 어미에만 적용
        // "어", "아" 같은 일반 어미에는 적용하지 않음 (ㄷ불규칙 깨짐 방지)
        if !merged && i + 1 < tokens.len() && tokens[i].pos == "VV" && tokens[i + 1].pos == "EF" {
            let surface = &tokens[i].surface;
            let next_surface = &tokens[i + 1].surface;
            // ㄹ 어미 패턴: 까, 까요, 게, 게요, 래, 래요 등
            let rieul_endings = ["까", "까요", "게", "게요", "래", "래요", "지", "지요"];
            let should_move_rieul = rieul_endings.iter().any(|e| *e == next_surface);
            // ㄹ을 떼어내기: 올 → 오
            if should_move_rieul {
                if let Some(last_char) = surface.chars().last() {
                    // 받침이 ㄹ인 경우 (종성 ㄹ = 0x11AF)
                    // 올 = 오 + ㅗ + ㄹ => 떼면 오
                    let code = last_char as u32;
                    if (0xAC00..=0xD7A3).contains(&code) {
                        let final_jamo = (code - 0xAC00) % 28;
                        if final_jamo == 8 {
                            // ㄹ 받침
                            // ㄹ을 떼면 새로운 글자
                            let new_code = code - 8;
                            if let Some(new_char) = char::from_u32(new_code) {
                                // 어미에 ㄹ을 붙임
                                let new_ending = format!("ㄹ{next_surface}");
                                let new_stem: String = surface
                                    .chars()
                                    .take(surface.chars().count() - 1)
                                    .chain(std::iter::once(new_char))
                                    .collect();

                                let start = tokens[i].start_pos;
                                let end = tokens[i + 1].end_pos;
                                let stem_len = new_stem.chars().count();
                                tokens[i] =
                                    SejongToken::new(&new_stem, "VV", start, start + stem_len);
                                tokens[i + 1] =
                                    SejongToken::new(&new_ending, "EF", start + stem_len, end);
                                // merged = true; // 마지막 패턴이므로 불필요
                            }
                        }
                    }
                }
            }
        }

        i += 1;
    }
}

/// 잘못된 분해 패턴 보정
///
/// mecab-ko-dic의 일부 항목은 복합 동사(갔다오다, 왔다갔다 등)의 활용형을
/// 독립된 과거형 종결어미로 잘못 분석함. 이 함수에서 보정.
///
/// 또한 형용사의 관형형+종결어미 패턴(좋/VA + 은/ETM + 다/EF)을
/// 올바른 종결 패턴(좋/VA + 다/EF)으로 보정.
///
/// 예: "갔다" → 갔다오/VV + ㄴ/ETM (잘못) → 갔/VV + 다/EF (올바름)
/// 예: "좋다" → 좋/VA + 은/ETM + 다/EF (잘못) → 좋/VA + 다/EF (올바름)
pub(super) fn apply_decomposition_corrections(tokens: &mut Vec<SejongToken>) {
    // "X오/VV + ㄴ/ETM" 패턴을 "X/VV + 다/EF"로 보정
    // 대상: 갔다, 왔다, 봤다, 했다 등 과거형 동사
    let verb_patterns: &[(&str, &str, &str)] = &[
        // (잘못된 어간, 올바른 어간, 올바른 어간 품사)
        ("갔다오", "갔", "VV"),
        ("왔다가", "왔", "VV"),
        ("갔다가", "갔", "VV"),
    ];

    let mut i = 0;
    while i < tokens.len() {
        let mut matched = false;

        // 패턴 1: X오/VV + ㄴ/ETM → X/VV + 다/EF
        if i + 1 < tokens.len()
            && tokens[i].pos == "VV"
            && tokens[i + 1].surface == "ㄴ"
            && tokens[i + 1].pos == "ETM"
        {
            for &(wrong_stem, correct_stem, stem_pos) in verb_patterns {
                if tokens[i].surface == wrong_stem {
                    let start = tokens[i].start_pos;
                    let end = tokens[i + 1].end_pos;

                    tokens[i] = SejongToken::new(
                        correct_stem,
                        stem_pos,
                        start,
                        start + correct_stem.chars().count(),
                    );
                    tokens[i + 1] =
                        SejongToken::new("다", "EF", start + correct_stem.chars().count(), end);

                    matched = true;
                    break;
                }
            }
        }

        // 패턴 2: X/VA + 은/ETM + 다/EF → X/VA + 다/EF
        // (형용사 관형형 + 종결어미 패턴 보정)
        if !matched
            && i + 2 < tokens.len()
            && tokens[i].pos == "VA"
            && tokens[i + 1].surface == "은"
            && tokens[i + 1].pos == "ETM"
            && tokens[i + 2].surface == "다"
            && tokens[i + 2].pos == "EF"
        {
            // VA 어간 유지, 은/ETM 제거, 다/EF를 VA 바로 뒤로 이동
            let start = tokens[i].start_pos;
            let end = tokens[i + 2].end_pos;

            // 원래 어간 유지
            tokens[i].end_pos = start + tokens[i].surface.chars().count();

            // 중간 토큰(은/ETM) 제거하고 다/EF 위치 조정
            tokens[i + 1] = SejongToken::new("다", "EF", tokens[i].end_pos, end);

            // 세 번째 토큰 제거 (나중에 처리)
            tokens.remove(i + 2);

            matched = true;
        }

        i += if matched { 2 } else { 1 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(surface: &str, pos: &str) -> SejongToken {
        let end = surface.chars().count();
        SejongToken::new(surface, pos, 0, end)
    }

    fn tok_at(surface: &str, pos: &str, start: usize, end: usize) -> SejongToken {
        SejongToken::new(surface, pos, start, end)
    }

    #[test]
    fn test_apply_vv_seyo_splits_gaseyo() {
        // "가세요/VV" → "가/VV + 세요/EF"
        let tokens = vec![tok("가세요", "VV")];
        let result = apply_vv_seyo_splits(tokens);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].surface, "가");
        assert_eq!(result[0].pos, "VV");
        assert_eq!(result[1].surface, "세요");
        assert_eq!(result[1].pos, "EF");
    }

    #[test]
    fn test_apply_vv_seyo_splits_passthrough_no_match() {
        // "가다/VV" 는 변환 없이 그대로
        let tokens = vec![tok("가다", "VV")];
        let result = apply_vv_seyo_splits(tokens);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].surface, "가다");
    }

    #[test]
    fn test_apply_token_merges_날씨_pattern() {
        // 패턴 2: "날/NNG + 씨/EP + 가/EF" → "날씨/NNG + 가/JKS"
        let mut tokens = vec![
            tok_at("날", "NNG", 0, 1),
            tok_at("씨", "EP", 1, 2),
            tok_at("가", "EF", 2, 3),
        ];
        apply_token_merges(&mut tokens);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].surface, "날씨");
        assert_eq!(tokens[0].pos, "NNG");
        assert_eq!(tokens[1].surface, "가");
        assert_eq!(tokens[1].pos, "JKS");
    }

    #[test]
    fn test_apply_token_merges_gada_nng_correction() {
        // 패턴 8: "가다/NNG" → "가다/VV"
        let mut tokens = vec![tok("가다", "NNG")];
        apply_token_merges(&mut tokens);
        assert_eq!(tokens[0].pos, "VV");
    }

    #[test]
    fn test_apply_decomposition_corrections_va_eun_pattern() {
        // 패턴 2: "좋/VA + 은/ETM + 다/EF" → "좋/VA + 다/EF"
        let mut tokens = vec![
            tok_at("좋", "VA", 0, 1),
            tok_at("은", "ETM", 1, 2),
            tok_at("다", "EF", 2, 3),
        ];
        apply_decomposition_corrections(&mut tokens);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].surface, "좋");
        assert_eq!(tokens[0].pos, "VA");
        assert_eq!(tokens[1].surface, "다");
        assert_eq!(tokens[1].pos, "EF");
    }
}
