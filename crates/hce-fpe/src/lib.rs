extern crate alloc;

use alloc::sync::Arc;
use hce_core::{Domain, Feistel, FixedShuffle, Fpe};

pub enum CipherKind {
    Feistel8,
    Shuffle4,
}

pub fn build_cipher(
    kind: CipherKind,
    key: Option<&[u8]>,
    domain: Domain,
) -> Arc<dyn Fpe + Send + Sync> {
    match kind {
        CipherKind::Feistel8 => {
            let k = key.unwrap_or(&hce_core::FIXED_PLAIN_KEY);
            Arc::new(Feistel::with_domain(k, domain))
        }
        CipherKind::Shuffle4 => Arc::new(FixedShuffle::with_domain(domain)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feistel8_roundtrip() {
        let f = build_cipher(
            CipherKind::Feistel8,
            Some(b"test-32-byte-key-for-feistel!!!"),
            Domain::Bits(128),
        );
        let p = 0xDEADBEEFu128;
        assert_eq!(f.decrypt(f.encrypt(p, b"t"), b"t"), p);
    }

    #[test]
    fn shuffle4_roundtrip() {
        let f = build_cipher(CipherKind::Shuffle4, None, Domain::Bits(128));
        let p = 0xABCDEFu128;
        assert_eq!(f.decrypt(f.encrypt(p, b""), b""), p);
    }

    #[test]
    fn different_kinds_produce_different_output() {
        let f1 = build_cipher(
            CipherKind::Feistel8,
            Some(b"key-32-bytes-for-testing--xyz!!"),
            Domain::Bits(128),
        );
        let f2 = build_cipher(CipherKind::Shuffle4, None, Domain::Bits(128));
        let p = 12345u128;
        assert_ne!(f1.encrypt(p, b"t"), f2.encrypt(p, b""));
    }

    #[test]
    fn feistel8_default_key_works() {
        let f = build_cipher(CipherKind::Feistel8, None, Domain::Bits(128));
        assert_eq!(f.decrypt(f.encrypt(0, b"x"), b"x"), 0);
    }

    #[test]
    fn shuffle4_respects_domain() {
        let f = build_cipher(CipherKind::Shuffle4, None, Domain::Bits(64));
        let p = 0x123456789ABCDEF0u128;
        let c = f.encrypt(p, b"");
        assert!(c < (1u128 << 64));
        assert_eq!(f.decrypt(c, b""), p);
    }
}
