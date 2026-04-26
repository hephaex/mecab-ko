use super::apply_context_corrections;
use super::compound_and_irregular::apply_compound_and_irregular_corrections;
use super::conjugation::apply_conjugation_corrections;
use super::sentence_final::apply_sentence_final_corrections;
use super::suffix_and_dependency::apply_suffix_and_dependency_corrections;
use super::verb_and_morpheme::apply_verb_and_morpheme_corrections;
use crate::sejong::types::SejongToken;

fn tok(surface: &str, pos: &str) -> SejongToken {
        let end = surface.chars().count();
        SejongToken::new(surface, pos, 0, end)
    }

    fn tok_at(surface: &str, pos: &str, start: usize, end: usize) -> SejongToken {
        SejongToken::new(surface, pos, start, end)
    }

    // ── 185차 보정: 첫 번째 토큰 하/XSV → VV ────────────────────────────
    #[test]
    fn test_correction_185_ha_xsv_to_vv_at_start() {
        let mut tokens = vec![tok("하", "XSV"), tok("니까", "EC")];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[0].pos, "VV", "하/XSV at position 0 should become VV");
    }

    #[test]
    fn test_correction_185_ha_xsv_not_changed_if_not_first() {
        // 첫 번째 토큰이 아니면 변환 없음
        let mut tokens = vec![tok("먹", "VV"), tok("하", "XSV")];
        apply_context_corrections(&mut tokens);
        assert_eq!(
            tokens[1].pos, "XSV",
            "하/XSV not at position 0 must stay XSV"
        );
    }

    // ── 188차 보정: 그래/VV → 그러/VV 표면형 정규화 ────────────────────
    #[test]
    fn test_correction_188_geurae_vv_normalized_to_geuro() {
        let mut tokens = vec![tok("그래", "VV")];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[0].surface, "그러");
        assert_eq!(tokens[0].pos, "VV");
    }

    // ── 193차 보정: ETN 표면형 초성 ㅁ → 호환 ㅁ ────────────────────────
    #[test]
    fn test_correction_193_etn_jamo_normalization() {
        // 초성 ᄆ(U+1106)을 호환 ㅁ(U+3141)으로 정규화
        let mut tokens = vec![tok("\u{1106}", "ETN")];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[0].surface, "ㅁ");
    }

    // ── 194차 보정: 따라/NNB + 서/VV + 어/EC → 따라서/MAG ───────────────
    #[test]
    fn test_correction_194_tara_merge_to_tararso() {
        let mut tokens = vec![
            tok_at("따라", "NNB", 0, 2),
            tok_at("서", "VV", 2, 3),
            tok_at("어", "EC", 3, 4),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].surface, "따라서");
        assert_eq!(tokens[0].pos, "MAG");
    }

    // ── 196차 보정: XPN 복합어 분리 맨손/NNG → 맨/XPN 손/NNG ────────────
    #[test]
    fn test_correction_196_xpn_compound_split() {
        let mut tokens = vec![tok_at("맨손", "NNG", 0, 2)];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].surface, "맨");
        assert_eq!(tokens[0].pos, "XPN");
        assert_eq!(tokens[1].surface, "손");
        assert_eq!(tokens[1].pos, "NNG");
    }

    // ── 255차 보정: 어/EF + 요/JX → 어요/EF 병합 ───────────────────────
    #[test]
    fn test_correction_255_eo_yo_merge() {
        let mut tokens = vec![tok_at("어", "EF", 0, 1), tok_at("요", "JX", 1, 2)];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].surface, "어요");
        assert_eq!(tokens[0].pos, "EF");
    }

    // ── 보호 테스트: 빈 입력 무변환 ────────────────────────────────────
    #[test]
    fn test_protection_empty_input_unchanged() {
        let mut tokens: Vec<SejongToken> = vec![];
        apply_context_corrections(&mut tokens);
        assert!(tokens.is_empty(), "empty token list must remain empty");
    }

    // ── 보호 테스트: 200차 밤낮/NNG → 밤/NNG 낮/NNG 분리 ─────────────
    #[test]
    fn test_protection_200_bamnak_split() {
        let mut tokens = vec![tok_at("밤낮", "NNG", 0, 2)];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 2, "밤낮 should be split into two tokens");
        assert_eq!(tokens[0].surface, "밤");
        assert_eq!(tokens[0].pos, "NNG");
        assert_eq!(tokens[1].surface, "낮");
        assert_eq!(tokens[1].pos, "NNG");
    }

    // ── 보호 테스트: 202차 복합명사 병합 여론+조사 → 여론조사 ───────────
    #[test]
    fn test_protection_202_compound_noun_merge() {
        let mut tokens = vec![
            tok_at("여론", "NNG", 0, 2),
            tok_at("조사", "NNG", 2, 4),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1, "여론 + 조사 should merge into one token");
        assert_eq!(tokens[0].surface, "여론조사");
        assert_eq!(tokens[0].pos, "NNG");
        assert_eq!(tokens[0].start_pos, 0);
        assert_eq!(tokens[0].end_pos, 4);
    }

    // ── 보호 테스트: 207차 부사로 잘못 분석된 진짜/MAG → NNG ────────────
    #[test]
    fn test_protection_207_jinja_mag_to_nng() {
        let mut tokens = vec![tok("진짜", "MAG")];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[0].pos, "NNG", "진짜/MAG must become NNG");
        assert_eq!(tokens[0].surface, "진짜");
    }

    // ── 보호 테스트: 248차 외래어 NNP → NNG 변환 ─────────────────────
    #[test]
    fn test_protection_248_foreign_word_nnp_to_nng() {
        let mut tokens = vec![tok("알고리즘", "NNP")];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[0].pos, "NNG", "알고리즘/NNP must become NNG");
    }

    // ── 보호 테스트: 251차 그/NP + 동안/NNG → 그동안/NNG 병합 ───────────
    #[test]
    fn test_protection_251_geudongan_merge() {
        let mut tokens = vec![
            tok_at("그", "NP", 0, 1),
            tok_at("동안", "NNG", 1, 3),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1, "그 + 동안 should merge into 그동안");
        assert_eq!(tokens[0].surface, "그동안");
        assert_eq!(tokens[0].pos, "NNG");
        assert_eq!(tokens[0].start_pos, 0);
        assert_eq!(tokens[0].end_pos, 3);
    }

    // ── 보호 테스트: 253차 의성어 야옹/NNG → IC 변환 ──────────────────
    #[test]
    fn test_protection_253_onomatopoeia_yaong_to_ic() {
        let mut tokens = vec![tok("야옹", "NNG")];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[0].pos, "IC", "야옹/NNG must become IC");
        assert_eq!(tokens[0].surface, "야옹");
    }

    // ── 보호 테스트: 254차 ㅓ요/EF → 어요/EF 표면형 정규화 ─────────────
    #[test]
    fn test_protection_254_jamo_eo_yo_normalization() {
        let mut tokens = vec![tok("ㅓ요", "EF")];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[0].surface, "어요", "ㅓ요/EF surface must normalize to 어요");
        assert_eq!(tokens[0].pos, "EF");
    }

    // ── 보호 테스트: 256차 졸리/VV → VA 변환 ─────────────────────────
    #[test]
    fn test_protection_256_jollri_vv_to_va() {
        let mut tokens = vec![tok("졸리", "VV")];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[0].pos, "VA", "졸리/VV must become VA");
        assert_eq!(tokens[0].surface, "졸리");
    }

    // ── 보호 테스트: 187차 서울특별시/NNP → 서울/NNP 특별시/NNG 분리 ──
    #[test]
    fn test_protection_187_seoul_teukbyeolsi_split() {
        let mut tokens = vec![tok_at("서울특별시", "NNP", 0, 5)];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 2, "서울특별시 must be split into two tokens");
        assert_eq!(tokens[0].surface, "서울");
        assert_eq!(tokens[0].pos, "NNP");
        assert_eq!(tokens[1].surface, "특별시");
        assert_eq!(tokens[1].pos, "NNG");
    }

    // ── 보호 테스트: 247차 하/XSV + 여/XSN → 어/EC 변환 ─────────────────
    // 247차는 표면형이 "하"이고 pos가 "XSV"인 경우에만 적용됨.
    // 문장 중간에 "여/XSN"을 배치하여 문장 끝 EC→EF 보정(8차/161차)이
    // 간섭하지 않도록 뒤에 추가 토큰을 붙임.
    #[test]
    fn test_protection_247_ha_yeo_xsn_to_ec() {
        let mut tokens = vec![
            tok("공부", "NNG"),
            tok("하", "XSV"),
            tok("여", "XSN"),
            tok("주", "VX"),    // 뒤에 토큰을 추가해 "여"가 문장 끝이 아니도록 함
        ];
        apply_context_corrections(&mut tokens);
        // 247차 보정: 여/XSN 표면형 → 어, pos → EC
        assert_eq!(tokens[2].surface, "어", "여/XSN surface must change to 어");
        assert_eq!(tokens[2].pos, "EC", "여/XSN pos must change to EC");
    }

    // ── 보호 테스트: 228차 하/XSV + ㄹ/ETM + 머/NP + 님/XSN → 할머님/NNG 병합 ──
    #[test]
    fn test_protection_228_halmeonym_merge() {
        let mut tokens = vec![
            tok_at("하", "XSV", 0, 1),
            tok_at("ㄹ", "ETM", 1, 2),
            tok_at("머", "NP", 2, 3),
            tok_at("님", "XSN", 3, 4),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1, "하+ㄹ+머+님 must merge into 할머님");
        assert_eq!(tokens[0].surface, "할머님");
        assert_eq!(tokens[0].pos, "NNG");
        assert_eq!(tokens[0].start_pos, 0);
        assert_eq!(tokens[0].end_pos, 4);
    }

    // ── 보호 테스트: 230차 시/NNG + 가/VV + ㄴ/ETM → 시간/NNG 병합 ─────────
    #[test]
    fn test_protection_230_sigan_merge() {
        let mut tokens = vec![
            tok_at("시", "NNG", 0, 1),
            tok_at("가", "VV", 1, 2),
            tok_at("ㄴ", "ETM", 2, 3),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1, "시+가+ㄴ must merge into 시간");
        assert_eq!(tokens[0].surface, "시간");
        assert_eq!(tokens[0].pos, "NNG");
    }

    // ── 보호 테스트: 231차 주/VX + 말/NNG → 주말/NNG 병합 ──────────────────
    #[test]
    fn test_protection_231_jumal_merge() {
        let mut tokens = vec![
            tok_at("주", "VX", 0, 1),
            tok_at("말", "NNG", 1, 2),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1, "주+말 must merge into 주말");
        assert_eq!(tokens[0].surface, "주말");
        assert_eq!(tokens[0].pos, "NNG");
    }

    // ── 보호 테스트: 232차 가/VV + ㄹ/ETM + 등/NNG → 갈등/NNG 병합 ──────────
    #[test]
    fn test_protection_232_galdeung_merge() {
        let mut tokens = vec![
            tok_at("가", "VV", 0, 1),
            tok_at("ㄹ", "ETM", 1, 2),
            tok_at("등", "NNG", 2, 3),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1, "가+ㄹ+등 must merge into 갈등");
        assert_eq!(tokens[0].surface, "갈등");
        assert_eq!(tokens[0].pos, "NNG");
    }

    // ── 보호 테스트: 234차 SL 뒤 가/VV + 어/EC → 가/JKS 병합 ─────────────
    #[test]
    fn test_protection_234_sl_ga_vv_to_jks() {
        let mut tokens = vec![
            tok("MBTI", "SL"),
            tok("가", "VV"),
            tok("어", "EC"),
            tok("뭐", "NP"),
        ];
        apply_context_corrections(&mut tokens);
        // 가/VV가 가/JKS로 바뀌고 어/EC는 제거됨
        assert_eq!(tokens[1].surface, "가");
        assert_eq!(tokens[1].pos, "JKS", "가/VV after SL must become JKS");
        assert_eq!(tokens.len(), 3, "어/EC must be removed");
    }

    // ── 보호 테스트: 236차 지/VX + ㄴ/ETM + 행/NNG → 진행/NNG 병합 ──────────
    #[test]
    fn test_protection_236_jinheng_merge() {
        let mut tokens = vec![
            tok_at("지", "VX", 0, 1),
            tok_at("ㄴ", "ETM", 1, 2),
            tok_at("행", "NNG", 2, 3),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1, "지+ㄴ+행 must merge into 진행");
        assert_eq!(tokens[0].surface, "진행");
        assert_eq!(tokens[0].pos, "NNG");
    }

    // ── 보호 테스트: 238차 하/VV + 아/EC → 하/VV + 어/EC 표면형 통일 ────────
    #[test]
    fn test_protection_238_ha_a_to_ha_eo() {
        let mut tokens = vec![
            tok("사랑하", "VV"),
            tok("아", "EC"),
            tok("주", "VX"),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[1].surface, "어", "아 after 하-ending VV must change to 어");
        assert_eq!(tokens[1].pos, "EC");
    }

    // ── 보호 테스트: 239차 주/VX + 워/NNG → 줍/VV + 어/EF (ㅂ불규칙) ─────
    #[test]
    fn test_protection_239_jup_irregular_weo_nng() {
        let mut tokens = vec![
            tok_at("주", "VX", 0, 1),
            tok_at("워", "NNG", 1, 2),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[0].surface, "줍", "주/VX before 워/NNG must become 줍");
        assert_eq!(tokens[0].pos, "VV");
        assert_eq!(tokens[1].surface, "어");
    }

    // ── 보호 테스트: 241차 무거/NNG + 우면/NNG → 무겁/VA + 으면/EC ─────────
    #[test]
    fn test_protection_241_mugeop_irregular() {
        let mut tokens = vec![
            tok_at("무거", "NNG", 0, 2),
            tok_at("우면", "NNG", 2, 4),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[0].surface, "무겁", "무거/NNG must become 무겁/VA");
        assert_eq!(tokens[0].pos, "VA");
        assert_eq!(tokens[1].surface, "으면");
        assert_eq!(tokens[1].pos, "EC");
    }

    // ── 보호 테스트: 242차 이르면/MAJ → 이르/VV + 면/EC 분리 ───────────────
    #[test]
    fn test_protection_242_ireumyeon_maj_to_vv_ec() {
        let mut tokens = vec![
            tok("이르", "VV"),
            tok("어", "EF"),
            tok("이르면", "MAJ"),
        ];
        apply_context_corrections(&mut tokens);
        // EF 뒤의 이르면/MAJ은 동사 활용으로 분리됨
        assert_eq!(tokens[2].surface, "이르", "이르면/MAJ must split: stem=이르");
        assert_eq!(tokens[2].pos, "VV");
        assert_eq!(tokens[3].surface, "면");
        assert_eq!(tokens[3].pos, "EC");
    }

    // ── 보호 테스트: 244차 VX + 안/MAG + 으며/EC → 안 제거 ─────────────────
    // 54차 보정이 있/VX를 있/VV로 바꾸므로, 있/VX가 보조동사로 유지되려면
    // 앞에 고/EC가 있어야 함.  따라서 [가/VV, 고/EC, 있/VX, 안/MAG, 으며/EC]
    // → 안/MAG 제거 후 4 토큰.
    #[test]
    fn test_protection_244_an_mag_removal() {
        let mut tokens = vec![
            tok("가", "VV"),
            tok("고", "EC"),
            tok("있", "VX"),
            tok("안", "MAG"),
            tok("으며", "EC"),
        ];
        apply_context_corrections(&mut tokens);
        // 안/MAG이 제거되어 4 토큰만 남아야 함
        assert_eq!(tokens.len(), 4, "안/MAG between VX and 으며/EC must be removed");
        assert_eq!(tokens[2].surface, "있");
        assert_eq!(tokens[3].surface, "으며");
    }

    // ── 보호 테스트: 167차 NNG + 적/XSN → 성공적/NNG 병합 ──────────────────
    #[test]
    fn test_protection_167_jeok_merge() {
        let mut tokens = vec![
            tok_at("성공", "NNG", 0, 2),
            tok_at("적", "XSN", 2, 3),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1, "성공 + 적 must merge into 성공적");
        assert_eq!(tokens[0].surface, "성공적");
        assert_eq!(tokens[0].pos, "NNG");
    }

    // ── 보호 테스트: 168차 NNG 뒤의 의/JKB → 의/JKG 변환 ──────────────────
    #[test]
    fn test_protection_168_ui_jkb_to_jkg() {
        let mut tokens = vec![
            tok("나라", "NNG"),
            tok("의", "JKB"),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[1].surface, "의");
        assert_eq!(tokens[1].pos, "JKG", "의/JKB after NNG must become JKG");
    }

    // ── 보호 테스트: 86차 ㄴ/ETM + 다/EF → ㄴ다/EF 병합 ───────────────────
    #[test]
    fn test_protection_86_nda_etm_ef_merge() {
        let mut tokens = vec![
            tok_at("가", "VV", 0, 1),
            tok_at("ㄴ", "ETM", 1, 2),
            tok_at("다", "EF", 2, 3),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 2, "ㄴ + 다 must merge into ㄴ다");
        assert_eq!(tokens[1].surface, "ㄴ다");
        assert_eq!(tokens[1].pos, "EF");
    }

    // ── 보호 테스트: 87차 EC 뒤의 보조동사 VV → VX 변환 ────────────────────
    #[test]
    fn test_protection_87_aux_vv_to_vx_after_ec() {
        let mut tokens = vec![
            tok("먹", "VV"),
            tok("어", "EC"),
            tok("버리", "VV"),
            tok("었", "EP"),
            tok("다", "EF"),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[2].surface, "버리");
        assert_eq!(tokens[2].pos, "VX", "버리/VV after EC must become VX");
    }

    // ── 보호 테스트: 88차 NNG + 되/VV → 되/XSV 변환 ────────────────────────
    #[test]
    fn test_protection_88_doe_vv_to_xsv_after_nng() {
        let mut tokens = vec![
            tok("공개", "NNG"),
            tok("되", "VV"),
            tok("었", "EP"),
            tok("다", "EF"),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[1].surface, "되");
        assert_eq!(tokens[1].pos, "XSV", "되/VV after NNG must become XSV");
    }

    // ── 보호 테스트: 265차 VV 뒤 문장 끝 네/IC → 네/EF 변환 ─────────────────
    #[test]
    fn test_protection_265_ne_ic_to_ef_after_vv() {
        let mut tokens = vec![
            tok("킹받", "VV"),
            tok("네", "IC"),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[1].surface, "네");
        assert_eq!(tokens[1].pos, "EF", "네/IC at sentence end after VV must become EF");
    }

    // ── 보호 테스트: 259차 채/VV + 아/EF → 채/NNB (의존명사) ──────────────
    #[test]
    fn test_protection_259_chae_vv_to_nnb() {
        let mut tokens = vec![
            tok_at("채", "VV", 0, 1),
            tok_at("아", "EF", 1, 2),
        ];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1, "채/VV + 아/EF must reduce to single 채/NNB");
        assert_eq!(tokens[0].surface, "채");
        assert_eq!(tokens[0].pos, "NNB");
    }

    // ── 보호 테스트: 209차 빈 POS + ASCII → SL ───────────────────────────────
    #[test]
    fn test_protection_209_empty_pos_ascii_to_sl() {
        let mut tokens = vec![tok("HELLO", "")];
        apply_context_corrections(&mut tokens);
        assert_eq!(
            tokens[0].pos, "SL",
            "empty POS for ASCII surface 'HELLO' must become SL (Pass 209)"
        );
    }

    // ── 보호 테스트: 209차 빈 POS + 한글 → NNG ───────────────────────────────
    #[test]
    fn test_protection_209_empty_pos_korean_to_nng() {
        let mut tokens = vec![tok("사랑", "")];
        apply_context_corrections(&mut tokens);
        assert_eq!(
            tokens[0].pos, "NNG",
            "empty POS for Korean surface '사랑' must become NNG (Pass 209)"
        );
    }

    // ── 보호 테스트: 223차 XR(어근) → NNG 변환 ───────────────────────────────
    #[test]
    fn test_protection_223_xr_to_nng() {
        let mut tokens = vec![tok("아름답", "XR")];
        apply_context_corrections(&mut tokens);
        assert_eq!(
            tokens[0].pos, "NNG",
            "XR root token '아름답' must be converted to NNG (Pass 223)"
        );
    }

    // ── 보호 테스트 (M1): 174차 형용사 어근 뒤 하/XSV → XSA ────────────────
    #[test]
    fn test_protection_conjugation_174_xsv_to_xsa() {
        // 14차가 하/XSV+다/EF→하/VV 변환하므로 "어요/EF" 사용
        let mut tokens = vec![tok("행복", "NNG"), tok("하", "XSV"), tok("어요", "EF")];
        apply_context_corrections(&mut tokens);
        // 174차: 행복 + 하/XSV → 하/XSA, 215차: 미안/심심만 VA 병합
        // 행복은 va_merge_roots에 없으므로 XSA 유지
        let ha_token = tokens.iter().find(|t| t.surface == "하");
        assert!(ha_token.is_some(), "하 token must exist");
        assert_eq!(ha_token.unwrap().pos, "XSA", "하/XSV after adj root must become XSA (Pass 174)");
    }

    // ── 보호 테스트 (M1): post_conjugation 86차 ㄴ다 병합 ───────────────────
    #[test]
    fn test_protection_post_conjugation_86_nda_merge() {
        let mut tokens = vec![
            tok_at("가", "VV", 0, 1),
            tok_at("ㄴ", "ETM", 1, 2),
            tok_at("다", "EF", 2, 3),
        ];
        apply_context_corrections(&mut tokens);
        let nda = tokens.iter().find(|t| t.surface == "ㄴ다");
        assert!(nda.is_some(), "ㄴ/ETM + 다/EF must merge to ㄴ다/EF (Pass 86)");
        assert_eq!(nda.unwrap().pos, "EF");
    }

    // ── 보호 테스트 (M1): post_conjugation 88차 NNG+되→XSV ─────────────────
    #[test]
    fn test_protection_post_conjugation_88_nng_doe_xsv() {
        let mut tokens = vec![tok("공개", "NNG"), tok("되", "VV"), tok("었", "EP")];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[1].pos, "XSV", "되/VV after NNG must become XSV (Pass 88)");
    }

    // ── 보호 테스트: particle_and_ending 21차 VCP 삽입 ──────────────────────
    #[test]
    fn test_protection_particle_21_vcp_insertion() {
        let mut tokens = vec![tok("학생", "NNG"), tok("이", "EP"), tok("다", "EF")];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens[1].pos, "VCP", "이/EP after NNG must become VCP (Pass 21)");
    }

    // ── 보호 테스트: compound_noun 196차 XPN 분리 ───────────────────────────
    #[test]
    fn test_protection_compound_196_xpn_maeson() {
        let mut tokens = vec![tok_at("맨손", "NNG", 0, 2)];
        apply_context_corrections(&mut tokens);
        assert_eq!(tokens.len(), 2, "맨손 must split into 맨/XPN + 손/NNG");
        assert_eq!(tokens[0].surface, "맨");
        assert_eq!(tokens[0].pos, "XPN");
        assert_eq!(tokens[1].surface, "손");
        assert_eq!(tokens[1].pos, "NNG");
    }

    // ── 보호 테스트 (sentence_final): 89차 문장 끝 어요/EC → EF ─────────────
    #[test]
    fn test_protection_sentence_final_89_eoyo_ec_to_ef() {
        // Pass 89: 문장 끝 "어요/EC" → "어요/EF"
        // apply_sentence_final_corrections 직접 호출
        let mut tokens = vec![tok("먹", "VV"), tok("어요", "EC")];
        apply_sentence_final_corrections(&mut tokens);
        assert_eq!(
            tokens.last().unwrap().pos,
            "EF",
            "어요/EC at sentence end must become EF (Pass 89)"
        );
    }

    // ── 보호 테스트 (sentence_final): 91차 ㄴ다/EC → EF ─────────────────────
    #[test]
    fn test_protection_sentence_final_91_nda_ec_to_ef() {
        // Pass 91: "ㄴ다/EC" → "ㄴ다/EF" (문장 내 모든 위치)
        // apply_sentence_final_corrections 직접 호출
        let mut tokens = vec![tok("가", "VV"), tok("ㄴ다", "EC")];
        apply_sentence_final_corrections(&mut tokens);
        assert_eq!(
            tokens.last().unwrap().pos,
            "EF",
            "ㄴ다/EC must become EF (Pass 91)"
        );
    }

    // ── 보호 테스트 (sentence_final): 154차 문장 끝 다/NNG → EF ─────────────
    #[test]
    fn test_protection_sentence_final_154_da_nng_to_ef() {
        // Pass 154: 문장 끝 "다/NNG" → "다/EF"
        // VV 뒤의 "다/NNG"도 EF로 변환
        // apply_sentence_final_corrections 직접 호출
        let mut tokens = vec![tok("하", "VV"), tok("다", "NNG")];
        apply_sentence_final_corrections(&mut tokens);
        assert_eq!(
            tokens.last().unwrap().pos,
            "EF",
            "다/NNG at sentence end after VV must become EF (Pass 154)"
        );
    }

    // ── 보호 테스트 (conjugation): 174차 직접 호출 XSV → XSA ────────────────
    #[test]
    fn test_protection_conjugation_174_direct_call() {
        // Pass 174: 형용사 어근 뒤 하/XSV → XSA
        // "행복/NNG + 하/XSV + 어요/EF" → 하/XSA
        // apply_conjugation_corrections 직접 호출 (apply_context_corrections 경유 아님)
        let mut tokens = vec![tok("행복", "NNG"), tok("하", "XSV"), tok("어요", "EF")];
        apply_conjugation_corrections(&mut tokens);
        assert_eq!(
            tokens[1].pos,
            "XSA",
            "하/XSV after adj root 행복 must become XSA (Pass 174 direct call)"
        );
    }

    // ── 보호 테스트 (conjugation): 217차 VA + 으면/EF → EC 직접 호출 ─────────
    #[test]
    fn test_protection_conjugation_217_va_eumyeon_ef_to_ec() {
        // Pass 217: VA 뒤 으면/EF → 으면/EC (연결어미)
        // apply_conjugation_corrections 직접 호출 (apply_context_corrections 경유 아님)
        let mut tokens = vec![tok("예쁘", "VA"), tok("으면", "EF")];
        apply_conjugation_corrections(&mut tokens);
        assert_eq!(tokens[0].pos, "VA", "예쁘/VA must remain VA");
        assert_eq!(
            tokens[1].pos,
            "EC",
            "으면/EF after VA must become EC (Pass 217 direct call)"
        );
    }

    // ── 보호 테스트 (verb_morpheme): 24차 직접 호출 가기/NNG → VV+ETN ────────
    #[test]
    fn test_protection_verb_morpheme_24_verb_gi_splitting() {
        // Pass 24: 명사형 어미 분리 "가기/NNG" → "가/VV + 기/ETN"
        let mut tokens = vec![tok("가기", "NNG")];
        apply_verb_and_morpheme_corrections(&mut tokens);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].surface, "가");
        assert_eq!(tokens[0].pos, "VV");
        assert_eq!(tokens[1].surface, "기");
        assert_eq!(tokens[1].pos, "ETN");
    }

    // ── 보호 테스트 (compound_irregular): 228차 직접 호출 할머님 병합 ─────────
    #[test]
    fn test_protection_compound_irregular_228_halmeonim() {
        // Pass 228: "하/XSV + ㄹ/ETM + 머/NP + 님/XSN" → "할머님/NNG"
        let mut tokens = vec![
            tok("하", "XSV"),
            tok("ㄹ", "ETM"),
            tok("머", "NP"),
            tok("님", "XSN"),
        ];
        apply_compound_and_irregular_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].surface, "할머님");
        assert_eq!(tokens[0].pos, "NNG");
    }

    // ── 보호 테스트 (suffix_dep): 167차 직접 호출 NNG + 적/XSN 병합 ───────────
    #[test]
    fn test_protection_suffix_dep_167_jeok_xsn_merge() {
        // Pass 167: NNG + "적/XSN" → NNG 병합 (e.g. "역사적" → "역사적/NNG")
        let mut tokens = vec![
            tok("역사", "NNG"),
            tok("적", "XSN"),
        ];
        apply_suffix_and_dependency_corrections(&mut tokens);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].surface, "역사적");
        assert_eq!(tokens[0].pos, "NNG");
    }
