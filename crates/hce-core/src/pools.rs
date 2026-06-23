include!(concat!(env!("OUT_DIR"), "/packs.rs"));

use crate::LanguageLevel;

static ONSET_SINGLE: [char; 15] = [
    'b', 'd', 'g', 'k', 'm', 'n', 'p', 't', 's', 'l', 'r', 'f', 'h', 'j', 'w',
];

static PARSER_CLUSTERS: [&str; 20] = [
    "bl", "br", "dr", "fl", "fr", "gl", "gr", "kl", "kr", "pl", "pr", "sl", "sr", "tr", "st", "sp",
    "sk", "sm", "sw", "tw",
];

static VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];

static CODA: [char; 3] = ['n', 'l', 'r'];

pub fn onset_str(idx: u8) -> &'static str {
    debug_assert!(
        (idx as usize) < universal::ONSET.len(),
        "onset idx {} out of range",
        idx
    );
    universal::ONSET.get(idx as usize).unwrap_or(&"")
}

pub fn vnuc_str(idx: u8) -> &'static str {
    debug_assert!(
        (idx as usize) < universal::VNUC.len(),
        "vnuc idx {} out of range",
        idx
    );
    universal::VNUC.get(idx as usize).unwrap_or(&"")
}

pub fn onset_str_for(idx: u8, level: LanguageLevel) -> &'static str {
    match level {
        LanguageLevel::Universal => universal::ONSET.get(idx as usize).unwrap_or(&""),
        LanguageLevel::Eu => eu::ONSET.get(idx as usize).unwrap_or(&""),
        LanguageLevel::En => en::ONSET.get(idx as usize).unwrap_or(&""),
        LanguageLevel::Numeric => numeric::ONSET.get(idx as usize).unwrap_or(&""),
    }
}

pub fn vnuc_str_for(idx: u8, _level: LanguageLevel) -> &'static str {
    vnuc_str(idx)
}

fn find_cluster_pos(cl: &str) -> Option<u8> {
    if let Some(pos) = universal::CLUSTERS.iter().position(|&c| c == cl) {
        return Some((pos + 15) as u8);
    }
    if let Some(pos) = eu::CLUSTERS.iter().position(|&c| c == cl) {
        return Some((pos + 15) as u8);
    }
    if let Some(pos) = en::CLUSTERS.iter().position(|&c| c == cl) {
        return Some((pos + 15) as u8);
    }
    if let Some(pos) = numeric::CLUSTERS.iter().position(|&c| c == cl) {
        return Some((pos + 15) as u8);
    }
    None
}

pub fn onset_index_from_chars(first: char, second: Option<char>) -> Option<u8> {
    if let Some(s) = second {
        for cl in PARSER_CLUSTERS.iter() {
            let mut chars = cl.chars();
            if chars.next() == Some(first) && chars.next() == Some(s) {
                if let Some(pos) = find_cluster_pos(cl) {
                    return Some(pos);
                }
            }
        }
    }
    for (i, &sc) in ONSET_SINGLE.iter().enumerate() {
        if first == sc {
            return Some(i as u8);
        }
    }
    None
}

pub fn vnuc_index_from_chars(first: char, second: Option<char>) -> Option<u8> {
    if let Some(s) = second {
        for (vi, &v) in VOWELS.iter().enumerate() {
            if v == first {
                for (ci, &c) in CODA.iter().enumerate() {
                    if c == s {
                        return Some((5 + vi * 3 + ci) as u8);
                    }
                }
            }
        }
    }
    for (i, &vc) in VOWELS.iter().enumerate() {
        if first == vc {
            return Some(i as u8);
        }
    }
    None
}

pub fn onset_index_from_str(s: &str) -> Option<u8> {
    if s.len() == 1 {
        let c = s.chars().next()?;
        for (i, &sc) in ONSET_SINGLE.iter().enumerate() {
            if c == sc {
                return Some(i as u8);
            }
        }
    }
    if s.len() == 2 {
        for cl in PARSER_CLUSTERS.iter() {
            if s == *cl {
                return find_cluster_pos(cl);
            }
        }
    }
    None
}

pub fn vnuc_index_from_str(s: &str) -> Option<u8> {
    if s.len() == 1 {
        let c = s.chars().next()?;
        for (i, &vc) in VOWELS.iter().enumerate() {
            if c == vc {
                return Some(i as u8);
            }
        }
    }
    if s.len() == 2 {
        let vc = s.chars().next()?;
        let cc = s.chars().nth(1)?;
        for (vi, &v) in VOWELS.iter().enumerate() {
            if v == vc {
                for (ci, &c) in CODA.iter().enumerate() {
                    if c == cc {
                        return Some((5 + vi * 3 + ci) as u8);
                    }
                }
            }
        }
    }
    None
}

pub fn is_coda_char(c: char) -> bool {
    CODA.contains(&c)
}

pub fn is_vowel_char(c: char) -> bool {
    VOWELS.contains(&c)
}

pub fn is_consonant_char(c: char) -> bool {
    ONSET_SINGLE.contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants;

    #[test]
    fn onset_display_covers_all() {
        for i in 0..constants::ONSET_RADIX {
            let s = onset_str(i);
            assert!(!s.is_empty(), "onset {} empty", i);
        }
    }

    #[test]
    fn vnuc_display_covers_all() {
        for i in 0..constants::VNUC_RADIX {
            let s = vnuc_str(i);
            assert!(!s.is_empty(), "vnuc {} empty", i);
        }
    }

    #[test]
    fn roundtrip_indices() {
        for i in 0..constants::ONSET_RADIX {
            let s = onset_str(i);
            let idx = onset_index_from_str(s).unwrap();
            assert_eq!(idx, i);
        }
        for i in 0..constants::VNUC_RADIX {
            let s = vnuc_str(i);
            let idx = vnuc_index_from_str(s).unwrap();
            assert_eq!(idx, i);
        }
    }
}
