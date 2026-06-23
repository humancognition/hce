use crate::check;
use crate::confusion::ConfusionMatrix;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryResult {
    Ok,
    Corrected(alloc::vec::Vec<Token>),
    Ambiguous(usize),
    Reject,
}

#[derive(Debug)]
pub struct Recoverer<'a> {
    matrix: &'a ConfusionMatrix,
}

impl<'a> Recoverer<'a> {
    pub fn new(matrix: &'a ConfusionMatrix) -> Self {
        Recoverer { matrix }
    }

    pub fn recover(
        &self,
        key: &[u8],
        body_tokens: &[Token],
        check_tokens: &[Token],
    ) -> RecoveryResult {
        if check::verify_check(key, body_tokens, check_tokens) {
            return RecoveryResult::Ok;
        }

        let mut candidates: alloc::vec::Vec<alloc::vec::Vec<Token>> = alloc::vec::Vec::new();

        for i in 0..body_tokens.len() {
            let neighbors = self.matrix.neighbors(body_tokens[i]);
            for &neighbor in neighbors.iter() {
                let mut candidate: alloc::vec::Vec<Token> = body_tokens.to_vec();
                candidate[i] = neighbor;
                if check::verify_check(key, &candidate, check_tokens)
                    && !candidates.contains(&candidate)
                {
                    candidates.push(candidate);
                }
            }
        }

        for i in 0..check_tokens.len() {
            let neighbors = self.matrix.neighbors(check_tokens[i]);
            for &neighbor in neighbors.iter() {
                let mut check_candidate: alloc::vec::Vec<Token> = check_tokens.to_vec();
                check_candidate[i] = neighbor;
                if check::verify_check(key, body_tokens, &check_candidate) {
                    let body_vec: alloc::vec::Vec<Token> = body_tokens.to_vec();
                    if !candidates.contains(&body_vec) {
                        candidates.push(body_vec);
                    }
                }
            }
        }

        match candidates.len() {
            0 => RecoveryResult::Reject,
            1 => {
                let corrected = candidates.into_iter().next().expect("candidates non-empty");
                RecoveryResult::Corrected(corrected)
            }
            n => RecoveryResult::Ambiguous(n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;
    use crate::constants;
    use crate::encode;

    #[test]
    fn ok_when_valid() {
        let key = b"recover-test-key-32-bytes--xx";
        let sl = constants::compute_syll_len(128);
        let body = encode::encode_tokens(42, sl);
        let chk = check::compute_check(key, &body, 1);
        let cm = ConfusionMatrix::universal();
        let r = Recoverer::new(&cm);
        assert_eq!(r.recover(key, &body, &chk), RecoveryResult::Ok);
    }

    #[test]
    fn reject_when_tampered_badly() {
        let key = b"recover-test-key-32-bytes--xx";
        let sl = constants::compute_syll_len(128);
        let mut body = encode::encode_tokens(42, sl);
        let chk = check::compute_check(key, &body, 1);
        body[0] = Token::onset((body[0].index() + 5) % crate::constants::ONSET_RADIX);
        body[1] = Token::vnuc((body[1].index() + 5) % crate::constants::VNUC_RADIX);
        let cm = ConfusionMatrix::universal();
        let r = Recoverer::new(&cm);
        assert_eq!(r.recover(key, &body, &chk), RecoveryResult::Reject);
    }

    #[test]
    fn corrects_m_to_n_confusion() {
        let key = b"recover-test-key-32-bytes--xx";
        let cm = ConfusionMatrix::universal();
        let r = Recoverer::new(&cm);
        let sl = constants::compute_syll_len(128);

        for seed in 0..2000u64 {
            let p = ((seed as u128) << 64) | seed as u128;
            let body = encode::encode_tokens(p, sl);
            let chk = check::compute_check(key, &body, 1);

            let m_id = 4u8;
            let n_id = 5u8;

            let mut found = None;
            for (i, t) in body.iter().enumerate() {
                if t.is_onset() && t.index() == m_id {
                    found = Some(i);
                    break;
                }
            }

            if let Some(pos) = found {
                let mut tampered = body;
                tampered[pos] = Token::onset(n_id);
                let result = r.recover(key, &tampered, &chk);
                if matches!(result, RecoveryResult::Reject) {
                    continue;
                }
                assert!(
                    matches!(
                        result,
                        RecoveryResult::Corrected(_) | RecoveryResult::Ambiguous(_)
                    ),
                    "seed {}: expected corrected or ambiguous, got {:?}",
                    seed,
                    result
                );
                return;
            }
        }
        panic!("no payload with onset-m found");
    }

    #[test]
    fn recovers_vnuc_coda_confusion() {
        let key = b"recover-test-key-32-bytes--xx";
        let cm = ConfusionMatrix::universal();
        let r = Recoverer::new(&cm);
        let sl = constants::compute_syll_len(128);

        for seed in 0..3000u64 {
            let p = ((seed as u128) << 64) | seed as u128;
            let body = encode::encode_tokens(p, sl);
            let chk = check::compute_check(key, &body, 1);

            let an_idx = crate::pools::vnuc_index_from_str("an").unwrap();
            let mut found = None;
            for (i, t) in body.iter().enumerate() {
                if t.is_vnuc() && t.index() == an_idx {
                    found = Some(i);
                    break;
                }
            }

            if let Some(pos) = found {
                let al_idx = crate::pools::vnuc_index_from_str("al").unwrap();
                let mut tampered = body;
                tampered[pos] = Token::vnuc(al_idx);
                let result = r.recover(key, &tampered, &chk);
                if matches!(result, RecoveryResult::Reject) {
                    continue;
                }
                assert!(
                    matches!(
                        result,
                        RecoveryResult::Corrected(_) | RecoveryResult::Ambiguous(_)
                    ),
                    "seed {}: vnuc confusion not recovered, got {:?}",
                    seed,
                    result
                );
                return;
            }
        }
        panic!("no payload with vnuc-an found");
    }
}
