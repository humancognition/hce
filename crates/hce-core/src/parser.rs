use crate::pools;
use crate::token::Token;
use crate::LanguageLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidOnset(usize),
    InvalidVnuc(usize),
    EmptyInput,
}

#[derive(Clone)]
pub struct Parser {
    level: LanguageLevel,
}

impl Parser {
    pub fn new(level: LanguageLevel) -> Self {
        Parser { level }
    }

    pub fn parse(&self, input: &str) -> Result<alloc::vec::Vec<Token>, ParseError> {
        let chars: alloc::vec::Vec<char> = input.chars().collect();
        let n = chars.len();
        if n == 0 {
            return Err(ParseError::EmptyInput);
        }

        let mut tokens = alloc::vec::Vec::new();
        let mut pos: usize = 0;

        loop {
            if pos >= n {
                break;
            }
            if tokens.len() % 2 == 0 {
                if let Some((token, adv)) = self.try_onset(&chars, pos) {
                    tokens.push(token);
                    pos += adv;
                } else {
                    return Err(ParseError::InvalidOnset(pos));
                }
            } else {
                if let Some((token, adv)) = self.try_vnuc(&chars, pos) {
                    tokens.push(token);
                    pos += adv;
                } else {
                    return Err(ParseError::InvalidVnuc(pos));
                }
            }
        }
        Ok(tokens)
    }

    fn try_onset(&self, chars: &[char], pos: usize) -> Option<(Token, usize)> {
        let n = chars.len();
        if pos >= n {
            return None;
        }

        let first = chars[pos];
        let second = if pos + 1 < n {
            Some(chars[pos + 1])
        } else {
            None
        };

        if let Some(idx) = pools::onset_index_from_chars(first, second, self.level) {
            let adv = if idx >= 15 { 2 } else { 1 };
            return Some((Token::onset(idx), adv));
        }
        None
    }

    fn try_vnuc(&self, chars: &[char], pos: usize) -> Option<(Token, usize)> {
        let n = chars.len();
        if pos >= n {
            return None;
        }

        let vowel = chars[pos];
        if !pools::is_vowel_char(vowel, self.level) {
            return None;
        }

        if pos + 1 >= n {
            return pools::vnuc_index_from_chars(vowel, None, self.level)
                .map(|idx| (Token::vnuc(idx), 1));
        }

        let next = chars[pos + 1];
        if !pools::is_coda_char(next, self.level) {
            return pools::vnuc_index_from_chars(vowel, None, self.level)
                .map(|idx| (Token::vnuc(idx), 1));
        }

        if pos + 2 >= n {
            return pools::vnuc_index_from_chars(vowel, Some(next), self.level)
                .map(|idx| (Token::vnuc(idx), 2));
        }

        let after_next = chars[pos + 2];
        if pools::is_consonant_char(after_next, self.level) {
            return pools::vnuc_index_from_chars(vowel, Some(next), self.level)
                .map(|idx| (Token::vnuc(idx), 2));
        }

        pools::vnuc_index_from_chars(vowel, None, self.level).map(|idx| (Token::vnuc(idx), 1))
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new(LanguageLevel::Universal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pools;
    use crate::LanguageLevel;

    fn parser() -> Parser {
        Parser::new(LanguageLevel::Universal)
    }

    #[test]
    fn empty_input_errors() {
        assert_eq!(parser().parse(""), Err(ParseError::EmptyInput));
    }

    #[test]
    fn unknown_char_in_onset_rejected() {
        assert!(parser().parse("x").is_err());
    }

    #[test]
    fn vowel_in_onset_rejected() {
        assert!(parser().parse("aea").is_err());
    }

    #[test]
    fn consecutive_codas_rejected() {
        assert!(parser().parse("arn").is_err());
    }

    #[test]
    fn double_coda_fails() {
        assert!(parser().parse("banr").is_err());
    }

    #[test]
    fn exhaustive_2syll_roundtrip() {
        let p = parser();
        for o0 in 0u8..35 {
            for v0 in 0u8..20 {
                for o1 in 0u8..35 {
                    for v1 in 0u8..20 {
                        let s = alloc::format!(
                            "{}{}{}{}",
                            pools::onset_str(o0),
                            pools::vnuc_str(v0),
                            pools::onset_str(o1),
                            pools::vnuc_str(v1),
                        );
                        let tokens = p.parse(&s).unwrap();
                        assert_eq!(tokens.len(), 4);
                        assert!(tokens[0].is_onset());
                        assert!(tokens[1].is_vnuc());
                        assert_eq!(tokens[0].index(), o0);
                        assert_eq!(tokens[1].index(), v0);
                        assert_eq!(tokens[2].index(), o1);
                        assert_eq!(tokens[3].index(), v1);
                    }
                }
            }
        }
    }

    #[test]
    fn coda_r_absorbed_before_consonant() {
        let t = parser().parse("karko").unwrap();
        assert_eq!(t.len(), 4);
    }

    #[test]
    fn coda_n_not_absorbed_before_vowel() {
        assert!(parser().parse("kano").is_err());
    }

    #[test]
    fn onset_cluster_parsed_as_unit() {
        let t = parser().parse("stan").unwrap();
        assert_eq!(t.len(), 2);
        assert!(t[0].is_onset());
        assert_eq!(t[0].index(), 29);
    }
}
