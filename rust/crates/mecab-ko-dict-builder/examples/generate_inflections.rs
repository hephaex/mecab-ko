//! 동사/형용사 활용형 생성 예제
//!
//! 고빈도 동사 목록을 기반으로 Inflect.csv 형식의 활용형을 생성합니다.
//!
//! # 실행 방법
//! ```bash
//! cargo run --example generate_inflections > generated_inflections.csv
//! ```

use mecab_ko_dict_builder::inflect_gen::{InflectGenerator, IrregularType, VerbEntry};

/// 고빈도 동사 목록 (세종 코퍼스 기반 상위 100)
const HIGH_FREQ_VERBS: &[(&str, &str, IrregularType)] = &[
    // 규칙 동사
    ("가", "VV", IrregularType::Regular),
    ("오", "VV", IrregularType::Regular),
    ("보", "VV", IrregularType::Regular),
    ("주", "VV", IrregularType::Regular),
    ("받", "VV", IrregularType::Regular),
    ("먹", "VV", IrregularType::Regular),
    ("자", "VV", IrregularType::Regular),
    ("읽", "VV", IrregularType::Regular),
    ("쓰", "VV", IrregularType::Regular),
    ("찾", "VV", IrregularType::Regular),
    ("잡", "VV", IrregularType::Regular),
    ("타", "VV", IrregularType::Regular),
    ("내", "VV", IrregularType::Regular),
    ("넣", "VV", IrregularType::Regular),
    ("빼", "VV", IrregularType::Regular),
    ("차", "VV", IrregularType::Regular),
    ("펴", "VV", IrregularType::Regular),
    ("켜", "VV", IrregularType::Regular),
    ("끄", "VV", IrregularType::Regular),
    ("서", "VV", IrregularType::Regular),
    ("앉", "VV", IrregularType::Regular),
    ("입", "VV", IrregularType::Regular),
    ("벗", "VV", IrregularType::Regular),
    ("씻", "VV", IrregularType::Regular),
    ("닦", "VV", IrregularType::Regular),
    ("던지", "VV", IrregularType::Regular),
    ("잡", "VV", IrregularType::Regular),
    ("놓", "VV", IrregularType::Regular),
    ("두", "VV", IrregularType::Regular),
    ("세우", "VV", IrregularType::Regular),
    // ㄷ 불규칙
    ("듣", "VV", IrregularType::DieutIrregular),
    ("걷", "VV", IrregularType::DieutIrregular),
    ("싣", "VV", IrregularType::DieutIrregular),
    ("묻", "VV", IrregularType::DieutIrregular),
    // ㅂ 불규칙
    ("돕", "VV", IrregularType::BiupIrregular),
    ("눕", "VV", IrregularType::BiupIrregular),
    ("굽", "VV", IrregularType::BiupIrregular),
    ("줍", "VV", IrregularType::BiupIrregular),
    ("덥", "VA", IrregularType::BiupIrregular),
    ("춥", "VA", IrregularType::BiupIrregular),
    ("가깝", "VA", IrregularType::BiupIrregular),
    ("어렵", "VA", IrregularType::BiupIrregular),
    ("쉽", "VA", IrregularType::BiupIrregular),
    ("무겁", "VA", IrregularType::BiupIrregular),
    ("가볍", "VA", IrregularType::BiupIrregular),
    ("아름답", "VA", IrregularType::BiupIrregular),
    ("고맙", "VA", IrregularType::BiupIrregular),
    // ㄹ 불규칙
    ("살", "VV", IrregularType::LieulIrregular),
    ("알", "VV", IrregularType::LieulIrregular),
    ("만들", "VV", IrregularType::LieulIrregular),
    ("열", "VV", IrregularType::LieulIrregular),
    ("닫", "VV", IrregularType::LieulIrregular),
    ("팔", "VV", IrregularType::LieulIrregular),
    ("놀", "VV", IrregularType::LieulIrregular),
    ("울", "VV", IrregularType::LieulIrregular),
    ("풀", "VV", IrregularType::LieulIrregular),
    ("길", "VA", IrregularType::LieulIrregular),
    ("멀", "VA", IrregularType::LieulIrregular),
    // 르 불규칙
    ("모르", "VV", IrregularType::ReuIrregular),
    ("부르", "VV", IrregularType::ReuIrregular),
    ("오르", "VV", IrregularType::ReuIrregular),
    ("고르", "VV", IrregularType::ReuIrregular),
    ("흐르", "VV", IrregularType::ReuIrregular),
    ("빠르", "VA", IrregularType::ReuIrregular),
    ("다르", "VA", IrregularType::ReuIrregular),
    // ㅅ 불규칙
    ("낫", "VV", IrregularType::SiotIrregular),
    ("짓", "VV", IrregularType::SiotIrregular),
    ("긋", "VV", IrregularType::SiotIrregular),
    // ㅎ 불규칙
    ("그렇", "VA", IrregularType::HieutIrregular),
    ("이렇", "VA", IrregularType::HieutIrregular),
    ("저렇", "VA", IrregularType::HieutIrregular),
    ("어떻", "VA", IrregularType::HieutIrregular),
    ("빨갛", "VA", IrregularType::HieutIrregular),
    ("파랗", "VA", IrregularType::HieutIrregular),
    ("노랗", "VA", IrregularType::HieutIrregular),
    ("하얗", "VA", IrregularType::HieutIrregular),
    ("까맣", "VA", IrregularType::HieutIrregular),
    // 하다 동사
    ("하", "VV", IrregularType::HadaSpecial),
    ("되", "VV", IrregularType::Regular),
];

/// 하다 동사 어근 (명사+하다)
const HADA_STEMS: &[&str] = &[
    "공부", "운동", "청소", "요리", "독서", "여행", "사랑", "싸움", "말", "생각", "기억", "시작",
    "끝", "일", "준비", "결정", "선택", "변화", "발전", "노력", "성공", "실패", "참여", "활용",
    "연구", "분석", "검토", "확인", "추가", "삭제", "수정", "저장", "검색", "입력", "출력",
];

fn main() {
    let generator = InflectGenerator::new();

    // 기본 동사 활용형 생성
    for (stem, pos, irregular_type) in HIGH_FREQ_VERBS {
        let verb = VerbEntry {
            stem: stem.to_string(),
            pos: pos.to_string(),
            irregular_type: *irregular_type,
            base_cost: -500, // 기본 비용
        };

        for inflection in generator.generate(&verb) {
            println!("{}", inflection.to_csv_line());
        }
    }

    // 하다 동사 활용형 생성
    for stem in HADA_STEMS {
        let verb = VerbEntry {
            stem: format!("{stem}하"),
            pos: "VV".to_string(),
            irregular_type: IrregularType::HadaSpecial,
            base_cost: -500,
        };

        for inflection in generator.generate(&verb) {
            println!("{}", inflection.to_csv_line());
        }
    }
}
