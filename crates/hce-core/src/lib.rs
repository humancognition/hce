#![no_std]

extern crate alloc;

mod check;
mod chunk;
mod confusion;
mod constants;
mod crc;
mod encode;
mod fpe;
mod normalize;
mod parser;
mod pools;
mod recover;
mod render;
mod timestamp;
mod token;
mod version;
mod wire;

pub use check::{compute_check, verify_check};
pub use chunk::validate_separator;
pub use chunk::{ChunkSpec, ChunkStrategy, DefaultEngine};
pub use confusion::ConfusionMatrix;
pub use constants::{
    body_tokens_for, compute_syll_len, DEFAULT_BIT_WIDTH, ONSET_RADIX, VNUC_RADIX,
};
pub use encode::{decode_tokens, encode_tokens};
pub use fpe::{build_tweak, Domain, Feistel, FixedShuffle, Fpe};
pub use normalize::Normalizer;
pub use pools::{is_coda_char, is_vowel_char};
pub use recover::{Recoverer, RecoveryResult};
pub use timestamp::{ts_prefix, TsGranularity};
pub use token::Token;
pub use version::{reassemble_uuid, split_uuid, UuidInfo};
pub use wire::{bytes_to_int, int_to_bytes};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageLevel {
    Universal,
    Eu,
    En,
    Numeric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HceMode {
    Sealed,
    Open,
    Plain,
}

impl LanguageLevel {
    pub fn to_byte(self) -> u8 {
        match self {
            LanguageLevel::Universal => 0,
            LanguageLevel::Eu => 1,
            LanguageLevel::En => 2,
            LanguageLevel::Numeric => 3,
        }
    }

    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(LanguageLevel::Universal),
            1 => Some(LanguageLevel::Eu),
            2 => Some(LanguageLevel::En),
            3 => Some(LanguageLevel::Numeric),
            _ => None,
        }
    }
}

impl HceMode {
    pub fn to_byte(self) -> u8 {
        match self {
            HceMode::Sealed => 0,
            HceMode::Open => 1,
            HceMode::Plain => 2,
        }
    }

    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(HceMode::Sealed),
            1 => Some(HceMode::Open),
            2 => Some(HceMode::Plain),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HceError {
    NormalizeError,
    KeyRequired,
    RecoveryNotSupported,
    IntegrityFailure,
}

impl From<normalize::NormalizeError> for HceError {
    fn from(_: normalize::NormalizeError) -> Self {
        HceError::NormalizeError
    }
}

impl core::fmt::Display for HceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HceError::NormalizeError => write!(f, "input normalization failed"),
            HceError::KeyRequired => write!(f, "key required for this mode"),
            HceError::RecoveryNotSupported => write!(f, "recovery not supported in plain mode"),
            HceError::IntegrityFailure => {
                write!(f, "check verification failed — input may be corrupted")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HceCase {
    Upper,
    Lower,
}

#[derive(Clone)]
struct Cipher(alloc::sync::Arc<dyn Fpe + Send + Sync>);

pub static FIXED_PLAIN_KEY: [u8; 32] = [
    0x1f, 0x2e, 0x3d, 0x4c, 0x5b, 0x6a, 0x79, 0x88, 0x97, 0xa6, 0xb5, 0xc4, 0xd3, 0xe2, 0xf1, 0x00,
    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x10,
];

use zeroize::Zeroizing;

#[derive(Clone)]
pub struct Hce {
    key: Option<Zeroizing<alloc::vec::Vec<u8>>>,
    level: LanguageLevel,
    mode: HceMode,
    bit_width: u32,
    current_domain: Domain,
    ts_epoch: i64,
    ts_granularity: TsGranularity,
    cipher: Cipher,
    tweak: [u8; 4],
    check_syllables: usize,
    case: HceCase,
    chunk_spec: ChunkSpec,
    chunk_engine: alloc::sync::Arc<dyn ChunkStrategy + Send + Sync>,
    normalizer: Normalizer,
    custom_cipher: bool,
}

impl core::fmt::Debug for Hce {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Hce")
            .field("level", &self.level)
            .field("mode", &self.mode)
            .field("bit_width", &self.bit_width)
            .field("check_syllables", &self.check_syllables)
            .field("case", &self.case)
            .finish()
    }
}

impl Hce {
    pub fn new(key: Option<&[u8]>, level: LanguageLevel, mode: HceMode) -> Self {
        if let (Some(k), mode) = (&key, mode) {
            if mode != HceMode::Plain && k.is_empty() {
                panic!("key must not be empty for sealed/open modes");
            }
        }
        let domain = Domain::Bits(128);
        let cipher = match (key, mode) {
            (_, HceMode::Plain) => Cipher(alloc::sync::Arc::new(Feistel::with_domain(
                &FIXED_PLAIN_KEY,
                domain,
            ))),
            (Some(k), _) => Cipher(alloc::sync::Arc::new(Feistel::with_domain(k, domain))),
            (None, _) => Cipher(alloc::sync::Arc::new(Feistel::with_domain(
                &[0u8; 32], domain,
            ))),
        };

        Hce {
            key: match (key, mode) {
                (Some(k), _) => Some(Zeroizing::new(k.to_vec())),
                (None, HceMode::Plain) => None,
                (None, _) => Some(Zeroizing::new(alloc::vec![0u8; 32])),
            },
            mode,
            level,
            bit_width: constants::DEFAULT_BIT_WIDTH,
            ts_epoch: constants::TS_EPOCH_MS,
            ts_granularity: TsGranularity::Month,
            cipher,
            tweak: build_tweak(level.to_byte(), mode.to_byte(), domain),
            check_syllables: 1,
            case: HceCase::Upper,
            chunk_spec: ChunkSpec::natural(),
            chunk_engine: alloc::sync::Arc::new(DefaultEngine),
            normalizer: Normalizer::new(),
            current_domain: domain,
            custom_cipher: false,
        }
    }

