use super::Adapter;

pub struct XidAdapter;

impl Adapter for XidAdapter {
    fn bit_width(&self) -> u32 {
        96
    }

    fn to_bytes(&self, id: &[u8]) -> Vec<u8> {
        let mut buf = vec![0u8; 12];
        let n = id.len().min(12);
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
        if bytes.len() < 4 {
            return None;
        }
        Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i64 * 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HceCodec;
    use hce_core::{Hce, HceMode, LanguageLevel};

    #[test]
    fn xid_roundtrip() {
        let hce = Hce::new(
            Some(b"adapter-test-key-32-bytes-here!"),
            LanguageLevel::Universal,
            HceMode::Sealed,
        );
        let codec = HceCodec::new(hce, XidAdapter);
        let data: [u8; 12] = [
            0x67, 0x5a, 0x1b, 0x2c, 0x8f, 0x3d, 0x9a, 0x6c, 0x1e, 0x0b, 0x4d, 0x27,
        ];
        let encoded = codec.encode(&data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 16);
        assert_eq!(&decoded[4..16], &data[..]);
    }

    #[test]
    fn xid_syll_len() {
        assert_eq!(hce_core::compute_syll_len(96), 11);
    }
}
