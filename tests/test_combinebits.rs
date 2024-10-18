extern crate splitbits;

use splitbits::{combinebits, combinehex};
use ux::{u1, u4, u7, u9, u12};

#[test]
fn combine_overflow() {
    let a: u8 = 0b1010_0101;
    let result = combinebits!("0aaa aaaa");
    assert_eq!(result, 0b0010_0101);
}

// Same as default behavior.
#[test]
fn combine_overflow_truncate() {
    let a: u8 = 0b1010_0101;
    let result = combinebits!(overflow=truncate, "0aaa aaaa");
    assert_eq!(result, 0b0010_0101);
}

#[test]
#[should_panic(expected = "Variable a is too big for its location in the template. 0b10100101 > 0b1111111")]
fn combine_overflow_panic() {
    let a: u8 = 0b1010_0101;
    let _ = combinebits!(overflow=panic, "0aaa aaaa");
}

#[test]
fn combine_overflow_corrupt() {
    let a: u8 = 0b1010_0101;
    let result = combinebits!(overflow=corrupt, "0aaa aaaa");
    assert_eq!(result, 0b1010_0101);
}

#[test]
fn combine_overflow_saturate() {
    let a: u8 = 0b1010_0101;
    let result = combinebits!(overflow=saturate, "0aaa aaaa");
    assert_eq!(result, 0b0111_1111);
}

#[test]
fn combine_trivial() {
    let a: u16 = 0b1010_0101_0000_1111;
    let result = combinebits!("aaaa aaaa aaaa aaaa");
    assert_eq!(result, a);
}

#[test]
fn combine_two() {
    let a: u8 = 0b1010_0101;
    let b: u8 = 0b0000_1111;
    let result = combinebits!("aaaa aaaa bbbb bbbb");
    assert_eq!(result, 0b1010_0101_0000_1111u16);
}

#[test]
fn combine_into_multiple_segments() {
    let a: u8 = 0b1010_0101;
    let b: u8 = 0b0000_1111;
    let result = combinebits!("aaaa bbbb abbb aaba");
    assert_eq!(result, 0b1010_0000_0111_1011u16);
}

#[test]
fn combine_args_into_multiple_segments() {
    let first: u8 = 0b1010_0101;
    let second: u8 = 0b0000_1111;
    let result = combinebits!(first, second, "aaaa bbbb abbb aaba");
    assert_eq!(result, 0b1010_0000_0111_1011u16);
}

#[test]
fn combine_different_sizes() {
    let a = true;
    let b: u8 = 0b00_1111;
    let c: u16 = 0b1_0101_0101_0101;
    let d: u64 = 0b1000_1000_1000_1000_1000_1000_1000_1000_1000_1000;
    let result = combinebits!("cccc cccc cccc cabb bbbb 1011 dddd dddd dddd dddd dddd dddd dddd dddd dddd dddd");
    assert_eq!(result,       0b1010_1010_1010_1100_1111_1011_1000_1000_1000_1000_1000_1000_1000_1000_1000_1000u64);
}

#[test]
fn combine_ux() {
    let a = true;
    let b = u1::new(1);
    let c = u7::new(0b000_0000);
    let d: u8 = 0b00_1111;
    let e = u9::new(0b1_0101_0101);
    let result = combinebits!("0000 0000 0000 1a0b ccdd ddde eeee eeee");
    assert_eq!(result,       0b0000_0000_0000_1101_0001_1111_0101_0101);
}

#[test]
fn combine_hex() {
    let a = u4::new(0xA);
    let b: u8 = 0xF0;
    let c = u12::new(0xEEE);
    let result = combinehex!("a1bbDccc");
    assert_eq!(result,      0xA1F0DEEE);
}

#[test]
fn combine_arguments() {
    let first: u16 = 0b1_0101_0101_0101;
    let second = true;
    let third: u8 = 0b00_1111;
    let fourth: u64 = 0b1000_1000_1000_1000_1000_1000_1000_1000_1000_1000;
    let result = combinebits!(
        first, second, third, fourth,
         "cccc cccc cccc cabb bbbb 1011 dddd dddd dddd dddd dddd dddd dddd dddd dddd dddd");
    assert_eq!(result,
        0b1010_1010_1010_1100_1111_1011_1000_1000_1000_1000_1000_1000_1000_1000_1000_1000u64);
}

#[test]
fn combine_arguments_literal_arg() {
    let result = combinebits!(0b1010_0101, "aaaa aaaa");
    assert_eq!(result, 0b1010_0101);
}

#[test]
fn combine_arguments_overflow() {
    let arg = 0b1010_0101;
    let result = combinebits!(arg, "0aaa aaaa");
    assert_eq!(result, 0b0010_0101);
}

// Same as default behavior.
#[test]
fn combine_arguments_overflow_truncate() {
    let arg = 0b1010_0101;
    let result = combinebits!(overflow=truncate, arg, "0aaa aaaa");
    assert_eq!(result, 0b0010_0101);
}

#[test]
#[should_panic(expected = "Variable a is too big for its location in the template. 0b10100101 > 0b1111111")]
fn combine_arguments_overflow_panic() {
    let arg = 0b1010_0101;
    let _ = combinebits!(overflow=panic, arg, "0aaa aaaa");
}

#[test]
fn combine_arguments_overflow_corrupt() {
    let arg = 0b1010_0101;
    let result = combinebits!(overflow=corrupt, arg, "0aaa aaaa");
    assert_eq!(result, 0b1010_0101);
}

#[test]
fn combine_arguments_overflow_saturate() {
    let arg = 0b1010_0101;
    let result = combinebits!(overflow=saturate, arg, "0aaa aaaa");
    assert_eq!(result, 0b0111_1111);
}
