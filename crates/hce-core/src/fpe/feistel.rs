use super::{Domain, Fpe};
use crate::constants::FPE_ROUNDS;
use hmac::digest::KeyInit;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct Feistel {
    pre_keyed: HmacSha256,
    rounds: u8,
    half: u32,
    half_mask: u64,
    domain_max: u128,
}

impl core::fmt::Debug for Feistel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Feistel")
            .field("rounds", &self.rounds)
            .field("half", &self.half)
            .field("half_mask", &self.half_mask)
            .field("domain_max", &self.domain_max)
            .finish()
    }
}

impl Feistel {
    pub fn new(key: &[u8]) -> Self {
        Self::with_domain(key, Domain::Bits(128))
    }

    pub fn with_domain(key: &[u8], domain: Domain) -> Self {
        let half = match domain {
            Domain::Bits(n) => (n / 2).max(1),
            Domain::Modulus(m) => {
                let bits = 128 - m.leading_zeros();
                (bits / 2).max(1)
            }
        };
        let half_mask = if half >= 64 {
            u64::MAX
        } else {
            (1u64 << half) - 1
        };
        let domain_max = domain.max_value();

        Feistel {
            pre_keyed: HmacSha256::new_from_slice(key).expect("HMAC accepts any key length"),
            rounds: FPE_ROUNDS,
            half,
            half_mask,
            domain_max,
        }
    }

    pub fn with_rounds(key: &[u8], rounds: u8) -> Self {
        let mut f = Self::new(key);
        f.rounds = rounds;
        f
    }

    fn f(&self, round: u8, half_val: u64, tweak: &[u8]) -> u64 {
        let mut mac = self.pre_keyed.clone();
        mac.update(&[round]);
        mac.update(tweak);
        mac.update(&half_val.to_be_bytes());
        let result = mac.finalize().into_bytes();
        let wide = u64::from_be_bytes([
            result[0], result[1], result[2], result[3], result[4], result[5], result[6], result[7],
        ]);
        wide & self.half_mask
    }
}

impl Fpe for Feistel {
    fn encrypt(&self, plain: u128, tweak: &[u8]) -> u128 {
        let mask = (1u128 << self.half) - 1;
        let max = self.domain_max;
        let mut c = {
            let mut left = (plain >> self.half) as u64;
            let mut right = (plain & mask) as u64;
            for i in 0..self.rounds {
                let f = self.f(i, right, tweak);
                (left, right) = (right, left ^ f);
            }
            ((left as u128) << self.half) | (right as u128)
        };
        let mut attempts = 0;
        while c > max && attempts < 256 {
            let mut left = (c >> self.half) as u64;
            let mut right = (c & mask) as u64;
            for i in 0..self.rounds {
                let f = self.f(i, right, tweak);
                (left, right) = (right, left ^ f);
            }
            c = ((left as u128) << self.half) | (right as u128);
            attempts += 1;
        }
        debug_assert!(
            c <= max,
            "cycle-walk failed after {} attempts: c={} max={}",
            attempts,
            c,
            max
        );
        c
    }

