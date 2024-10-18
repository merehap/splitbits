extern crate splitbits;

use splitbits::{replacebits, replacehex};
use ux::{u4, u28};

#[test]
fn replace() {
    let a = 0b101u16;
    let b = 0b00001u8;
    let c = 0b0101u128;
    let d = false;
    let result = replacebits!(0b1001_1010_1100_1111u16, "aaab bbbb .d.. cccc");
    assert_eq!(result,        0b1010_0001_1000_0101u16);
}

#[test]
fn replace_var() {
    let var = 0b1001_1010_1100_1111u16;
    let a = 0b101u16;
    let b = 0b00001u8;
    let c = 0b0101u128;
    let d = false;
    let result = replacebits!(var, "aaab bbbb .d.. cccc");
    assert_eq!(result,        0b1010_0001_1000_0101u16);
}

#[test]
fn replace_with_literal() {
    let a = 0b101u16;
    let b = 0b00001u8;
    let c = 0b0101u128;
    let d = false;
    let result = replacebits!(0b1001_1010_1100_1111u16, "aaab bbbb 0d11 cccc");
    assert_eq!(result,        0b1010_0001_0011_0101u16);
}

#[test]
fn replace_segments() {
    let a = 0b101u16;
    let b = 0b00001u8;
    let c = 0b0101u128;
    let d = true;
    let result = replacebits!(0b1001_1010_1100_1111u16, "aabb bbab .c.. ccdc");
    assert_eq!(result,        0b1000_0011_1000_1011u16);
}

#[test]
fn replace_too_big() {
    let a = 0b110u16;
    let b = 0b00001u8;
    let c = 0b0101u128;
    let d = false;
    let result = replacebits!(0b0001_1010_1100_1111u16, ".aab bbbb .d.. cccc");
    assert_eq!(result,        0b0100_0001_1000_0101u16);
}

// Same behavior as default.
#[test]
fn replace_too_big_truncate() {
    let a = 0b110u16;
    let b = 0b00001u8;
    let c = 0b0101u128;
    let d = false;
    let result = replacebits!(overflow=truncate, 0b0001_1010_1100_1111u16, ".aab bbbb .d.. cccc");
    assert_eq!(result,                           0b0100_0001_1000_0101u16);
}

#[test]
#[should_panic(expected = "Variable a is too big for its location in the template. 0b110 > 0b11")]
fn replace_too_big_panic() {
    let a = 0b110u16;
    let b = 0b00001u8;
    let c = 0b0101u128;
    let d = false;
    let _ = replacebits!(overflow=panic, 0b0001_1010_1100_1111u16, ".aab bbbb .d.. cccc");
}

#[test]
fn replace_too_big_saturate() {
    let a = 0b110u16;
    let b = 0b00001u8;
    let c = 0b0101u128;
    let d = false;
    let result = replacebits!(overflow=saturate, 0b0001_1010_1100_1111u16, ".aab bbbb .d.. cccc");
    assert_eq!(result,                           0b0110_0001_1000_0101u16);
}

#[test]
fn replace_too_big_corrupt() {
    let a = 0b1101u16;
    let b = 0b00001u8;
    let c = 0b0101u128;
    let d = false;
    let result = replacebits!(overflow=corrupt, 0b0001_1010_1100_1111u16, ".aab bbbb .d.. cccc");
    assert_eq!(result,                          0b1010_0001_1000_0101u16);
}

#[test]
fn replacehex_ux() {
    let a = u4::new(0xE);
    let c: u8 = 0x2A;
    let b = u28::new(0x90210AB);
    let result = replacehex!(0xABCD_EF01_2345_6789, "0a.. cc.b bbbb bb1D");
    assert_eq!(result,       0x0ECD_2A09_0210_AB1Du64);
}
