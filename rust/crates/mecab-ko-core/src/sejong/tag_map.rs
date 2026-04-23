//! 품사 태그 매핑 테이블

use std::collections::HashMap;

/// 품사 태그 매핑 테이블 초기화
#[allow(clippy::too_many_lines)]
#[must_use]
pub(super) fn init_tag_map() -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    // 동사 + 어미
    map.insert(
        "VV+EF".to_string(),
        vec!["VV".to_string(), "EF".to_string()],
    );
    map.insert(
        "VV+EC".to_string(),
        vec!["VV".to_string(), "EC".to_string()],
    );
    map.insert(
        "VV+ETM".to_string(),
        vec!["VV".to_string(), "ETM".to_string()],
    );
    map.insert(
        "VV+ETN".to_string(),
        vec!["VV".to_string(), "ETN".to_string()],
    );
    map.insert(
        "VV+EP".to_string(),
        vec!["VV".to_string(), "EP".to_string()],
    );
    map.insert(
        "VV+EP+EF".to_string(),
        vec!["VV".to_string(), "EP".to_string(), "EF".to_string()],
    );
    map.insert(
        "VV+EP+EC".to_string(),
        vec!["VV".to_string(), "EP".to_string(), "EC".to_string()],
    );
    // 존칭+과거 복합형 (VV+EP+EP+EF): 오시+었+습니다
    map.insert(
        "VV+EP+EP+EF".to_string(),
        vec![
            "VV".to_string(),
            "EP".to_string(),
            "EP".to_string(),
            "EF".to_string(),
        ],
    );

    // 형용사 + 어미
    map.insert(
        "VA+EF".to_string(),
        vec!["VA".to_string(), "EF".to_string()],
    );
    map.insert(
        "VA+EC".to_string(),
        vec!["VA".to_string(), "EC".to_string()],
    );
    map.insert(
        "VA+ETM".to_string(),
        vec!["VA".to_string(), "ETM".to_string()],
    );
    map.insert(
        "VA+EP".to_string(),
        vec!["VA".to_string(), "EP".to_string()],
    );
    map.insert(
        "VA+EP+EF".to_string(),
        vec!["VA".to_string(), "EP".to_string(), "EF".to_string()],
    );

    // 선어말어미 단독 (EP)
    map.insert(
        "VV+EP".to_string(),
        vec!["VV".to_string(), "EP".to_string()],
    );
    map.insert(
        "VA+EP".to_string(),
        vec!["VA".to_string(), "EP".to_string()],
    );

    // 선어말어미 + 연결어미 (EP+EC)
    map.insert(
        "VV+EP+EC".to_string(),
        vec!["VV".to_string(), "EP".to_string(), "EC".to_string()],
    );
    map.insert(
        "VA+EP+EC".to_string(),
        vec!["VA".to_string(), "EP".to_string(), "EC".to_string()],
    );

    // 선어말어미 + 관형형어미 (EP+ETM)
    map.insert(
        "VV+EP+ETM".to_string(),
        vec!["VV".to_string(), "EP".to_string(), "ETM".to_string()],
    );
    map.insert(
        "VA+EP+ETM".to_string(),
        vec!["VA".to_string(), "EP".to_string(), "ETM".to_string()],
    );

    // 형용사 명사형어미 (VA+ETN)
    map.insert(
        "VA+ETN".to_string(),
        vec!["VA".to_string(), "ETN".to_string()],
    );

    // 보조용언 + 어미
    map.insert(
        "VX+EF".to_string(),
        vec!["VX".to_string(), "EF".to_string()],
    );
    map.insert(
        "VX+EC".to_string(),
        vec!["VX".to_string(), "EC".to_string()],
    );
    map.insert(
        "VX+EP".to_string(),
        vec!["VX".to_string(), "EP".to_string()],
    );
    map.insert(
        "VX+EP+EF".to_string(),
        vec!["VX".to_string(), "EP".to_string(), "EF".to_string()],
    );

    // 피동/사동 구문 (VV+VX+EF)
    map.insert(
        "VV+VX+EF".to_string(),
        vec!["VV".to_string(), "VX".to_string(), "EF".to_string()],
    );

    // 긍정/부정 지정사 + 어미
    map.insert(
        "VCP+EF".to_string(),
        vec!["VCP".to_string(), "EF".to_string()],
    );
    map.insert(
        "VCN+EF".to_string(),
        vec!["VCN".to_string(), "EF".to_string()],
    );

    // EP+EF (선어말어미+종결어미 복합) - "입니다" 같은 패턴
    map.insert(
        "EP+EF".to_string(),
        vec!["VCP".to_string(), "EF".to_string()],
    );

    // EP+EP (존칭+과거 복합) - "셨" = "시/EP + 었/EP"
    map.insert(
        "EP+EP".to_string(),
        vec!["EP".to_string(), "EP".to_string()],
    );

    // 체언 + 격조사
    map.insert(
        "NNG+JKS".to_string(),
        vec!["NNG".to_string(), "JKS".to_string()],
    );
    map.insert(
        "NNG+JKC".to_string(),
        vec!["NNG".to_string(), "JKC".to_string()],
    );
    map.insert(
        "NNG+JKG".to_string(),
        vec!["NNG".to_string(), "JKG".to_string()],
    );
    map.insert(
        "NNG+JKO".to_string(),
        vec!["NNG".to_string(), "JKO".to_string()],
    );
    map.insert(
        "NNG+JKB".to_string(),
        vec!["NNG".to_string(), "JKB".to_string()],
    );
    map.insert(
        "NNG+JKV".to_string(),
        vec!["NNG".to_string(), "JKV".to_string()],
    );
    map.insert(
        "NNG+JKQ".to_string(),
        vec!["NNG".to_string(), "JKQ".to_string()],
    );
    map.insert(
        "NNP+JKS".to_string(),
        vec!["NNP".to_string(), "JKS".to_string()],
    );
    map.insert(
        "NNP+JKO".to_string(),
        vec!["NNP".to_string(), "JKO".to_string()],
    );
    map.insert(
        "NNP+JKB".to_string(),
        vec!["NNP".to_string(), "JKB".to_string()],
    );
    map.insert(
        "NP+JKS".to_string(),
        vec!["NP".to_string(), "JKS".to_string()],
    );
    map.insert(
        "NP+JKO".to_string(),
        vec!["NP".to_string(), "JKO".to_string()],
    );
    map.insert(
        "NP+JKB".to_string(),
        vec!["NP".to_string(), "JKB".to_string()],
    );

    // 체언 + 보조사
    map.insert(
        "NNG+JX".to_string(),
        vec!["NNG".to_string(), "JX".to_string()],
    );
    map.insert(
        "NNP+JX".to_string(),
        vec!["NNP".to_string(), "JX".to_string()],
    );
    map.insert(
        "NP+JX".to_string(),
        vec!["NP".to_string(), "JX".to_string()],
    );

    // 체언 + 접속조사
    map.insert(
        "NNG+JC".to_string(),
        vec!["NNG".to_string(), "JC".to_string()],
    );

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tag_map_verb_endings() {
        let map = init_tag_map();
        assert_eq!(
            map.get("VV+EF"),
            Some(&vec!["VV".to_string(), "EF".to_string()])
        );
        assert_eq!(
            map.get("VV+EC"),
            Some(&vec!["VV".to_string(), "EC".to_string()])
        );
        assert_eq!(
            map.get("VV+ETM"),
            Some(&vec!["VV".to_string(), "ETM".to_string()])
        );
    }

    #[test]
    fn test_init_tag_map_three_way_splits() {
        let map = init_tag_map();
        assert_eq!(
            map.get("VV+EP+EF"),
            Some(&vec!["VV".to_string(), "EP".to_string(), "EF".to_string()])
        );
        assert_eq!(
            map.get("VA+EP+EF"),
            Some(&vec!["VA".to_string(), "EP".to_string(), "EF".to_string()])
        );
    }

    #[test]
    fn test_init_tag_map_noun_particle_entries() {
        let map = init_tag_map();
        assert_eq!(
            map.get("NNG+JKS"),
            Some(&vec!["NNG".to_string(), "JKS".to_string()])
        );
        assert_eq!(
            map.get("NNG+JX"),
            Some(&vec!["NNG".to_string(), "JX".to_string()])
        );
        // 알 수 없는 키는 None
        assert!(map.get("UNKNOWN+TAG").is_none());
    }
}
