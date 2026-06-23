use crate::constants;
use crate::token::Token;

pub fn encode_tokens(mut p: u128, syll_len: usize) -> alloc::vec::Vec<Token> {
    let mut tokens = alloc::vec::Vec::with_capacity(syll_len * 2);
    for _ in 0..syll_len {
        let o = (p % (constants::ONSET_RADIX as u128)) as u8;
        p /= constants::ONSET_RADIX as u128;
        let v = (p % (constants::VNUC_RADIX as u128)) as u8;
        p /= constants::VNUC_RADIX as u128;
        tokens.push(Token::onset(o));
        tokens.push(Token::vnuc(v));
    }
    tokens
}

pub fn decode_tokens(tokens: &[Token]) -> u128 {
    let syll_len = tokens.len() / 2;
    let mut p: u128 = 0;
    for s in (0..syll_len).rev() {
        let o = tokens[s * 2].index() as u128;
        let v = tokens[s * 2 + 1].index() as u128;
        p = p * (constants::VNUC_RADIX as u128) + v;
        p = p * (constants::ONSET_RADIX as u128) + o;
    }
    p
}

pub fn syllable_stamp(tokens: &[Token]) -> u64 {
    debug_assert!(
        tokens.len() >= 2,
        "syllable_stamp requires at least 2 tokens"
    );
    let o = tokens[0].index() as u64;
    let v = tokens[1].index() as u64;
    o * constants::VNUC_RADIX as u64 + v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_zero() {
        let t = encode_tokens(0, 14);
        assert_eq!(decode_tokens(&t), 0);
    }

    #[test]
    fn roundtrip_max_u128() {
        let t = encode_tokens(u128::MAX, 14);
        assert_eq!(decode_tokens(&t), u128::MAX);
    }

    #[test]
    fn roundtrip_batch() {
        let n = 500_000u64;
        let mut seed = 1u64;
        for _ in 0..n {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let p = ((seed as u128) << 64) | (seed as u128);
            let t = encode_tokens(p, 14);
            assert_eq!(decode_tokens(&t), p, "failed at seed {}", seed);
        }
    }

    #[test]
    fn deterministic() {
        let a = encode_tokens(42, 14);
        let b = encode_tokens(42, 14);
        assert_eq!(a, b);
    }

    #[test]
    fn monotonic_first() {
        assert_eq!(encode_tokens(0, 14)[0].index(), 0);
        assert_eq!(encode_tokens(1, 14)[0].index(), 1);
    }

    #[test]
    fn variable_syll_len() {
        for bits in &[64u32, 96, 128] {
            let sl = constants::compute_syll_len(*bits);
            let t = encode_tokens(42, sl);
            assert_eq!(t.len(), sl * 2);
            assert_eq!(decode_tokens(&t), 42);
        }
    }

    #[test]
    fn syll_len_math() {
        assert_eq!(constants::compute_syll_len(128), 14);
        assert_eq!(constants::compute_syll_len(64), 7);
        assert_eq!(constants::compute_syll_len(96), 11);
    }
}
