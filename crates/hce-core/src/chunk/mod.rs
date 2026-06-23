mod engine;
mod strategy;

pub use engine::DefaultEngine;
pub use strategy::ChunkStrategy;

use crate::pools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSpec {
    pub mode: ChunkMode,
    pub target_k: usize,
    pub char_size: Option<usize>,
    pub pattern: alloc::vec::Vec<usize>,
    pub min_chars: usize,
    pub max_chars: usize,
    pub alpha: u8,
    pub separator: char,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkMode {
    Natural,
    Fixed,
    None_,
    Pattern,
}

impl ChunkSpec {
    pub fn natural() -> Self {
        ChunkSpec {
            mode: ChunkMode::Natural,
            target_k: 5,
            char_size: None,
            pattern: alloc::vec::Vec::new(),
            min_chars: 0,
            max_chars: 0,
            alpha: 0,
            separator: '-',
        }
    }

    pub fn natural_with_alpha(alpha: u8) -> Self {
        let mut s = Self::natural();
        s.alpha = alpha.min(100);
        s
    }

    pub fn natural_with_separator(sep: char) -> Self {
        let mut s = Self::natural();
        s.separator = sep;
        s
    }

    pub fn fixed(chars_per_chunk: usize) -> Self {
        ChunkSpec {
            mode: ChunkMode::Fixed,
            target_k: 0,
            char_size: Some(chars_per_chunk),
            pattern: alloc::vec::Vec::new(),
            min_chars: 0,
            max_chars: 0,
            alpha: 0,
            separator: '-',
        }
    }

    pub fn none() -> Self {
        ChunkSpec {
            mode: ChunkMode::None_,
            target_k: 0,
            char_size: None,
            pattern: alloc::vec::Vec::new(),
            min_chars: 0,
            max_chars: 0,
            alpha: 0,
            separator: '\0',
        }
    }

    pub fn pattern(syl_per_chunk: &[usize]) -> Self {
        ChunkSpec {
            mode: ChunkMode::Pattern,
            target_k: 0,
            char_size: None,
            pattern: syl_per_chunk.to_vec(),
            min_chars: 0,
            max_chars: 0,
            alpha: 0,
            separator: '-',
        }
    }
}

pub fn validate_separator(sep: char) -> bool {
    if sep == '-' || sep == ' ' || sep == '.' || sep == '\0' {
        return true;
    }
    if sep.is_alphabetic() {
        return false;
    }
    if pools::is_coda_char(sep) || pools::is_vowel_char(sep) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natural_has_defaults() {
        let s = ChunkSpec::natural();
        assert_eq!(s.target_k, 5);
        assert_eq!(s.separator, '-');
    }

    #[test]
    fn validate_separator_rejects_letters() {
        assert!(!validate_separator('A'));
        assert!(!validate_separator('n'));
    }

    #[test]
    fn validate_separator_accepts_symbols() {
        assert!(validate_separator('-'));
        assert!(validate_separator('.'));
        assert!(validate_separator(' '));
    }
}