    #[must_use]
    pub fn with_timestamp_config(mut self, epoch_ms: i64, granularity: TsGranularity) -> Self {
        self.ts_epoch = epoch_ms;
        self.ts_granularity = granularity;
        self
    }

    pub fn with_chunk_spec(mut self, spec: ChunkSpec) -> Self {
        self.chunk_spec = spec;
        self
    }

    pub fn with_chunk_strategy(
        mut self,
        engine: alloc::sync::Arc<dyn ChunkStrategy + Send + Sync>,
    ) -> Self {
        self.chunk_engine = engine;
        self
    }

    pub fn with_check_syllables(mut self, n: usize) -> Self {
        self.check_syllables = n.clamp(1, 8);
        self
    }

    pub fn with_case(mut self, case: HceCase) -> Self {
        self.case = case;
        self
    }

    #[must_use]
    pub fn with_bit_width(mut self, bits: u32) -> Self {
        self.bit_width = bits.clamp(16, 128);
        let domain = Domain::Bits(self.bit_width);
        self.current_domain = domain;
        self.rebuild_cipher(domain);
        self
    }

    pub fn with_modulus(mut self, modulus: u128) -> Self {
        let modulus = modulus.max(2);
        self.bit_width = 128 - modulus.leading_zeros();
        let domain = Domain::Modulus(modulus);
        self.current_domain = domain;
        self.rebuild_cipher(domain);
        self
    }

    pub fn with_domain(mut self, domain: Domain) -> Self {
        let domain = match domain {
            Domain::Bits(n) if n < 16 => Domain::Bits(16),
            Domain::Bits(n) if n > 128 => Domain::Bits(128),
            Domain::Modulus(m) if m < 2 => Domain::Modulus(2),
            other => other,
        };
        self.bit_width = match domain {
            Domain::Bits(n) => n,
            Domain::Modulus(m) => 128 - m.leading_zeros(),
        };
        self.current_domain = domain;
        self.rebuild_cipher(domain);
        self
    }

