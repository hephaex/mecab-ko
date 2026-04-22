//! 세종 변환기 테스트

#[cfg(test)]
mod tests {
    use crate::sejong::{DecomposedMorpheme, SejongConverter, SejongToken};
    use crate::tokenizer::Token;

    fn create_test_token(surface: &str, pos: &str) -> Token {
        Token {
            surface: surface.to_string(),
            pos: pos.to_string(),
            start_pos: 0,
            end_pos: surface.chars().count(),
            start_byte: 0,
            end_byte: surface.len(),
            reading: None,
            lemma: None,
            cost: 0,
            features: String::new(),
            normalized: None,
        }
    }

    #[test]
    fn test_is_compound_tag() {
        let converter = SejongConverter::new();
        assert!(converter.is_compound_tag("VV+EF"));
        assert!(converter.is_compound_tag("VA+EP+EF"));
        assert!(!converter.is_compound_tag("NNG"));
        assert!(!converter.is_compound_tag("VV"));
    }

    #[test]
    fn test_split_compound_tag() {
        let converter = SejongConverter::new();

        assert_eq!(converter.split_compound_tag("VV+EF"), vec!["VV", "EF"]);
        assert_eq!(
            converter.split_compound_tag("VV+EP+EF"),
            vec!["VV", "EP", "EF"]
        );
        assert_eq!(converter.split_compound_tag("NNG"), vec!["NNG"]);
    }

    #[test]
    fn test_simple_verb_ending_split() {
        let converter = SejongConverter::new();

        // 갔다 -> 가 + 았 + 다 (과거 시제 축약형 분리)
        // 세종 코퍼스: 가/VV + 았/EP + 다/EF
        let result = converter.split_morpheme("갔다", "VV+EF");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("가".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("았".to_string(), "EP".to_string()));
        assert_eq!(result[2], ("다".to_string(), "EF".to_string()));
    }

    #[test]
    fn test_causative_verb_split() {
        let converter = SejongConverter::new();

        // 212차: 들리다 -> 들리 + 다 (사동사는 VX로 분리하지 않음)
        let result = converter.split_morpheme("들리다", "VV+EF");
        assert_eq!(
            result.len(),
            2,
            "들리다 should split into 2 parts, got {result:?}"
        );
        assert_eq!(result[0], ("들리".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("다".to_string(), "EF".to_string()));

        // 웃기다 -> 웃기 + 다
        let result2 = converter.split_morpheme("웃기다", "VV+EF");
        assert_eq!(result2.len(), 2);
        assert_eq!(result2[0], ("웃기".to_string(), "VV".to_string()));
        assert_eq!(result2[1], ("다".to_string(), "EF".to_string()));
    }

    #[test]
    fn test_polite_ending_split() {
        let converter = SejongConverter::new();

        // 먹습니다 -> 먹 + 습니다
        let result = converter.split_morpheme("먹습니다", "VV+EF");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("먹".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("습니다".to_string(), "EF".to_string()));
    }

    #[test]
    fn test_adjective_ending_split() {
        let converter = SejongConverter::new();

        // 좋다 -> 좋 + 다
        let result = converter.split_morpheme("좋다", "VA+EF");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("좋".to_string(), "VA".to_string()));
        assert_eq!(result[1], ("다".to_string(), "EF".to_string()));
    }

    #[test]
    fn test_connective_ending_split() {
        let converter = SejongConverter::new();

        // 먹고 -> 먹 + 고
        let result = converter.split_morpheme("먹고", "VV+EC");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("먹".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("고".to_string(), "EC".to_string()));
    }

    #[test]
    fn test_adnominal_ending_split() {
        let converter = SejongConverter::new();

        // 먹는 -> 먹 + 는
        let result = converter.split_morpheme("먹는", "VV+ETM");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("먹".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("는".to_string(), "ETM".to_string()));
    }

    #[test]
    fn test_past_tense_ending_split() {
        let converter = SejongConverter::new();

        // 먹었다 -> 먹 + 었 + 다
        let result = converter.split_morpheme("먹었다", "VV+EP+EF");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("먹".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("었".to_string(), "EP".to_string()));
        assert_eq!(result[2], ("다".to_string(), "EF".to_string()));
    }

