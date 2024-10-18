extern crate splitbits;

use splitbits::splitbits_named_into;

#[test]
fn named_existing_types() {
    let greatest: u128; let of: u8; let all: u32; let time: u16;
    (greatest, of, all, time) = splitbits_named_into!(
        0b1101110111110001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(greatest, 0b110111011u128);
    assert_eq!(of, 0b111u8);
    assert_eq!(all, 0u32);
    assert_eq!(time, 0b001u16);

    let (greatest, of, all, time): (u128, u8, u32, u16) = splitbits_named_into!(
        0b1101110111110001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(greatest, 0b110111011u128);
    assert_eq!(of, 0b111u8);
    assert_eq!(all, 0u32);
    assert_eq!(time, 0b001u16);
}