    pub fn with_cipher(mut self, fpe: alloc::sync::Arc<dyn Fpe + Send + Sync>) -> Self {
        self.cipher = Cipher(fpe);
        self.custom_cipher = true;
        self
    }

    pub fn domain(&self) -> Domain {
        self.current_domain
    }

    pub fn bit_width(&self) -> u32 {
        self.bit_width
    }

    fn rebuild_cipher(&mut self, domain: Domain) {
        self.tweak[2] = match domain {
            Domain::Bits(n) => (n as u8).min(128),
            Domain::Modulus(m) => 128u32.saturating_sub(m.leading_zeros()) as u8,
        };
        if self.custom_cipher {
            return;
        }
        self.cipher = match (&self.key, self.mode) {
            (_, HceMode::Plain) => Cipher(alloc::sync::Arc::new(Feistel::with_domain(
                &FIXED_PLAIN_KEY,
                domain,
            ))),
            (Some(k), _) => Cipher(alloc::sync::Arc::new(Feistel::with_domain(k, domain))),
            (None, _) => Cipher(alloc::sync::Arc::new(Feistel::with_domain(
                &[0u8; 32], domain,
            ))),
        };
    }

    pub fn encode(&self, bytes: &[u8]) -> alloc::string::String {
        let bw = self.byte_width();
        let mut buf = alloc::vec::Vec::with_capacity(bw);
        let n = bytes.len().min(bw);
        if matches!(self.current_domain, Domain::Modulus(_)) {
            buf.resize(bw.saturating_sub(n), 0);
            buf.extend_from_slice(&bytes[bytes.len().saturating_sub(n)..]);
        } else {
            buf.extend_from_slice(&bytes[..n]);
            buf.resize(bw, 0);
        }
        self.encode_bytes(&buf)
    }

    fn byte_width(&self) -> usize {
        self.bit_width.div_ceil(8) as usize
    }

    fn encode_bytes(&self, data: &[u8]) -> alloc::string::String {
        let mut buf = [0u8; 16];
        let n = data.len().min(16);
        buf[16 - n..].copy_from_slice(&data[..n]);
        self.encode_payload(version::split_uuid(&buf))
    }

    fn encode_payload(&self, info: version::UuidInfo) -> alloc::string::String {
        let mask = if self.bit_width >= 128 {
            u128::MAX
        } else {
            (1u128 << self.bit_width) - 1
        };
        let payload = info.payload & mask;
        let syll_len = constants::compute_syll_len(self.bit_width);

        let cipher = self.cipher.0.encrypt(payload, &self.tweak);

        let body = encode_tokens(cipher, syll_len);

        let check_tokens = match self.mode {
            HceMode::Plain => crc::compute_crc_check(&body, self.check_syllables),
            _ => {
                let key = self
                    .key
                    .as_deref()
                    .expect("key required for sealed/open modes");
                check::compute_check(key, &body, self.check_syllables)
            }
        };

        let body_str = render::render_body(
            &body,
            &self.chunk_spec,
            &*self.chunk_engine,
            self.case,
            self.level,
        );
        let check_str = render::render_check(&check_tokens, self.case, self.level);

        let prefix = if self.mode == HceMode::Open && info.has_ts {
            let ts = info.ts_ms.expect("has_ts implies ts_ms is Some");
            timestamp::ts_prefix(ts, self.ts_epoch, self.ts_granularity)
        } else {
            alloc::string::String::new()
        };

        let sep: alloc::string::String = if self.chunk_spec.separator == '\0' {
            alloc::string::String::new()
        } else {
            alloc::string::String::from(self.chunk_spec.separator)
        };

        if prefix.is_empty() {
            alloc::format!("{}{}{}", body_str, sep, check_str)
        } else {
            alloc::format!("{}{}{}{}{}", prefix, sep, body_str, sep, check_str)
        }
    }

    pub fn decode(&self, input: &str) -> Result<alloc::vec::Vec<u8>, HceError> {
        let uuid = self.decode_hce(input)?;
        Ok(uuid.to_vec())
    }

