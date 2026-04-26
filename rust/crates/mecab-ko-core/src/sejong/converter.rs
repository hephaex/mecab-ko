//! 세종 코퍼스 형식 변환기

use std::collections::HashMap;

use crate::tokenizer::Token;

use super::corrections::apply_context_corrections;
use super::ending_rules::init_ending_rules;
use super::hangul::normalize_jamo as hangul_normalize_jamo;
use super::lexicon::apply_lexicon_overrides;
use super::postprocess::{
    apply_decomposition_corrections, apply_token_merges, apply_vv_seyo_splits,
};
use super::splitter::{is_compound_tag, split_compound_tag, split_morpheme};
use super::tag_map::tag_map;
use super::types::{DecomposedMorpheme, EndingRule, SejongToken};

/// 세종 코퍼스 형식 변환기
pub struct SejongConverter {
    /// 품사 태그 매핑 테이블 (복합 → 분리) — 전역 정적 참조
    tag_map: &'static HashMap<String, Vec<String>>,
    /// 어미 분리 규칙
    ending_rules: Vec<EndingRule>,
    /// 분석결과 컬럼 사용 여부 (불규칙 활용 지원)
    use_decomposition: bool,
}

impl Default for SejongConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl SejongConverter {
    /// 기본 설정으로 변환기 생성
    #[must_use]
    pub fn new() -> Self {
        Self {
            tag_map: tag_map(),
            ending_rules: init_ending_rules(),
            use_decomposition: true, // 기본값: 분석결과 컬럼 활용
        }
    }

    /// 분석결과 사용 여부 설정
    ///
    /// `true`이면 mecab-ko-dic의 12번째 컬럼(분석결과)을 우선 사용합니다.
    /// 불규칙 활용을 정확하게 처리하려면 `true`로 설정하세요.
    #[must_use]
    pub const fn with_decomposition(mut self, use_decomposition: bool) -> Self {
        self.use_decomposition = use_decomposition;
        self
    }

    /// 분석결과 컬럼에서 형태소 분해 정보 파싱
    ///
    /// 형식: `stem/POS/*+ending/POS/*+...`
    /// 예시: `가깝/VA/*+아/EC/*` → [("가깝", "VA"), ("아", "EC")]
    #[must_use]
    pub fn parse_decomposition(decomposition: &str) -> Vec<DecomposedMorpheme> {
        if decomposition.is_empty() || decomposition == "*" {
            return Vec::new();
        }

        let mut result = Vec::new();

        // '+' 로 분리하여 각 형태소 파싱
        for part in decomposition.split('+') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            // 형식: surface/POS/* 또는 surface/POS
            let segments: Vec<&str> = part.split('/').collect();
            if segments.len() >= 2 {
                let surface = segments[0].to_string();
                let pos = segments[1].to_string();

                // 빈 표면형이나 '*' 는 스킵
                if !surface.is_empty() && surface != "*" && !pos.is_empty() && pos != "*" {
                    result.push(DecomposedMorpheme { surface, pos });
                }
            }
        }

