use super::feistel::Feistel;
use super::{Domain, Fpe};

#[derive(Debug, Clone)]
pub struct FixedShuffle {
    inner: Feistel,
}

impl FixedShuffle {
    pub fn new() -> Self {
        FixedShuffle {
            inner: Feistel::with_domain(&crate::FIXED_PLAIN_KEY, Domain::Bits(128)),
        }
    }

    pub fn with_domain(domain: Domain) -> Self {
        FixedShuffle {
            inner: Feistel::with_domain(&crate::FIXED_PLAIN_KEY, domain),
        }
    }
}

impl Fpe for FixedShuffle {
    fn encrypt(&self, plain: u128, tweak: &[u8]) -> u128 {
        self.inner.encrypt(plain, tweak)
    }

    fn decrypt(&self, cipher: u128, tweak: &[u8]) -> u128 {
        self.inner.decrypt(cipher, tweak)
    }
}

impl Default for FixedShuffle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let s = FixedShuffle::new();
        for v in [0u128, 1, u128::MAX] {
            assert_eq!(s.decrypt(s.encrypt(v, b""), b""), v);
        }
    }

    #[test]
    fn not_identity() {
        let s = FixedShuffle::new();
        assert_ne!(s.encrypt(0, b""), 0);
        assert_ne!(s.encrypt(1, b""), 1);
    }

    #[test]
    fn batch_roundtrip() {
        let s = FixedShuffle::new();
        let mut seed = 0xABCDu64;
        for _ in 0..1000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let v = ((seed as u128) << 64) | (seed as u128);
            assert_eq!(s.decrypt(s.encrypt(v, b""), b""), v);
        }
    }

    #[test]
    fn implements_fpe_trait() {
        let s: alloc::sync::Arc<dyn Fpe + Send + Sync> = alloc::sync::Arc::new(FixedShuffle::new());
        assert_eq!(s.decrypt(s.encrypt(42, b""), b""), 42);
    }
}
