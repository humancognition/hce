use super::Adapter;

pub struct UlidAdapter;

impl Adapter for UlidAdapter {
    fn bit_width(&self) -> u32 {
        128
    }

    fn to_bytes(&self, id: &[u8]) -> Vec<u8> {
        let mut buf = vec![0u8; 16];
        let n = id.len().min(16);
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
        if bytes.len() < 6 {
            return None;
        }
        let mut b = [0u8; 8];
        b[2..8].copy_from_slice(&bytes[0..6]);
        Some(i64::from_be_bytes(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HceCodec;
    use hce_core::{Hce, HceMode, LanguageLevel};

    #[test]
    fn ulid_roundtrip() {
        let hce = Hce::new(
            Some(b"adapter-test-key-32-bytes-here!"),
            LanguageLevel::Universal,
            HceMode::Sealed,
        );
        let codec = HceCodec::new(hce, UlidAdapter);
        let mut data = [0u8; 16];
        data[..6].copy_from_slice(&[0x01, 0x95, 0xe3, 0xa0, 0x7c, 0x2e]);
        let encoded = codec.encode(&data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(&decoded[..16], &data[..]);
    }

    #[test]
    fn ulid_syll_len() {
        assert_eq!(hce_core::compute_syll_len(128), 14);
    }
}
