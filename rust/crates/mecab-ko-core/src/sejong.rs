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

        // 선어말어미 단독 (EP)
        self.tag_map
            .insert("VV+EP".to_string(), vec!["VV".to_string(), "EP".to_string()]);
        self.tag_map
            .insert("VA+EP".to_string(), vec!["VA".to_string(), "EP".to_string()]);

        // 선어말어미 + 연결어미 (EP+EC)
        self.tag_map.insert(
            "VV+EP+EC".to_string(),
            vec!["VV".to_string(), "EP".to_string(), "EC".to_string()],
        );
        self.tag_map.insert(
            "VA+EP+EC".to_string(),
            vec!["VA".to_string(), "EP".to_string(), "EC".to_string()],
        );

        // 선어말어미 + 관형형어미 (EP+ETM)
        self.tag_map.insert(
            "VV+EP+ETM".to_string(),
            vec!["VV".to_string(), "EP".to_string(), "ETM".to_string()],
        );
        self.tag_map.insert(
            "VA+EP+ETM".to_string(),
            vec!["VA".to_string(), "EP".to_string(), "ETM".to_string()],
        );

        // 형용사 명사형어미 (VA+ETN)
        self.tag_map.insert(
            "VA+ETN".to_string(),
            vec!["VA".to_string(), "ETN".to_string()],
        );

        // 보조용언 + 어미
        self.tag_map
            .insert("VX+EF".to_string(), vec!["VX".to_string(), "EF".to_string()]);
        self.tag_map
            .insert("VX+EC".to_string(), vec!["VX".to_string(), "EC".to_string()]);
        self.tag_map
            .insert("VX+EP".to_string(), vec!["VX".to_string(), "EP".to_string()]);
        self.tag_map.insert(
            "VX+EP+EF".to_string(),
            vec!["VX".to_string(), "EP".to_string(), "EF".to_string()],
        );

        // 피동/사동 구문 (VV+VX+EF)
        self.tag_map.insert(
            "VV+VX+EF".to_string(),
            vec!["VV".to_string(), "VX".to_string(), "EF".to_string()],
        );

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
        self.tag_map.insert(
            "NP+JKB".to_string(),
            vec!["NP".to_string(), "JKB".to_string()],
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
        // 종결어미 (EF) - 확장
        self.ending_rules.push(EndingRule::new(
            "VV+EF",
            vec![
                // 격식체 종결어미
                "습니다", "ㅂ니다", "습니까", "ㅂ니까", // 합쇼체
                "는다", "ㄴ다", "는가", "ㄴ가", "냐", "니", // 해라체
                "오", "소", // 하오체
                "네", "세", "게", // 하게체
                // 비격식체 종결어미
                "어요", "아요", "요", // 해요체
                "어", "아", // 해체
                // 기타 종결어미
                "지", "죠", "죠", // 추측/확인
                "군", "구나", "구먼", // 감탄
                "래", "려", "자", // 청유
                "마", "거라", "너라", // 명령
                "랴", "리", // 의문
                "다", // 기본 종결
            ],
            vec!["VV", "EF"],
        ));

        self.ending_rules.push(EndingRule::new(
            "VA+EF",
            vec![
                // 격식체
                "습니다", "ㅂ니다", "습니까", "ㅂ니까",
                // 비격식체
                "다", "어요", "아요", "요", "어", "아",
                // 기타
                "지", "죠", "네", "군", "구나",
            ],
            vec!["VA", "EF"],
        ));

        // XSV (파생접미사) + 종결어미 (EF)
        self.ending_rules.push(EndingRule::new(
            "XSV+EF",
            vec![
                // 격식체
                "습니다", "ㅂ니다", "습니까", "ㅂ니까",
                // 비격식체
                "다", "어요", "아요", "요", "어", "아",
                // 축약형
                "ㄴ다", "ㅆ다",
                // 기타
                "지", "죠", "네",
            ],
            vec!["XSV", "EF"],
        ));

        // XSV + 연결어미 (EC)
        self.ending_rules.push(EndingRule::new(
            "XSV+EC",
            vec![
                "고", "면", "으면", "서", "어서", "아서",
                "면서", "으면서", "니까", "으니까", "지만",
            ],
            vec!["XSV", "EC"],
        ));

        // VX (보조용언) + 종결어미 (EF)
        self.ending_rules.push(EndingRule::new(
            "VX+EF",
            vec![
                // 격식체
                "습니다", "ㅂ니다", "습니까", "ㅂ니까",
                // 비격식체
                "어요", "아요", "요", "어", "아", "다",
                // 축약형
                "ㄴ다", "ㅆ다",
                // 기타
                "지", "죠", "네",
            ],
            vec!["VX", "EF"],
        ));

        // VV + VX + EF (피동/사동 구문) - 특별 처리
        // 예: 보이다 → 보/VV + 이/VX + 다/EF
        // 패턴: 어간 + (이/히/리/기) + 다
        self.ending_rules.push(EndingRule::new(
            "VV+VX+EF",
            vec![
                "이다", "히다", "리다", "기다", // 피동/사동 기본형
            ],
            vec!["VV", "VX", "EF"],
        ));

        // VX + 연결어미 (EC)
        self.ending_rules.push(EndingRule::new(
            "VX+EC",
            vec![
                "고", "면", "으면", "서", "어서", "아서",
                "면서", "으면서", "니까", "으니까", "지만",
            ],
            vec!["VX", "EC"],
        ));

        // 연결어미 (EC) - 확장
        // NOTE: 긴 패턴을 먼저 배치하여 올바른 매칭 (예: "아서" > "서")
        self.ending_rules.push(EndingRule::new(
            "VV+EC",
            vec![
                // 긴 패턴 우선 (복합 연결어미)
                "고나서", "자마자", "으면서", "으니까", "더라도", "으므로",
                // 3글자 연결어미
                "어서", "아서", "면서", "든지", "든가", "기에", "길래",
                "거든", "다면", "어도", "아도", "도록", "듯이", "데도",
                // 2글자 연결어미
                "니까", "므로", "지만", "거나", "다가", "더니", "ㄴ데", "는데",
                // 1글자 연결어미 (마지막에 배치)
                "고", "며", "나", "니", "면", "서", "자", "게", "듯",
            ],
            vec!["VV", "EC"],
        ));

        // NOTE: 긴 패턴을 먼저 배치하여 올바른 매칭
        self.ending_rules.push(EndingRule::new(
            "VA+EC",
            vec![
                // 긴 패턴 우선
                "더라도", "으면서", "으니까",
                // 3글자 연결어미
                "어서", "아서", "면서", "어도", "아도", "도록",
                // 2글자 연결어미
                "니까", "므로", "지만", "거나", "거든", "다면", "ㄴ데", "는데", "더니",
                // 1글자 연결어미 (마지막)
                "고", "며", "나", "니", "면", "서", "게",
            ],
            vec!["VA", "EC"],
        ));

        // 관형형어미 (ETM) - 확장
        self.ending_rules.push(EndingRule::new(
            "VV+ETM",
            vec![
                "는",  // 현재 관형형 (먹는)
                "ㄴ", "은", // 과거 관형형 (먹은)
                "ㄹ", "을", // 미래 관형형 (먹을)
                "던",  // 회상 관형형 (먹던)
            ],
            vec!["VV", "ETM"],
        ));

        self.ending_rules.push(EndingRule::new(
            "VA+ETM",
            vec![
                "ㄴ", "은", // 현재 관형형 (좋은, 큰)
                "ㄹ", "을", // 미래 관형형 (좋을)
                "던",  // 회상 관형형 (좋던)
            ],
            vec!["VA", "ETM"],
        ));

        // 명사형어미 (ETN) - 확장
        self.ending_rules.push(EndingRule::new(
            "VV+ETN",
            vec![
                "기",  // 명사형 (먹기)
                "ㅁ", "음", // 명사형 (먹음)
            ],
            vec!["VV", "ETN"],
        ));

        self.ending_rules.push(EndingRule::new(
            "VA+ETN",
            vec![
                "기",  // 명사형 (좋기)
                "ㅁ", "음", // 명사형 (좋음)
            ],
            vec!["VA", "ETN"],
        ));

        // 선어말어미 (EP) 단독 - 시제/양태
        self.ending_rules.push(EndingRule::new(
            "VV+EP",
            vec![
                "었", "았", "였", // 과거 시제
                "겠", // 추측/의지
                "시", "으시", // 높임
                "더", // 회상
            ],
            vec!["VV", "EP"],
        ));

        self.ending_rules.push(EndingRule::new(
            "VA+EP",
            vec![
                "었", "았", "였", // 과거 시제
                "겠", // 추측
                "시", "으시", // 높임
            ],
            vec!["VA", "EP"],
        ));

        // 선어말어미 + 종결어미 (EP+EF) - 확장
        // NOTE: 긴 패턴을 먼저 배치하여 올바른 매칭
        self.ending_rules.push(EndingRule::new(
            "VV+EP+EF",
            vec![
                // 긴 패턴 우선 (5글자 이상)
                "으셨습니다", "으셨습니까", "으셨어요",
                // 4글자 패턴
                "었습니다", "았습니다", "였습니다",
                "었습니까", "았습니까", "였습니까",
                "겠습니다", "셨습니다",
                // 3글자 패턴
                "었어요", "았어요", "였어요",
                "었어", "았어", "였어",
                "었다", "았다", "였다",
                "었니", "았니", "였니",
                "었지", "았지", "였지",
                "었나", "았나", "였나",
                "겠어요", "겠다", "겠지", "겠어",
                "셨어요", "셨다", "셨어",
                "시네요", "으시네요",
                // 2글자 패턴
                "더라", "더군", "더니", "던데",
            ],
            vec!["VV", "EP", "EF"],
        ));

        self.ending_rules.push(EndingRule::new(
            "VA+EP+EF",
            vec![
                // 과거 + 격식체
                "었습니다", "았습니다", "였습니다",
                "었습니까", "았습니까", "였습니까",
                // 과거 + 비격식체
                "었어요", "았어요", "였어요",
                "었어", "았어", "였어",
                "었다", "았다", "였다",
                "었지", "았지", "였지",
                // 추측
                "겠습니다", "겠어요", "겠다", "겠지",
                // 높임
                "시네요", "으시네요", "셨어요", "셨다",
            ],
            vec!["VA", "EP", "EF"],
        ));

        // 선어말어미 + 연결어미 (EP+EC)
        self.ending_rules.push(EndingRule::new(
            "VV+EP+EC",
            vec![
                // 과거 + 연결
                "었고", "았고", "였고",
                "었는데", "았는데", "였는데",
                "었지만", "았지만", "였지만",
                "었으면", "았으면", "였으면",
                "었어도", "았어도", "였어도",
                "었으니", "았으니", "였으니",
                // 추측 + 연결
                "겠고", "겠는데", "겠지만",
            ],
            vec!["VV", "EP", "EC"],
        ));

        self.ending_rules.push(EndingRule::new(
            "VA+EP+EC",
            vec![
                "었고", "았고", "였고",
                "었는데", "았는데", "였는데",
                "었지만", "았지만", "였지만",
                "었으면", "았으면", "였으면",
            ],
            vec!["VA", "EP", "EC"],
        ));

        // 선어말어미 + 관형형어미 (EP+ETM)
        self.ending_rules.push(EndingRule::new(
            "VV+EP+ETM",
            vec![
                "었던", "았던", "였던", // 과거 회상 관형형
                "겠을", // 추측 미래 관형형
            ],
            vec!["VV", "EP", "ETM"],
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
        self.ending_rules.push(EndingRule::new(
            "NP+JKB",
            vec!["에", "에서", "에게", "로", "으로", "한테", "보다", "처럼", "같이", "까지", "부터", "와", "과"],
            vec!["NP", "JKB"],
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

        // 축약형 처리를 먼저 시도 (해요→하+어요, 했어요→하+았+어요 등)
        // 이 처리가 일반 규칙보다 우선해야 함 (해요가 해+요로 분리되는 것 방지)

        // 3개 태그 축약형 처리 (했어요, 갔어요 등)
        if let Some(result) = self.try_split_contracted(surface, pos) {
            return result;
        }

        // 2개 태그 축약형 처리 (해요, 돼요 등)
        if let Some(result) = self.try_split_contracted_two_tags(surface, pos) {
            return result;
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

    /// 축약형 동사 분리 시도
    /// 예: 했어요 → 하 + 았 + 어요, 갔어요 → 가 + 았 + 어요
    fn try_split_contracted(&self, surface: &str, pos: &str) -> Option<Vec<(String, String)>> {
        let tags = self.split_compound_tag(pos);
        if tags.len() != 3 {
            return None;
        }

        // 축약형 패턴 정의: (축약된 어간, 원래 어간, 선어말어미)
        // 하다류: 하+았 → 했, 하+았+어 → 했어
        // 가다류: 가+았 → 갔, 오+았 → 왔
        // 보다류: 보+았 → 봤
        let contracted_stems = [
            ("했", "하", "았"),
            ("갔", "가", "았"),
            ("왔", "오", "았"),
            ("봤", "보", "았"),
            ("샀", "사", "았"),
            ("잤", "자", "았"),
            ("됐", "되", "었"),
        ];

        let chars: Vec<char> = surface.chars().collect();
        if chars.is_empty() {
            return None;
        }

        // 첫 글자가 축약형 어간인지 확인
        let first_char = chars[0].to_string();
        for (contracted, stem, prefinal) in &contracted_stems {
            if first_char == *contracted {
                let ending: String = chars[1..].iter().collect();
                if !ending.is_empty() {
                    // 종결어미 패턴 확인
                    let ef_patterns = ["어요", "어", "다", "지", "니", "나", "습니다", "습니까"];
                    for ef in &ef_patterns {
                        if ending == *ef || ending.ends_with(ef) {
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

        None
    }

    /// 2개 태그 축약형 동사 분리 시도
    /// 예: 해요 → 하 + 어요, 돼요 → 되 + 어요
    /// VV+EF, VA+EF 에서 '하다/되다' 축약형 처리
    fn try_split_contracted_two_tags(
        &self,
        surface: &str,
        pos: &str,
    ) -> Option<Vec<(String, String)>> {
        let tags = self.split_compound_tag(pos);
        if tags.len() != 2 {
            return None;
        }

        // 축약형 패턴: (축약된 1음절, 원래 어간, 연결되는 어미 접두사)
        // 해요 = 하+어요, 해 = 하+어, 했다 = 하+았+다 (이건 3태그라 위에서 처리)
        let contracted_patterns = [
            ("해", "하", "어"),  // 하+어 → 해
            ("돼", "되", "어"),  // 되+어 → 돼
            ("봬", "뵈", "어"),  // 뵈+어 → 봬
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
            ("봤", "보", "았"),  // 보+았 → 봤
            ("갔", "가", "았"),  // 가+았 → 갔
            ("왔", "오", "았"),  // 오+았 → 왔
            ("샀", "사", "았"),  // 사+았 → 샀
            ("잤", "자", "았"),  // 자+았 → 잤
            ("됐", "되", "었"),  // 되+었 → 됐
            ("했", "하", "았"),  // 하+았 → 했
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
            // VV + VX + EF (피동/사동) 특수 처리
            // 예: "이다" → "이/VX + 다/EF"
            if tags[1] == "VX" && tags[2] == "EF" {
                let (vx_part, ef_part) = Self::split_causative_ending(ending);
                result.push((stem.to_string(), tags[0].clone()));
                result.push((vx_part, tags[1].clone()));
                result.push((ef_part, tags[2].clone()));
            } else {
                // 어간 + 선어말어미 + 종결어미 (예: VV + EP + EF)
                // 어미 부분에서 선어말어미와 종결어미 분리 시도
                let (prefinal, final_ending) = Self::split_prefinal_ending(ending);
                result.push((stem.to_string(), tags[0].clone()));
                result.push((prefinal, tags[1].clone()));
                result.push((final_ending, tags[2].clone()));
            }
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
    fn split_prefinal_ending(ending: &str) -> (String, String) {
        // 복합 선어말어미 패턴 (긴 것부터 먼저 매칭)
        // 시제 + 높임: 으셨, 셨, 으시었, 시었
        let compound_prefinal_patterns = [
            "으셨", "셨", "으시었", "시었", // 높임 + 과거
            "으시겠", "시겠", // 높임 + 추측
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
            ("뭐", "NP"),  // 무엇의 준말
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
            // ===== 의문대명사 (NP) =====
            ("어디", "NP"),
            ("언제", "NP"),
            ("어느", "NP"),
            ("어떤", "NP"),
            // ===== 의문부사 (MAG) =====
            ("왜", "MAG"),
            ("어찌", "MAG"),
            ("어떻게", "MAG"),
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
            ("수", "NNB"),   // 할 수 있다
            ("등", "NNB"),   // 사과, 배 등
            ("지", "NNB"),   // 만난 지
            ("양", "NNB"),   // 하는 양
            ("따라", "NNB"), // ~에 따라
            ("중", "NNB"),   // 진행 중
            ("년", "NNB"),   // 십 년
            ("잔", "NNB"),   // 커피 한 잔
            ("원", "NNB"),   // 만 원
            ("분", "NNB"),   // 삼십 분
            ("시", "NNB"),   // 열 시
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
            ("세종", "NNP"),
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
            ("안양", "NNP"),
            ("안산", "NNP"),
            ("파주", "NNP"),
            ("김해", "NNP"),
            ("구미", "NNP"),
            ("진주", "NNP"),
            ("익산", "NNP"),
            ("군산", "NNP"),
            ("여수", "NNP"),
            ("순천", "NNP"),
            ("목포", "NNP"),
            ("강릉", "NNP"),
            ("춘천", "NNP"),
            ("원주", "NNP"),
            ("속초", "NNP"),
            ("제천", "NNP"),
            ("충주", "NNP"),
            ("안동", "NNP"),
            ("경주", "NNP"),
            ("거제", "NNP"),
            ("통영", "NNP"),
            ("양산", "NNP"),
            ("김포", "NNP"),
            ("시흥", "NNP"),
            ("화성", "NNP"),
            ("평택", "NNP"),
            ("의정부", "NNP"),
            ("부천", "NNP"),
            // ===== 고유명사 (NNP) - 서울 주요 지역 =====
            ("강남", "NNP"),
            ("강북", "NNP"),
            ("강서", "NNP"),
            ("강동", "NNP"),
            ("서초", "NNP"),
            ("송파", "NNP"),
            ("영등포", "NNP"),
            ("마포", "NNP"),
            ("종로", "NNP"),
            ("동대문", "NNP"),
            ("성동", "NNP"),
            ("광진", "NNP"),
            ("노원", "NNP"),
            ("도봉", "NNP"),
            ("중랑", "NNP"),
            ("성북", "NNP"),
            ("관악", "NNP"),
            ("동작", "NNP"),
            ("금천", "NNP"),
            ("구로", "NNP"),
            ("양천", "NNP"),
            ("명동", "NNP"),
            ("이태원", "NNP"),
            ("홍대", "NNP"),
            ("신촌", "NNP"),
            ("압구정", "NNP"),
            ("잠실", "NNP"),
            ("여의도", "NNP"),
            // ===== 고유명사 (NNP) - 국가명 (확장) =====
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
            ("멕시코", "NNP"),
            ("아르헨티나", "NNP"),
            ("네덜란드", "NNP"),
            ("벨기에", "NNP"),
            ("스위스", "NNP"),
            ("오스트리아", "NNP"),
            ("폴란드", "NNP"),
            ("체코", "NNP"),
            ("헝가리", "NNP"),
            ("그리스", "NNP"),
            ("터키", "NNP"),
            ("이집트", "NNP"),
            ("남아공", "NNP"),
            ("사우디", "NNP"),
            ("이란", "NNP"),
            ("이라크", "NNP"),
            ("파키스탄", "NNP"),
            ("방글라데시", "NNP"),
            ("인도네시아", "NNP"),
            ("필리핀", "NNP"),
            ("말레이시아", "NNP"),
            ("싱가포르", "NNP"),
            ("대만", "NNP"),
            ("홍콩", "NNP"),
            ("북한", "NNP"),
            ("몽골", "NNP"),
            ("뉴질랜드", "NNP"),
            ("스웨덴", "NNP"),
            ("노르웨이", "NNP"),
            ("덴마크", "NNP"),
            ("핀란드", "NNP"),
            ("아이슬란드", "NNP"),
            ("포르투갈", "NNP"),
            ("우크라이나", "NNP"),
            // ===== 고유명사 (NNP) - 기업/브랜드 (확장) =====
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
            ("틱톡", "NNP"),
            ("텔레그램", "NNP"),
            ("디스코드", "NNP"),
            ("넷플릭스", "NNP"),
            ("디즈니", "NNP"),
            ("스포티파이", "NNP"),
            ("테슬라", "NNP"),
            ("엔비디아", "NNP"),
            ("인텔", "NNP"),
            ("소니", "NNP"),
            ("도요타", "NNP"),
            ("혼다", "NNP"),
            ("닌텐도", "NNP"),
            ("엘지", "NNP"),
            ("에스케이", "NNP"),
            ("롯데", "NNP"),
            ("신세계", "NNP"),
            ("쿠팡", "NNP"),
            ("배달의민족", "NNP"),
            ("당근마켓", "NNP"),
            ("토스", "NNP"),
            ("야놀자", "NNP"),
            ("무신사", "NNP"),
            ("마켓컬리", "NNP"),
            ("오늘의집", "NNP"),
            ("직방", "NNP"),
            ("리디", "NNP"),
            // ===== 고유명사 (NNP) - 대학교 =====
            ("서울대", "NNP"),
            ("연세대", "NNP"),
            ("고려대", "NNP"),
            ("카이스트", "NNP"),
            ("포스텍", "NNP"),
            ("성균관대", "NNP"),
            ("한양대", "NNP"),
            ("이화여대", "NNP"),
            ("경희대", "NNP"),
            ("서강대", "NNP"),
            ("중앙대", "NNP"),
            ("한국외대", "NNP"),
            ("건국대", "NNP"),
            ("동국대", "NNP"),
            ("홍익대", "NNP"),
            ("부산대", "NNP"),
            ("경북대", "NNP"),
            ("전남대", "NNP"),
            ("충남대", "NNP"),
            ("전북대", "NNP"),
            // ===== 고유명사 (NNP) - 인물/유명인 =====
            ("이순신", "NNP"),
            ("세종대왕", "NNP"),
            ("정조", "NNP"),
            ("안중근", "NNP"),
            ("김구", "NNP"),
            ("유관순", "NNP"),
            ("반기문", "NNP"),
            ("손흥민", "NNP"),
            ("김연아", "NNP"),
            ("방탄소년단", "NNP"),
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
            // ===== 형용사 (VA) - 자주 잘못 인식되는 것 =====
            ("덥", "VA"),
            ("춥", "VA"),
            ("좁", "VA"),
            ("넓", "VA"),
            ("깊", "VA"),
            ("얕", "VA"),
        ]
        .into_iter()
        .collect();

        // 강제 매핑이 필요한 품사 집합 (잘못 인식되는 품사들)
        let overridable_poses: std::collections::HashSet<&str> =
            ["EF", "EC", "EP", "VV", "VA", "NNG", "NNP", "IC", "MAG"].into_iter().collect();

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
    #[allow(clippy::useless_let_if_seq, clippy::too_many_lines)]
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
            if !merged && tokens[i].pos == "VV" && tokens[i].surface.ends_with("지") {
                let surface = &tokens[i].surface;
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
            if !merged
                && i + 1 < tokens.len()
                && tokens[i].pos == "VV"
                && tokens[i + 1].pos == "EF"
            {
                let surface = &tokens[i].surface;
                let next_surface = &tokens[i + 1].surface;
                // ㄹ을 떼어내기: 올 → 오
                if let Some(last_char) = surface.chars().last() {
                    // 받침이 ㄹ인 경우 (종성 ㄹ = 0x11AF)
                    // 올 = 오 + ㅗ + ㄹ => 떼면 오
                    let code = last_char as u32;
                    if code >= 0xAC00 && code <= 0xD7A3 {
                        let final_jamo = (code - 0xAC00) % 28;
                        if final_jamo == 8 {
                            // ㄹ 받침
                            // ㄹ을 떼면 새로운 글자
                            let new_code = code - 8;
                            if let Some(new_char) = char::from_u32(new_code) {
                                // 어미에 ㄹ을 붙임
                                let new_ending = format!("ㄹ{}", next_surface);
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
    #[allow(clippy::too_many_lines)]
    fn apply_context_corrections(tokens: &mut Vec<SejongToken>) {
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

        // 의문대명사 집합 (이 뒤의 VV는 조사가 아님)
        let interrogatives: std::collections::HashSet<&str> =
            ["어디", "언제", "뭐", "무엇", "누구", "어느", "어떤", "왜", "어찌"].into_iter().collect();

        for i in 1..tokens.len() {
            let prev_surface = &tokens[i - 1].surface;
            let prev_pos = &tokens[i - 1].pos;
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;

            // 체언 뒤의 잘못 태그된 품사를 조사로 보정
            // ETN: "을" 등이 명사형어미로 잘못 태그되는 경우
            // EF/EC: "가", "는" 등이 어미로 잘못 태그되는 경우
            // EP: "씨" 등이 선어말어미로 잘못 태그되는 경우
            // JKB: "께서" 등이 부사격조사로 잘못 태그되는 경우 → JKS로 보정
            if noun_poses.contains(prev_pos.as_str())
                && (curr_pos == "EF" || curr_pos == "EC" || curr_pos == "ETN" || curr_pos == "EP" || curr_pos == "VV" || curr_pos == "VA" || curr_pos == "JKB")
            {
                // 다음 토큰이 EP(선어말어미)인 경우 동사의 일부이므로 조사로 보정하지 않음
                // 예: 학교/NNG 가/VV 았/EP 다/EF -> "가"는 동사 "가다"의 어간
                let next_is_ep = i + 1 < tokens.len() && tokens[i + 1].pos == "EP";

                // 다음 토큰이 EF/EC인 경우 현재 토큰은 동사의 어간이므로 조사로 보정하지 않음
                // 예: 어디/NP 가/VV 니/EF -> "가"는 동사 "가다"의 어간
                let next_is_ending = i + 1 < tokens.len()
                    && (tokens[i + 1].pos == "EF" || tokens[i + 1].pos == "EC");

                // 의문대명사 뒤의 VV는 동사로 유지 (조사가 아님)
                // 예: 어디 가니, 뭐 하니
                let prev_is_interrogative = interrogatives.contains(prev_surface.as_str());

                // "께서"는 항상 주격조사 (동사 어간이 될 수 없음)
                let is_definite_particle = curr_surface == "께서";

                if is_definite_particle || (!next_is_ep && !next_is_ending && !prev_is_interrogative)
                {
                    if let Some(&correct_pos) = particle_map.get(curr_surface.as_str()) {
                        corrections.push((i, correct_pos.to_string()));
                    }
                }
            }
        }

        // 보정 적용
        for (idx, new_pos) in corrections {
            tokens[idx].pos = new_pos;
        }

        // 2차 보정: 동사/형용사 뒤의 관형형어미(ETM) 보정
        let verb_poses: std::collections::HashSet<&str> = ["VV", "VA", "VX"].into_iter().collect();
        let etm_map: HashMap<&str, &str> = [
            ("는", "ETM"), // 현재 관형형: 가는, 먹는
            ("ㄴ", "ETM"), // 과거 관형형: 간, 먹은
            ("은", "ETM"), // 과거 관형형: 먹은
            ("ㄹ", "ETM"), // 미래 관형형: 갈, 먹을
            ("을", "ETM"), // 미래 관형형: 먹을
            ("던", "ETM"), // 회상 관형형: 가던, 먹던
        ]
        .into_iter()
        .collect();

        let mut etm_corrections: Vec<(usize, String)> = Vec::new();

        for i in 1..tokens.len() {
            let prev_pos = &tokens[i - 1].pos;
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;

            // 동사/형용사 뒤의 JX/EF를 ETM으로 보정
            if verb_poses.contains(prev_pos.as_str())
                && (curr_pos == "JX" || curr_pos == "EF" || curr_pos == "EC")
            {
                if let Some(&correct_pos) = etm_map.get(curr_surface.as_str()) {
                    etm_corrections.push((i, correct_pos.to_string()));
                }
            }
        }

        // ETM 보정 적용
        for (idx, new_pos) in etm_corrections {
            tokens[idx].pos = new_pos;
        }

        // 3차 보정: XSV (파생접미사) 보정
        // 일반명사 뒤의 "하다/되다" 계열을 XSV로 보정
        // 패턴: NNG + 하/했/해/되/됐 → NNG + XSV
        // 주의: NP(대명사) 뒤에는 적용하지 않음 (예: "뭐 하니"에서 "하"는 VV)
        let xsv_patterns: HashMap<&str, bool> = [
            ("하", true),   // 하다
            ("해", true),   // 해요 (하+어)
            ("했", true),   // 했다 (하+았)
            ("되", true),   // 되다
            ("됐", true),   // 됐다 (되+었)
        ]
        .into_iter()
        .collect();

        // XSV 보정 대상: 일반명사만 (대명사 NP 제외)
        let xsv_trigger_poses: std::collections::HashSet<&str> =
            ["NNG", "NNP", "NNB"].into_iter().collect();

        let mut xsv_corrections: Vec<(usize, String)> = Vec::new();

        for i in 1..tokens.len() {
            let prev_pos = &tokens[i - 1].pos;
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;

            // 일반명사 뒤의 VV/EF를 XSV로 보정 (대명사 NP 제외)
            if xsv_trigger_poses.contains(prev_pos.as_str())
                && (curr_pos == "VV" || curr_pos == "EF" || curr_pos == "VA")
                && xsv_patterns.contains_key(curr_surface.as_str()) {
                    xsv_corrections.push((i, "XSV".to_string()));
                }
        }

        // XSV 보정 적용
        for (idx, new_pos) in xsv_corrections {
            tokens[idx].pos = new_pos;
        }

        // 4차 보정: 축약된 연결어미 복원
        // 동사 뒤의 "서"를 "아서/어서"로 복원 (모음 조화)
        // 예: 만나/VV + 서/EC → 만나/VV + 아서/EC
        let mut ec_restorations: Vec<(usize, String)> = Vec::new();

        for i in 1..tokens.len() {
            let prev_surface = &tokens[i - 1].surface;
            let prev_pos = &tokens[i - 1].pos;
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;

            // VV/VA 뒤의 "서"를 복원
            if (prev_pos == "VV" || prev_pos == "VA") && curr_surface == "서" && curr_pos == "EC" {
                // 어간의 마지막 모음에 따라 아서/어서 결정
                // ㅏ, ㅗ → 아서 (양성모음)
                // 그 외 → 어서 (음성모음)
                if let Some(last_char) = prev_surface.chars().last() {
                    let vowel = Self::extract_vowel(last_char);
                    let restored = if vowel == 'ㅏ' || vowel == 'ㅗ' {
                        "아서"
                    } else {
                        "어서"
                    };
                    ec_restorations.push((i, restored.to_string()));
                }
            }
        }

        // 연결어미 복원 적용
        for (idx, new_surface) in ec_restorations {
            tokens[idx].surface = new_surface;
        }

        // 4-2차 보정: 축약된 종결어미 복원
        // 동사 뒤의 "요"를 "아요/어요"로 복원 (모음 조화)
        // 예: 가/VV + 요/EF → 가/VV + 아요/EF
        let mut ef_restorations: Vec<(usize, String)> = Vec::new();

        for i in 1..tokens.len() {
            let prev_surface = &tokens[i - 1].surface;
            let prev_pos = &tokens[i - 1].pos;
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;

            // VV/VA 뒤의 "요"를 복원
            if (prev_pos == "VV" || prev_pos == "VA") && curr_surface == "요" && curr_pos == "EF" {
                // 어간의 마지막 모음에 따라 아요/어요 결정
                // ㅏ, ㅗ → 아요 (양성모음)
                // 그 외 → 어요 (음성모음)
                if let Some(last_char) = prev_surface.chars().last() {
                    let vowel = Self::extract_vowel(last_char);
                    let restored = if vowel == 'ㅏ' || vowel == 'ㅗ' {
                        "아요"
                    } else {
                        "어요"
                    };
                    ef_restorations.push((i, restored.to_string()));
                }
            }
        }

        // 종결어미 복원 적용
        for (idx, new_surface) in ef_restorations {
            tokens[idx].surface = new_surface;
        }

        // 5차 보정: "하면/XSV + 서/EC" → "하/XSV + 면서/EC" 변환
        // MeCab이 "하면서"를 "하면" + "서"로 잘못 분리하는 문제 해결
        let mut ec_merge_corrections: Vec<(usize, String, String)> = Vec::new();

        for i in 1..tokens.len() {
            let prev_surface = &tokens[i - 1].surface;
            let prev_pos = &tokens[i - 1].pos;
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;

            // XSV/VV/VA 뒤 "서/EC" 패턴 체크
            if (prev_pos == "XSV" || prev_pos == "VV" || prev_pos == "VA")
                && curr_surface == "서"
                && curr_pos == "EC"
            {
                // "하면" → "하", "서" → "면서"
                if prev_surface.ends_with("면") {
                    let new_prev = prev_surface.trim_end_matches("면").to_string();
                    ec_merge_corrections.push((i - 1, new_prev, "면서".to_string()));
                }
            }
        }

        // EC 병합 보정 적용
        for (prev_idx, new_prev_surface, new_curr_surface) in ec_merge_corrections {
            if !new_prev_surface.is_empty() {
                tokens[prev_idx].surface = new_prev_surface;
            }
            tokens[prev_idx + 1].surface = new_curr_surface;
        }

        // 6차 보정: (비활성화) JC → JKB 변환은 평가 데이터와 불일치
        // "친구와/JC 만나다" - JC 유지 (평가 데이터 기준)

        // 7차 보정: "합니/VV + 다/EF" → "합니다/EF"
        // MeCab이 "합니다"를 "합니 + 다"로 분리하는 문제 해결
        let mut merge_indices: Vec<usize> = Vec::new();

        for i in 0..tokens.len().saturating_sub(1) {
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;

            // 합니/VV + 다/EF → 합니다/EF
            if curr_surface == "합니" && curr_pos == "VV"
                && next_surface == "다" && next_pos == "EF"
            {
                merge_indices.push(i);
            }
        }

        // 역순으로 병합 (인덱스 변화 방지)
        for idx in merge_indices.into_iter().rev() {
            let merged = format!("{}{}", tokens[idx].surface, tokens[idx + 1].surface);
            tokens[idx].surface = merged;
            tokens[idx].pos = "EF".to_string();
            tokens[idx].end_pos = tokens[idx + 1].end_pos;
            tokens.remove(idx + 1);
        }

        // 8차 보정: 문장 끝 "니/EC" → "니/EF"
        // "하니/VV+니/EC" → "하니/VV+니/EF" (종결어미로 사용될 때)
        if let Some(last) = tokens.last_mut() {
            if last.surface == "니" && last.pos == "EC" {
                last.pos = "EF".to_string();
            }
        }

        // 9차 보정: "하/XSV + 아야/EC" → "하/VV + 아야/EC"
        // "준비해야" 등에서 "하다"는 VV로 분석
        let mut xsv_to_vv_indices: Vec<usize> = Vec::new();

        for i in 0..tokens.len().saturating_sub(1) {
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;

            // 하/XSV + 아야/EC → 하/VV + 아야/EC
            if (curr_surface == "하" || curr_surface == "해" || curr_surface == "했")
                && curr_pos == "XSV"
                && next_pos == "EC"
                && (next_surface == "아야" || next_surface == "어야" || next_surface == "야")
            {
                xsv_to_vv_indices.push(i);
            }
        }

        for idx in xsv_to_vv_indices {
            tokens[idx].pos = "VV".to_string();
        }

        // 10차 보정: "고/EC + 나/NP" 다음에 서/EC가 아니면 "고나서" 패턴 아님
        // 일단 단순한 보정: "먹고/EC 나서/EC" → "먹/VV 고나서/EC"
        // 이 패턴은 apply_token_merges에서 처리하는 것이 더 적절

        // 11차 보정: NP + 세요/EF → NP + 이/VCP + 세요/EF
        // "누구세요"에서 계사 "이다"가 생략된 경우 복원
        let mut vcp_insert_indices: Vec<usize> = Vec::new();

        for i in 0..tokens.len().saturating_sub(1) {
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;

            // NP + 세요/EF 또는 NP + 에요/EF → NP + 이/VCP + 세요/EF
            if curr_pos == "NP"
                && next_pos == "EF"
                && (next_surface == "세요" || next_surface == "에요" || next_surface == "예요")
            {
                vcp_insert_indices.push(i + 1);
            }
        }

        // 역순으로 삽입 (인덱스 변화 방지)
        for idx in vcp_insert_indices.into_iter().rev() {
            let start = tokens[idx].start_pos;
            tokens.insert(idx, SejongToken::new("이", "VCP", start, start));
        }

        // 12차 보정: "지/NNB" + "않/VX" → "지/EC" + "않/VX"
        // "하지 않아요"에서 "지"는 연결어미(EC)
        let mut nnb_to_ec_indices: Vec<usize> = Vec::new();

        for i in 0..tokens.len().saturating_sub(1) {
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;

            // 지/NNB + 않/VX → 지/EC + 않/VX
            if curr_surface == "지" && curr_pos == "NNB"
                && next_surface == "않" && next_pos == "VX"
            {
                nnb_to_ec_indices.push(i);
            }
        }

        for idx in nnb_to_ec_indices {
            tokens[idx].pos = "EC".to_string();
        }

        // 13차 보정: 동사 기본형 분리 (Xda/VV → X/VV + 다/EF)
        // 가다, 먹다, 오다, 보다, 하다 등 기본형을 분리
        // 주의: 단독 사용 시만 분리 (문장 내에서는 어간+어미로 분석됨)
        let base_verbs: std::collections::HashSet<&str> = [
            "가다", "오다", "보다", "먹다", "되다", "주다", "받다",
            "쓰다", "읽다", "듣다", "말다", "살다", "죽다", "자다", "일다",
            "앉다", "서다", "놓다", "두다", "치다", "잡다", "놀다", "울다",
        ].into_iter().collect();

        // "하다"는 별도 처리 (XSV인 경우만 VV로 변환 후 분리)

        let mut verb_split_indices: Vec<usize> = Vec::new();

        for (i, token) in tokens.iter().enumerate() {
            if token.pos == "VV" && base_verbs.contains(token.surface.as_str()) {
                verb_split_indices.push(i);
            }
        }

        // 역순으로 분리 (인덱스 변화 방지)
        for idx in verb_split_indices.into_iter().rev() {
            let surface = &tokens[idx].surface;
            if let Some(stem) = surface.strip_suffix("다") {
                if !stem.is_empty() {
                    let start = tokens[idx].start_pos;
                    let end = tokens[idx].end_pos;
                    let stem_len = stem.chars().count();
                    tokens[idx] = SejongToken::new(stem, "VV", start, start + stem_len);
                    tokens.insert(idx + 1, SejongToken::new("다", "EF", start + stem_len, end));
                }
            }
        }

        // 14차 보정: "하/XSV + 다/EF" → "하/VV + 다/EF"
        // 단독 "하다"는 VV로 분석
        let mut xsv_da_to_vv_indices: Vec<usize> = Vec::new();

        for i in 0..tokens.len().saturating_sub(1) {
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;

            // 하/XSV + 다/EF → 하/VV + 다/EF
            if curr_surface == "하" && curr_pos == "XSV"
                && next_surface == "다" && next_pos == "EF"
            {
                xsv_da_to_vv_indices.push(i);
            }
        }

        for idx in xsv_da_to_vv_indices {
            tokens[idx].pos = "VV".to_string();
        }

        // 15차 보정: 복합명사+기(NNG) + 전(NNG) → 어간+기(ETN) + 전(NNG)
        // "가기 전에", "먹기 전에" 등 명사형어미 분리
        let mut gi_split_indices: Vec<usize> = Vec::new();

        for i in 0..tokens.len().saturating_sub(1) {
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;

            // X기/NNG + 전/NNG → X/VV + 기/ETN + 전/NNG
            if curr_pos == "NNG"
                && curr_surface.ends_with("기")
                && curr_surface.chars().count() >= 2
                && (next_surface == "전" || next_surface == "위해" || next_surface == "시작")
            {
                gi_split_indices.push(i);
            }
        }

        // 역순으로 분리 (인덱스 변화 방지)
        for idx in gi_split_indices.into_iter().rev() {
            let surface = &tokens[idx].surface;
            if let Some(stem) = surface.strip_suffix("기") {
                if !stem.is_empty() {
                    let start = tokens[idx].start_pos;
                    let end = tokens[idx].end_pos;
                    let stem_len = stem.chars().count();
                    tokens[idx] = SejongToken::new(stem, "VV", start, start + stem_len);
                    tokens.insert(idx + 1, SejongToken::new("기", "ETN", start + stem_len, end));
                }
            }
        }

        // 16차 보정: 잘못된 "는/JX + 들/XSN + 이/JKS" 패턴 수정
        // 사전 버그로 인해 "들이"가 "는+들+이"로 분해됨
        // 실제로는 "들+이"여야 함 → "는/JX" 토큰 삭제
        let mut jx_delete_indices: Vec<usize> = Vec::new();

        for i in 0..tokens.len().saturating_sub(2) {
            let curr_surface = &tokens[i].surface;
            let curr_pos = &tokens[i].pos;
            let next_surface = &tokens[i + 1].surface;
            let next_pos = &tokens[i + 1].pos;
            let next2_surface = &tokens[i + 2].surface;
            let next2_pos = &tokens[i + 2].pos;

            // "는/JX + 들/XSN + 이/JKS" 패턴 감지
            if curr_surface == "는"
                && curr_pos == "JX"
                && next_surface == "들"
                && next_pos == "XSN"
                && next2_surface == "이"
                && next2_pos == "JKS"
            {
                jx_delete_indices.push(i);
            }
        }

        // 역순으로 삭제 (인덱스 변화 방지)
        for idx in jx_delete_indices.into_iter().rev() {
            tokens.remove(idx);
        }
    }

    /// 한글 음절에서 모음 추출
    fn extract_vowel(ch: char) -> char {
        let code = ch as u32;
        // 한글 음절 범위: 0xAC00 ~ 0xD7A3
        if (0xAC00..=0xD7A3).contains(&code) {
            // 모음 인덱스 = ((code - 0xAC00) / 28) % 21
            let vowel_idx = ((code - 0xAC00) / 28) % 21;
            // 모음: ㅏ ㅐ ㅑ ㅒ ㅓ ㅔ ㅕ ㅖ ㅗ ㅘ ㅙ ㅚ ㅛ ㅜ ㅝ ㅞ ㅟ ㅠ ㅡ ㅢ ㅣ
            let vowels = ['ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ'];
            vowels[vowel_idx as usize]
        } else {
            // 한글 자모 범위인 경우 그대로 반환
            ch
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

        // 갔다 -> 가 + 았 + 다 (과거 시제 축약형 분리)
        // 세종 코퍼스: 가/VV + 았/EP + 다/EF
        let result = converter.split_morpheme("갔다", "VV+EF");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("가".to_string(), "VV".to_string()));
        assert_eq!(result[1], ("았".to_string(), "EP".to_string()));
        assert_eq!(result[2], ("다".to_string(), "EF".to_string()));
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
        use super::DecomposedMorpheme;

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