    fn decode_hce(&self, input: &str) -> Result<[u8; 16], HceError> {
        let body_tokens_count = constants::body_tokens_for(self.bit_width);
        let check_count = self.check_syllables * 2;
        let (body_tokens, check_tokens) =
            self.normalizer
                .normalize(input, body_tokens_count, check_count)?;

        match self.mode {
            HceMode::Plain => {
                if !crc::verify_crc_check(&body_tokens, &check_tokens) {
                    return Err(HceError::IntegrityFailure);
                }
            }
            _ => {
                let key = self.key.as_deref().ok_or(HceError::KeyRequired)?;
                if !check::verify_check(key, &body_tokens, &check_tokens) {
                    return Err(HceError::IntegrityFailure);
                }
            }
        }

        let cipher = encode::decode_tokens(&body_tokens);

        let payload = self.cipher.0.decrypt(cipher, &self.tweak);

        let payload = if self.bit_width >= 128 {
            payload
        } else {
            payload & ((1u128 << self.bit_width) - 1)
        };

        Ok(version::reassemble_uuid(payload))
    }

    pub fn recover(&self, input: &str) -> Result<RecoveryResult, HceError> {
        match self.mode {
            HceMode::Plain => Err(HceError::RecoveryNotSupported),
            _ => {
                let key = self.key.as_deref().ok_or(HceError::KeyRequired)?;
                let body_tokens_count = constants::body_tokens_for(self.bit_width);
                let check_count = self.check_syllables * 2;
                let (body_tokens, check_tokens) =
                    self.normalizer
                        .normalize(input, body_tokens_count, check_count)?;
                let matrix = ConfusionMatrix::universal();
                let recoverer = Recoverer::new(&matrix);
                Ok(recoverer.recover(key, &body_tokens, &check_tokens))
            }
        }
    }

    pub fn decode_corrected(&self, result: &RecoveryResult) -> Option<alloc::vec::Vec<u8>> {
        match result {
            RecoveryResult::Corrected(tokens) => {
                let cipher = encode::decode_tokens(tokens);
                let mut payload = self.cipher.0.decrypt(cipher, &self.tweak);
                if self.bit_width < 128 {
                    payload &= (1u128 << self.bit_width) - 1;
                }
                Some(version::reassemble_uuid(payload).to_vec())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    fn test_key() -> &'static [u8] {
        b"test-key--32-bytes-long-key-here"
    }

    fn test_uuid_v7() -> [u8; 16] {
        [
            0x01, 0x95, 0xe3, 0xa0, 0x7c, 0x2e, 0x7b, 0x41, 0x8f, 0x3d, 0x9a, 0x6c, 0x1e, 0x0b,
            0x4d, 0x27,
        ]
    }

    fn test_uuid_v4() -> [u8; 16] {
        [
            0xf8, 0x1d, 0x4f, 0xae, 0x7d, 0xec, 0x4d, 0x10, 0xa7, 0x65, 0x00, 0xa0, 0xc9, 0x1e,
            0x6b, 0xf6,
        ]
    }

    #[test]
    fn sealed_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let encoded = hce.encode(&test_uuid_v7());
        let decoded = hce.decode_hce(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v7());
    }

    #[test]
    fn plain_roundtrip() {
        let hce = Hce::new(None, LanguageLevel::Universal, HceMode::Plain);
        let encoded = hce.encode(&test_uuid_v4());
        let decoded = hce.decode_hce(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v4());
    }

    #[test]
    fn open_has_prefix() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Open);
        let encoded = hce.encode(&test_uuid_v7());
        assert!(encoded.starts_with('K'));
        let decoded = hce.decode_hce(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v7());
    }