    fn decrypt(&self, cipher: u128, tweak: &[u8]) -> u128 {
        let mask = (1u128 << self.half) - 1;
        let max = self.domain_max;
        let mut p = {
            let mut left = (cipher >> self.half) as u64;
            let mut right = (cipher & mask) as u64;
            for i in (0..self.rounds).rev() {
                let f = self.f(i, left, tweak);
                (left, right) = (right ^ f, left);
            }
            ((left as u128) << self.half) | (right as u128)
        };
        let mut attempts = 0;
        while p > max && attempts < 256 {
            let mut left = (p >> self.half) as u64;
            let mut right = (p & mask) as u64;
            for i in (0..self.rounds).rev() {
                let f = self.f(i, left, tweak);
                (left, right) = (right ^ f, left);
            }
            p = ((left as u128) << self.half) | (right as u128);
            attempts += 1;
        }
        debug_assert!(
            p <= max,
            "cycle-walk decrypt failed after {} attempts",
            attempts
        );
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_128() {
        let f = Feistel::new(b"32-bytes-key-for-testing-hmac!!");
        let p = 0x1234567890abcdef01234567890abcdeu128;
        assert_eq!(f.decrypt(f.encrypt(p, b"t"), b"t"), p);
    }

    #[test]
    fn roundtrip_64() {
        let f = Feistel::with_domain(b"32-bytes-key-for-testing-hmac!!", Domain::Bits(64));
        let p = 0x123456789abcdef0u128;
        assert_eq!(f.decrypt(f.encrypt(p, b"t"), b"t"), p);
        let c = f.encrypt(0, b"t");
        assert!(c <= Domain::Bits(64).max_value());
    }

    #[test]
    fn roundtrip_96() {
        let f = Feistel::with_domain(b"32-bytes-key-for-testing-hmac!!", Domain::Bits(96));
        let p = 0x123456789abcdef0u128;
        assert_eq!(f.decrypt(f.encrypt(p, b"t"), b"t"), p);
    }

    #[test]
    fn deterministic() {
        let f = Feistel::new(b"32-bytes-key-for-testing-hmac!!");
        let p = 42u128;
        assert_eq!(f.encrypt(p, b"t"), f.encrypt(p, b"t"));
    }

    #[test]
    fn different_tweak_different_output() {
        let f = Feistel::new(b"32-bytes-key-for-testing-hmac!!");
        let p = 42u128;
        assert_ne!(f.encrypt(p, b"a"), f.encrypt(p, b"b"));
    }

    #[test]
    fn batch_roundtrip() {
        let f = Feistel::new(b"32-bytes-key-for-testing-hmac!!");
        for i in 0..1000u128 {
            assert_eq!(f.decrypt(f.encrypt(i, b"x"), b"x"), i);
        }
    }

    #[test]
    fn random_batch_100k() {
        let f = Feistel::new(b"32-bytes-key-for-testing-hmac!!");
        let mut seed = 0x5EEDu64;
        for _ in 0..100_000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let p = ((seed as u128) << 64) | ((seed ^ 0xDEADBEEF) as u128);
            assert_eq!(f.decrypt(f.encrypt(p, b"x"), b"x"), p);
        }
    }

    #[test]
    fn distribution_is_uniform() {
        let f = Feistel::new(b"32-bytes-key-for-testing-hmac!!");
        let mut buckets = [0u32; 16];
        let mut seed = 0xABCDu64;
        for _ in 0..32_000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let p = ((seed as u128) << 64) | (seed as u128);
            let c = f.encrypt(p, b"t");
            let nibble = (c & 0xF) as usize;
            buckets[nibble] += 1;
        }
        let avg = 32_000 / 16;
        for &b in &buckets {
            let dev = b.abs_diff(avg);
            assert!(
                dev < avg / 5,
                "bucket deviation too large: {} vs avg {}",
                b,
                avg
            );
        }
    }

    #[test]
    fn modulus_domain_roundtrip() {
        let n: u128 = 1_000_000_000_000;
        let f = Feistel::with_domain(b"32-bytes-key-for-testing-hmac!!", Domain::Modulus(n));
        for &p in &[0u128, 1, n - 1, n / 2, 42] {
            let c = f.encrypt(p, b"t");
            assert!(c < n, "cipher {} >= modulus {}", c, n);
            assert_eq!(f.decrypt(c, b"t"), p);
        }
    }

    #[test]
    fn modulus_batch() {
        let n: u128 = 1_000_000;
        let f = Feistel::with_domain(b"32-bytes-key-for-testing-hmac!!", Domain::Modulus(n));
        for p in 0..5000u128 {
            let c = f.encrypt(p, b"t");
            assert!(c < n);
            assert_eq!(f.decrypt(c, b"t"), p);
        }
    }
}
