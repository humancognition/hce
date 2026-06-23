mod feistel;
mod shuffle;

pub use feistel::Feistel;
pub use shuffle::FixedShuffle;

pub trait Fpe: Send + Sync {
    fn encrypt(&self, plain: u128, tweak: &[u8]) -> u128;
    fn decrypt(&self, cipher: u128, tweak: &[u8]) -> u128;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Bits(u32),
    Modulus(u128),
}

impl Domain {
    pub fn max_value(self) -> u128 {
        match self {
            Domain::Bits(n) if n >= 128 => u128::MAX,
            Domain::Bits(n) => (1u128 << n) - 1,
            Domain::Modulus(m) if m > 0 => m - 1,
            Domain::Modulus(_) => 1,
        }
    }
}

pub fn build_tweak(level_byte: u8, mode_byte: u8, domain: Domain) -> [u8; 4] {
    let (d, dt) = match domain {
        Domain::Bits(n) => (n as u8, 0u8),
        Domain::Modulus(m) => (128u32.saturating_sub(m.leading_zeros()) as u8, 1u8),
    };
    [level_byte, mode_byte, d, dt]
}
