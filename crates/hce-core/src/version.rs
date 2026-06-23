use crate::wire;

pub struct UuidInfo {
    pub payload: u128,
    pub ts_ms: Option<i64>,
    pub has_ts: bool,
    pub version: u8,
}

pub fn split_uuid(bytes: &[u8; 16]) -> UuidInfo {
    let version = (bytes[6] >> 4) & 0xF;
    let has_ts = matches!(version, 1 | 6 | 7);

    let ts_ms = if has_ts {
        Some(extract_timestamp(bytes, version))
    } else {
        None
    };

    UuidInfo {
        payload: wire::bytes_to_int(bytes),
        ts_ms,
        has_ts,
        version,
    }
}

fn extract_timestamp(bytes: &[u8; 16], version: u8) -> i64 {
    match version {
        7 => {
            let mut b = [0u8; 8];
            b[2..8].copy_from_slice(&bytes[0..6]);
            i64::from_be_bytes(b)
        }
        6 | 1 => {
            let time_low = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            let time_mid = u16::from_be_bytes([bytes[4], bytes[5]]);
            let time_hi = u16::from_be_bytes([bytes[6] & 0x0F, bytes[7]]);
            let ticks = ((time_low as u64) << 32) | ((time_mid as u64) << 16) | (time_hi as u64);
            (ticks as i64) / 10_000 - (crate::constants::UUID_V1_EPOCH / 10_000)
        }
        _ => 0,
    }
}

pub fn reassemble_uuid(payload: u128) -> [u8; 16] {
    wire::int_to_bytes(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v7_has_ts() {
        let uuid: [u8; 16] = [
            0x01, 0x95, 0xe3, 0xa0, 0x7c, 0x2e, 0x7b, 0x41, 0x8f, 0x3d, 0x9a, 0x6c, 0x1e, 0x0b,
            0x4d, 0x27,
        ];
        let info = split_uuid(&uuid);
        assert_eq!(info.version, 7);
        assert!(info.has_ts);
    }

    #[test]
    fn v4_no_ts() {
        let uuid: [u8; 16] = [
            0xf8, 0x1d, 0x4f, 0xae, 0x7d, 0xec, 0x4d, 0x10, 0xa7, 0x65, 0x00, 0xa0, 0xc9, 0x1e,
            0x6b, 0xf6,
        ];
        let info = split_uuid(&uuid);
        assert_eq!(info.version, 4);
        assert!(!info.has_ts);
    }

    #[test]
    fn roundtrip() {
        let uuid: [u8; 16] = [
            0xf8, 0x1d, 0x4f, 0xae, 0x7d, 0xec, 0x4d, 0x10, 0xa7, 0x65, 0x00, 0xa0, 0xc9, 0x1e,
            0x6b, 0xf6,
        ];
        assert_eq!(reassemble_uuid(wire::bytes_to_int(&uuid)), uuid);
    }
}
