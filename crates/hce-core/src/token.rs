#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token(u8);

impl Token {
    pub const fn onset(idx: u8) -> Self {
        Token(idx)
    }

    pub const fn vnuc(idx: u8) -> Self {
        Token(idx | 0x80)
    }

    pub const fn is_onset(self) -> bool {
        self.0 & 0x80 == 0
    }

    pub const fn is_vnuc(self) -> bool {
        self.0 & 0x80 != 0
    }

    pub fn index(self) -> u8 {
        self.0 & 0x7F
    }

    pub fn display(&self) -> &'static str {
        if self.is_onset() {
            crate::pools::onset_str(self.index())
        } else {
            crate::pools::vnuc_str(self.index())
        }
    }

    pub fn display_for(&self, level: crate::LanguageLevel) -> &'static str {
        if self.is_onset() {
            crate::pools::onset_str_for(self.index(), level)
        } else {
            crate::pools::vnuc_str_for(self.index(), level)
        }
    }
}

pub fn tokens_to_string(tokens: &[Token]) -> alloc::string::String {
    let mut s = alloc::string::String::new();
    for t in tokens {
        s.push_str(t.display());
    }
    s
}
