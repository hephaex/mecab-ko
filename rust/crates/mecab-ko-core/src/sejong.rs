//! 세종 코퍼스 호환 모듈
//!
//! mecab-ko-dic 출력을 세종 코퍼스 형식으로 변환합니다.
//!
//! # 배경
//!
//! mecab-ko-dic과 세종 코퍼스는 토큰화 기준이 다릅니다:
//! - mecab-ko-dic: 어미 결합 (갔다/VV+EF)
//! - 세종 코퍼스: 어미 분리 (갔/VV 다/EF)
//!
//! # 분석결과 활용
//!
//! mecab-ko-dic의 12번째 컬럼에는 형태소 분해 정보가 저장되어 있습니다:
//! - 형식: `stem/POS/*+ending/POS/*`
//! - 예시: `가깝/VA/*+아/EC/*` (가까와 → 가깝 + 아)
//!
//! 이 정보를 활용하면 불규칙 활용도 정확하게 분리할 수 있습니다.
//!
//! # 예제
//!
//! ```rust,no_run
//! use mecab_ko_core::sejong::{SejongConverter, SejongToken};
//! use mecab_ko_core::tokenizer::Tokenizer;
//!
//! let mut tokenizer = Tokenizer::new().unwrap();
//! let converter = SejongConverter::new();
//!
//! let tokens = tokenizer.tokenize("갔다");
//! let sejong_tokens = converter.convert_tokens(&tokens);
//!
//! // "갔다/VV+EF" -> ["갔/VV", "다/EF"]
//! ```

use crate::tokenizer::Token;
use std::collections::HashMap;

/// 세종 코퍼스 호환 토큰
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SejongToken {
    /// 표면형
    pub surface: String,
    /// 세종 품사 태그
    pub pos: String,
    /// 원본 텍스트 시작 위치
    pub start_pos: usize,
    /// 원본 텍스트 끝 위치
    pub end_pos: usize,
    /// 분리 전 원본 표면형 (복합 형태소일 경우)
    pub original_surface: Option<String>,
    /// 분리 전 원본 품사 (복합 태그일 경우)
    pub original_pos: Option<String>,
}

impl SejongToken {
    /// 새 세종 토큰 생성
    #[must_use]
    pub fn new(surface: &str, pos: &str, start_pos: usize, end_pos: usize) -> Self {
        Self {
            surface: surface.to_string(),
            pos: pos.to_string(),
            start_pos,
            end_pos,
            original_surface: None,
            original_pos: None,
        }
    }

    /// 분리된 토큰 생성 (원본 정보 포함)
    #[must_use]
    pub fn from_split(
        surface: &str,
        pos: &str,
        start_pos: usize,
        end_pos: usize,
        original_surface: &str,
        original_pos: &str,
    ) -> Self {
        Self {
            surface: surface.to_string(),
            pos: pos.to_string(),
            start_pos,
            end_pos,
            original_surface: Some(original_surface.to_string()),
            original_pos: Some(original_pos.to_string()),
        }
    }

    /// 세종 형식 문자열 반환 (표면형/품사)
    #[must_use]
    pub fn to_sejong_format(&self) -> String {
        format!("{}/{}", self.surface, self.pos)
    }
}

/// 어미 분리 규칙
#[derive(Debug, Clone)]
pub struct EndingRule {
    /// 대상 품사 패턴 (예: "VV+EF")
    pub pos_pattern: String,
    /// 어미 목록 (우선순위 순)
    pub endings: Vec<String>,
    /// 분리 후 품사 태그들
    pub target_tags: Vec<String>,
}

impl EndingRule {
    /// 새 어미 분리 규칙 생성
    #[must_use]
    pub fn new(pos_pattern: &str, endings: Vec<&str>, target_tags: Vec<&str>) -> Self {
        Self {
            pos_pattern: pos_pattern.to_string(),
            endings: endings.into_iter().map(String::from).collect(),
            target_tags: target_tags.into_iter().map(String::from).collect(),
        }
    }
}

/// 분석결과에서 파싱된 형태소
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecomposedMorpheme {
    /// 형태소 표면형
    pub surface: String,
    /// 품사 태그
    pub pos: String,
}

/// 세종 코퍼스 형식 변환기
pub struct SejongConverter {
    /// 품사 태그 매핑 테이블 (복합 → 분리)
    tag_map: HashMap<String, Vec<String>>,
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
        let mut converter = Self {
            tag_map: HashMap::new(),
            ending_rules: Vec::new(),
            use_decomposition: true, // 기본값: 분석결과 컬럼 활용
        };
        converter.init_tag_map();
        converter.init_ending_rules();
        converter
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

