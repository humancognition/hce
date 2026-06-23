use hce_core::{ChunkSpec, Hce, HceCase, HceMode, LanguageLevel, RecoveryResult, TsGranularity};
use hce_fpe::{build_cipher, CipherKind};
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[link_section = "hce_metadata"]
#[no_mangle]
pub static HCE_META: [u8; 75] =
    *b"hce-core:0.1.0\0onset:29\0vnuc:20\0coda:nlr\0syll:14\0fpe:feistel8\0hmac:sha256\0\0";

#[wasm_bindgen]
pub struct HceCodec {
    inner: Hce,
}

#[wasm_bindgen]
impl HceCodec {
    #[wasm_bindgen(constructor)]
    pub fn new(key: Option<String>, level: String, mode: String) -> Result<HceCodec, JsValue> {
        let level = match level.as_str() {
            "universal" | "u" => LanguageLevel::Universal,
            "eu" | "e" => LanguageLevel::Eu,
            "en" => LanguageLevel::En,
            "numeric" | "n" => LanguageLevel::Numeric,
            _ => {
                return Err(JsValue::from_str(
                    "invalid level: use universal, eu, en, or numeric",
                ))
            }
        };
        let mode = match mode.as_str() {
            "sealed" | "s" => HceMode::Sealed,
            "open" | "o" => HceMode::Open,
            "plain" | "p" => HceMode::Plain,
            _ => {
                return Err(JsValue::from_str(
                    "invalid mode: use sealed, open, or plain",
                ))
            }
        };
        let key_bytes = match &key {
            Some(k)
                if k.len() >= 2 && k.len() % 2 == 0 && k.chars().all(|c| c.is_ascii_hexdigit()) =>
            {
                let n = k.len() / 2;
                let mut bytes = Vec::with_capacity(n);
                for i in 0..n {
                    let hex = &k[i * 2..i * 2 + 2];
                    bytes.push(
                        u8::from_str_radix(hex, 16)
                            .map_err(|_| JsValue::from_str("invalid hex key"))?,
                    );
                }
                Some(bytes)
            }
            Some(k) => Some(k.clone().into_bytes()),
            None => None,
        };
        Ok(HceCodec {
            inner: Hce::new(key_bytes.as_deref(), level, mode),
        })
    }

    #[wasm_bindgen(js_name = withBitWidth)]
    pub fn with_bit_width(mut self, bits: u32) -> HceCodec {
        self.inner = self.inner.with_bit_width(bits);
        self
    }

    #[wasm_bindgen(js_name = withDomainModulus)]
    pub fn with_domain_modulus(mut self, modulus: String) -> Result<HceCodec, JsValue> {
        let m = modulus
            .parse::<u128>()
            .map_err(|_| JsValue::from_str("invalid modulus: must be a decimal integer"))?;
        if m < 2 {
            return Err(JsValue::from_str("modulus must be at least 2"));
        }
        self.inner = self.inner.with_modulus(m);
        Ok(self)
    }

    #[wasm_bindgen(js_name = withCipher)]
    pub fn with_cipher(mut self, kind: String, key: Option<String>) -> Result<HceCodec, JsValue> {
        let ck = match kind.as_str() {
            "feistel" | "feistel8" => CipherKind::Feistel8,
            "shuffle" | "shuffle4" => CipherKind::Shuffle4,
            _ => {
                return Err(JsValue::from_str(
                    "unknown cipher: use 'feistel' or 'shuffle'",
                ))
            }
        };
        let key_bytes = key.as_deref().map(|k| k.as_bytes().to_vec());
        let raw_key = key_bytes.as_deref();
        let fpe = build_cipher(ck, raw_key, self.inner.domain());
        self.inner = self.inner.with_cipher(fpe);
        Ok(self)
    }

    #[wasm_bindgen(js_name = withCase)]
    pub fn with_case(mut self, uppercase: bool) -> HceCodec {
        self.inner = self.inner.with_case(if uppercase {
            HceCase::Upper
        } else {
            HceCase::Lower
        });
        self
    }

    #[wasm_bindgen(js_name = withCheckSyllables)]
    pub fn with_check_syllables(mut self, n: usize) -> HceCodec {
        self.inner = self.inner.with_check_syllables(n);
        self
    }

    #[wasm_bindgen(js_name = withTimestampConfig)]
    pub fn with_timestamp_config(
        mut self,
        epoch_ms: i64,
        granularity: String,
    ) -> Result<HceCodec, JsValue> {
        let g = match granularity.as_str() {
            "second" => TsGranularity::Second,
            "minute" => TsGranularity::Minute,
            "hour" => TsGranularity::Hour,
            "day" => TsGranularity::Day,
            "week" => TsGranularity::Week,
            "month" => TsGranularity::Month,
            _ => {
                return Err(JsValue::from_str(
                    "invalid granularity: use second, minute, hour, day, week, or month",
                ))
            }
        };
        self.inner = self.inner.with_timestamp_config(epoch_ms, g);
        Ok(self)
    }

    #[wasm_bindgen(js_name = withSeparator)]
    pub fn with_separator(mut self, sep: String) -> Result<HceCodec, JsValue> {
        let c = sep
            .chars()
            .next()
            .ok_or(JsValue::from_str("empty separator"))?;
        if !hce_core::validate_separator(c) {
            return Err(JsValue::from_str(
                "invalid separator: must be non-alphabetic, e.g. '-', '.', '_'",
            ));
        }
        self.inner = self
            .inner
            .with_chunk_spec(ChunkSpec::natural_with_separator(c));
        Ok(self)
    }

    #[wasm_bindgen(js_name = withChunkNone)]
    pub fn with_chunk_none(mut self) -> HceCodec {
        self.inner = self.inner.with_chunk_spec(ChunkSpec::none());
        self
    }

    #[wasm_bindgen(js_name = withChunkFixed)]
    pub fn with_chunk_fixed(mut self, char_size: usize) -> HceCodec {
        self.inner = self.inner.with_chunk_spec(ChunkSpec::fixed(char_size));
        self
    }

    #[wasm_bindgen(js_name = withChunkPattern)]
    pub fn with_chunk_pattern(mut self, pattern: Vec<usize>) -> HceCodec {
        self.inner = self.inner.with_chunk_spec(ChunkSpec::pattern(&pattern));
        self
    }

    pub fn encode(&self, data: &[u8]) -> String {
        self.inner.encode(data)
    }

    pub fn decode(&self, input: &str) -> Result<Vec<u8>, JsValue> {
        self.inner
            .decode(input)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    pub fn recover(&self, input: &str) -> Result<Vec<u8>, JsValue> {
        match self.inner.recover(input) {
            Ok(RecoveryResult::Ok) => self
                .inner
                .decode(input)
                .map_err(|e| JsValue::from_str(&format!("{}", e))),
            Ok(RecoveryResult::Corrected(tokens)) => self
                .inner
                .decode_corrected(&RecoveryResult::Corrected(tokens))
                .ok_or_else(|| JsValue::from_str("decode failed after correction")),
            Ok(RecoveryResult::Ambiguous(n)) => Err(JsValue::from_str(&format!("ambiguous:{}", n))),
            Ok(RecoveryResult::Reject) => Err(JsValue::from_str("reject")),
            Err(e) => Err(JsValue::from_str(&format!("{}", e))),
        }
    }
}
