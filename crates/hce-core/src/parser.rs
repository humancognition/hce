use crate::pools;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidOnset(usize),
    InvalidVnuc(usize),
    EmptyInput,
}

#[derive(Clone)]
pub struct Parser;

impl Parser {
    pub const fn new() -> Self {
        Parser
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

        if let Some(idx) = pools::onset_index_from_chars(first, second) {
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
        if !pools::is_vowel_char(vowel) {
            return None;
        }

        if pos + 1 >= n {
            return pools::vnuc_index_from_chars(vowel, None).map(|idx| (Token::vnuc(idx), 1));
        }

        let next = chars[pos + 1];
        if !pools::is_coda_char(next) {
            return pools::vnuc_index_from_chars(vowel, None).map(|idx| (Token::vnuc(idx), 1));
        }

        if pos + 2 >= n {
            return pools::vnuc_index_from_chars(vowel, Some(next))
                .map(|idx| (Token::vnuc(idx), 2));
        }

        let after_next = chars[pos + 2];
        if pools::is_consonant_char(after_next) {
            return pools::vnuc_index_from_chars(vowel, Some(next))
                .map(|idx| (Token::vnuc(idx), 2));
        }

        pools::vnuc_index_from_chars(vowel, None).map(|idx| (Token::vnuc(idx), 1))
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pools;

    #[test]
    fn parse_single_syllable() {
        let p = Parser::new();
        let tokens = p.parse("ko").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(tokens[0].is_onset());
        assert!(tokens[1].is_vnuc());
    }

    #[test]
    fn parse_with_coda() {
        let p = Parser::new();
        let tokens = p.parse("kon").unwrap();
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn parse_cluster_onset() {
        let p = Parser::new();
        let tokens = p.parse("tran").unwrap();
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn parse_multiple_syllables() {
        let p = Parser::new();
        let tokens = p.parse("kobra").unwrap();
        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn coda_s_not_mistaken_for_next_onset_start() {
        let p = Parser::new();
        assert!(p.parse("kas").is_ok());
    }

    #[test]
    fn coda_n_followed_by_consonant() {
        let p = Parser::new();
        let tokens = p.parse("kanko").unwrap();
        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn invalid_onset_rejected() {
        assert!(Parser::new().parse("x").is_err());
    }

    #[test]
    fn invalid_vnuc_rejected() {
        assert!(Parser::new().parse("kx").is_err());
    }

    #[test]
    fn empty_input_errors() {
        assert_eq!(Parser::new().parse(""), Err(ParseError::EmptyInput));
    }

    #[test]
    fn coda_l_before_onset() {
        let t = Parser::new().parse("kalko").unwrap();
        assert_eq!(t.len(), 4);
    }

    #[test]
    fn coda_r_before_onset() {
        let t = Parser::new().parse("karko").unwrap();
        assert_eq!(t.len(), 4);
    }

    #[test]
    fn all_onset_tokens_parse() {
        let p = Parser::new();
        for i in 0u8..15 {
            let s = pools::onset_str(i);
            let t = p.parse(&alloc::format!("{}a", s)).unwrap();
            assert_eq!(t[0].index(), i);
        }
        for i in 15u8..29 {
            let s = pools::onset_str(i);
            let t = p.parse(&alloc::format!("{}a", s)).unwrap();
            assert_eq!(t[0].index(), i);
        }
    }

    #[test]
    fn all_vnuc_tokens_parse() {
        let p = Parser::new();
        for i in 0u8..5 {
            let s = pools::vnuc_str(i);
            let t = p.parse(&alloc::format!("k{}", s)).unwrap();
            assert_eq!(t[1].index(), i);
        }
        for i in 5u8..20 {
            let s = pools::vnuc_str(i);
            let t = p.parse(&alloc::format!("k{}", s)).unwrap();
            assert_eq!(t[1].index(), i);
        }
    }
}
