#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Base {
    Binary = 2,
    Hexadecimal = 16,
}

impl Base {
    pub fn bits_per_digit(self) -> usize {
        match self {
            Base::Binary => 1,
            Base::Hexadecimal => 4,
        }
    }
}
