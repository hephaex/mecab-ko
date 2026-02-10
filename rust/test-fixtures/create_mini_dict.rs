//! Test fixture generator for minimal MeCab-Ko dictionary
//!
//! This utility creates a minimal test dictionary with common Korean words
//! for integration testing without requiring a full system dictionary.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::{LittleEndian, WriteBytesExt};

/// Creates a minimal test dictionary with common Korean words
///
/// # Errors
///
/// Returns an error if file writing fails
pub fn create_mini_dict<P: AsRef<Path>>(dict_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    let dict_dir = dict_dir.as_ref();
    std::fs::create_dir_all(dict_dir)?;

    // Create entries.csv with common Korean words
    create_entries_csv(dict_dir)?;

    // Create sys.dic (Trie) with the same words
    create_trie(dict_dir)?;

    // Create matrix.bin with minimal connection costs
    create_matrix(dict_dir)?;

    Ok(())
}

/// Creates entries.csv with common Korean words
fn create_entries_csv<P: AsRef<Path>>(dict_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    let path = dict_dir.as_ref().join("entries.csv");
    let mut file = File::create(path)?;

    // Format: surface,left_id,right_id,cost,feature
    // Common Korean words with proper POS tags
    let entries = [
        // Greetings
        ("안녕", 1, 1, 100, "NNG,*,T,안녕,*,*,*,*"),
        ("하", 2, 2, 50, "XSV,*,F,하,*,*,*,*"),
        ("세요", 3, 3, 50, "EP+EF,*,F,세요,*,*,*,*"),
        // Thank you
        ("감사", 4, 4, 100, "NNG,*,F,감사,*,*,*,*"),
        ("합니다", 5, 5, 50, "XSV+EF,*,F,합니다,*,*,*,*"),
        // Common nouns
        ("한국어", 6, 6, 150, "NNG,*,F,한국어,*,*,*,*"),
        ("사람", 7, 7, 100, "NNG,*,T,사람,*,*,*,*"),
        ("시간", 8, 8, 100, "NNG,*,T,시간,*,*,*,*"),
        ("책", 9, 9, 80, "NNG,*,T,책,*,*,*,*"),
        // Common verbs
        ("가", 10, 10, 80, "VV,*,F,가,*,*,*,*"),
        ("다", 11, 11, 30, "EF,*,F,다,*,*,*,*"),
        ("먹", 12, 12, 80, "VV,*,T,먹,*,*,*,*"),
        ("었", 13, 13, 40, "EP,*,T,었,*,*,*,*"),
        // Particles
        ("은", 14, 14, 40, "JX,*,T,은,*,*,*,*"),
        ("는", 15, 15, 40, "JX,*,T,는,*,*,*,*"),
        ("을", 16, 16, 40, "JKO,*,T,을,*,*,*,*"),
        ("를", 17, 17, 40, "JKO,*,T,를,*,*,*,*"),
        ("이", 18, 18, 40, "JKS,*,F,이,*,*,*,*"),
        ("가", 19, 19, 40, "JKS,*,F,가,*,*,*,*"),
        // Pronouns
        ("나", 20, 20, 100, "NP,*,F,나,*,*,*,*"),
        ("너", 21, 21, 100, "NP,*,F,너,*,*,*,*"),
    ];

    for (surface, left_id, right_id, cost, feature) in &entries {
        writeln!(file, "{},{},{},{},{}", surface, left_id, right_id, cost, feature)?;
    }

    Ok(())
}

/// Creates sys.dic (Trie) with dictionary entries
fn create_trie<P: AsRef<Path>>(dict_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    // This requires the yada crate for building the trie
    // We'll create a simple trie with the same entries
    use yada::builder::DoubleArrayBuilder;

    // The trie value should be the index in the entries.csv file (0-based)
    // So the first entry in CSV (안녕) has index 0, second (하) has index 1, etc.
    let mut entries: Vec<(&[u8], u32)> = vec![
        ("안녕".as_bytes(), 0u32),
        ("하".as_bytes(), 1u32),
        ("세요".as_bytes(), 2u32),
        ("감사".as_bytes(), 3u32),
        ("합니다".as_bytes(), 4u32),
        ("한국어".as_bytes(), 5u32),
        ("사람".as_bytes(), 6u32),
        ("시간".as_bytes(), 7u32),
        ("책".as_bytes(), 8u32),
        ("가".as_bytes(), 9u32), // verb form
        ("다".as_bytes(), 10u32),
        ("먹".as_bytes(), 11u32),
        ("었".as_bytes(), 12u32),
        ("은".as_bytes(), 13u32),
        ("는".as_bytes(), 14u32),
        ("을".as_bytes(), 15u32),
        ("를".as_bytes(), 16u32),
        ("이".as_bytes(), 17u32),
        // ("가".as_bytes(), 18u32), // particle - duplicate key, skip for now
        ("나".as_bytes(), 19u32),
        ("너".as_bytes(), 20u32),
    ];

    // Sort entries by key (required by yada)
    entries.sort_by(|a, b| a.0.cmp(b.0));

    // Build trie using yada API
    let trie_bytes = DoubleArrayBuilder::build(&entries)
        .ok_or("Failed to build trie")?;

    let path = dict_dir.as_ref().join("sys.dic");
    let mut file = File::create(path)?;
    file.write_all(&trie_bytes)?;

    Ok(())
}

/// Creates matrix.bin with minimal connection costs
fn create_matrix<P: AsRef<Path>>(dict_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    let path = dict_dir.as_ref().join("matrix.bin");
    let mut file = File::create(path)?;

    // Matrix dimensions
    let lsize: u16 = 25; // left context IDs (0-24)
    let rsize: u16 = 25; // right context IDs (0-24)

    // Write header
    file.write_u16::<LittleEndian>(lsize)?;
    file.write_u16::<LittleEndian>(rsize)?;

    // Write connection costs (all initialized to a default cost)
    // Lower costs mean better connections
    let default_cost: i16 = 100;

    for _left_id in 0..lsize {
        for _right_id in 0..rsize {
            file.write_i16::<LittleEndian>(default_cost)?;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get output directory from command line arg, or use default
    let args: Vec<String> = std::env::args().collect();
    let output_dir = if args.len() > 1 {
        &args[1]
    } else {
        "mini-dict"
    };

    println!("Creating minimal test dictionary in: {}", output_dir);
    create_mini_dict(output_dir)?;
    println!("Success! Dictionary created at {}", output_dir);
    println!("Files:");
    println!("  - entries.csv");
    println!("  - sys.dic");
    println!("  - matrix.bin");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mini_dict() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let result = create_mini_dict(temp_dir.path());
        assert!(result.is_ok(), "Failed to create mini dict: {:?}", result);

        // Verify files exist
        assert!(temp_dir.path().join("entries.csv").exists());
        assert!(temp_dir.path().join("sys.dic").exists());
        assert!(temp_dir.path().join("matrix.bin").exists());
    }
}
