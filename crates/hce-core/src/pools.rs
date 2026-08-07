include!(concat!(env!("OUT_DIR"), "/packs.rs"));
include!(concat!(env!("OUT_DIR"), "/parser_tables.rs"));

use crate::LanguageLevel;

macro_rules! with_level {
    ($level:expr, $table:ident) => {
        match $level {
            LanguageLevel::Universal => &parser_tables::universal::$table,
            LanguageLevel::Eu => &parser_tables::eu::$table,
            LanguageLevel::En => &parser_tables::en::$table,
            LanguageLevel::Numeric => &parser_tables::numeric::$table,
            LanguageLevel::Ko => &parser_tables::ko::$table,
        }
    };
}

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
        LanguageLevel::Ko => ko::ONSET.get(idx as usize).unwrap_or(&""),
    }
}

pub fn vnuc_str_for(idx: u8, level: LanguageLevel) -> &'static str {
    match level {
        LanguageLevel::Universal => universal::VNUC.get(idx as usize).unwrap_or(&""),
        LanguageLevel::Eu => eu::VNUC.get(idx as usize).unwrap_or(&""),
        LanguageLevel::En => en::VNUC.get(idx as usize).unwrap_or(&""),
        LanguageLevel::Numeric => numeric::VNUC.get(idx as usize).unwrap_or(&""),
        LanguageLevel::Ko => ko::VNUC.get(idx as usize).unwrap_or(&""),
    }
}

fn find_cluster_pos(cl: &str, level: LanguageLevel) -> Option<u8> {
    let clusters = with_level!(level, PARSER_CLUSTERS);
    clusters
        .iter()
        .position(|&c| c == cl)
        .map(|pos| (pos + 15) as u8)
}

pub fn onset_index_from_chars(
    first: char,
    second: Option<char>,
    level: LanguageLevel,
) -> Option<u8> {
    if let Some(s) = second {
        let clusters = with_level!(level, PARSER_CLUSTERS);
        for cl in clusters.iter() {
            let mut chars = cl.chars();
            if chars.next() == Some(first) && chars.next() == Some(s) {
                if let Some(pos) = find_cluster_pos(cl, level) {
                    return Some(pos);
                }
            }
        }
    }
    let singles = with_level!(level, ONSET_SINGLE);
    for (i, &sc) in singles.iter().enumerate() {
        if first == sc {
            return Some(i as u8);
        }
    }
    None
}

pub fn vnuc_index_from_chars(
    first: char,
    second: Option<char>,
    level: LanguageLevel,
) -> Option<u8> {
    let vowels = with_level!(level, VOWELS);
    let coda = with_level!(level, CODA);
    if let Some(s) = second {
        for (vi, &v) in vowels.iter().enumerate() {
            if v == first {
                for (ci, &c) in coda.iter().enumerate() {
                    if c == s {
                        return Some((5 + vi * 3 + ci) as u8);
                    }
                }
            }
        }
    }
    for (i, &vc) in vowels.iter().enumerate() {
        if first == vc {
            return Some(i as u8);
        }
    }
    None
}

pub fn onset_index_from_str(s: &str, level: LanguageLevel) -> Option<u8> {
    if s.len() == 1 {
        let c = s.chars().next()?;
        let singles = with_level!(level, ONSET_SINGLE);
        for (i, &sc) in singles.iter().enumerate() {
            if c == sc {
                return Some(i as u8);
            }
        }
    }
    if s.len() == 2 {
        let clusters = with_level!(level, PARSER_CLUSTERS);
        for cl in clusters.iter() {
            if s == *cl {
                return find_cluster_pos(cl, level);
            }
        }
    }
    None
}

pub fn vnuc_index_from_str(s: &str, level: LanguageLevel) -> Option<u8> {
    if s.len() == 1 {
        let c = s.chars().next()?;
        let vowels = with_level!(level, VOWELS);
        for (i, &vc) in vowels.iter().enumerate() {
            if c == vc {
                return Some(i as u8);
            }
        }
    }
    if s.len() == 2 {
        let vc = s.chars().next()?;
        let cc = s.chars().nth(1)?;
        let vowels = with_level!(level, VOWELS);
        let coda = with_level!(level, CODA);
        for (vi, &v) in vowels.iter().enumerate() {
            if v == vc {
                for (ci, &c) in coda.iter().enumerate() {
                    if c == cc {
                        return Some((5 + vi * 3 + ci) as u8);
                    }
                }
            }
        }
    }
    None
}

pub fn is_coda_char(c: char, level: LanguageLevel) -> bool {
    with_level!(level, CODA).contains(&c)
}

pub fn is_vowel_char(c: char, level: LanguageLevel) -> bool {
    with_level!(level, VOWELS).contains(&c)
}

pub fn is_consonant_char(c: char, level: LanguageLevel) -> bool {
    with_level!(level, ONSET_SINGLE).contains(&c)
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
            let idx = onset_index_from_str(s, LanguageLevel::Universal).unwrap();
            assert_eq!(idx, i);
        }
        for i in 0..constants::VNUC_RADIX {
            let s = vnuc_str(i);
            let idx = vnuc_index_from_str(s, LanguageLevel::Universal).unwrap();
            assert_eq!(idx, i);
        }
    }
}
