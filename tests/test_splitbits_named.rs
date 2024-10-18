extern crate splitbits;

use splitbits::splitbits_named;

#[test]
fn onefield() {
    let field = splitbits_named!(
        0b1101_1101_1000_0100_0000_0000_1111_1001,
         ".... bbbb bbbb bbbb bbbb bbb. .... ....",
    );

    assert_eq!(field, 0b110_1100_0010_0000_0000u32);
}

#[test]
fn named() {
    let (greatest, of, all, time) = splitbits_named!(
        0b1101110111110001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(greatest, 0b110111011u16);
    assert_eq!(of, 0b111u8);
    assert_eq!(all, false);
    assert_eq!(time, 0b001u8);
}

#[test]
fn named_preserve_ordering() {
    let (greatest, of, all, time) = splitbits_named!(0b1000_1011, "bdda.cc.");
    assert_eq!(greatest, true);
    assert_eq!(of, 0b00u8);
    assert_eq!(all, false);
    assert_eq!(time, 0b01u8);
}

#[test]
fn named_existing_variables() {
    let greatest; let of; let all; let time;
    (greatest, of, all, time) = splitbits_named!(
        0b1101110111110001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(greatest, 0b110111011u16);
    assert_eq!(of, 0b111u8);
    assert_eq!(all, false);
    assert_eq!(time, 0b001u8);
}
