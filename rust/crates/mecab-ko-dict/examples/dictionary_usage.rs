//! Dictionary usage examples
//!
//! This example demonstrates how to use the `SystemDictionary` and `UserDictionary`.

use mecab_ko_dict::{DictionaryLoader, Matrix, UserDictionaryBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MeCab-Ko Dictionary Example ===\n");

    // 1. Try to load system dictionary
    println!("1. Loading system dictionary...");
    match DictionaryLoader::find_dicdir() {
        Ok(dicdir) => {
            println!("   Found dictionary at: {}", dicdir.display());

            match DictionaryLoader::load_system(&dicdir) {
                Ok(dict) => {
                    println!("   Successfully loaded system dictionary");
                    println!(
                        "   Matrix size: {} x {}",
                        dict.matrix().left_size(),
                        dict.matrix().right_size()
                    );
                    println!("   Entry count: {}", dict.entry_count());
                }
                Err(e) => {
                    println!("   Failed to load dictionary: {e}");
                }
            }
        }
        Err(e) => {
            println!("   {e}");
            println!("   This is expected if mecab-ko-dic is not installed.");
        }
    }

    // 2. Create user dictionary
    println!("\n2. Creating user dictionary...");
    let user_dict = UserDictionaryBuilder::new()
        .default_cost(-1000)
        .add("딥러닝", "NNG")
        .add("머신러닝", "NNG")
        .add_with_cost("자연어처리", "NNG", -800)
        .add_full("챗GPT", "NNP", -1000, Some("챗지피티"))
        .add_full("클로드", "NNP", -1000, Some("클로드"))
        .build_with_trie()?;

    println!(
        "   Created user dictionary with {} entries",
        user_dict.len()
    );

    // 3. Lookup entries
    println!("\n3. Looking up entries...");
    for word in &["딥러닝", "챗GPT", "클로드"] {
        let entries = user_dict.lookup(word);
        println!("   '{}': {} entries found", word, entries.len());
        for entry in entries {
            println!(
                "      - POS: {}, Cost: {}, Reading: {:?}",
                entry.pos, entry.cost, entry.reading
            );
        }
    }

    // 4. Common prefix search
    println!("\n4. Common prefix search...");
    if let Some(trie) = user_dict.get_trie() {
        let text = "딥러닝모델을사용한자연어처리시스템";
        let results: Vec<_> = trie.common_prefix_search(text).collect();
        println!("   Text: '{text}'");
        println!("   Found {} prefix matches:", results.len());
        for (value, byte_len) in results {
            let matched = &text[..byte_len];
            println!("      - '{matched}' (value: {value}, bytes: {byte_len})");
        }
    }

    // 5. Load from CSV
    println!("\n5. Loading from CSV...");
    let csv = r"
# AI 용어 사전
GPT,NNP,-1000,지피티
LLM,NNP,-1000,엘엘엠
트랜스포머,NNG,-800,트랜스포머
어텐션,NNG,-700,어텐션
";

    let csv_dict = UserDictionaryBuilder::new()
        .load_str(csv)?
        .build_with_trie()?;

    println!("   Loaded {} entries from CSV", csv_dict.len());
    for word in &["GPT", "LLM", "트랜스포머"] {
        let entries = csv_dict.lookup(word);
        if !entries.is_empty() {
            println!(
                "   '{}': POS={}, Cost={}",
                word, entries[0].pos, entries[0].cost
            );
        }
    }

    println!("\n=== Example completed successfully ===");

    Ok(())
}