    #[test]
    fn sealed_v4_no_prefix() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let encoded = hce.encode(&test_uuid_v4());
        assert!(!encoded.starts_with('K'));
    }

    #[test]
    fn open_v4_no_prefix() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Open);
        let encoded = hce.encode(&test_uuid_v4());
        assert!(!encoded.starts_with('K'));
    }

    #[test]
    fn chunk_count_is_5() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let encoded = hce.encode(&test_uuid_v7());
        let no_check = encoded.rsplit_once('-').unwrap().0;
        assert_eq!(no_check.split('-').count(), 5);
    }

    #[test]
    fn output_length_in_range() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let encoded = hce.encode(&test_uuid_v7());
        let clean: String = encoded.chars().filter(|c| *c != '-').collect();
        assert!(
            clean.len() >= 36 && clean.len() <= 56,
            "got length {}",
            clean.len()
        );
    }

    #[test]
    fn tampered_decode_fails() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let encoded = hce.encode(&test_uuid_v7());
        let mut chars: alloc::vec::Vec<char> = encoded.chars().collect();
        for (i, &original) in chars.iter().enumerate() {
            if ('B'..='Z').contains(&original) {
                chars[i] = if original == 'B' { 'D' } else { 'B' };
                break;
            }
        }
        let tampered: String = chars.into_iter().collect();
        assert!(tampered != encoded, "tampering had no effect");
        assert!(matches!(
            hce.decode_hce(&tampered),
            Err(HceError::IntegrityFailure)
        ));
    }

    #[test]
    fn default_key_produces_valid_output() {
        let hce = Hce::new(None, LanguageLevel::Universal, HceMode::Sealed);
        let encoded = hce.encode(&test_uuid_v7());
        let decoded = hce.decode_hce(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v7());
    }

    #[test]
    fn fixed_chunk_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_chunk_spec(ChunkSpec::fixed(7));
        let encoded = hce.encode(&test_uuid_v4());
        let decoded = hce.decode_hce(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v4());
    }

    #[test]
    fn none_chunk_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_chunk_spec(ChunkSpec::none());
        let encoded = hce.encode(&test_uuid_v4());
        let decoded = hce.decode_hce(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v4());
    }

    #[test]
    fn pattern_chunk_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_chunk_spec(ChunkSpec::pattern(&[3, 3, 3, 3, 2]));
        let encoded = hce.encode(&test_uuid_v4());
        let decoded = hce.decode_hce(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v4());
    }

    #[test]
    fn natural_is_default() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let encoded = hce.encode(&test_uuid_v7());
        let no_check = encoded.rsplit_once('-').unwrap().0;
        assert_eq!(no_check.split('-').count(), 5);
    }

    #[test]
    fn decode_ignores_all_separators() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let encoded_hyphen = hce.encode(&test_uuid_v7());
        let hce_none = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_chunk_spec(ChunkSpec::none());
        let encoded_none = hce_none.encode(&test_uuid_v7());

        let from_hyphen = hce.decode_hce(&encoded_hyphen).unwrap();
        let from_none = hce.decode_hce(&encoded_none).unwrap();
        assert_eq!(from_hyphen, from_none);
        assert_eq!(from_hyphen, test_uuid_v7());
    }

    #[test]
    fn lower_case_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_case(HceCase::Lower);
        let encoded = hce.encode(&test_uuid_v7());
        assert!(encoded.chars().all(|c| !c.is_uppercase() || c == '-'));
        let decoded = hce.decode(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v7().to_vec());
    }

    #[test]
    fn bit_width_64_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_bit_width(64);
        let data: [u8; 8] = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = hce.encode(&data);
        let decoded = hce.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 16);
        assert_eq!(&decoded[8..16], &data[..]);
    }

    #[test]
    fn bit_width_96_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_bit_width(96);
        let mut data = [0u8; 12];
        data.copy_from_slice(b"hello-world!");
        let encoded = hce.encode(&data);
        let decoded = hce.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 16);
        assert_eq!(&decoded[4..16], &data[..]);
    }

    #[test]
    fn bit_width_128_is_default() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let encoded = hce.encode(&test_uuid_v7());
        let decoded = hce.decode(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v7().to_vec());
    }

    #[test]
    fn check_syllables_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_check_syllables(2);
        let encoded = hce.encode(&test_uuid_v7());
        let decoded = hce.decode(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v7().to_vec());
    }

    #[test]
    fn custom_strategy_roundtrip() {
        struct HalfSyllables;
        impl ChunkStrategy for HalfSyllables {
            fn boundaries(&self, tokens: &[Token], _spec: &ChunkSpec) -> alloc::vec::Vec<usize> {
                let n = tokens.len() / 2;
                alloc::vec![n / 2]
            }
        }
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_chunk_strategy(alloc::sync::Arc::new(HalfSyllables));
        let encoded = hce.encode(&test_uuid_v7());
        let body = encoded.rsplit_once('-').unwrap().0;
        assert_eq!(body.split('-').count(), 2);
        let decoded = hce.decode(&encoded).unwrap();
        assert_eq!(decoded, test_uuid_v7().to_vec());
    }

    #[test]
    fn recover_corrected_contains_tokens() {
        use crate::constants;
        let key = b"recover-corrected-key-32-bytes";
        let cm = ConfusionMatrix::universal();
        let r = Recoverer::new(&cm);
        let sl = constants::compute_syll_len(128);

        for seed in 0..3000u64 {
            let p = ((seed as u128) << 64) | seed as u128;
            let body = encode::encode_tokens(p, sl);
            let chk = check::compute_check(key, &body, 1);

            let mut found = None;
            for (i, t) in body.iter().enumerate() {
                if t.is_onset() && t.index() == 4 {
                    found = Some((i, t.index()));
                    break;
                }
            }
            if let Some((pos, _)) = found {
                let mut tampered = body;
                tampered[pos] = Token::onset(5);
                let result = r.recover(key, &tampered, &chk);
                match result {
                    RecoveryResult::Corrected(ref corrected) => {
                        let val = encode::decode_tokens(corrected);
                        assert_eq!(val, p, "corrected value mismatch at seed {}", seed);
                        return;
                    }
                    RecoveryResult::Reject => continue,
                    _ => {
                        assert!(
                            matches!(result, RecoveryResult::Corrected(_)),
                            "seed {}: unexpected result",
                            seed
                        );
                    }
                }
            }
        }
        panic!("no recoverable payload found");
    }

    #[test]
    fn bit_width_32_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_bit_width(32);
        let data: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];
        let encoded = hce.encode(&data);
        let decoded = hce.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 16);
        assert_eq!(&decoded[12..16], &data[..]);
    }

    #[test]
    fn custom_cipher_different_output() {
        let hfeistel = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let hshuffle = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_cipher(alloc::sync::Arc::new(FixedShuffle::new()));
        let uuid = test_uuid_v7();
        let e1 = hfeistel.encode(&uuid);
        let e2 = hshuffle.encode(&uuid);
        assert_ne!(
            e1, e2,
            "FixedShuffle must produce different output from Feistel"
        );
        assert_eq!(
            hshuffle.decode(&e2).unwrap(),
            uuid.to_vec(),
            "FixedShuffle roundtrip"
        );
    }

    #[test]
    fn with_modulus_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_modulus(1_000_000);
        let data = [0u8, 0u8, 42u8];
        let encoded = hce.encode(&data);
        let decoded = hce.decode(&encoded).unwrap();
        assert_eq!(
            decoded.last(),
            Some(&42),
            "modulus roundtrip: enc={:?}",
            encoded
        );
    }

    #[test]
    fn modulus_boundaries() {
        let key = Some(test_key());
        for modulus in [256u128, 1_000, 1_000_000, 1_000_000_000_000] {
            let hce =
                Hce::new(key, LanguageLevel::Universal, HceMode::Sealed).with_modulus(modulus);
            for &val in &[0u8, 1, 255] {
                let enc = hce.encode(&[val]);
                let dec = hce.decode(&enc).unwrap();
                assert_eq!(*dec.last().unwrap(), val, "modulus={} val={}", modulus, val);
            }
        }
    }

    #[test]
    fn modulus_all_modes() {
        let key = Some(test_key());
        let hce_s = Hce::new(key, LanguageLevel::Universal, HceMode::Sealed).with_modulus(10_000);
        let hce_o = Hce::new(key, LanguageLevel::Universal, HceMode::Open).with_modulus(10_000);
        let hce_p = Hce::new(None, LanguageLevel::Universal, HceMode::Plain).with_modulus(10_000);

        for hce in [&hce_s, &hce_o, &hce_p] {
            let enc = hce.encode(&[7u8]);
            let dec = hce.decode(&enc).unwrap();
            assert_eq!(dec.last(), Some(&7));
        }
    }

    #[test]
    fn custom_cipher_all_domains() {
        let key = Some(test_key());
        for bw in [64u32, 128] {
            let h = Hce::new(key, LanguageLevel::Universal, HceMode::Sealed)
                .with_bit_width(bw)
                .with_cipher(alloc::sync::Arc::new(FixedShuffle::with_domain(
                    Domain::Bits(bw),
                )));
            let byte_len = (bw.div_ceil(8)) as usize;
            let input: alloc::vec::Vec<u8> = (0..byte_len).map(|i| (i + 1) as u8).collect();
            let enc = h.encode(&input);
            let dec = h.decode(&enc).unwrap();
            assert_eq!(&dec[16 - byte_len..], &input[..], "shuffle bw={}", bw);
        }
    }

    #[test]
    fn feistel_upper_bits_preserved() {
        let f = Feistel::with_domain(b"32-bytes-key-for-testing-hmac!!", Domain::Bits(128));
        let p = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128;
        assert_eq!(f.decrypt(f.encrypt(p, b"t"), b"t"), p);
    }

    #[test]
    fn encode_zero_length_input() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let encoded = hce.encode(&[]);
        assert!(!encoded.is_empty(), "empty input should still encode");
        let decoded = hce.decode(&encoded).unwrap();
        assert!(!decoded.is_empty());
    }

    #[test]
    fn max_value_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed);
        let data = [0xFFu8; 16];
        let enc = hce.encode(&data);
        assert_eq!(hce.decode(&enc).unwrap(), data.to_vec());
    }

    #[test]
    fn custom_cipher_preserves_tweak() {
        let h1 = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed).with_cipher(
            alloc::sync::Arc::new(FixedShuffle::with_domain(Domain::Bits(128))),
        );
        let h2 = Hce::new(Some(test_key()), LanguageLevel::Eu, HceMode::Sealed).with_cipher(
            alloc::sync::Arc::new(FixedShuffle::with_domain(Domain::Bits(128))),
        );
        let uuid = test_uuid_v7();
        let e1 = h1.encode(&uuid);
        let e2 = h2.encode(&uuid);
        assert_ne!(e1, e2, "different tweaks must produce different output");
    }

    #[test]
    fn every_bit_width_roundtrip() {
        let key = Some(test_key());
        for bw in [
            16u32, 17, 24, 31, 32, 33, 48, 63, 64, 65, 80, 96, 112, 127, 128,
        ] {
            let hce = Hce::new(key, LanguageLevel::Universal, HceMode::Sealed).with_bit_width(bw);
            let byte_len = (bw.div_ceil(8)) as usize;
            let input: alloc::vec::Vec<u8> = (0..byte_len)
                .map(|i| (i as u8).wrapping_mul(17).wrapping_add(1))
                .collect();
            let enc = hce.encode(&input);
            let dec = hce.decode(&enc).unwrap();
            assert_eq!(&dec[16 - byte_len..], &input[..], "bw={}", bw);
        }
    }

    #[test]
    fn unsigned_overflow_guard() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Sealed)
            .with_modulus(u128::MAX - 1);
        let data = [0x7Fu8; 16];
        let enc = hce.encode(&data);
        let dec = hce.decode(&enc).unwrap();
        assert_eq!(dec, data.to_vec());
    }
}
