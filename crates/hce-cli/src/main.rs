use clap::{Parser, Subcommand, ValueEnum};
use hce_core::{ChunkSpec, Hce, HceCase, HceMode, LanguageLevel, TsGranularity};
use hce_fpe::{build_cipher, CipherKind};
use std::io::{self, Read};

#[derive(Parser)]
#[command(
    name = "hce",
    version,
    about = "Lossless, pronounceable, encrypted codec for any identifier — reversible, format-preserving, human-readable"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Encode {
        #[arg(short, long)]
        key: Option<String>,

        #[arg(short, long, value_enum, default_value_t = LevelArg::Universal)]
        level: LevelArg,

        #[arg(short, long, value_enum, default_value_t = ModeArg::Sealed)]
        mode: ModeArg,

        #[arg(short = 'w', long, default_value = "128")]
        bits: u32,

        #[arg(long)]
        modulus: Option<String>,

        #[arg(long, value_enum)]
        cipher: Option<CipherCliArg>,

        #[arg(long, value_enum)]
        case: Option<CaseArg>,

        #[arg(long, default_value = "1")]
        check_syllables: usize,

        #[arg(long, default_value = "-")]
        separator: char,

        #[arg(long)]
        chunk_none: bool,

        #[arg(long)]
        chunk_fixed: Option<usize>,

        #[arg(long, value_parser = parse_pattern)]
        chunk_pattern: Option<Vec<usize>>,

        #[arg(long)]
        timestamp_epoch: Option<i64>,

        #[arg(long, value_enum)]
        timestamp_granularity: Option<GranularityArg>,

        input: Option<String>,
    },

    Decode {
        #[arg(short, long)]
        key: Option<String>,

        #[arg(short, long, value_enum, default_value_t = LevelArg::Universal)]
        level: LevelArg,

        #[arg(short, long, value_enum, default_value_t = ModeArg::Sealed)]
        mode: ModeArg,

        #[arg(short = 'w', long, default_value = "128")]
        bits: u32,

        #[arg(long)]
        modulus: Option<String>,

        #[arg(long, value_enum)]
        cipher: Option<CipherCliArg>,

        #[arg(long, default_value = "1")]
        check_syllables: usize,

        input: String,
    },

    Recover {
        #[arg(short, long)]
        key: Option<String>,

        #[arg(short, long, value_enum, default_value_t = LevelArg::Universal)]
        level: LevelArg,

        #[arg(short, long, value_enum, default_value_t = ModeArg::Sealed)]
        mode: ModeArg,

        #[arg(short = 'w', long, default_value = "128")]
        bits: u32,

        input: String,
    },

    Kat {
        #[arg(default_value = "shared/kat")]
        kat_dir: String,

        #[arg(short, long)]
        key: Option<String>,
    },
}

#[derive(Clone, ValueEnum)]
enum LevelArg {
    #[value(name = "universal", alias = "u")]
    Universal,
    #[value(name = "eu", alias = "e")]
    Eu,
    #[value(name = "en")]
    En,
    #[value(name = "numeric", alias = "n")]
    Numeric,
}

#[derive(Clone, ValueEnum)]
enum ModeArg {
    #[value(name = "sealed", alias = "s")]
    Sealed,
    #[value(name = "open", alias = "o")]
    Open,
    #[value(name = "plain", alias = "p")]
    Plain,
}

#[derive(Clone, ValueEnum)]
enum CaseArg {
    #[value(name = "upper")]
    Upper,
    #[value(name = "lower")]
    Lower,
}

#[derive(Clone, ValueEnum)]
enum GranularityArg {
    #[value(name = "second")]
    Second,
    #[value(name = "minute")]
    Minute,
    #[value(name = "hour")]
    Hour,
    #[value(name = "day")]
    Day,
    #[value(name = "week")]
    Week,
    #[value(name = "month")]
    Month,
}

#[derive(Clone, ValueEnum)]
enum CipherCliArg {
    #[value(name = "feistel")]
    Feistel,
    #[value(name = "shuffle")]
    Shuffle,
}

fn parse_pattern(s: &str) -> Result<Vec<usize>, String> {
    s.split(',')
        .map(|n| n.parse::<usize>().map_err(|e| e.to_string()))
        .collect()
}

impl From<LevelArg> for LanguageLevel {
    fn from(a: LevelArg) -> Self {
        match a {
            LevelArg::Universal => LanguageLevel::Universal,
            LevelArg::Eu => LanguageLevel::Eu,
            LevelArg::En => LanguageLevel::En,
            LevelArg::Numeric => LanguageLevel::Numeric,
        }
    }
}