    /// feature 문자열에서 분석결과(12번째 컬럼) 추출
    ///
    /// mecab-ko-dic CSV 형식:
    /// `품사,의미분류,종성,읽기,타입,첫품사,끝품사,분석결과`
    /// (0~7, 총 8개 필드이지만 인덱스 7이 분석결과)
    #[must_use]
    pub fn extract_decomposition(features: &str) -> Option<String> {
        let fields: Vec<&str> = features.split(',').collect();
        // 분석결과는 8번째 필드 (인덱스 7) 또는 그 이후
        // Inflect 타입의 경우 인덱스 7에 분석결과가 있음
        if fields.len() >= 8 {
            let decomp = fields[7].trim();
            if !decomp.is_empty() && decomp != "*" {
                return Some(decomp.to_string());
            }
        }
        None
    }

    /// 품사 태그 매핑 테이블 초기화
    #[allow(clippy::too_many_lines)]
    fn init_tag_map(&mut self) {
        // 동사 + 어미
        self.tag_map
            .insert("VV+EF".to_string(), vec!["VV".to_string(), "EF".to_string()]);
        self.tag_map
            .insert("VV+EC".to_string(), vec!["VV".to_string(), "EC".to_string()]);
        self.tag_map.insert(
            "VV+ETM".to_string(),
            vec!["VV".to_string(), "ETM".to_string()],
        );
        self.tag_map.insert(
            "VV+ETN".to_string(),
            vec!["VV".to_string(), "ETN".to_string()],
        );
        self.tag_map
            .insert("VV+EP".to_string(), vec!["VV".to_string(), "EP".to_string()]);
        self.tag_map.insert(
            "VV+EP+EF".to_string(),
            vec!["VV".to_string(), "EP".to_string(), "EF".to_string()],
        );
        self.tag_map.insert(
            "VV+EP+EC".to_string(),
            vec!["VV".to_string(), "EP".to_string(), "EC".to_string()],
        );

        // 형용사 + 어미
        self.tag_map
            .insert("VA+EF".to_string(), vec!["VA".to_string(), "EF".to_string()]);
        self.tag_map
            .insert("VA+EC".to_string(), vec!["VA".to_string(), "EC".to_string()]);
        self.tag_map.insert(
            "VA+ETM".to_string(),
            vec!["VA".to_string(), "ETM".to_string()],
        );
        self.tag_map
            .insert("VA+EP".to_string(), vec!["VA".to_string(), "EP".to_string()]);
        self.tag_map.insert(
            "VA+EP+EF".to_string(),
            vec!["VA".to_string(), "EP".to_string(), "EF".to_string()],
        );

        // 보조용언 + 어미
        self.tag_map
            .insert("VX+EF".to_string(), vec!["VX".to_string(), "EF".to_string()]);
        self.tag_map
            .insert("VX+EC".to_string(), vec!["VX".to_string(), "EC".to_string()]);

        // 긍정/부정 지정사 + 어미
        self.tag_map.insert(
            "VCP+EF".to_string(),
            vec!["VCP".to_string(), "EF".to_string()],
        );
        self.tag_map.insert(
            "VCN+EF".to_string(),
            vec!["VCN".to_string(), "EF".to_string()],
        );

        // 체언 + 격조사
        self.tag_map.insert(
            "NNG+JKS".to_string(),
            vec!["NNG".to_string(), "JKS".to_string()],
        );
        self.tag_map.insert(
            "NNG+JKC".to_string(),
            vec!["NNG".to_string(), "JKC".to_string()],
        );
        self.tag_map.insert(
            "NNG+JKG".to_string(),
            vec!["NNG".to_string(), "JKG".to_string()],
        );
        self.tag_map.insert(
            "NNG+JKO".to_string(),
            vec!["NNG".to_string(), "JKO".to_string()],
        );
        self.tag_map.insert(
            "NNG+JKB".to_string(),
            vec!["NNG".to_string(), "JKB".to_string()],
        );
        self.tag_map.insert(
            "NNG+JKV".to_string(),
            vec!["NNG".to_string(), "JKV".to_string()],
        );
        self.tag_map.insert(
            "NNG+JKQ".to_string(),
            vec!["NNG".to_string(), "JKQ".to_string()],
        );
        self.tag_map.insert(
            "NNP+JKS".to_string(),
            vec!["NNP".to_string(), "JKS".to_string()],
        );
        self.tag_map.insert(
            "NNP+JKO".to_string(),
            vec!["NNP".to_string(), "JKO".to_string()],
        );
        self.tag_map.insert(
            "NNP+JKB".to_string(),
            vec!["NNP".to_string(), "JKB".to_string()],
        );
        self.tag_map.insert(
            "NP+JKS".to_string(),
            vec!["NP".to_string(), "JKS".to_string()],
        );
        self.tag_map.insert(
            "NP+JKO".to_string(),
            vec!["NP".to_string(), "JKO".to_string()],
        );

        // 체언 + 보조사
        self.tag_map.insert(
            "NNG+JX".to_string(),
            vec!["NNG".to_string(), "JX".to_string()],
        );
        self.tag_map.insert(
            "NNP+JX".to_string(),
            vec!["NNP".to_string(), "JX".to_string()],
        );
        self.tag_map.insert(
            "NP+JX".to_string(),
            vec!["NP".to_string(), "JX".to_string()],
        );

        // 체언 + 접속조사
        self.tag_map.insert(
            "NNG+JC".to_string(),
            vec!["NNG".to_string(), "JC".to_string()],
        );
    }

