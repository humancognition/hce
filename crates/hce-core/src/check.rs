use crate::constants;
use crate::token::Token;
use hmac::digest::KeyInit;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn compute_check(
    key: &[u8],
    body_tokens: &[Token],
    check_syllables: usize,
) -> alloc::vec::Vec<Token> {
    let n = if check_syllables > 8 {
        8
    } else {
        check_syllables
    };

    let norm_body = crate::token::tokens_to_string(body_tokens);

    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(norm_body.as_bytes());
    let digest = mac.finalize().into_bytes();

    let mut out = alloc::vec::Vec::with_capacity(n * 2);
    for j in 0..n {
        let o = u16::from_be_bytes([digest[4 * j], digest[4 * j + 1]]) as u32
            % (constants::ONSET_RADIX as u32);
        let v = u16::from_be_bytes([digest[4 * j + 2], digest[4 * j + 3]]) as u32
            % (constants::VNUC_RADIX as u32);
        out.push(Token::onset(o as u8));
        out.push(Token::vnuc(v as u8));
    }
    out
}

pub fn verify_check(key: &[u8], body_tokens: &[Token], check_tokens: &[Token]) -> bool {
    let expected = compute_check(key, body_tokens, check_tokens.len() / 2);
    check_tokens.len() == expected.len() && *check_tokens == expected[..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants;
    use crate::encode;

    #[test]
    fn matches() {
        let key = b"check-test-key--32-bytes-xxxxx";
        let body = encode::encode_tokens(42, constants::compute_syll_len(128));
        let chk = compute_check(key, &body, 1);
        assert!(verify_check(key, &body, &chk));
    }

    #[test]
    fn detects_tamper() {
        let key = b"check-test-key--32-bytes-xxxxx";
        let sl = constants::compute_syll_len(128);
        let mut body = encode::encode_tokens(42, sl);
        let chk = compute_check(key, &body, 1);
        body[0] = Token::onset((body[0].index() + 1) % constants::ONSET_RADIX);
        assert!(!verify_check(key, &body, &chk));
    }
}
