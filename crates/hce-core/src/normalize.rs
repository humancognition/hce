use crate::parser::{ParseError, Parser};
use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizeError {
    Parse(ParseError),
    TooFewTokens,
}

#[derive(Clone)]
pub struct Normalizer {
    parser: Parser,
}

impl Normalizer {
    pub const fn new() -> Self {
        Normalizer {
            parser: Parser::new(),
        }
    }

    pub fn normalize(
        &self,
        input: &str,
        body_tokens: usize,
        check_tokens: usize,
    ) -> Result<(alloc::vec::Vec<Token>, alloc::vec::Vec<Token>), NormalizeError> {
        let cleaned: alloc::string::String = input
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_lowercase())
            .collect();

        let all_tokens = self.parser.parse(&cleaned).map_err(NormalizeError::Parse)?;

        let min = body_tokens + check_tokens;
        if all_tokens.len() < min {
            return Err(NormalizeError::TooFewTokens);
        }

        let check_start = all_tokens.len() - check_tokens;
        let body_start = check_start - body_tokens;

        let body: alloc::vec::Vec<Token> = all_tokens[body_start..check_start].to_vec();
        let check: alloc::vec::Vec<Token> = all_tokens[check_start..].to_vec();

        Ok((body, check))
    }
}

impl Default for Normalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    #[test]
    fn strips_hyphens_and_case() {
        let input = "Ab-KoNa-TreS";
        let cleaned: String = input
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_lowercase())
            .collect();
        assert_eq!(cleaned, "abkonatres");
    }

    #[test]
    fn normalize_roundtrip() {
        use super::Normalizer;
        use crate::constants;
        use crate::encode;
        let n = Normalizer::new();
        let sl = constants::compute_syll_len(128);
        let body = encode::encode_tokens(42, sl);
        let post: String = body.iter().map(|t| t.display()).collect();
        let input = alloc::format!("{}-ba", post);
        let (parsed_body, parsed_check) = n.normalize(&input, body.len(), 2).unwrap();
        assert_eq!(parsed_body, body);
        assert_eq!(parsed_check.len(), 2);
    }
}