    /// 어미 분리 규칙 초기화
    #[allow(clippy::too_many_lines)]
    fn init_ending_rules(&mut self) {
        // 종결어미 (EF)
        self.ending_rules.push(EndingRule::new(
            "VV+EF",
            vec![
                "습니다", "ㅂ니다", "는다", "ㄴ다", "다", "어요", "아요", "요", "어", "아", "지",
                "네", "군", "구나",
            ],
            vec!["VV", "EF"],
        ));

        self.ending_rules.push(EndingRule::new(
            "VA+EF",
            vec![
                "습니다", "ㅂ니다", "다", "어요", "아요", "요", "어", "아", "지", "네", "군",
            ],
            vec!["VA", "EF"],
        ));

        // 연결어미 (EC)
        self.ending_rules.push(EndingRule::new(
            "VV+EC",
            vec![
                "고", "면", "서", "니", "니까", "어서", "아서", "며", "지만", "는데", "ㄴ데",
            ],
            vec!["VV", "EC"],
        ));

        self.ending_rules.push(EndingRule::new(
            "VA+EC",
            vec![
                "고", "면", "서", "니", "니까", "어서", "아서", "며", "지만", "ㄴ데",
            ],
            vec!["VA", "EC"],
        ));

        // 관형형어미 (ETM)
        self.ending_rules.push(EndingRule::new(
            "VV+ETM",
            vec!["는", "ㄴ", "ㄹ", "을", "던"],
            vec!["VV", "ETM"],
        ));

        self.ending_rules.push(EndingRule::new(
            "VA+ETM",
            vec!["ㄴ", "ㄹ", "을", "던"],
            vec!["VA", "ETM"],
        ));

        // 명사형어미 (ETN)
        self.ending_rules.push(EndingRule::new(
            "VV+ETN",
            vec!["기", "ㅁ", "음"],
            vec!["VV", "ETN"],
        ));

        // 선어말어미 + 종결어미 (EP+EF)
        self.ending_rules.push(EndingRule::new(
            "VV+EP+EF",
            vec![
                "었습니다",
                "았습니다",
                "였습니다",
                "었어요",
                "았어요",
                "였어요",
                "었다",
                "았다",
                "였다",
                "었어",
                "았어",
            ],
            vec!["VV", "EP", "EF"],
        ));

        self.ending_rules.push(EndingRule::new(
            "VA+EP+EF",
            vec![
                "었습니다",
                "았습니다",
                "였습니다",
                "었어요",
                "았어요",
                "였어요",
                "었다",
                "았다",
                "였다",
            ],
            vec!["VA", "EP", "EF"],
        ));

        // ========== 조사 분리 규칙 ==========

        // 주격조사 (JKS)
        self.ending_rules.push(EndingRule::new(
            "NNG+JKS",
            vec!["이", "가", "께서"],
            vec!["NNG", "JKS"],
        ));
        self.ending_rules.push(EndingRule::new(
            "NNP+JKS",
            vec!["이", "가", "께서"],
            vec!["NNP", "JKS"],
        ));
        self.ending_rules.push(EndingRule::new(
            "NP+JKS",
            vec!["이", "가", "께서"],
            vec!["NP", "JKS"],
        ));

        // 목적격조사 (JKO)
        self.ending_rules.push(EndingRule::new(
            "NNG+JKO",
            vec!["을", "를"],
            vec!["NNG", "JKO"],
        ));
        self.ending_rules.push(EndingRule::new(
            "NNP+JKO",
            vec!["을", "를"],
            vec!["NNP", "JKO"],
        ));
        self.ending_rules.push(EndingRule::new(
            "NP+JKO",
            vec!["을", "를"],
            vec!["NP", "JKO"],
        ));

        // 부사격조사 (JKB)
        self.ending_rules.push(EndingRule::new(
            "NNG+JKB",
            vec!["에", "에서", "에게", "로", "으로", "한테", "보다", "처럼", "같이", "까지", "부터", "와", "과"],
            vec!["NNG", "JKB"],
        ));
        self.ending_rules.push(EndingRule::new(
            "NNP+JKB",
            vec!["에", "에서", "에게", "로", "으로", "한테", "보다", "처럼", "같이", "까지", "부터", "와", "과"],
            vec!["NNP", "JKB"],
        ));

        // 관형격조사 (JKG)
        self.ending_rules.push(EndingRule::new(
            "NNG+JKG",
            vec!["의"],
            vec!["NNG", "JKG"],
        ));
        self.ending_rules.push(EndingRule::new(
            "NNP+JKG",
            vec!["의"],
            vec!["NNP", "JKG"],
        ));

        // 호격조사 (JKV)
        self.ending_rules.push(EndingRule::new(
            "NNG+JKV",
            vec!["아", "야", "여", "이여"],
            vec!["NNG", "JKV"],
        ));
        self.ending_rules.push(EndingRule::new(
            "NNP+JKV",
            vec!["아", "야", "여", "이여"],
            vec!["NNP", "JKV"],
        ));

        // 보조사 (JX)
        self.ending_rules.push(EndingRule::new(
            "NNG+JX",
            vec!["은", "는", "도", "만", "까지", "부터", "마저", "조차", "라도", "밖에", "요"],
            vec!["NNG", "JX"],
        ));
        self.ending_rules.push(EndingRule::new(
            "NNP+JX",
            vec!["은", "는", "도", "만", "까지", "부터", "마저", "조차", "라도", "밖에", "요"],
            vec!["NNP", "JX"],
        ));
        self.ending_rules.push(EndingRule::new(
            "NP+JX",
            vec!["은", "는", "도", "만", "까지", "부터", "마저", "조차", "라도", "밖에", "요"],
            vec!["NP", "JX"],
        ));

        // 접속조사 (JC)
        self.ending_rules.push(EndingRule::new(
            "NNG+JC",
            vec!["와", "과", "이랑", "랑", "하고"],
            vec!["NNG", "JC"],
        ));
    }

