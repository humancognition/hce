use hce_core::{Hce, HceMode, LanguageLevel};
use serde::Deserialize;

#[derive(Deserialize)]
struct KatEntry {
    index: usize,
    uuid: String,
    #[allow(dead_code)]
    key_hex: String,
    level: String,
    mode: String,
    hce: String,
}

#[derive(Deserialize)]
struct KatFile {
    vectors: Vec<KatEntry>,
}

fn parse_uuid_hex(s: &str) -> [u8; 16] {
    let clean: String = s.chars().filter(|c| *c != '-').collect();
    let mut bytes = [0u8; 16];
    for i in 0..16 {
        let hex = &clean[i * 2..i * 2 + 2];
        bytes[i] = u8::from_str_radix(hex, 16).unwrap();
    }
    bytes
}

fn level_from_str(s: &str) -> LanguageLevel {
    match s {
        "eu" => LanguageLevel::Eu,
        "en" => LanguageLevel::En,
        "numeric" => LanguageLevel::Numeric,
        _ => LanguageLevel::Universal,
    }
}

fn test_kat_file(filename: &str) {
    let key = b"hce-kat-standard-key-32-bytes!!";
    let path = format!(
        "{}/../../shared/kat/{}",
        env!("CARGO_MANIFEST_DIR"),
        filename
    );
    let data =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("KAT file not found: {}", path));
    let kat: KatFile = serde_json::from_str(&data).expect("invalid KAT JSON");

    let mut counts = [0usize; 3];

    for entry in &kat.vectors {
        let uuid_bytes = parse_uuid_hex(&entry.uuid);
        let level = level_from_str(&entry.level);

        match entry.mode.as_str() {
            "sealed" => {
                let hce = Hce::new(Some(key), level, HceMode::Sealed);
                let encoded = hce.encode(&uuid_bytes);
                assert_eq!(
                    encoded, entry.hce,
                    "sealed mismatch at index {} (level {})",
                    entry.index, entry.level
                );
                let decoded = hce.decode(&encoded).unwrap();
                assert_eq!(
                    decoded,
                    uuid_bytes.to_vec(),
                    "sealed decode mismatch at index {}",
                    entry.index
                );
                counts[0] += 1;
            }
            "open" => {
                let hce = Hce::new(Some(key), level, HceMode::Open);
                let encoded = hce.encode(&uuid_bytes);
                assert_eq!(
                    encoded, entry.hce,
                    "open mismatch at index {} (level {})",
                    entry.index, entry.level
                );
                let decoded = hce.decode(&encoded).unwrap();
                assert_eq!(
                    decoded,
                    uuid_bytes.to_vec(),
                    "open decode mismatch at index {}",
                    entry.index
                );
                counts[1] += 1;
            }
            "plain" => {
                let hce = Hce::new(None, level, HceMode::Plain);
                let encoded = hce.encode(&uuid_bytes);
                assert_eq!(
                    encoded, entry.hce,
                    "plain mismatch at index {} (level {})",
                    entry.index, entry.level
                );
                let decoded = hce.decode(&encoded).unwrap();
                assert_eq!(
                    decoded,
                    uuid_bytes.to_vec(),
                    "plain decode mismatch at index {}",
                    entry.index
                );
                counts[2] += 1;
            }
            _ => panic!("unknown mode: {}", entry.mode),
        }
    }

    eprintln!(
        "  {}: {} sealed, {} open, {} plain — OK",
        filename, counts[0], counts[1], counts[2]
    );
}

#[test]
fn kat_universal() {
    test_kat_file("universal.json");
}

#[test]
fn kat_eu() {
    test_kat_file("eu.json");
}

#[test]
fn kat_en() {
    test_kat_file("en.json");
}

#[test]
fn kat_numeric() {
    test_kat_file("numeric.json");
}
