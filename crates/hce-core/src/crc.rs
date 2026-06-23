use crate::constants;
use crate::token::Token;

pub fn compute_crc_check(body_tokens: &[Token], check_syllables: usize) -> alloc::vec::Vec<Token> {
    let norm = crate::token::tokens_to_string(body_tokens);
    let crc = crc32_ieee(norm.as_bytes());

    let mut out = alloc::vec::Vec::with_capacity(check_syllables * 2);
    let mut remaining = crc;
    for _ in 0..check_syllables {
        let o = (remaining % (constants::ONSET_RADIX as u32)) as u8;
        remaining /= constants::ONSET_RADIX as u32;
        let v = (remaining % (constants::VNUC_RADIX as u32)) as u8;
        remaining /= constants::VNUC_RADIX as u32;
        out.push(Token::onset(o));
        out.push(Token::vnuc(v));
    }
    out
}

pub fn verify_crc_check(body_tokens: &[Token], check_tokens: &[Token]) -> bool {
    let expected = compute_crc_check(body_tokens, check_tokens.len() / 2);
    *check_tokens == expected[..]
}

fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = if crc & 1 != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants;
    use crate::encode;

    #[test]
    fn matches() {
        let sl = constants::compute_syll_len(128);
        let body = encode::encode_tokens(42, sl);
        let chk = compute_crc_check(&body, 1);
        assert!(verify_crc_check(&body, &chk));
    }

    #[test]
    fn detects_tamper() {
        let sl = constants::compute_syll_len(128);
        let mut body = encode::encode_tokens(42, sl);
        let chk = compute_crc_check(&body, 1);
        body[1] = Token::vnuc((body[1].index() + 1) % constants::VNUC_RADIX);
        assert!(!verify_crc_check(&body, &chk));
    }
}
