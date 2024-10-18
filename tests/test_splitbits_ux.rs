extern crate splitbits;

use splitbits::{splitbits_ux, splithex_ux, splithex_named_ux, splithex_named_into_ux};
use ux::{u1, u2, u3, u9, u19};

#[test]
fn ux() {
    let fields = splitbits_ux!(
        0b1101_1101_1000_0100_0000_0000_1111_1001,
         "aaaa bbbb bbbb bbbb bbbb bbbi aajj kaaa",
    );

    assert_eq!(fields.a, u9::new(0b1_1011_1001));
    assert_eq!(fields.b, u19::new(0b110_1100_0010_0000_0000));
    assert_eq!(fields.i, false);
    assert_eq!(fields.j, u2::new(0b11u8));
    assert_eq!(fields.k, true);
}

#[test]
fn ux_other() {
    let _ = splithex_ux!(
        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
         "aaaa bbbb cccc .... .... .... dddd ....",
    );
    let (_, _, _, _) = splithex_named_ux!(
        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
         "aaaa bbbb cccc .... .... .... dddd ....",
    );
    let (_, _, _, _) : (u16, u32, u64, u128) = splithex_named_into_ux!(
        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
         "aaaa bbbb cccc .... .... .... dddd ....",
    );
    let _ = splithex_named_ux!(
        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
         "aaaa .... .... .... .... .... .... ....",
    );
}

#[test]
fn min_u1() {
    let fields = splitbits_ux!(
        min=u1,
        0b1101110111111001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(fields.a, u9::new(0b110111011u16));

    assert_eq!(fields.d, u3::new(0b111u8));
    assert_eq!(fields.e, u1::new(0b1));
    assert_eq!(fields.f, u3::new(0b001u8));
}

#[test]
fn min_u2() {
    let fields = splitbits_ux!(
        min=u2,
        0b1101110111111001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(fields.a, u9::new(0b110111011u16));

    assert_eq!(fields.d, u3::new(0b111u8));
    assert_eq!(fields.e, u2::new(0b1u8));
    assert_eq!(fields.f, u3::new(0b001u8));
}
