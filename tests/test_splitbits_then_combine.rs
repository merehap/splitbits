extern crate splitbits;

use splitbits::{splitbits_then_combine, splithex_then_combine};

#[test]
fn split_then_combine_trivial() {
    let result = splitbits_then_combine!(0b1001_1011, "aaaa aaaa", "aaaa aaaa");
    assert_eq!(result, 0b1001_1011u8);
}

#[test]
fn split_then_combine_swap() {
    let result = splitbits_then_combine!(0b1001_1010, "aaab bbbb", "bbbb baaa");
    assert_eq!(result, 0b1101_0100u8);
}

#[test]
fn split_then_combine_with_literals() {
    let result = splitbits_then_combine!(0b1001_1011, "..aa ....", "101a a011");
    assert_eq!(result, 0b1010_1011u8);
}

#[test]
fn split_then_combine_upsize() {
    let result = splitbits_then_combine!(
        0b1001_0000, "aaaa aaaa",
        0b1111_1010, "bbbb bbbb",
                     "aaaa aaaa bbbb bbbb",
    );
    assert_eq!(result, 0b1001_0000_1111_1010u16);
}

#[test]
fn split_multiple_then_combine() {
    let result = splitbits_then_combine!(
        0b1001_0000, "aaaa ....",
        0b1111_1010, ".... ..cc",
        0b1111_0011, ".... bb..",
                     "aaaa bbcc",
    );
    assert_eq!(result, 0b1001_0010u8);
}

#[test]
fn split_then_combine_fields() {
    let result = splitbits_then_combine!(
        0b1001_0000_1111_0000, "aaaa .... .... ....",
        0b1111_1010_0111_0000, ".... bbbb aaaa ....",
                               "0000 bbbb aaaa aaaa",
    );
    assert_eq!(result, 0b0000_1010_1001_0111u16);
}

#[test]
fn split_then_combine_many_fragments() {
    let result = splitbits_then_combine!(
        0b1001_0000_1111_0000, "c.aa .... ...a ....",
        0b1111_1010_0111_0000, "..a. ..a. bb.a ....",
        0b1111_0011_1111_0000, "..aa aaa. .... ...a",
                               "bb1a aaaa aaaa aaac",
    );
    assert_eq!(result, 0b0110_1111_1110_0101u16);
}

#[test]
fn split_then_combine_into_fragments() {
    let result = splitbits_then_combine!(
        0b1001_0000_1111_0000, "aaaa .... .... ....",
        0b1111_1010_0111_0000, ".... bbbb aaaa ....",
                               "aa00 bb01 aaab baaa",
    );
    assert_eq!(result, 0b1000_1001_0101_0111u16);
}

#[test]
fn split_then_combine_segments_into_fragments() {
    let result = splitbits_then_combine!(
        0b1001_0000_1111_0000, "c.aa .... ...a ....",
        0b1111_1010_0111_0000, "..a. ..a. bb.a ....",
        0b1111_0011_1111_0000, "..aa aaa. .... ...a",
                               "ba1b aaaa aaaa aaca",
    );
    assert_eq!(result, 0b0011_1111_1110_0110u16);
}

#[test]
fn splithex_then_combine() {
    let result = splithex_then_combine!(0xABCD_EF01_2345_6789, "aaab bbbb bbbb bbbb", "bbbb bbbb bbbb baaa");
    assert_eq!(result, 0xDEF0_1234_5678_9ABCu64);
}

#[test]
fn splithex_then_combine_with_literals() {
    let result = splithex_then_combine!(
        0xABCD_EF01_2345_6789_1900_0088_4321_9876,
         ".aa. ..bb b..c dd.. ..ee eeee ee.. ....",
         "ddAB cCbb bDEF aa01 2345 6789 eeee eeee");
    assert_eq!(result, 0x67AB_5C01_2DEF_BC01_2345_6789_0000_8843);
}
