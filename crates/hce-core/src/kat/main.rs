use hce_core::{
    compute_syll_len, Hce, HceMode, LanguageLevel, TsGranularity, ONSET_RADIX, VNUC_RADIX,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize)]
struct KatEntry {
    index: usize,
    uuid: String,
    key_hex: String,
    level: String,
    mode: String,
    hce: String,
}

#[derive(Serialize)]
struct KatFile {
    version: u32,
    onset_radix: u8,
    vnuc_radix: u8,
    syll_len: usize,
    check_syll: usize,
    level: String,
    vectors: Vec<KatEntry>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let count: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(250);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(42);
    let output_dir: PathBuf = args
        .get(3)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("shared/kat"));

    let key = b"hce-kat-standard-key-32-bytes!!";
    let key_hex: String = key.iter().map(|b| format!("{:02x}", b)).collect();

    let levels = [
        (LanguageLevel::Universal, "universal"),
        (LanguageLevel::Eu, "eu"),
        (LanguageLevel::En, "en"),
        (LanguageLevel::Numeric, "numeric"),
    ];

    let modes = [
        (HceMode::Sealed, "sealed"),
        (HceMode::Open, "open"),
        (HceMode::Plain, "plain"),
    ];

    let mut rng = StdRng::seed_from_u64(seed);

    for &(level, level_name) in &levels {
        let mut entries = Vec::with_capacity(count);
        let hce = |mode: HceMode| -> Hce {
            match mode {
                HceMode::Plain => Hce::new(None, level, mode),
                _ => Hce::new(Some(key), level, mode)
                    .with_timestamp_config(1_700_000_000_000, TsGranularity::Month),
            }
        };

        let sealed = hce(HceMode::Sealed);
        let open = hce(HceMode::Open);
        let plain = hce(HceMode::Plain);

        for i in 0..count {
            let mut uuid_bytes = [0u8; 16];
            rng.fill_bytes(&mut uuid_bytes);
            uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x70;
            uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80;

            let uuid_hex: String = uuid_bytes.iter().map(|b| format!("{:02x}", b)).collect();

            let (mode, mode_str) = modes[i % 3];

            let hce_str = match mode {
                HceMode::Sealed => sealed.encode(&uuid_bytes),
                HceMode::Open => open.encode(&uuid_bytes),
                HceMode::Plain => plain.encode(&uuid_bytes),
            };

            entries.push(KatEntry {
                index: i,
                uuid: uuid_hex,
                key_hex: key_hex.clone(),
                level: level_name.to_string(),
                mode: mode_str.to_string(),
                hce: hce_str,
            });
        }

        let kat = KatFile {
            version: 1,
            onset_radix: ONSET_RADIX,
            vnuc_radix: VNUC_RADIX,
            syll_len: compute_syll_len(128),
            check_syll: 1,
            level: level_name.to_string(),
            vectors: entries,
        };

        fs::create_dir_all(&output_dir).ok();
        let output = output_dir.join(format!("{}.json", level_name));
        let json = serde_json::to_string_pretty(&kat).unwrap();
        let mut f = fs::File::create(&output).unwrap();
        f.write_all(json.as_bytes()).unwrap();
        f.write_all(b"\n").unwrap();

        eprintln!("  {}: {} vectors → {}", level_name, count, output.display());
    }

    eprintln!("Done — 4 levels × {} = {} total vectors", count, count * 4);
}