impl From<ModeArg> for HceMode {
    fn from(a: ModeArg) -> Self {
        match a {
            ModeArg::Sealed => HceMode::Sealed,
            ModeArg::Open => HceMode::Open,
            ModeArg::Plain => HceMode::Plain,
        }
    }
}

impl From<CaseArg> for HceCase {
    fn from(a: CaseArg) -> Self {
        match a {
            CaseArg::Upper => HceCase::Upper,
            CaseArg::Lower => HceCase::Lower,
        }
    }
}

impl From<GranularityArg> for TsGranularity {
    fn from(a: GranularityArg) -> Self {
        match a {
            GranularityArg::Second => TsGranularity::Second,
            GranularityArg::Minute => TsGranularity::Minute,
            GranularityArg::Hour => TsGranularity::Hour,
            GranularityArg::Day => TsGranularity::Day,
            GranularityArg::Week => TsGranularity::Week,
            GranularityArg::Month => TsGranularity::Month,
        }
    }
}

fn parse_hex_input(hex: &str) -> Result<Vec<u8>, String> {
    let trimmed = hex.trim();
    if trimmed.is_empty() {
        return Err("empty input".into());
    }
    hex::decode(trimmed).map_err(|e| format!("invalid hex: {}", e))
}

fn parse_key_arg(hex: &Option<String>) -> Result<Option<Vec<u8>>, String> {
    match hex {
        Some(k) => Ok(Some(
            hex::decode(k.trim()).map_err(|e| format!("invalid key hex: {}", e))?,
        )),
        None => Ok(None),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encode {
            key,
            level,
            mode,
            bits,
            modulus,
            cipher,
            case,
            check_syllables,
            separator,
            chunk_none,
            chunk_fixed,
            chunk_pattern,
            timestamp_epoch,
            timestamp_granularity,
            input,
        } => {
            let mode: HceMode = mode.into();
            let key_bytes = match parse_key_arg(&key) {
                Ok(Some(k)) if mode != HceMode::Plain => Some(k),
                Ok(_) if mode == HceMode::Plain => None,
                Ok(_) => {
                    eprintln!("error: --key required for sealed/open modes");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            };

            let mut hce = if let Some(ref m) = modulus {
                let n = m.trim().parse::<u128>().unwrap_or_else(|_| {
                    eprintln!("error: invalid modulus: {}", m);
                    std::process::exit(1);
                });
                Hce::new(key_bytes.as_deref(), level.into(), mode).with_modulus(n)
            } else {
                Hce::new(key_bytes.as_deref(), level.into(), mode).with_bit_width(bits)
            };

            if let Some(c) = case {
                hce = hce.with_case(c.into());
            }
            if let Some(ref ck) = cipher {
                let ckind = match ck {
                    CipherCliArg::Feistel => CipherKind::Feistel8,
                    CipherCliArg::Shuffle => CipherKind::Shuffle4,
                };
                let dom = hce.domain();
                let fpe = build_cipher(ckind, key_bytes.as_deref(), dom);
                hce = hce.with_cipher(fpe);
            }
            hce = hce.with_check_syllables(check_syllables);

            if chunk_none {
                hce = hce.with_chunk_spec(ChunkSpec::none());
            } else if let Some(sz) = chunk_fixed {
                hce = hce.with_chunk_spec(ChunkSpec::fixed(sz));
            } else if let Some(ref pat) = chunk_pattern {
                hce = hce.with_chunk_spec(ChunkSpec::pattern(pat));
            } else {
                hce = hce.with_chunk_spec(ChunkSpec::natural_with_separator(separator));
            }

            if let (Some(epoch), Some(gran)) = (timestamp_epoch, timestamp_granularity) {
                hce = hce.with_timestamp_config(epoch, gran.into());
            }

            let data = match input {
                Some(ref hex_str) => match parse_hex_input(hex_str) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("error: {}", e);
                        std::process::exit(1);
                    }
                },
                None => {
                    let mut buf = Vec::new();
                    if let Err(e) = io::stdin().read_to_end(&mut buf) {
                        eprintln!("error reading stdin: {}", e);
                        std::process::exit(1);
                    }
                    buf
                }
            };

            println!("{}", hce.encode(&data));
        }

        Commands::Decode {
            key,
            level,
            mode,
            bits,
            modulus,
            cipher,
            check_syllables,
            input,
        } => {
            let mode: HceMode = mode.into();
            let key_bytes = match parse_key_arg(&key) {
                Ok(Some(k)) if mode != HceMode::Plain => Some(k),
                Ok(_) if mode == HceMode::Plain => None,
                Ok(_) => {
                    eprintln!("error: --key required for sealed/open modes");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            };

            let mut hce = if let Some(ref m) = modulus {
                let n = m.trim().parse::<u128>().unwrap_or_else(|_| {
                    eprintln!("error: invalid modulus: {}", m);
                    std::process::exit(1);
                });
                Hce::new(key_bytes.as_deref(), level.into(), mode).with_modulus(n)
            } else {
                Hce::new(key_bytes.as_deref(), level.into(), mode).with_bit_width(bits)
            };

            if let Some(ref ck) = cipher {
                let ckind = match ck {
                    CipherCliArg::Feistel => CipherKind::Feistel8,
                    CipherCliArg::Shuffle => CipherKind::Shuffle4,
                };
                let dom = hce.domain();
                let fpe = build_cipher(ckind, key_bytes.as_deref(), dom);
                hce = hce.with_cipher(fpe);
            }

            hce = hce.with_check_syllables(check_syllables);

            match hce.decode(&input) {
                Ok(bytes) => println!("{}", hex::encode(&bytes)),
                Err(e) => {
                    eprintln!("decode error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Recover {
            key,
            level,
            mode,
            bits,
            input,
        } => {
            let mode: HceMode = mode.into();
            let key_bytes = match parse_key_arg(&key) {
                Ok(Some(k)) if mode != HceMode::Plain => Some(k),
                Ok(_) if mode == HceMode::Plain => None,
                Ok(_) => {
                    eprintln!("error: --key required for sealed/open modes");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            };

            let hce = Hce::new(key_bytes.as_deref(), level.into(), mode).with_bit_width(bits);
            match hce.recover(&input) {
                Ok(result) => match hce.decode_corrected(&result) {
                    Some(bytes) => println!("{}", hex::encode(&bytes)),
                    None => match hce.decode(&input) {
                        Ok(bytes) => println!("{} (no correction needed)", hex::encode(&bytes)),
                        Err(e) => {
                            eprintln!("recover error: {}", e);
                            std::process::exit(1);
                        }
                    },
                },
                Err(e) => {
                    eprintln!("recover error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Kat { kat_dir, key } => {
            let key_hex = key.unwrap_or_else(|| hex::encode(b"hce-kat-standard-key-32-bytes!!"));
            let key_bytes = match hex::decode(key_hex.trim()) {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("fatal: invalid kat key hex: {}", e);
                    std::process::exit(1);
                }
            };

            let levels = [
                ("universal", LanguageLevel::Universal),
                ("eu", LanguageLevel::Eu),
                ("en", LanguageLevel::En),
                ("numeric", LanguageLevel::Numeric),
            ];

            let mut total = 0;
            let mut failures = 0;

            for (name, level) in &levels {
                let path = format!("{}/{}.json", kat_dir, name);
                let data = match std::fs::read_to_string(&path) {
                    Ok(d) => d,
                    Err(_) => {
                        eprintln!("  skip {}: file not found", name);
                        continue;
                    }
                };

                let kat: serde_json::Value = match serde_json::from_str(&data) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("  {}: FAIL (invalid json: {})", name, e);
                        failures += 1;
                        continue;
                    }
                };

                let vectors = match kat["vectors"].as_array() {
                    Some(v) => v,
                    None => {
                        eprintln!("  {}: FAIL (no vectors array)", name);
                        failures += 1;
                        continue;
                    }
                };

                let mut level_ok = true;
                for entry in vectors {
                    let mode_str = entry["mode"].as_str().unwrap_or("");
                    let expected = entry["hce"].as_str().unwrap_or("");
                    let uuid_hex = entry["uuid"].as_str().unwrap_or("");

                    if mode_str.is_empty() || expected.is_empty() || uuid_hex.is_empty() {
                        failures += 2;
                        level_ok = false;
                        continue;
                    }

                    let uuid_bytes = match hex::decode(uuid_hex) {
                        Ok(v) => v,
                        Err(_) => {
                            failures += 2;
                            level_ok = false;
                            continue;
                        }
                    };

                    let (mode, kat_key) = match mode_str {
                        "plain" => (HceMode::Plain, None),
                        "open" => (HceMode::Open, Some(key_bytes.as_slice())),
                        _ => (HceMode::Sealed, Some(key_bytes.as_slice())),
                    };

                    let hce = Hce::new(kat_key, *level, mode);
                    let encoded = hce.encode(&uuid_bytes);
                    total += 2;
                    if encoded != expected {
                        failures += 1;
                    }
                    match hce.decode(&encoded) {
                        Ok(decoded) if decoded == uuid_bytes => {}
                        _ => {
                            failures += 1;
                        }
                    }
                }

                if level_ok {
                    eprintln!("  {}: OK", name);
                }
            }

            if failures > 0 {
                eprintln!("FAIL: {} of {} checks failed", failures, total);
                std::process::exit(1);
            } else {
                eprintln!("ALL {} checks passed", total);
            }
        }
    }
}
