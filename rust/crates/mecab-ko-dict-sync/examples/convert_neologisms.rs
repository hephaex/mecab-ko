//! Example: Convert NIKL format entries to MeCab-Ko user dictionary format.
//!
//! This example demonstrates how to use the dictionary converter to transform
//! entries from various Korean dictionary formats into MeCab-Ko compatible CSV.

use mecab_ko_dict_sync::{ConverterEntry, DictConverter};

fn main() {
    // Create converter with default POS mappings
    let converter = DictConverter::new();

    // Example entries from various sources
    let entries = vec![
        // Tech/IT terms
        ConverterEntry {
            surface: "챗GPT".to_string(),
            pos: "고유명사".to_string(),
            reading: Some("챗지피티".to_string()),
            frequency: Some(5000),
        },
        ConverterEntry {
            surface: "메타버스".to_string(),
            pos: "명사".to_string(),
            reading: Some("메타버스".to_string()),
            frequency: Some(1200),
        },
        ConverterEntry {
            surface: "생성AI".to_string(),
            pos: "명사".to_string(),
            reading: Some("생성에이아이".to_string()),
            frequency: Some(800),
        },
        // Social/Cultural terms
        ConverterEntry {
            surface: "워라밸".to_string(),
            pos: "명사".to_string(),
            reading: Some("워라밸".to_string()),
            frequency: Some(600),
        },
        ConverterEntry {
            surface: "갓생".to_string(),
            pos: "명사".to_string(),
            reading: Some("갓생".to_string()),
            frequency: Some(400),
        },
        // Internet/SNS terms
        ConverterEntry {
            surface: "밈".to_string(),
            pos: "명사".to_string(),
            reading: Some("밈".to_string()),
            frequency: Some(300),
        },
        ConverterEntry {
            surface: "숏폼".to_string(),
            pos: "명사".to_string(),
            reading: Some("숏폼".to_string()),
            frequency: Some(250),
        },
        // Abbreviations
        ConverterEntry {
            surface: "TMI".to_string(),
            pos: "명사".to_string(),
            reading: Some("티엠아이".to_string()),
            frequency: Some(150),
        },
        // Interjections
        ConverterEntry {
            surface: "ㅋㅋㅋ".to_string(),
            pos: "감탄사".to_string(),
            reading: Some("크크크".to_string()),
            frequency: Some(2000),
        },
    ];

    println!("=== NIKL to MeCab-Ko Dictionary Converter ===\n");
    println!("Converting {} entries...\n", entries.len());

    // Convert all entries to CSV format
    match converter.convert_to_csv(&entries) {
        Ok(csv_lines) => {
            println!("# Generated MeCab-Ko User Dictionary");
            println!("# Format: 표면형,좌ID,우ID,비용,품사,*,*,*,읽기,원형,읽기,*\n");

            for (i, (entry, csv_line)) in entries.iter().zip(csv_lines.iter()).enumerate() {
                println!("# Entry {}: {} ({})", i + 1, entry.surface, entry.pos);
                println!("{csv_line}");

                // Show analysis
                let user_entry = converter.convert_entry(entry).unwrap();
                println!(
                    "#   -> MeCab POS: {}, Cost: {}, Freq: {:?}\n",
                    user_entry.pos, user_entry.cost, entry.frequency
                );
            }

            println!("\n=== Conversion Summary ===");
            println!("Total entries: {}", csv_lines.len());
            println!("Format: MeCab-Ko user dictionary CSV");

            // Count by cost category
            let high_priority = entries
                .iter()
                .filter(|e| e.frequency.unwrap_or(0) >= 1000)
                .count();
            let medium_priority = entries
                .iter()
                .filter(|e| {
                    let freq = e.frequency.unwrap_or(0);
                    freq >= 100 && freq < 1000
                })
                .count();
            let low_priority = entries
                .iter()
                .filter(|e| e.frequency.unwrap_or(0) < 100)
                .count();

            println!("  High priority (freq ≥ 1000): {high_priority}");
            println!("  Medium priority (100-999): {medium_priority}");
            println!("  Low priority (< 100): {low_priority}");

            // Show POS tag distribution
            println!("\n=== POS Tag Distribution ===");
            let mut pos_counts = std::collections::HashMap::new();
            for entry in &entries {
                let mecab_pos = converter.map_pos(&entry.pos).unwrap();
                *pos_counts.entry(mecab_pos).or_insert(0) += 1;
            }

            for (pos, count) in pos_counts {
                println!("  {pos}: {count}");
            }
        }
        Err(e) => {
            eprintln!("Error converting entries: {e}");
        }
    }
}
