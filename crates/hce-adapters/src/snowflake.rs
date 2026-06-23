use super::Adapter;

pub struct SnowflakeAdapter {
    epoch_ms: i64,
}

impl SnowflakeAdapter {
    pub fn new(epoch_ms: i64) -> Self {
        SnowflakeAdapter { epoch_ms }
    }

    pub fn twitter() -> Self {
        Self::new(1_280_000_000_000)
    }

    pub fn discord() -> Self {
        Self::new(1_420_000_000_000)
    }
}

impl Adapter for SnowflakeAdapter {
    fn bit_width(&self) -> u32 {
        64
    }

    fn to_bytes(&self, id: &[u8]) -> Vec<u8> {
        let mut buf = vec![0u8; 8];
        let n = id.len().min(8);
        buf[..n].copy_from_slice(&id[..n]);
        buf
    }

    fn parse_bytes(&self, bytes: &[u8]) -> Vec<u8> {
        bytes.to_vec()
    }

    fn has_timestamp(&self) -> bool {
        true
    }

    fn extract_timestamp(&self, bytes: &[u8]) -> Option<i64> {
        if bytes.len() < 8 {
            return None;
        }
        let v = u64::from_be_bytes(bytes[..8].try_into().ok()?);
        let ts = (v >> 22) as i64;
        Some(ts + self.epoch_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HceCodec;
    use hce_core::{Hce, HceMode, LanguageLevel};

    #[test]
    fn snowflake_roundtrip() {
        let hce = Hce::new(
            Some(b"adapter-test-key-32-bytes-here!"),
            LanguageLevel::Universal,
            HceMode::Sealed,
        );
        let codec = HceCodec::new(hce, SnowflakeAdapter::twitter());
        let data: [u8; 8] = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = codec.encode(&data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 16);
        assert_eq!(&decoded[8..16], &data[..]);
    }

    #[test]
    fn snowflake_syll_len() {
        assert_eq!(hce_core::compute_syll_len(64), 7);
    }
}
