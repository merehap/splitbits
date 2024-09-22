// The numeric base that the Template input data is in.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Base {
    Binary = 2,
    Hexadecimal = 16,
}

impl Base {
    // How many bits are needed to represent a single digit of this Base.
    pub const fn bits_per_digit(self) -> usize {
        match self {
            Self::Binary => 1,
            Self::Hexadecimal => 4,
        }
    }
}
