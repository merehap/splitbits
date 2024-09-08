#[derive(Clone, Copy)]
pub struct Location {
    pub len: u8,
    pub mask_offset: u8,
}

impl Location {
    pub const fn len(self) -> u8 {
        self.len
    }

    pub const fn mask_offset(self) -> u8 {
        self.mask_offset
    }

    pub fn to_mask(self) -> u128 {
        self.to_unshifted_mask() << self.mask_offset
    }

    pub fn to_unshifted_mask(self) -> u128 {
        2u128.pow(u32::from(self.len)) - 1
    }
}
