extern crate splitbits;

use splitbits::{splithex, splithex_named, splithex_named_into};

#[test]
fn hex() {
    // IPV6
    let fields = splithex!(
        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
         "aaaa bbbb cccc dddd eeee ffff gggg hhhh",
    );
    assert_eq!(fields.a, 0x2001u16);
    assert_eq!(fields.b, 0x0db8u16);
    assert_eq!(fields.c, 0x85a3u16);
    assert_eq!(fields.d, 0x0000u16);
    assert_eq!(fields.e, 0x0000u16);
    assert_eq!(fields.f, 0x8a2eu16);
    assert_eq!(fields.g, 0x0370u16);
    assert_eq!(fields.h, 0x7334u16);
}

#[test]
fn hex_named() {
    // IPV6
    let (everybody, wants, to, rule, the, world) = splithex_named!(
        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
         "aaaa bbbb cccc .... eeee ffff hhhh hhhh",
    );
    assert_eq!(everybody, 0x2001u16);
    assert_eq!(wants, 0x0db8u16);
    assert_eq!(to, 0x85a3u16);
    assert_eq!(rule, 0x0000u16);
    assert_eq!(the, 0x8a2eu16);
    assert_eq!(world, 0x0370_7334u32);
}

#[test]
fn hex_named_into() {
    // IPV6
    let (everybody, wants, to, rule, the, world): (u16, u128, u8, u32, u128, u64) =
        splithex_named_into!(
            0x2101_0db8_85a3_0000_0000_8a2e_0370_7334,
             "abbb bbbb bbbb bbbb bbbe ffff ffff hhii",
        );
    assert_eq!(everybody, 0x2u16);
    assert_eq!(wants, 0x10_10db_885a_3000_0000u128);
    assert_eq!(to, 0x0u8);
    assert_eq!(rule, 0x8a2e_0370u32);
    assert_eq!(the, 0x73u128);
    assert_eq!(world, 0x34u64);
}

#[test]
fn onehex() {
    // IPV6
    let value = splithex_named!(
        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
         ".... bbbb .... .... .... .... .... ....",
    );
    assert_eq!(value, 0x0db8u16);
}

