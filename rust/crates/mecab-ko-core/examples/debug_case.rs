use mecab_ko_core::evaluate::TestDataset;
use mecab_ko_core::sejong::SejongConverter;
use mecab_ko_core::tokenizer::Tokenizer;

fn main() {
    let dict_path = "../../../data/mecab-ko-dic-2.1.1-20180720";
    let mut tokenizer = Tokenizer::with_dict(dict_path).expect("tokenizer");
    let converter = SejongConverter::new();
    let dataset = TestDataset::from_tsv("../../../data/eval/sample.tsv").expect("dataset");

    // NR 오류 분석
    let mut errors = Vec::new();
    for sentence in &dataset.sentences {
        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);

        for (i, gold) in sentence.tokens.iter().enumerate() {
            if gold.pos == "NR" {
                let pred = if i < sejong_tokens.len() {
                    format!("{}/{}", sejong_tokens[i].surface, sejong_tokens[i].pos)
                } else {
                    "MISSING".to_string()
                };

                if !pred.ends_with("/NR") {
                    errors.push((sentence.text.clone(), gold.surface.clone(), pred));
                }
            }
        }
    }

    println!("NR 오류 {}개:\n", errors.len());
    for (text, gold, pred) in &errors {
        println!("{text}: {gold} → {pred}");
    }
}
