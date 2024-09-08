#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Base {
    Binary = 2,
    Hexadecimal = 16,
}

impl Base {
    pub const fn bits_per_digit(self) -> usize {
        match self {
            Self::Binary => 1,
            Self::Hexadecimal => 4,
        }
    }
}
