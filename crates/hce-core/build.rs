use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("packs.rs");
    let packs_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("packs");

    let mut f = BufWriter::new(File::create(&dest).unwrap());

    writeln!(f).unwrap();

    for entry in fs::read_dir(&packs_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let name = path.file_stem().unwrap().to_str().unwrap();
        let json: serde_json::Value = serde_json::from_reader(File::open(&path).unwrap()).unwrap();

        let singles: Vec<String> = json["single_consonants"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let clusters: Vec<String> = json["clusters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let vowels: Vec<String> = json["vowels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let coda: Vec<String> = json["coda"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();

        let singles_str: Vec<String> = singles.iter().map(|s| format!("\"{}\"", s)).collect();
        let clusters_str: Vec<String> = clusters.iter().map(|s| format!("\"{}\"", s)).collect();

        let onset_size = singles.len() + clusters.len();
        let vnuc_size = vowels.len() + vowels.len() * coda.len();

        let vnuc_display: Vec<String> = {
            let mut v = Vec::new();
            for vw in &vowels {
                v.push(format!("\"{}\"", vw));
            }
            for vw in &vowels {
                for cd in &coda {
                    v.push(format!("\"{}{}\"", vw, cd));
                }
            }
            v
        };

        writeln!(f, "#[allow(dead_code)]").unwrap();
        writeln!(f, "pub mod {} {{", name).unwrap();
        writeln!(f, "    pub const ONSET_RADIX: u8 = {};", onset_size).unwrap();
        writeln!(f, "    pub const VNUC_RADIX: u8 = {};", vnuc_size).unwrap();
        writeln!(
            f,
            "    pub const ONSET: [&str; {}] = [{}];",
            onset_size,
            singles_str
                .iter()
                .chain(clusters_str.iter())
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
        writeln!(
            f,
            "    pub const VNUC: [&str; {}] = [{}];",
            vnuc_size,
            vnuc_display.join(", ")
        )
        .unwrap();
        writeln!(
            f,
            "    pub const SINGLES: [char; {}] = [{}];",
            singles.len(),
            singles
                .iter()
                .map(|s| format!("'{}'", s.chars().next().unwrap()))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
        writeln!(
            f,
            "    pub const CLUSTERS: [&str; {}] = [{}];",
            clusters.len(),
            clusters_str.join(", ")
        )
        .unwrap();
        writeln!(
            f,
            "    pub const VOWELS: [char; {}] = [{}];",
            vowels.len(),
            vowels
                .iter()
                .map(|s| format!("'{}'", s.chars().next().unwrap()))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
        writeln!(
            f,
            "    pub const CODA: [char; {}] = [{}];",
            coda.len(),
            coda.iter()
                .map(|s| format!("'{}'", s.chars().next().unwrap()))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
        writeln!(f, "}}").unwrap();
        writeln!(f).unwrap();
    }

    println!("cargo:rerun-if-changed=packs/");
}
