pub const ONSET_RADIX: u8 = 35;
pub const VNUC_RADIX: u8 = 20;
pub const DEFAULT_BIT_WIDTH: u32 = 128;

pub const FPE_ROUNDS: u8 = 8;

pub const TS_EPOCH_MS: i64 = 1_700_000_000_000;
pub const UUID_V1_EPOCH: i64 = 0x01b2_1dd2_1381_4000i64;

pub fn compute_syll_len(bits: u32) -> usize {
    let cap = ONSET_RADIX as u128 * VNUC_RADIX as u128;
    let mut acc: u128 = 1;
    for s in 1..=128 {
        acc = match acc.checked_mul(cap) {
            Some(v) => v,
            None => return s,
        };
        let enough = match bits {
            0 => true,
            1..=64 => {
                acc > if bits == 64 {
                    u64::MAX as u128
                } else {
                    (1u128 << bits) - 1
                }
            }
            65..=127 => acc > (1u128 << bits) - 1,
            _ => acc > u128::MAX / 2 || acc == u128::MAX,
        };
        if enough {
            return s;
        }
    }
    128
}

pub fn body_tokens_for(bits: u32) -> usize {
    compute_syll_len(bits) * 2
}