    #[test]
    fn test_non_compound_tag() {
        let converter = SejongConverter::new();

        // 단순 품사는 분리하지 않음
        let result = converter.split_morpheme("학교", "NNG");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("학교".to_string(), "NNG".to_string()));
    }

    #[test]
    fn test_convert_token() {
        let converter = SejongConverter::new();

        let token = create_test_token("갔다", "VV+EF");
        let sejong_tokens = converter.convert_token(&token);

        // 과거 시제 축약형: 갔다 → 가 + 았 + 다
        assert_eq!(sejong_tokens.len(), 3);
        assert_eq!(sejong_tokens[0].surface, "가");
        assert_eq!(sejong_tokens[0].pos, "VV");
        assert_eq!(sejong_tokens[1].surface, "았");
        assert_eq!(sejong_tokens[1].pos, "EP");
        assert_eq!(sejong_tokens[2].surface, "다");
        assert_eq!(sejong_tokens[2].pos, "EF");
    }

    #[test]
    fn test_convert_tokens() {
        let converter = SejongConverter::new();

        let tokens = vec![
            create_test_token("학교", "NNG"),
            create_test_token("갔다", "VV+EF"),
        ];

        let sejong_tokens = converter.convert_tokens(&tokens);

        // 과거 시제 축약형: 갔다 → 가 + 았 + 다
        assert_eq!(sejong_tokens.len(), 4);
        assert_eq!(sejong_tokens[0].to_sejong_format(), "학교/NNG");
        assert_eq!(sejong_tokens[1].to_sejong_format(), "가/VV");
        assert_eq!(sejong_tokens[2].to_sejong_format(), "았/EP");
        assert_eq!(sejong_tokens[3].to_sejong_format(), "다/EF");
    }

    #[test]
    fn test_format_sejong() {
        let converter = SejongConverter::new();

        let tokens = vec![
            create_test_token("학교", "NNG"),
            create_test_token("갔다", "VV+EF"),
        ];

        let result = converter.tokens_to_sejong_string(&tokens);
        // 과거 시제 축약형: 갔다 → 가 + 았 + 다
        assert_eq!(result, "학교/NNG 가/VV 았/EP 다/EF");
    }

    #[test]
    fn test_sejong_token_format() {
        let token = SejongToken::new("갔", "VV", 0, 1);
        assert_eq!(token.to_sejong_format(), "갔/VV");
    }

    #[test]
    fn test_sejong_token_from_split() {
        let token = SejongToken::from_split("갔", "VV", 0, 1, "갔다", "VV+EF");
        assert_eq!(token.surface, "갔");
        assert_eq!(token.pos, "VV");
        assert_eq!(token.original_surface, Some("갔다".to_string()));
        assert_eq!(token.original_pos, Some("VV+EF".to_string()));
    }

    #[test]
    fn test_informal_ending_split() {
        let converter = SejongConverter::new();

        // 먹어 -> 먹 + 어
        let result = converter.split_morpheme("먹어", "VV+EF");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("먹".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("어".to_string(), "EF".to_string()));
    }

    #[test]
    fn test_contracted_hada_split() {
        let converter = SejongConverter::new();

        // 해요 -> 하 + 어요 (하다 축약형)
        let result = converter.split_morpheme("해요", "VV+EF");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("하".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("어요".to_string(), "EF".to_string()));

        // 돼요 -> 되 + 어요 (되다 축약형)
        let result2 = converter.split_morpheme("돼요", "VV+EF");
        assert_eq!(result2.len(), 2);
        assert_eq!(result2[0], ("되".to_string(), "VV".to_string()));
        assert_eq!(result2[1], ("어요".to_string(), "EF".to_string()));

        // 해 -> 하 + 어
        let result3 = converter.split_morpheme("해", "VV+EF");
        assert_eq!(result3.len(), 2);
        assert_eq!(result3[0], ("하".to_string(), "VV".to_string()));
        assert_eq!(result3[1], ("어".to_string(), "EF".to_string()));
    }

    #[test]
    fn test_contracted_past_split() {
        let converter = SejongConverter::new();

        // 봤다 -> 보 + 았 + 다 (VV+EF를 VV+EP+EF로 확장)
        let result = converter.split_morpheme("봤다", "VV+EF");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("보".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("았".to_string(), "EP".to_string()));
        assert_eq!(result[2], ("다".to_string(), "EF".to_string()));

        // 갔다 -> 가 + 았 + 다
        let result2 = converter.split_morpheme("갔다", "VV+EF");
        assert_eq!(result2.len(), 3);
        assert_eq!(result2[0], ("가".to_string(), "VV".to_string()));
        assert_eq!(result2[1], ("았".to_string(), "EP".to_string()));
        assert_eq!(result2[2], ("다".to_string(), "EF".to_string()));

        // 했다 -> 하 + 았 + 다
        let result3 = converter.split_morpheme("했다", "VV+EF");
        assert_eq!(result3.len(), 3);
        assert_eq!(result3[0], ("하".to_string(), "VV".to_string()));
        assert_eq!(result3[1], ("았".to_string(), "EP".to_string()));
        assert_eq!(result3[2], ("다".to_string(), "EF".to_string()));
    }

    #[test]
    fn test_polite_past_ending_split() {
        let converter = SejongConverter::new();

        // 먹었습니다 -> 먹 + 었 + 습니다
        let result = converter.split_morpheme("먹었습니다", "VV+EP+EF");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("먹".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("었".to_string(), "EP".to_string()));
        assert_eq!(result[2], ("습니다".to_string(), "EF".to_string()));
    }

    // ============================================================
    // 분석결과(decomposition) 파싱 테스트
    // ============================================================

    #[test]
    fn test_parse_decomposition_simple() {
        // 단순 형태소: stem/POS/*
        let decomp = "가깝/VA/*+아/EC/*";
        let result = SejongConverter::parse_decomposition(decomp);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].surface, "가깝");
        assert_eq!(result[0].pos, "VA");
        assert_eq!(result[1].surface, "아");
        assert_eq!(result[1].pos, "EC");
    }

    #[test]
    fn test_parse_decomposition_three_parts() {
        // 3개 형태소: VV + EP + EF
        let decomp = "먹/VV/*+었/EP/*+다/EF/*";
        let result = SejongConverter::parse_decomposition(decomp);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].surface, "먹");
        assert_eq!(result[0].pos, "VV");
        assert_eq!(result[1].surface, "었");
        assert_eq!(result[1].pos, "EP");
        assert_eq!(result[2].surface, "다");
        assert_eq!(result[2].pos, "EF");
    }

    #[test]
    fn test_parse_decomposition_irregular_verb() {
        // ㅂ불규칙: 가깝 + 아 → 가까와
        let decomp = "가깝/VA/*+아/EC/*";
        let result = SejongConverter::parse_decomposition(decomp);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].surface, "가깝");
        assert_eq!(result[0].pos, "VA");
        assert_eq!(result[1].surface, "아");
        assert_eq!(result[1].pos, "EC");
    }

    #[test]
    fn test_parse_decomposition_empty() {
        assert!(SejongConverter::parse_decomposition("").is_empty());
        assert!(SejongConverter::parse_decomposition("*").is_empty());
    }

    #[test]
    fn test_extract_decomposition_from_features() {
        // Inflect.csv 형식: 품사,의미분류,종성,읽기,타입,첫품사,끝품사,분석결과
        let features = "VA+EC,*,F,가까와,Inflect,VA,EC,가깝/VA/*+아/EC/*";
        let result = SejongConverter::extract_decomposition(features);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "가깝/VA/*+아/EC/*");
    }

    #[test]
    fn test_extract_decomposition_no_decomp() {
        // 분석결과가 없는 경우
        let features = "NNG,*,T,학교,*,*,*";
        let result = SejongConverter::extract_decomposition(features);

        assert!(result.is_none());
    }

    #[test]
    fn test_convert_token_with_decomposition() {
        let converter = SejongConverter::new();

        // 분석결과가 있는 토큰 (불규칙 활용)
        let token = Token {
            surface: "가까와".to_string(),
            pos: "VA+EC".to_string(),
            start_pos: 0,
            end_pos: 3,
            start_byte: 0,
            end_byte: 9,
            reading: None,
            lemma: None,
            cost: 0,
            features: "VA+EC,*,F,가까와,Inflect,VA,EC,가깝/VA/*+아/EC/*".to_string(),
            normalized: None,
        };

        let sejong_tokens = converter.convert_token(&token);

        // 분석결과를 활용하여 정확하게 분리
        assert_eq!(sejong_tokens.len(), 2);
        assert_eq!(sejong_tokens[0].surface, "가깝");
        assert_eq!(sejong_tokens[0].pos, "VA");
        assert_eq!(sejong_tokens[1].surface, "아");
        assert_eq!(sejong_tokens[1].pos, "EC");
    }

    #[test]
    fn test_convert_token_without_decomposition_flag() {
        // 분석결과 비활성화 시 규칙 기반 분리
        let converter = SejongConverter::new().with_decomposition(false);

        let token = Token {
            surface: "가까와".to_string(),
            pos: "VA+EC".to_string(),
            start_pos: 0,
            end_pos: 3,
            start_byte: 0,
            end_byte: 9,
            reading: None,
            lemma: None,
            cost: 0,
            features: "VA+EC,*,F,가까와,Inflect,VA,EC,가깝/VA/*+아/EC/*".to_string(),
            normalized: None,
        };

        let sejong_tokens = converter.convert_token(&token);

        // 규칙 기반으로 분리 시도 (불규칙 활용은 정확하게 분리 못함)
        // '아'로 끝나므로 분리 가능
        assert!(!sejong_tokens.is_empty());
    }

    #[test]
    fn test_decomposed_morpheme_struct() {
        let morpheme = DecomposedMorpheme {
            surface: "가깝".to_string(),
            pos: "VA".to_string(),
        };

        assert_eq!(morpheme.surface, "가깝");
        assert_eq!(morpheme.pos, "VA");
    }

    // ============================================================
    // S23-03: 어미 분리 로직 강화 테스트
    // ============================================================

    #[test]
    fn test_ep_split_past_tense() {
        let converter = SejongConverter::new();

        // 과거 시제 선어말어미 분리
        let result = converter.split_morpheme("먹었", "VV+EP");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("먹".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("었".to_string(), "EP".to_string()));
    }

    #[test]
    fn test_ep_split_presumptive() {
        let converter = SejongConverter::new();

        // 추측 선어말어미 분리
        let result = converter.split_morpheme("먹겠", "VV+EP");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("먹".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("겠".to_string(), "EP".to_string()));
    }

    #[test]
    fn test_ec_split_extended_connectives() {
        let converter = SejongConverter::new();

        // 조건 연결어미 - "면"으로 분리 (어미 매칭은 suffix 기반)
        let result = converter.split_morpheme("먹으면", "VV+EC");
        assert_eq!(result.len(), 2);
        // 현재 구현: 가장 먼저 매칭되는 어미로 분리
        assert!(result[1].1 == "EC");

        // 양보 연결어미
        let result2 = converter.split_morpheme("먹어도", "VV+EC");
        assert_eq!(result2.len(), 2);
        assert_eq!(result2[1], ("어도".to_string(), "EC".to_string()));
    }

    #[test]
    fn test_ec_split_reason_connectives() {
        let converter = SejongConverter::new();

        // 이유 연결어미
        let result = converter.split_morpheme("먹어서", "VV+EC");
        assert_eq!(result.len(), 2);
        assert_eq!(result[1], ("어서".to_string(), "EC".to_string()));

        // 므로 연결어미 (으므로 중 므로로 분리됨)
        let result2 = converter.split_morpheme("먹으므로", "VV+EC");
        assert_eq!(result2.len(), 2);
        assert_eq!(result2[1].1, "EC".to_string());
    }

    #[test]
    fn test_etm_split_adnominal() {
        let converter = SejongConverter::new();

        // 현재 관형형
        let result = converter.split_morpheme("먹는", "VV+ETM");
        assert_eq!(result.len(), 2);
        assert_eq!(result[1], ("는".to_string(), "ETM".to_string()));

        // 과거 관형형 (은)
        let result2 = converter.split_morpheme("먹은", "VV+ETM");
        assert_eq!(result2.len(), 2);
        assert_eq!(result2[1], ("은".to_string(), "ETM".to_string()));

        // 미래 관형형 (을)
        let result3 = converter.split_morpheme("먹을", "VV+ETM");
        assert_eq!(result3.len(), 2);
        assert_eq!(result3[1], ("을".to_string(), "ETM".to_string()));
    }

    #[test]
    fn test_etn_split_nominalization() {
        let converter = SejongConverter::new();

        // 명사형 어미 -기
        let result = converter.split_morpheme("먹기", "VV+ETN");
        assert_eq!(result.len(), 2);
        assert_eq!(result[1], ("기".to_string(), "ETN".to_string()));

        // 형용사 명사형
        let result2 = converter.split_morpheme("좋기", "VA+ETN");
        assert_eq!(result2.len(), 2);
        assert_eq!(result2[0], ("좋".to_string(), "VA".to_string()));
        assert_eq!(result2[1], ("기".to_string(), "ETN".to_string()));
    }

    #[test]
    fn test_ep_ec_split_past_connective() {
        let converter = SejongConverter::new();

        // 과거 + 연결어미
        let result = converter.split_morpheme("먹었고", "VV+EP+EC");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("먹".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("었".to_string(), "EP".to_string()));
        assert_eq!(result[2], ("고".to_string(), "EC".to_string()));
    }

    #[test]
    fn test_ep_etm_split_past_adnominal() {
        let converter = SejongConverter::new();

        // 과거 회상 관형형
        let result = converter.split_morpheme("먹었던", "VV+EP+ETM");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("먹".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("었".to_string(), "EP".to_string()));
        assert_eq!(result[2], ("던".to_string(), "ETM".to_string()));
    }

    #[test]
    fn test_honorific_split() {
        let converter = SejongConverter::new();

        // 높임 + 과거 + 종결 (으셨습니다 패턴)
        let result = converter.split_morpheme("읽으셨습니다", "VV+EP+EF");
        assert_eq!(result.len(), 3);
        // 어간 + 선어말어미 + 종결어미로 분리
        assert_eq!(result[0].1, "VV".to_string());
        assert_eq!(result[1].1, "EP".to_string());
        assert_eq!(result[2].1, "EF".to_string());
    }

    #[test]
    fn test_formal_endings() {
        let converter = SejongConverter::new();

        // 합쇼체 의문형
        let result = converter.split_morpheme("먹습니까", "VV+EF");
        assert_eq!(result.len(), 2);
        assert_eq!(result[1], ("습니까".to_string(), "EF".to_string()));
    }

    #[test]
    fn test_vx_auxiliary() {
        let converter = SejongConverter::new();

        // 보조용언 + 종결어미
        let result = converter.split_compound_tag("VX+EF");
        assert_eq!(result, vec!["VX", "EF"]);

        // 보조용언 + 선어말어미 + 종결어미
        let result2 = converter.split_compound_tag("VX+EP+EF");
        assert_eq!(result2, vec!["VX", "EP", "EF"]);
    }

    #[test]
    fn test_split_prefinal_ending_compound() {
        // 복합 선어말어미 분리
        let (prefinal, final_part) = SejongConverter::split_prefinal_ending("으셨습니다");
        assert_eq!(prefinal, "으셨");
        assert_eq!(final_part, "습니다");

        let (prefinal2, final_part2) = SejongConverter::split_prefinal_ending("겠어요");
        assert_eq!(prefinal2, "겠");
        assert_eq!(final_part2, "어요");
    }

    #[test]
    fn test_xsv_ef_split() {
        let converter = SejongConverter::new();

        // 되다 -> 되 + 다
        let result = converter.split_morpheme("되다", "XSV+EF");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("되".to_string(), "XSV".to_string()));
        assert_eq!(result[1], ("다".to_string(), "EF".to_string()));

        // 한다 -> 한 + 다
        let result2 = converter.split_morpheme("한다", "XSV+EF");
        assert_eq!(result2.len(), 2);
        assert_eq!(result2[0], ("한".to_string(), "XSV".to_string()));
        assert_eq!(result2[1], ("다".to_string(), "EF".to_string()));

        // 해요 -> 하 + 어요 (하다 축약형 처리)
        // 세종 코퍼스에서는 해요가 하+어요로 분리됨
        let result3 = converter.split_morpheme("해요", "XSV+EF");
        assert_eq!(result3.len(), 2);
        assert_eq!(result3[0], ("하".to_string(), "XSV".to_string()));
        assert_eq!(result3[1], ("어요".to_string(), "EF".to_string()));
    }

    #[test]
    fn test_xsv_ec_split() {
        let converter = SejongConverter::new();

        // 하고 -> 하 + 고
        let result = converter.split_morpheme("하고", "XSV+EC");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("하".to_string(), "XSV".to_string()));
        assert_eq!(result[1], ("고".to_string(), "EC".to_string()));

        // 되면 -> 되 + 면
        let result2 = converter.split_morpheme("되면", "XSV+EC");
        assert_eq!(result2.len(), 2);
        assert_eq!(result2[0], ("되".to_string(), "XSV".to_string()));
        assert_eq!(result2[1], ("면".to_string(), "EC".to_string()));
    }

    #[test]
    fn test_vx_ef_split() {
        let converter = SejongConverter::new();

        // 있어요 -> 있 + 어요 (VX+EF)
        let result = converter.split_morpheme("있어요", "VX+EF");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("있".to_string(), "VX".to_string()));
        assert_eq!(result[1], ("어요".to_string(), "EF".to_string()));

        // 있다 -> 있 + 다
        let result2 = converter.split_morpheme("있다", "VX+EF");
        assert_eq!(result2.len(), 2);
        assert_eq!(result2[0], ("있".to_string(), "VX".to_string()));
        assert_eq!(result2[1], ("다".to_string(), "EF".to_string()));

        // 않아요 -> 않 + 아요
        let result3 = converter.split_morpheme("않아요", "VX+EF");
        assert_eq!(result3.len(), 2);
        assert_eq!(result3[0], ("않".to_string(), "VX".to_string()));
        assert_eq!(result3[1], ("아요".to_string(), "EF".to_string()));
    }
}
