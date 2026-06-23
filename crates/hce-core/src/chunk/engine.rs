use crate::chunk::strategy::ChunkStrategy;
use crate::chunk::{ChunkMode, ChunkSpec};
use crate::encode;
use crate::token::Token;

const MULT: u64 = 31;
const MODULUS: u64 = 97;

#[derive(Debug, Clone)]
pub struct DefaultEngine;

impl DefaultEngine {
    fn crypto_scores(tokens: &[Token], n: usize) -> alloc::vec::Vec<u64> {
        let mut crypto = alloc::vec::Vec::with_capacity(n);
        crypto.push(0);
        for i in 1..n {
            let prev = encode::syllable_stamp(&tokens[(i - 1) * 2..(i - 1) * 2 + 2]);
            let curr = encode::syllable_stamp(&tokens[i * 2..i * 2 + 2]);
            let score = (prev.wrapping_mul(MULT).wrapping_add(curr)) % MODULUS;
            crypto.push(score);
        }
        crypto
    }

    fn char_cumsum(tokens: &[Token], n: usize) -> alloc::vec::Vec<usize> {
        let mut cum = alloc::vec::Vec::with_capacity(n + 1);
        cum.push(0);
        for i in 0..n {
            let pair = &tokens[i * 2..i * 2 + 2];
            let cl = pair[0].display().len() + pair[1].display().len();
            cum.push(cum[i] + cl);
        }
        cum
    }
}

impl ChunkStrategy for DefaultEngine {
    fn boundaries(&self, tokens: &[Token], spec: &ChunkSpec) -> alloc::vec::Vec<usize> {
        match spec.mode {
            ChunkMode::Natural => self.boundaries_natural(tokens, spec),
            ChunkMode::Fixed => self.boundaries_fixed(tokens, spec),
            ChunkMode::None_ => alloc::vec::Vec::new(),
            ChunkMode::Pattern => self.boundaries_pattern(tokens, spec),
        }
    }
}

impl DefaultEngine {
    fn boundaries_natural(&self, tokens: &[Token], spec: &ChunkSpec) -> alloc::vec::Vec<usize> {
        let n = tokens.len() / 2;
        if n <= spec.target_k {
            return alloc::vec::Vec::new();
        }

        let crypto = Self::crypto_scores(tokens, n);
        let cumsum = Self::char_cumsum(tokens, n);
        let total_chars = cumsum[n];

        let mut boundaries = alloc::vec::Vec::new();
        let mut last: usize = 0;

        for s in 1..spec.target_k {
            let remaining = spec.target_k - s + 1;
            let remaining_chars = total_chars - cumsum[last];
            let ideal_per = remaining_chars / remaining;
            let margin = (ideal_per / 3).max(1);
            let lo_chars = if spec.min_chars > 0 {
                spec.min_chars
            } else {
                ideal_per.saturating_sub(margin)
            };
            let hi_chars = if spec.max_chars > 0 {
                spec.max_chars
            } else {
                ideal_per + margin
            };

            let mut best = last + 1;
            let mut best_score: u64 = 0;
            let ideal_syl = {
                let mut closest = last + 1;
                let mut closest_dist = usize::MAX;
                for pos in (last + 1)..n {
                    let chars = cumsum[pos] - cumsum[last];
                    let d = chars.abs_diff(ideal_per);
                    if d < closest_dist {
                        closest_dist = d;
                        closest = pos;
                    }
                }
                closest
            };

            for pos in (last + 1)..n {
                let chars_here = cumsum[pos] - cumsum[last];
                if chars_here < lo_chars || chars_here > hi_chars {
                    continue;
                }
                let dist = pos.abs_diff(ideal_syl);
                let boost = if dist == 0 {
                    spec.alpha
                } else if dist == 1 {
                    spec.alpha / 2
                } else {
                    0
                };
                let score = crypto[pos].saturating_add(boost as u64);
                if score > best_score || (score == best_score && pos < best) {
                    best_score = score;
                    best = pos;
                }
            }
            boundaries.push(best);
            last = best;
        }

        boundaries
    }

    fn boundaries_fixed(&self, tokens: &[Token], spec: &ChunkSpec) -> alloc::vec::Vec<usize> {
        let n = tokens.len() / 2;
        let char_size = spec.char_size.unwrap_or(5);
        let cumsum = Self::char_cumsum(tokens, n);

        let mut boundaries = alloc::vec::Vec::new();
        let mut target = char_size;

        loop {
            let mut pos: Option<usize> = None;
            let mut closest_dist = usize::MAX;
            for (i, &chars) in cumsum.iter().enumerate().skip(1) {
                if boundaries.contains(&i) {
                    continue;
                }
                let d = chars.abs_diff(target);
                if d < closest_dist {
                    closest_dist = d;
                    pos = Some(i);
                }
            }
            if let Some(p) = pos {
                boundaries.push(p);
                target = cumsum[p] + char_size;
            } else {
                break;
            }
            if target >= cumsum[n] {
                break;
            }
        }

        boundaries.sort_unstable();
        boundaries.dedup();
        boundaries
    }

    fn boundaries_pattern(&self, tokens: &[Token], spec: &ChunkSpec) -> alloc::vec::Vec<usize> {
        let n = tokens.len() / 2;
        let mut boundaries = alloc::vec::Vec::new();
        let mut pos: usize = 0;
        for &syls in &spec.pattern {
            pos += syls;
            if pos < n {
                boundaries.push(pos);
            }
        }
        boundaries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ChunkSpec;
    use crate::encode;

    #[test]
    fn natural_produces_k_groups() {
        let tokens = encode::encode_tokens(12345678901234567890u128, 14);
        let spec = ChunkSpec::natural();
        let engine = DefaultEngine;
        let b = engine.boundaries(&tokens, &spec);
        assert!(!b.is_empty(), "natural should produce boundaries");
    }

    #[test]
    fn fixed_produces_boundaries() {
        let tokens = encode::encode_tokens(42u128, 14);
        let spec = ChunkSpec::fixed(5);
        let engine = DefaultEngine;
        let b = engine.boundaries(&tokens, &spec);
        assert!(!b.is_empty(), "fixed should produce boundaries");
    }

    #[test]
    fn none_produces_empty() {
        let tokens = encode::encode_tokens(42u128, 14);
        let spec = ChunkSpec::none();
        let engine = DefaultEngine;
        let b = engine.boundaries(&tokens, &spec);
        assert!(b.is_empty());
    }

    #[test]
    fn pattern_produces_exact() {
        let tokens = encode::encode_tokens(42u128, 14);
        let spec = ChunkSpec::pattern(&[3, 3, 3, 3, 2]);
        let engine = DefaultEngine;
        let b = engine.boundaries(&tokens, &spec);
        assert_eq!(b, alloc::vec![3, 6, 9, 12]);
    }

    #[test]
    fn alpha_spectrum() {
        let tokens = encode::encode_tokens(12345678901234567890u128, 14);

        let b0 = DefaultEngine.boundaries(&tokens, &ChunkSpec::natural_with_alpha(0));
        let b100 = DefaultEngine.boundaries(&tokens, &ChunkSpec::natural_with_alpha(100));

        assert_eq!(b0.len(), 4);
        assert_eq!(b100.len(), 4);
        assert!(
            b0 != b100,
            "alpha=0 and alpha=100 should produce different boundaries"
        );
    }
}
