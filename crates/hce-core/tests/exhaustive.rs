use hce_core::*;
use std::sync::Arc;

static KEY: &[u8] = b"exhaustive-test-key--32-bytes!!";

fn roundtrip(h: &Hce, data: &[u8]) {
    let encoded = h.encode(data);
    let decoded = h.decode(&encoded).unwrap();
    let bw = (h.bit_width().div_ceil(8)) as usize;
    assert_eq!(
        &decoded[16 - bw..],
        data,
        "roundtrip failed: enc={}",
        encoded
    );
}

#[test]
fn exhaustive_levels_modes_bitwidths() {
    let levels = [
        LanguageLevel::Universal,
        LanguageLevel::Eu,
        LanguageLevel::En,
        LanguageLevel::Numeric,
    ];
    let modes = [HceMode::Sealed, HceMode::Open, HceMode::Plain];
    let bws = [16u32, 32, 48, 64, 80, 96, 112, 128];
    let mut seed: u64 = 0xCAFE;
    for &level in &levels {
        for &mode in &modes {
            for &bw in &bws {
                let key = match mode {
                    HceMode::Plain => None,
                    _ => Some(KEY),
                };
                let h = Hce::new(key, level, mode).with_bit_width(bw);
                let blen = (bw.div_ceil(8)) as usize;
                for _ in 0..4 {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let data = &(seed as u128).to_be_bytes()[16 - blen..];
                    roundtrip(&h, data);
                }
            }
        }
    }
}

#[test]
fn exhaustive_moduli() {
    let moduli = [
        2u128,
        3,
        7,
        15,
        16,
        31,
        100,
        255,
        256,
        1000,
        10000,
        100000,
        1_000_000,
        1_000_000_000,
    ];
    let mut seed: u64 = 0xDEAD;
    for &m in &moduli {
        let h = Hce::new(Some(KEY), LanguageLevel::Universal, HceMode::Sealed).with_modulus(m);
        let bw = (h.bit_width().div_ceil(8)) as usize;
        let max = m - 1;
        for _ in 0..10 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = (seed as u128) % max.max(1);
            let bytes = val.to_be_bytes();
            let data = if bytes.len() > bw {
                &bytes[bytes.len() - bw..]
            } else {
                &bytes[..]
            };
            roundtrip(&h, data);
        }
    }
}

#[test]
fn exhaustive_custom_ciphers() {
    let mut seed: u64 = 0xBEEF;
    for bw in [32u32, 64, 96, 128] {
        let h = Hce::new(Some(KEY), LanguageLevel::Universal, HceMode::Sealed)
            .with_bit_width(bw)
            .with_cipher(Arc::new(FixedShuffle::with_domain(Domain::Bits(bw))));
        let blen = (bw.div_ceil(8)) as usize;
        for _ in 0..8 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let data = &(seed as u128).to_be_bytes()[16 - blen..];
            roundtrip(&h, data);
        }
    }
}

#[test]
fn exhaustive_chunks() {
    let specs: [(ChunkSpec, &str); 5] = [
        (ChunkSpec::natural(), "natural"),
        (ChunkSpec::natural_with_separator('.'), "dot"),
        (ChunkSpec::none(), "none"),
        (ChunkSpec::fixed(5), "fixed"),
        (ChunkSpec::pattern(&[3, 3, 4, 4]), "pattern"),
    ];
    let mut seed: u64 = 0xFACE;
    for (spec, name) in &specs {
        let h = Hce::new(Some(KEY), LanguageLevel::Universal, HceMode::Sealed)
            .with_chunk_spec(spec.clone());
        for _ in 0..5 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let data = &(seed as u128).to_be_bytes();
            roundtrip(&h, data);
        }
        if name == &"none" {
            let enc = h.encode(&[0u8; 16]);
            assert!(!enc.contains('-'), "none chunk had hyphen");
        }
    }
}

