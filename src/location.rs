#[derive(Clone, Copy)]
pub struct Location {
    len: u8,
    mask_offset: u8,
}

impl Location {
    // TODO: Better construction method?
    pub fn new(len: u8, mask_offset: u8) -> Location {
        Location { len, mask_offset }
    }

    pub fn len(self) -> u8 {
        self.len
    }

    pub fn mask_offset(self) -> u8 {
        self.mask_offset
    }

    pub fn to_mask(self) -> u128 {
        self.to_unshifted_mask() << self.mask_offset
    }

    pub fn to_unshifted_mask(self) -> u128 {
        2u128.pow(u32::from(self.len)) - 1
    }
}
