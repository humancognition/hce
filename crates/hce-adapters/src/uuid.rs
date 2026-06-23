use super::Adapter;

pub struct UuidAdapter;

impl Adapter for UuidAdapter {
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
        if bytes.len() < 16 {
            return None;
        }
        let version = (bytes[6] >> 4) & 0xF;
        match version {
            7 => {
                let mut b = [0u8; 8];
                b[2..8].copy_from_slice(&bytes[0..6]);
                Some(i64::from_be_bytes(b))
            }
            1 | 6 => {
                let time_low = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                let time_mid = u16::from_be_bytes([bytes[4], bytes[5]]);
                let time_hi = u16::from_be_bytes([bytes[6] & 0x0F, bytes[7]]);
                let ticks: u64 =
                    ((time_low as u64) << 32) | ((time_mid as u64) << 16) | (time_hi as u64);
                let epoch_ticks: u64 = 0x01b2_1dd2_1381_4000u64;
                let diff = if ticks >= epoch_ticks {
                    ((ticks - epoch_ticks) / 10_000) as i64
                } else {
                    -(((epoch_ticks - ticks) / 10_000) as i64)
                };
                Some(diff)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HceCodec;
    use hce_core::{Hce, HceMode, LanguageLevel};

    #[test]
    fn uuid_v7_roundtrip() {
        let hce = Hce::new(
            Some(b"adapter-test-key-32-bytes-here!"),
            LanguageLevel::Universal,
            HceMode::Sealed,
        );
        let codec = HceCodec::new(hce, UuidAdapter);
        let uuid: [u8; 16] = [
            0x01, 0x95, 0xe3, 0xa0, 0x7c, 0x2e, 0x7b, 0x41, 0x8f, 0x3d, 0x9a, 0x6c, 0x1e, 0x0b,
            0x4d, 0x27,
        ];
        let encoded = codec.encode(&uuid);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(&decoded[..16], &uuid[..]);
    }

    #[test]
    fn uuid_v4_roundtrip() {
        let hce = Hce::new(None, LanguageLevel::Universal, HceMode::Plain);
        let codec = HceCodec::new(hce, UuidAdapter);
        let uuid: [u8; 16] = [
            0xf8, 0x1d, 0x4f, 0xae, 0x7d, 0xec, 0x4d, 0x10, 0xa7, 0x65, 0x00, 0xa0, 0xc9, 0x1e,
            0x6b, 0xf6,
        ];
        let encoded = codec.encode(&uuid);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(&decoded[..16], &uuid[..]);
    }

    #[test]
    fn uuid_syll_len() {
        assert_eq!(hce_core::compute_syll_len(128), 14);
    }
}