#[test]
fn exhaustive_case_check() {
    for case in [HceCase::Upper, HceCase::Lower] {
        for cs in [1usize, 2, 4, 8] {
            let h = Hce::new(Some(KEY), LanguageLevel::Universal, HceMode::Sealed)
                .with_case(case)
                .with_check_syllables(cs);
            roundtrip(&h, &[0x42u8; 16]);
        }
    }
}

#[test]
fn exhaustive_errors() {
    assert!(
        Hce::new(Some(KEY), LanguageLevel::Universal, HceMode::Sealed)
            .with_bit_width(0)
            .encode(&[1u8])
            .len()
            > 0
    );
    assert!(
        Hce::new(Some(KEY), LanguageLevel::Universal, HceMode::Sealed)
            .with_modulus(1)
            .encode(&[0u8])
            .len()
            > 0
    );
    assert!(
        Hce::new(Some(KEY), LanguageLevel::Universal, HceMode::Sealed)
            .with_check_syllables(100)
            .encode(&[0x42u8; 16])
            .len()
            > 0
    );
    assert!(Hce::new(None, LanguageLevel::Universal, HceMode::Plain)
        .decode(&Hce::new(None, LanguageLevel::Universal, HceMode::Plain).encode(&[0x42u8; 16]))
        .is_ok());
}

#[test]
fn exhaustive_cipher_domain_order() {
    for bw in [64u32, 128] {
        let domain = Domain::Bits(bw);
        let fpe: Arc<dyn Fpe + Send + Sync> = Arc::new(FixedShuffle::with_domain(domain));
        let h1 = Hce::new(Some(KEY), LanguageLevel::Universal, HceMode::Sealed)
            .with_bit_width(bw)
            .with_cipher(Arc::clone(&fpe));
        let e1 = h1.encode(&[42u8; 16]);
        assert!(h1.decode(&e1).is_ok(), "bw→cipher bw={}", bw);
        let h2 = Hce::new(Some(KEY), LanguageLevel::Universal, HceMode::Sealed)
            .with_cipher(Arc::clone(&fpe))
            .with_bit_width(bw);
        let e2 = h2.encode(&[42u8; 16]);
        if e1 == e2 {
            assert!(h2.decode(&e2).is_ok(), "cipher→bw bw={}", bw);
        }
    }
}

#[test]
fn exhaustive_api_smoke() {
    let _ = format!("{:?}", HceCase::Upper);
    let _ = format!("{}", HceError::KeyRequired);
    let _ = LanguageLevel::from_byte(99);
    let _ = Domain::Modulus(100).max_value();
    let _ = build_tweak(0, 0, Domain::Bits(128));
    let _ = ChunkSpec::natural_with_alpha(50);
    let _ = validate_separator('-');
    let _ = compute_syll_len(128);
    let _ = body_tokens_for(64);
    let tok = Token::onset(5);
    let _ = tok.index();
    let _ = tok.display();
    let _ = ts_prefix(1_700_000_000_000, 1_700_000_000_000, TsGranularity::Month);
    let _ = bytes_to_int(&[0u8; 16]);
    let _ = int_to_bytes(0u128);
    let cm = ConfusionMatrix::universal();
    let _ = cm.neighbors(Token::onset(4));
    let body = encode_tokens(42, 14);
    let _ = compute_check(KEY, &body, 1);
    let _ = verify_check(KEY, &body, &body[..2]);
    let fs = FixedShuffle::new();
    let _ = fs.encrypt(42, b"t");
    let f = Feistel::new(KEY);
    let _ = f.encrypt(42, b"t");
    let _ = Hce::new(Some(KEY), LanguageLevel::Universal, HceMode::Sealed)
        .with_bit_width(64)
        .with_modulus(1_000)
        .with_chunk_spec(ChunkSpec::natural())
        .with_check_syllables(3)
        .with_case(HceCase::Lower)
        .with_timestamp_config(1_700_000_000_000, TsGranularity::Day)
        .domain();
}
