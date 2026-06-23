use hce_core::Hce;

pub trait Adapter: Send + Sync {
    fn bit_width(&self) -> u32;
    fn to_bytes(&self, id: &[u8]) -> Vec<u8>;
    fn parse_bytes(&self, bytes: &[u8]) -> Vec<u8>;
    fn has_timestamp(&self) -> bool;
    fn extract_timestamp(&self, bytes: &[u8]) -> Option<i64>;
}

pub struct HceCodec<A: Adapter> {
    hce: Hce,
    adapter: A,
}

impl<A: Adapter> HceCodec<A> {
    pub fn new(hce: Hce, adapter: A) -> Self {
        let hce = hce.with_bit_width(adapter.bit_width());
        HceCodec { hce, adapter }
    }

    pub fn encode(&self, id: &[u8]) -> String {
        let bytes = self.adapter.to_bytes(id);
        self.hce.encode(&bytes)
    }

    pub fn decode(&self, hce_str: &str) -> Result<Vec<u8>, hce_core::HceError> {
        self.hce.decode(hce_str)
    }
}

pub struct RawAdapter {
    bits: u32,
}

impl RawAdapter {
    pub fn new(bits: u32) -> Self {
        RawAdapter {
            bits: bits.min(128),
        }
    }
}

impl Adapter for RawAdapter {
    fn bit_width(&self) -> u32 {
        self.bits
    }

    fn to_bytes(&self, id: &[u8]) -> Vec<u8> {
        let byte_len = self.bits.div_ceil(8) as usize;
        let mut buf = vec![0u8; byte_len];
        let n = id.len().min(byte_len);
        buf[..n].copy_from_slice(&id[..n]);
        buf
    }

    fn parse_bytes(&self, bytes: &[u8]) -> Vec<u8> {
        bytes.to_vec()
    }

    fn has_timestamp(&self) -> bool {
        false
    }

    fn extract_timestamp(&self, _bytes: &[u8]) -> Option<i64> {
        None
    }
}

mod objectid;
mod snowflake;
mod ulid;
mod uuid;
mod xid;

pub use objectid::ObjectIdAdapter;
pub use snowflake::SnowflakeAdapter;
pub use ulid::UlidAdapter;
pub use uuid::UuidAdapter;
pub use xid::XidAdapter;

#[cfg(test)]
mod tests {
    use super::*;
    use hce_core::{HceMode, LanguageLevel};

    fn test_key() -> &'static [u8] {
        b"adapter-test-key-32-bytes-here!"
    }

    #[test]
    fn raw_64_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Plain);
        let codec = HceCodec::new(hce, RawAdapter::new(64));
        let data = [0x12u8, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let encoded = codec.encode(&data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 16);
        assert_eq!(&decoded[8..16], &data[..]);
    }

    #[test]
    fn raw_32_roundtrip() {
        let hce = Hce::new(Some(test_key()), LanguageLevel::Universal, HceMode::Plain);
        let codec = HceCodec::new(hce, RawAdapter::new(32));
        let data = [0xdeu8, 0xad, 0xbe, 0xef];
        let encoded = codec.encode(&data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 16);
        assert_eq!(&decoded[12..16], &data[..]);
    }
}
