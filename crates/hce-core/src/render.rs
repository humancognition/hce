use crate::chunk::{ChunkSpec, ChunkStrategy};
use crate::token::Token;

fn push_uppercase(out: &mut alloc::string::String, s: &str) {
    for c in s.chars() {
        out.push(c.to_ascii_uppercase());
    }
}

fn push_lowercase(out: &mut alloc::string::String, s: &str) {
    for c in s.chars() {
        out.push(c.to_ascii_lowercase());
    }
}

fn push_case(out: &mut alloc::string::String, s: &str, case: crate::HceCase) {
    match case {
        crate::HceCase::Upper => push_uppercase(out, s),
        crate::HceCase::Lower => push_lowercase(out, s),
    }
}

pub fn render_body(
    tokens: &[Token],
    spec: &ChunkSpec,
    engine: &dyn ChunkStrategy,
    case: crate::HceCase,
    level: crate::LanguageLevel,
) -> alloc::string::String {
    let boundaries = engine.boundaries(tokens, spec);

    let n = tokens.len() / 2;
    let mut out = alloc::string::String::new();
    let mut b = 0;
    let sep = if spec.separator == '\0' {
        '\0'
    } else {
        spec.separator
    };
    let use_sep = spec.separator != '\0';

    for i in 0..n {
        if i > 0 && use_sep && b < boundaries.len() && boundaries[b] == i {
            out.push(sep);
            b += 1;
        }
        push_case(&mut out, tokens[i * 2].display_for(level), case);
        push_case(&mut out, tokens[i * 2 + 1].display_for(level), case);
    }
    out
}

pub fn render_check(
    tokens: &[Token],
    case: crate::HceCase,
    level: crate::LanguageLevel,
) -> alloc::string::String {
    let mut s = alloc::string::String::new();
    for i in 0..tokens.len() / 2 {
        push_case(&mut s, tokens[i * 2].display_for(level), case);
        push_case(&mut s, tokens[i * 2 + 1].display_for(level), case);
    }
    s
}
