//! 세종 코퍼스 데이터 타입 정의

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
