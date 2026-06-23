pub fn bytes_to_int(bytes: &[u8; 16]) -> u128 {
    let mut v: u128 = 0;
    let mut i = 0;
    while i < 16 {
        v = (v << 8) | (bytes[i] as u128);
        i += 1;
    }
    v
}

pub fn int_to_bytes(mut v: u128) -> [u8; 16] {
    let mut buf = [0u8; 16];
    let mut i: isize = 15;
    while i >= 0 {
        buf[i as usize] = (v & 0xFF) as u8;
        v >>= 8;
        i -= 1;
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_roundtrip() {
        assert_eq!(bytes_to_int(&[0u8; 16]), 0);
        assert_eq!(int_to_bytes(0), [0u8; 16]);
    }

    #[test]
    fn max_roundtrip() {
        let b = [0xFFu8; 16];
        assert_eq!(bytes_to_int(&b), u128::MAX);
        assert_eq!(int_to_bytes(u128::MAX), b);
    }

    #[test]
    fn known_vector() {
        let b: [u8; 16] = [
            0x01, 0x95, 0xe3, 0xa0, 0x7c, 0x2e, 0x7b, 0x41, 0x8f, 0x3d, 0x9a, 0x6c, 0x1e, 0x0b,
            0x4d, 0x27,
        ];
        let v = bytes_to_int(&b);
        assert_eq!(int_to_bytes(v), b);
    }
}
