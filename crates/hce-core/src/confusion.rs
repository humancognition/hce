use crate::token::Token;

#[derive(Debug, Clone)]
pub struct ConfusionMatrix {
    pairs: alloc::vec::Vec<(Token, alloc::vec::Vec<Token>)>,
}

impl ConfusionMatrix {
    pub fn universal() -> Self {
        let mut pairs = alloc::vec::Vec::new();

        add_sym(&mut pairs, Token::onset(4), Token::onset(5));
        add_sym(&mut pairs, Token::onset(0), Token::onset(6));
        add_sym(&mut pairs, Token::onset(1), Token::onset(7));
        add_sym(&mut pairs, Token::onset(9), Token::onset(10));
        add_sym(&mut pairs, Token::onset(2), Token::onset(3));

        for &onset_idx in &[15u8, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28] {
            let cluster = crate::pools::onset_str(onset_idx);
            if let (Some(first), Some(second)) = (cluster.chars().next(), cluster.chars().nth(1)) {
                let alt_second = if second == 'l' { 'r' } else { 'l' };
                let alt = alloc::format!("{}{}", first, alt_second);
                if let Some(alt_idx) = crate::pools::onset_index_from_str(&alt) {
                    add_sym(&mut pairs, Token::onset(onset_idx), Token::onset(alt_idx));
                }
            }
        }

        add_vnuc_sym(&mut pairs, "an", "al");
        add_vnuc_sym(&mut pairs, "al", "ar");
        add_vnuc_sym(&mut pairs, "en", "el");
        add_vnuc_sym(&mut pairs, "el", "er");
        add_vnuc_sym(&mut pairs, "in", "il");
        add_vnuc_sym(&mut pairs, "il", "ir");
        add_vnuc_sym(&mut pairs, "on", "ol");
        add_vnuc_sym(&mut pairs, "ol", "or");
        add_vnuc_sym(&mut pairs, "un", "ul");
        add_vnuc_sym(&mut pairs, "ul", "ur");

        ConfusionMatrix { pairs }
    }

    pub fn neighbors(&self, token: Token) -> &[Token] {
        for (k, v) in &self.pairs {
            if *k == token {
                return v;
            }
        }
        &[]
    }
}

fn add_sym(pairs: &mut alloc::vec::Vec<(Token, alloc::vec::Vec<Token>)>, a: Token, b: Token) {
    for (k, v) in pairs.iter_mut() {
        if *k == a && !v.contains(&b) {
            v.push(b);
        }
        if *k == b && !v.contains(&a) {
            v.push(a);
        }
    }
    if !pairs.iter().any(|(k, _)| *k == a) {
        pairs.push((a, alloc::vec![b]));
    }
    if !pairs.iter().any(|(k, _)| *k == b) {
        pairs.push((b, alloc::vec![a]));
    }
}

fn add_vnuc_sym(pairs: &mut alloc::vec::Vec<(Token, alloc::vec::Vec<Token>)>, a: &str, b: &str) {
    if let (Some(ai), Some(bi)) = (
        crate::pools::vnuc_index_from_str(a),
        crate::pools::vnuc_index_from_str(b),
    ) {
        add_sym(pairs, Token::vnuc(ai), Token::vnuc(bi));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pools;

    #[test]
    fn m_has_neighbor_n() {
        let cm = ConfusionMatrix::universal();
        assert!(!cm.neighbors(Token::onset(4)).is_empty());
    }

    #[test]
    fn is_nonempty() {
        let cm = ConfusionMatrix::universal();
        assert!(!cm.pairs.is_empty());
    }

    #[test]
    fn cluster_bl_has_neighbor_br() {
        let cm = ConfusionMatrix::universal();
        let bl_idx = pools::onset_index_from_str("bl").unwrap();
        let neighbors = cm.neighbors(Token::onset(bl_idx));
        let has_br = neighbors
            .iter()
            .any(|&t| t.index() == pools::onset_index_from_str("br").unwrap());
        assert!(has_br);
    }

    #[test]
    fn cluster_sl_has_neighbor_sr() {
        let cm = ConfusionMatrix::universal();
        let sl_idx = pools::onset_index_from_str("sl").unwrap();
        let neighbors = cm.neighbors(Token::onset(sl_idx));
        let has_sr = neighbors
            .iter()
            .any(|&t| t.index() == pools::onset_index_from_str("sr").unwrap());
        assert!(has_sr);
    }

    #[test]
    fn vnuc_an_has_neighbor_al() {
        let cm = ConfusionMatrix::universal();
        let an_idx = pools::vnuc_index_from_str("an").unwrap();
        let neighbors = cm.neighbors(Token::vnuc(an_idx));
        assert!(!neighbors.is_empty());
    }

    #[test]
    fn all_clusters_have_lr_swap() {
        let cm = ConfusionMatrix::universal();
        for idx in 15u8..29 {
            let s = pools::onset_str(idx);
            if s.len() != 2 {
                continue;
            }
            let neighbors = cm.neighbors(Token::onset(idx));
            if neighbors.is_empty() {
                continue;
            }
            let second = s.chars().nth(1).unwrap();
            let alt = if second == 'l' { 'r' } else { 'l' };
            let expected_second = neighbors.iter().any(|&t| {
                let ns = pools::onset_str(t.index());
                ns.len() == 2 && ns.chars().nth(1) == Some(alt)
            });
            assert!(
                expected_second,
                "cluster {} (idx {}) missing {}-swap neighbor",
                s, idx, alt
            );
        }
    }
}