    /// 복합 품사 태그인지 확인
    #[must_use]
    pub fn is_compound_tag(&self, pos: &str) -> bool {
        pos.contains('+')
    }

    /// 복합 품사 태그를 분리된 태그 목록으로 변환
    #[must_use]
    pub fn split_compound_tag(&self, pos: &str) -> Vec<String> {
        self.tag_map.get(pos).cloned().unwrap_or_else(|| {
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
    pub fn split_morpheme(&self, surface: &str, pos: &str) -> Vec<(String, String)> {
        // 복합 태그가 아니면 그대로 반환
        if !self.is_compound_tag(pos) {
            return vec![(surface.to_string(), pos.to_string())];
        }

        // 적용 가능한 규칙 찾기
        for rule in &self.ending_rules {
            if rule.pos_pattern == pos {
                // 어미 패턴 매칭 시도
                for ending in &rule.endings {
                    if surface.ends_with(ending) {
                        let stem_len = surface.chars().count() - ending.chars().count();
                        if stem_len > 0 {
                            let stem: String = surface.chars().take(stem_len).collect();

                            // 분리된 형태소 생성
                            return Self::create_split_morphemes(&stem, ending, &rule.target_tags);
                        }
                    }
                }
            }
        }

        // 규칙이 적용되지 않으면 태그만 분리
        let tags = self.split_compound_tag(pos);
        if tags.len() > 1 {
            // 어미를 분리할 수 없으면 표면형 전체에 첫 번째 태그 부여
            return vec![(surface.to_string(), tags[0].clone())];
        }

        vec![(surface.to_string(), pos.to_string())]
    }

    /// 분리된 형태소 생성 (어간 + 어미들)
    fn create_split_morphemes(
        stem: &str,
        ending: &str,
        tags: &[String],
    ) -> Vec<(String, String)> {
        let mut result = Vec::new();

        if tags.len() == 2 {
            // 어간 + 어미 (예: VV + EF)
            result.push((stem.to_string(), tags[0].clone()));
            result.push((ending.to_string(), tags[1].clone()));
        } else if tags.len() == 3 {
            // 어간 + 선어말어미 + 종결어미 (예: VV + EP + EF)
            // 어미 부분에서 선어말어미와 종결어미 분리 시도
            let (prefinal, final_ending) = Self::split_prefinal_ending(ending);
            result.push((stem.to_string(), tags[0].clone()));
            result.push((prefinal, tags[1].clone()));
            result.push((final_ending, tags[2].clone()));
        } else {
            // 기타 경우
            result.push((stem.to_string(), tags[0].clone()));
            if tags.len() > 1 {
                result.push((ending.to_string(), tags[tags.len() - 1].clone()));
            }
        }

        result
    }

    /// 선어말어미와 종결어미 분리
    fn split_prefinal_ending(ending: &str) -> (String, String) {
        // 선어말어미 패턴: 었, 았, 였, 겠 등
        let prefinal_patterns = ["었", "았", "였", "겠"];

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

    /// 토큰을 세종 형식으로 변환
    ///
    /// 변환 우선순위:
    /// 1. 분석결과 컬럼 사용 (`use_decomposition=true`, features에 분석결과 있는 경우)
    /// 2. 규칙 기반 어미 분리 (`ending_rules`)
    /// 3. 태그만 분리 (복합 태그인 경우)
    /// 4. 그대로 반환 (단순 태그인 경우)
    #[must_use]
    pub fn convert_token(&self, token: &Token) -> Vec<SejongToken> {
        // 1. 분석결과 컬럼 활용 시도
        if self.use_decomposition && !token.features.is_empty() {
            if let Some(decomp) = Self::extract_decomposition(&token.features) {
                let morphemes = Self::parse_decomposition(&decomp);
                if !morphemes.is_empty() {
                    return Self::morphemes_to_sejong_tokens(&morphemes, token);
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
        let mut sejong_tokens: Vec<SejongToken> = tokens
            .iter()
            .flat_map(|t| self.convert_token(t))
            .collect();

        // 잘못된 분해 패턴 보정 (갔다오/VV + ㄴ/ETM → 갔/VV + 다/EF)
        Self::apply_decomposition_corrections(&mut sejong_tokens);

        // 잘못 분해된 토큰 병합 (친/VV + 구와/NNG → 친구/NNG + 와/JC)
        Self::apply_token_merges(&mut sejong_tokens);

        // 고빈도 어휘 강제 매핑 (문맥 무관)
        Self::apply_lexicon_overrides(&mut sejong_tokens);

        // 컨텍스트 기반 품사 보정
        Self::apply_context_corrections(&mut sejong_tokens);

        sejong_tokens
    }

    /// 고빈도 어휘 강제 품사 매핑
    ///
    /// 문맥과 관계없이 특정 표면형은 항상 특정 품사로 지정
    #[allow(clippy::too_many_lines)]
    fn apply_lexicon_overrides(tokens: &mut [SejongToken]) {
        // 고빈도 어휘 -> 올바른 품사 매핑
        // 주의: 모호한 어휘(이, 그, 저, 등)는 문맥에 따라 다른 품사가 될 수 있으므로 제외
        let lexicon: HashMap<&str, &str> = [
            // ===== 인칭대명사 (NP) - 명확한 것만 =====
            ("나", "NP"),
            ("너", "NP"),
            ("우리", "NP"),
            ("그녀", "NP"),
            ("그것", "NP"),
            ("이것", "NP"),
            ("저것", "NP"),
            ("무엇", "NP"),
            ("누구", "NP"),
            ("자기", "NP"),
            ("자신", "NP"),
            ("당신", "NP"),
            ("여러분", "NP"),
            ("너희", "NP"),
            ("저희", "NP"),
            ("이분", "NP"),
            ("그분", "NP"),
            ("저분", "NP"),
            // ===== 의존명사 (NNB) =====
            ("것", "NNB"),
            ("줄", "NNB"),
            ("듯", "NNB"),
            ("데", "NNB"),
            ("바", "NNB"),
            ("만큼", "NNB"),
            ("대로", "NNB"),
            ("뿐", "NNB"),
            ("터", "NNB"),
            ("척", "NNB"),
            ("체", "NNB"),
            // ===== 일반명사 (NNG) - 시간/장소 (명확한 것만) =====
            ("때", "NNG"),
            ("곳", "NNG"),
            ("오늘", "NNG"),
            ("내일", "NNG"),
            ("어제", "NNG"),
            ("모레", "NNG"),
            ("올해", "NNG"),
            ("작년", "NNG"),
            ("내년", "NNG"),
            ("아침", "NNG"),
            ("점심", "NNG"),
            ("저녁", "NNG"),
            ("새벽", "NNG"),
            // ===== 일반명사 (NNG) - 사람/관계 =====
            ("사람", "NNG"),
            ("남자", "NNG"),
            ("여자", "NNG"),
            ("아이", "NNG"),
            ("어른", "NNG"),
            ("친구", "NNG"),
            ("가족", "NNG"),
            ("부모", "NNG"),
            ("아버지", "NNG"),
            ("어머니", "NNG"),
            ("동생", "NNG"),
            ("언니", "NNG"),
            ("오빠", "NNG"),
            ("선생", "NNG"),
            ("학생", "NNG"),
            ("회사원", "NNG"),
            // ===== 일반명사 (NNG) - 사물/개념 =====
            ("책", "NNG"),
            ("음식", "NNG"),
            ("옷", "NNG"),
            ("문제", "NNG"),
            ("생각", "NNG"),
            ("마음", "NNG"),
            ("느낌", "NNG"),
            ("정보", "NNG"),
            ("기술", "NNG"),
            ("방법", "NNG"),
            ("이유", "NNG"),
            ("결과", "NNG"),
            ("상황", "NNG"),
            ("경우", "NNG"),
            ("부분", "NNG"),
            ("전체", "NNG"),
            ("세계", "NNG"),
            ("나라", "NNG"),
            ("사회", "NNG"),
            ("정부", "NNG"),
            ("경제", "NNG"),
            ("문화", "NNG"),
            ("역사", "NNG"),
            ("과학", "NNG"),
            ("교육", "NNG"),
            ("날씨", "NNG"),
            // ===== 부사 (MAG) - 정도 =====
            ("매우", "MAG"),
            ("아주", "MAG"),
            ("너무", "MAG"),
            ("정말", "MAG"),
            ("진짜", "MAG"),
            ("꽤", "MAG"),
            ("좀", "MAG"),
            ("조금", "MAG"),
            ("많이", "MAG"),
            ("적게", "MAG"),
            ("더", "MAG"),
            ("덜", "MAG"),
            ("가장", "MAG"),
            ("제일", "MAG"),
            ("특히", "MAG"),
            // ===== 부사 (MAG) - 빈도/시간 =====
            ("항상", "MAG"),
            ("늘", "MAG"),
            ("자주", "MAG"),
            ("가끔", "MAG"),
            ("때때로", "MAG"),
            ("종종", "MAG"),
            ("별로", "MAG"),
            ("전혀", "MAG"),
            ("결코", "MAG"),
            ("이미", "MAG"),
            ("벌써", "MAG"),
            ("아직", "MAG"),
            ("곧", "MAG"),
            ("바로", "MAG"),
            ("즉시", "MAG"),
            ("먼저", "MAG"),
            ("나중에", "MAG"),
            ("드디어", "MAG"),
            ("마침내", "MAG"),
            // ===== 부사 (MAG) - 방법/양태 =====
            ("잘", "MAG"),
            ("못", "MAG"),
            ("안", "MAG"),
            ("빨리", "MAG"),
            ("천천히", "MAG"),
            ("갑자기", "MAG"),
            ("서서히", "MAG"),
            ("점점", "MAG"),
            ("차츰", "MAG"),
            ("함께", "MAG"),
            ("따로", "MAG"),
            ("혼자", "MAG"),
            ("직접", "MAG"),
            // ===== 접속부사 (MAJ) =====
            ("그래서", "MAJ"),
            ("그러나", "MAJ"),
            ("그런데", "MAJ"),
            ("그리고", "MAJ"),
            ("또한", "MAJ"),
            ("하지만", "MAJ"),
            ("따라서", "MAJ"),
            ("그러므로", "MAJ"),
            ("왜냐하면", "MAJ"),
            ("만약", "MAG"),
            ("혹시", "MAG"),
            ("아마", "MAG"),
            ("분명히", "MAG"),
            ("확실히", "MAG"),
            ("물론", "MAG"),
            ("역시", "MAG"),
            // ===== 관형사 (MM) - 명확한 것만 =====
            ("새", "MM"),
            ("헌", "MM"),
            ("옛", "MM"),
            ("첫", "MM"),
            ("모든", "MM"),
            ("어떤", "MM"),
            ("무슨", "MM"),
            ("어느", "MM"),
            ("여러", "MM"),
            // ===== 감탄사 (IC) =====
            ("아니요", "IC"),
            ("글쎄", "IC"),
            ("여보세요", "IC"),
            // ===== 고유명사 (NNP) - 광역시/도 =====
            ("서울", "NNP"),
            ("부산", "NNP"),
            ("대구", "NNP"),
            ("인천", "NNP"),
            ("광주", "NNP"),
            ("대전", "NNP"),
            ("울산", "NNP"),
            ("경기", "NNP"),
            ("강원", "NNP"),
            ("충북", "NNP"),
            ("충남", "NNP"),
            ("전북", "NNP"),
            ("전남", "NNP"),
            ("경북", "NNP"),
            ("경남", "NNP"),
            ("제주", "NNP"),
            // ===== 고유명사 (NNP) - 주요 도시 =====
            ("수원", "NNP"),
            ("성남", "NNP"),
            ("고양", "NNP"),
            ("용인", "NNP"),
            ("청주", "NNP"),
            ("천안", "NNP"),
            ("전주", "NNP"),
            ("포항", "NNP"),
            ("창원", "NNP"),
            // ===== 고유명사 (NNP) - 국가명 =====
            ("한국", "NNP"),
            ("대한민국", "NNP"),
            ("일본", "NNP"),
            ("중국", "NNP"),
            ("미국", "NNP"),
            ("영국", "NNP"),
            ("프랑스", "NNP"),
            ("독일", "NNP"),
            ("이탈리아", "NNP"),
            ("스페인", "NNP"),
            ("러시아", "NNP"),
            ("캐나다", "NNP"),
            ("호주", "NNP"),
            ("베트남", "NNP"),
            ("태국", "NNP"),
            ("인도", "NNP"),
            ("브라질", "NNP"),
            // ===== 고유명사 (NNP) - 기업/브랜드 =====
            ("삼성", "NNP"),
            ("현대", "NNP"),
            ("네이버", "NNP"),
            ("카카오", "NNP"),
            ("구글", "NNP"),
            ("애플", "NNP"),
            ("마이크로소프트", "NNP"),
            ("아마존", "NNP"),
            ("페이스북", "NNP"),
            ("트위터", "NNP"),
            ("인스타그램", "NNP"),
            ("유튜브", "NNP"),
            // ===== 수사 (SN) =====
            ("하나", "SN"),
            ("둘", "SN"),
            ("셋", "SN"),
            ("넷", "SN"),
            ("다섯", "SN"),
            ("여섯", "SN"),
            ("일곱", "SN"),
            ("여덟", "SN"),
            ("아홉", "SN"),
            ("스물", "SN"),
            ("서른", "SN"),
            ("마흔", "SN"),
            ("쉰", "SN"),
            ("예순", "SN"),
            ("일흔", "SN"),
            ("여든", "SN"),
            ("아흔", "SN"),
            // ===== 수관형사 (MM) =====
            ("첫째", "MM"),
            ("둘째", "MM"),
            ("셋째", "MM"),
            ("넷째", "MM"),
            // ===== 일반명사 (NNG) - 장소 =====
            ("학교", "NNG"),
            ("병원", "NNG"),
            ("은행", "NNG"),
            ("회사", "NNG"),
            ("공원", "NNG"),
            ("시장", "NNG"),
            ("백화점", "NNG"),
            ("편의점", "NNG"),
            ("카페", "NNG"),
            ("식당", "NNG"),
            ("호텔", "NNG"),
            ("도서관", "NNG"),
            ("공항", "NNG"),
            ("항구", "NNG"),
            // ===== 일반명사 (NNG) - 자연 =====
            ("바다", "NNG"),
            ("호수", "NNG"),
            ("나무", "NNG"),
            ("꽃", "NNG"),
            ("하늘", "NNG"),
            ("구름", "NNG"),
            ("바람", "NNG"),
            // ===== 일반명사 (NNG) - 신체 =====
            ("머리", "NNG"),
            ("얼굴", "NNG"),
            ("가슴", "NNG"),
        ]
        .into_iter()
        .collect();

        // 강제 매핑이 필요한 품사 집합 (잘못 인식되는 품사들)
        let overridable_poses: std::collections::HashSet<&str> =
            ["EF", "EC", "EP", "VV", "VA", "NNG"].into_iter().collect();

        for token in tokens.iter_mut() {
            // 오버라이드 대상 품사인 경우에만 적용
            if overridable_poses.contains(token.pos.as_str()) {
                if let Some(&correct_pos) = lexicon.get(token.surface.as_str()) {
                    token.pos = correct_pos.to_string();
                }
            }
        }
    }

    /// 잘못 분해된 토큰 병합
    ///
    /// 사전의 Viterbi 경로 선택 문제로 잘못 분해된 토큰들을 병합합니다.
    /// 예: "친/VV ᆫ/ETM 구와/NNG" → "친구/NNG 와/JC"
    /// 예: "날/NNG 씨/EP" → "날씨/NNG"
    fn apply_token_merges(tokens: &mut Vec<SejongToken>) {
        // 병합 규칙: (패턴, 결과)
        // 패턴: [(surface, pos), ...] - 매칭할 토큰 시퀀스
        // 결과: [(surface, pos), ...] - 병합 결과
        //
        // NOTE: 병합 규칙은 보수적으로 적용 (오탐 방지)
        // 특정 표면형과 품사의 조합만 병합

        let mut i = 0;
        while i < tokens.len() {
            let mut merged = false;

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
                let _end = tokens[i + 2].end_pos;

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
    fn apply_decomposition_corrections(tokens: &mut Vec<SejongToken>) {
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
            if i + 1 < tokens.len() && tokens[i].pos == "VV" && tokens[i + 1].surface == "ㄴ" && tokens[i + 1].pos == "ETM" {
                for &(wrong_stem, correct_stem, stem_pos) in verb_patterns {
                    if tokens[i].surface == wrong_stem {
                        let start = tokens[i].start_pos;
                        let end = tokens[i + 1].end_pos;

                        tokens[i] = SejongToken::new(correct_stem, stem_pos, start, start + correct_stem.chars().count());
                        tokens[i + 1] = SejongToken::new("다", "EF", start + correct_stem.chars().count(), end);

                        matched = true;
                        break;
                    }
                }
            }

            // 패턴 2: X/VA + 은/ETM + 다/EF → X/VA + 다/EF
            // (형용사 관형형 + 종결어미 패턴 보정)
            if !matched && i + 2 < tokens.len()
                && tokens[i].pos == "VA"
                && tokens[i + 1].surface == "은" && tokens[i + 1].pos == "ETM"
                && tokens[i + 2].surface == "다" && tokens[i + 2].pos == "EF"
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

    /// 컨텍스트 기반 품사 보정
    ///
    /// 체언(NNG, NNP, NP) 뒤의 어미(EF)를 조사로 보정
    fn apply_context_corrections(tokens: &mut [SejongToken]) {
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
            // 호격조사 (JKV)
            ("아", "JKV"),
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

        for i in 1..tokens.len() {
            let prev_pos = &tokens[i - 1].pos;
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;

            // 체언 뒤의 잘못 태그된 품사를 조사로 보정
            // ETN: "을" 등이 명사형어미로 잘못 태그되는 경우
            // EF/EC: "가", "는" 등이 어미로 잘못 태그되는 경우
            // EP: "씨" 등이 선어말어미로 잘못 태그되는 경우
            if noun_poses.contains(prev_pos.as_str())
                && (curr_pos == "EF" || curr_pos == "EC" || curr_pos == "ETN" || curr_pos == "EP" || curr_pos == "VV" || curr_pos == "VA")
            {
                if let Some(&correct_pos) = particle_map.get(curr_surface.as_str()) {
                    corrections.push((i, correct_pos.to_string()));
                }
            }
        }

        // 보정 적용
        for (idx, new_pos) in corrections {
            tokens[idx].pos = new_pos;
        }
    }

    /// 세종 형식 문자열로 변환
    #[must_use]
    pub fn format_sejong(&self, tokens: &[SejongToken]) -> String {
        tokens
            .iter()
            .map(SejongToken::to_sejong_format)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// 토큰을 세종 형식 문자열로 직접 변환
    #[must_use]
    pub fn tokens_to_sejong_string(&self, tokens: &[Token]) -> String {
        let sejong_tokens = self.convert_tokens(tokens);
        self.format_sejong(&sejong_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(
            converter.split_compound_tag("VV+EF"),
            vec!["VV", "EF"]
        );
        assert_eq!(
            converter.split_compound_tag("VV+EP+EF"),
            vec!["VV", "EP", "EF"]
        );
        assert_eq!(
            converter.split_compound_tag("NNG"),
            vec!["NNG"]
        );
    }

    #[test]
    fn test_simple_verb_ending_split() {
        let converter = SejongConverter::new();

        // 갔다 -> 갔 + 다
        let result = converter.split_morpheme("갔다", "VV+EF");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("갔".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("다".to_string(), "EF".to_string()));
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

        assert_eq!(sejong_tokens.len(), 2);
        assert_eq!(sejong_tokens[0].surface, "갔");
        assert_eq!(sejong_tokens[0].pos, "VV");
        assert_eq!(sejong_tokens[1].surface, "다");
        assert_eq!(sejong_tokens[1].pos, "EF");
    }

    #[test]
    fn test_convert_tokens() {
        let converter = SejongConverter::new();

        let tokens = vec![
            create_test_token("학교", "NNG"),
            create_test_token("갔다", "VV+EF"),
        ];

        let sejong_tokens = converter.convert_tokens(&tokens);

        assert_eq!(sejong_tokens.len(), 3);
        assert_eq!(sejong_tokens[0].to_sejong_format(), "학교/NNG");
        assert_eq!(sejong_tokens[1].to_sejong_format(), "갔/VV");
        assert_eq!(sejong_tokens[2].to_sejong_format(), "다/EF");
    }

    #[test]
    fn test_format_sejong() {
        let converter = SejongConverter::new();

        let tokens = vec![
            create_test_token("학교", "NNG"),
            create_test_token("갔다", "VV+EF"),
        ];

        let result = converter.tokens_to_sejong_string(&tokens);
        assert_eq!(result, "학교/NNG 갔/VV 다/EF");
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
        use super::DecomposedMorpheme;

        let morpheme = DecomposedMorpheme {
            surface: "가깝".to_string(),
            pos: "VA".to_string(),
        };

        assert_eq!(morpheme.surface, "가깝");
        assert_eq!(morpheme.pos, "VA");
    }
}
