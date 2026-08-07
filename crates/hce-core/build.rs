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

        assert_eq!(
            onset_size, 35,
            "pack '{}': render ONSET_RADIX must be 35, got {}",
            name, onset_size
        );
        assert_eq!(
            vnuc_size, 20,
            "pack '{}': render VNUC_RADIX must be 20, got {}",
            name, vnuc_size
        );
        assert_eq!(
            singles.len(),
            15,
            "pack '{}': render must have exactly 15 single consonants",
            name
        );
        assert_eq!(
            coda.len(),
            3,
            "pack '{}': render must have exactly 3 coda chars",
            name
        );
        assert_eq!(
            vowels.len(),
            5,
            "pack '{}': render must have exactly 5 vowels",
            name
        );

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

    let parser_path = Path::new(&out_dir).join("parser_tables.rs");
    let mut pf = BufWriter::new(File::create(&parser_path).unwrap());

    writeln!(pf, "pub mod parser_tables {{").unwrap();

    for entry in fs::read_dir(&packs_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let name = path.file_stem().unwrap().to_str().unwrap();
        let data: serde_json::Value = serde_json::from_reader(File::open(&path).unwrap()).unwrap();

        let parser_singles: Vec<String> = data["single_consonants"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let parser_clusters: Vec<String> = data["parser_clusters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let parser_vowels: Vec<String> = data["vowels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let parser_coda: Vec<String> = data["coda"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();

        let singles_chars: Vec<String> = parser_singles
            .iter()
            .map(|s| format!("'{}'", s.chars().next().unwrap()))
            .collect();
        let clusters_strs: Vec<String> = parser_clusters
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect();
        let vowels_chars: Vec<String> = parser_vowels
            .iter()
            .map(|s| format!("'{}'", s.chars().next().unwrap()))
            .collect();
        let coda_chars: Vec<String> = parser_coda
            .iter()
            .map(|s| format!("'{}'", s.chars().next().unwrap()))
            .collect();

        let onset_size = parser_singles.len() + parser_clusters.len();
        let vnuc_size = parser_vowels.len() + parser_vowels.len() * parser_coda.len();

        assert_eq!(
            onset_size, 35,
            "pack '{}': ONSET_RADIX must be 35, got {}",
            name, onset_size
        );
        assert_eq!(
            vnuc_size, 20,
            "pack '{}': VNUC_RADIX must be 20, got {}",
            name, vnuc_size
        );
        assert_eq!(
            parser_singles.len(),
            15,
            "pack '{}': must have exactly 15 single consonants",
            name
        );
        assert_eq!(
            parser_coda.len(),
            3,
            "pack '{}': must have exactly 3 coda characters",
            name
        );
        assert_eq!(
            parser_vowels.len(),
            5,
            "pack '{}': must have exactly 5 vowels",
            name
        );

        for cl in &parser_clusters {
            let first = cl.chars().next().unwrap();
            assert!(
                parser_singles.iter().any(|s| s.starts_with(first)),
                "pack '{}': cluster '{}' first char '{}' must be in single_consonants",
                name,
                cl,
                first
            );
        }

        writeln!(pf, "pub mod {} {{", name).unwrap();
        writeln!(
            pf,
            "    pub static ONSET_SINGLE: [char; {}] = [{}];",
            parser_singles.len(),
            singles_chars.join(", ")
        )
        .unwrap();
        writeln!(
            pf,
            "    pub static PARSER_CLUSTERS: [&str; {}] = [{}];",
            parser_clusters.len(),
            clusters_strs.join(", ")
        )
        .unwrap();
        writeln!(
            pf,
            "    pub static VOWELS: [char; {}] = [{}];",
            parser_vowels.len(),
            vowels_chars.join(", ")
        )
        .unwrap();
        writeln!(
            pf,
            "    pub static CODA: [char; {}] = [{}];",
            parser_coda.len(),
            coda_chars.join(", ")
        )
        .unwrap();
        writeln!(pf, "}}").unwrap();
    }

    writeln!(pf, "}}").unwrap();

    let first_pack = fs::read_dir(&packs_dir)
        .unwrap()
        .find_map(|e| {
            let p = e.unwrap().path();
            (p.extension().is_some_and(|e| e == "json")).then_some(p)
        })
        .unwrap();
    let data: serde_json::Value =
        serde_json::from_reader(File::open(&first_pack).unwrap()).unwrap();

    let json_singles: Vec<String> = data["single_consonants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let json_clusters: Vec<String> = data["clusters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let json_vowels: Vec<String> = data["vowels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let json_coda: Vec<String> = data["coda"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let radix_path = Path::new(&out_dir).join("radix_constants.rs");
    let mut rf = BufWriter::new(File::create(&radix_path).unwrap());
    writeln!(
        rf,
        "pub const ONSET_RADIX: u8 = {};",
        json_singles.len() + json_clusters.len()
    )
    .unwrap();
    writeln!(
        rf,
        "pub const VNUC_RADIX: u8 = {};",
        json_vowels.len() + json_vowels.len() * json_coda.len()
    )
    .unwrap();
}