        result
    }

    /// feature 문자열에서 분析결과(12번째 컬럼) 추출
    ///
    /// mecab-ko-dic CSV 형식:
    /// `품사,의미분류,종성,읽기,타입,첫품사,끝품사,분석결과`
    /// (0~7, 총 8개 필드이지만 인덱스 7이 분析결과)
    #[must_use]
    pub fn extract_decomposition(features: &str) -> Option<String> {
        let fields: Vec<&str> = features.split(',').collect();
        // 분析결과는 8번째 필드 (인덱스 7) 또는 그 이후
        // Inflect 타입의 경우 인덱스 7에 분析결과가 있음
        if fields.len() >= 8 {
            let decomp = fields[7].trim();
            if !decomp.is_empty() && decomp != "*" {
                return Some(decomp.to_string());
            }
        }
        None
    }

    // 166차: is_compound_token 함수 제거
    // 복합어 분리는 세종 코퍼스 표준에 맞게 선별적으로 적용 필요
    // 현재는 복합어를 분리하지 않음

    /// 복합 품사 태그인지 확인
    #[must_use]
    pub fn is_compound_tag(&self, pos: &str) -> bool {
        is_compound_tag(pos)
    }

    /// 복합 품사 태그를 분리된 태그 목록으로 변환
    #[must_use]
    pub fn split_compound_tag(&self, pos: &str) -> Vec<String> {
        split_compound_tag(self.tag_map, pos)
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
    pub fn split_morpheme(&self, surface: &str, pos: &str) -> Vec<(String, String)> {
        split_morpheme(surface, pos, self.tag_map, &self.ending_rules)
    }

    /// 토큰을 세종 형식으로 변환
    ///
    /// 변환 우선순위:
    /// 1. 분析결과 컬럼 사용 (`use_decomposition=true`, features에 분析결과 있는 경우)
    /// 2. 규칙 기반 어미 분리 (`ending_rules`)
    /// 3. 태그만 분리 (복합 태그인 경우)
    /// 4. 그대로 반환 (단순 태그인 경우)
    #[must_use]
    pub fn convert_token(&self, token: &Token) -> Vec<SejongToken> {
        // 143차: "는다/VV+EC"는 사전 분析결과가 잘못됨 (늘/VV+ㄴ다/EC)
        // 규칙 기반으로 직접 처리: "는다/VV+EC" → "는다/EF"
        let skip_decomposition = token.surface == "는다" && token.pos == "VV+EC";

        // 166차: Compound 분리 보류
        // 일부 복합어(밤낮)는 분리해야 하지만 대부분(인공지능, 서울특별시)은 분리하면 안 됨
        // 세종 코퍼스 표준에 따라 선별적 적용 필요 - 현재는 비활성화

        // 212차: 특수 동사는 분析결과 무시하고 규칙 기반 사용
        // "들리다", "놀리다" 등은 VV+EF로 처리해야 함 (VV+VX+EF 아님)
        let skip_decomp_verbs = ["들리다", "놀리다"];
        let force_rule_based =
            token.pos == "VV+EF" && skip_decomp_verbs.contains(&token.surface.as_str());

        // 1. 분析결과 컬럼 활용 시도
        if self.use_decomposition
            && !token.features.is_empty()
            && !skip_decomposition
            && !force_rule_based
        {
            if let Some(decomp) = Self::extract_decomposition(&token.features) {
                let morphemes = Self::parse_decomposition(&decomp);
                if !morphemes.is_empty() {
                    // 분析결과의 POS 태그 구조가 토큰 POS와 일치하는지 검증
                    let decomp_pos: String = morphemes
                        .iter()
                        .map(|m| m.pos.as_str())
                        .collect::<Vec<_>>()
                        .join("+");
                    if decomp_pos == token.pos {
                        // 245차 보정: 단일 형태소의 경우 표면형이 변경되면 decomposition 무시
                        if morphemes.len() == 1 && morphemes[0].surface != token.surface {
                            // 표면형이 다르면 decomposition 무시
                        } else {
                            return Self::morphemes_to_sejong_tokens(&morphemes, token);
                        }
                    }
                    // POS 구조가 일치하지 않으면 규칙 기반으로 폴백
                }
            }
        }

        // 2. 규칙 기반 어미 분리
        let morphemes = self.split_morpheme(&token.surface, &token.pos);

        if morphemes.len() == 1 {
            // 분리되지 않은 경우
            return vec![SejongToken::new(
                &token.surface,
                &morphemes[0].1,
                token.start_pos,
                token.end_pos,
            )];
        }

        // 분리된 경우
        let mut result = Vec::new();
        let mut current_pos = token.start_pos;

        for (surface, pos) in &morphemes {
            let char_len = surface.chars().count();
            let end_pos = current_pos + char_len;

            result.push(SejongToken::from_split(
                surface,
                pos,
                current_pos,
                end_pos,
                &token.surface,
                &token.pos,
            ));

            current_pos = end_pos;
        }

        result
    }

    /// 분해된 형태소를 `SejongToken`으로 변환
    fn morphemes_to_sejong_tokens(
        morphemes: &[DecomposedMorpheme],
        original_token: &Token,
    ) -> Vec<SejongToken> {
        let mut result = Vec::new();
        let mut current_pos = original_token.start_pos;

        for morpheme in morphemes {
            let char_len = morpheme.surface.chars().count();
            let end_pos = current_pos + char_len;

            result.push(SejongToken::from_split(
                &morpheme.surface,
                &morpheme.pos,
                current_pos,
                end_pos,
                &original_token.surface,
                &original_token.pos,
            ));

            current_pos = end_pos;
        }

        result
    }

    /// 토큰 목록을 세종 형식으로 변환
    #[must_use]
    pub fn convert_tokens(&self, tokens: &[Token]) -> Vec<SejongToken> {
        let mut sejong_tokens: Vec<SejongToken> =
            tokens.iter().flat_map(|t| self.convert_token(t)).collect();

        // 잘못된 분해 패턴 보정 (갔다오/VV + ㄴ/ETM → 갔/VV + 다/EF)
        apply_decomposition_corrections(&mut sejong_tokens);

        // 잘못 분해된 토큰 병합 (친/VV + 구와/NNG → 친구/NNG + 와/JC)
        apply_token_merges(&mut sejong_tokens);

        // 고빈도 어휘 강제 매핑 (문맥 무관)
        apply_lexicon_overrides(&mut sejong_tokens);

        // VV "세요" 패턴 분리 (가세요/VV → 가/VV + 세요/EF)
        sejong_tokens = apply_vv_seyo_splits(sejong_tokens);

        // 컨텍스트 기반 품사 보정
        apply_context_corrections(&mut sejong_tokens);

        sejong_tokens
    }

    /// 세종 형식 문자열로 변환 (자모 정규화 포함)
    #[must_use]
    pub fn format_sejong(&self, tokens: &[SejongToken]) -> String {
        tokens
            .iter()
            .map(|t| {
                let normalized_surface = hangul_normalize_jamo(&t.surface);
                format!("{}/{}", normalized_surface, t.pos)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// 토큰을 세종 형식 문자열로 직접 변환
    #[must_use]
    pub fn tokens_to_sejong_string(&self, tokens: &[Token]) -> String {
        let sejong_tokens = self.convert_tokens(tokens);
        self.format_sejong(&sejong_tokens)
    }

    /// 한글 자모 정규화 (외부 호환성 유지)
    ///
    /// ㅏ, ㅓ, ㅣ 등 자모가 포함된 문자열을 완성형으로 변환합니다.
    #[must_use]
    pub fn normalize_jamo(text: &str) -> String {
        hangul_normalize_jamo(text)
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn split_prefinal_ending(ending: &str) -> (String, String) {
        super::splitter::split_prefinal_ending(ending)
    }
}
