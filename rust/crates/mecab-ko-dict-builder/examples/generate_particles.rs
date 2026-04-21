//! 조사 결합형 생성 예제
//!
//! 고빈도 명사 + 조사 결합형을 `MeCab` Inflect.csv 형식으로 생성합니다.
//!
//! # 실행 방법
//! ```bash
//! cargo run --example generate_particles > generated_particles.csv
//! ```

use mecab_ko_hangul::has_jongseong;

/// 고빈도 명사 목록 (세종 코퍼스 기반 상위 100)
const HIGH_FREQ_NOUNS: &[&str] = &[
    // 대명사
    "나",
    "너",
    "저",
    "우리",
    "그",
    "이것",
    "저것",
    "그것",
    "누구",
    "무엇",
    // 고빈도 일반명사
    "사람",
    "일",
    "말",
    "시간",
    "년",
    "집",
    "물",
    "문제",
    "생각",
    "사회",
    "나라",
    "세계",
    "학교",
    "가족",
    "어머니",
    "아버지",
    "친구",
    "여자",
    "남자",
    "아이",
    "눈",
    "손",
    "마음",
    "정신",
    "힘",
    "돈",
    "책",
    "영화",
    "음식",
    "음악",
    "역사",
    "과학",
    "경제",
    "정치",
    "문화",
    "교육",
    "환경",
    "기술",
    "언어",
    "예술",
    "자연",
    "건강",
    "사랑",
    "행복",
    "진실",
    "평화",
    "자유",
    "희망",
    "미래",
    "과거",
    "현재",
    "오늘",
    "내일",
    "어제",
    "아침",
    "저녁",
    "밤",
    "낮",
    "봄",
    "여름",
    "가을",
    "겨울",
    "날씨",
    "하늘",
    "바다",
    "산",
    "강",
    "도시",
    "마을",
    "길",
    "차",
    "버스",
    "지하철",
    "비행기",
    "배",
    "회사",
    "직장",
    "학생",
    "선생님",
    "의사",
    "부모",
    "형제",
    "동생",
    "언니",
    "오빠",
    "누나",
    "할머니",
    "할아버지",
    "아들",
    "딸",
];

/// 조사 목록
struct Particle {
    /// 받침 있는 명사 뒤
    after_jong: &'static str,
    /// 받침 없는 명사 뒤
    after_no_jong: &'static str,
    /// 품사 태그
    pos: &'static str,
}

const PARTICLES: &[Particle] = &[
    // 주격조사
    Particle {
        after_jong: "이",
        after_no_jong: "가",
        pos: "JKS",
    },
    // 목적격조사
    Particle {
        after_jong: "을",
        after_no_jong: "를",
        pos: "JKO",
    },
    // 보조사
    Particle {
        after_jong: "은",
        after_no_jong: "는",
        pos: "JX",
    },
    Particle {
        after_jong: "도",
        after_no_jong: "도",
        pos: "JX",
    },
    Particle {
        after_jong: "만",
        after_no_jong: "만",
        pos: "JX",
    },
    // 부사격조사
    Particle {
        after_jong: "에",
        after_no_jong: "에",
        pos: "JKB",
    },
    Particle {
        after_jong: "에서",
        after_no_jong: "에서",
        pos: "JKB",
    },
    Particle {
        after_jong: "으로",
        after_no_jong: "로",
        pos: "JKB",
    },
    // 관형격조사
    Particle {
        after_jong: "의",
        after_no_jong: "의",
        pos: "JKG",
    },
    // 접속조사
    Particle {
        after_jong: "과",
        after_no_jong: "와",
        pos: "JC",
    },
];

fn main() {
    let cost = -500;

    for noun in HIGH_FREQ_NOUNS {
        let last_char = noun.chars().last().unwrap();
        let has_jong = has_jongseong(last_char).unwrap_or(false);

        for particle in PARTICLES {
            let particle_form = if has_jong {
                particle.after_jong
            } else {
                particle.after_no_jong
            };

            let surface = format!("{noun}{particle_form}");
            let compound_pos = format!("NNG+{}", particle.pos);
            let analysis = format!("{noun}/NNG/*+{particle_form}/{}/*", particle.pos);
            let jong = if surface
                .chars()
                .last()
                .is_some_and(|c| has_jongseong(c).unwrap_or(false))
            {
                "T"
            } else {
                "F"
            };

            println!(
                "{surface},0,0,{cost},{compound_pos},*,{jong},{surface},Inflect,NNG,{},{analysis}",
                particle.pos
            );
        }
    }

    // 대명사 + 조사 특수 처리
    let pronouns: &[(&str, &str)] = &[
        ("나", "NP"),
        ("너", "NP"),
        ("저", "NP"),
        ("우리", "NP"),
        ("그", "NP"),
    ];

    for (pronoun, pos) in pronouns {
        let last_char = pronoun.chars().last().unwrap();
        let has_jong = has_jongseong(last_char).unwrap_or(false);

        for particle in PARTICLES {
            let particle_form = if has_jong {
                particle.after_jong
            } else {
                particle.after_no_jong
            };

            let surface = format!("{pronoun}{particle_form}");
            let compound_pos = format!("{pos}+{}", particle.pos);
            let analysis = format!("{pronoun}/{pos}/*+{particle_form}/{}/*", particle.pos);
            let jong = if surface
                .chars()
                .last()
                .is_some_and(|c| has_jongseong(c).unwrap_or(false))
            {
                "T"
            } else {
                "F"
            };

            // 대명사는 더 낮은 비용 (우선순위 높음)
            let np_cost = -600;
            println!(
                "{surface},0,0,{np_cost},{compound_pos},*,{jong},{surface},Inflect,{pos},{},{analysis}",
                particle.pos
            );
        }
    }
}
