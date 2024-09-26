extern crate splitbits;

use splitbits::*;
use ux::{u1, u3, u4, u2, u7, u9, u12, u19, u28};

#[test]
fn u8() {
    let fields = splitbits!(0b11011101, "aaabbccc");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);
}

// Decimal constants should work.
#[test]
fn decimal() {
    let fields = splitbits!(221, "aaabbccc");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);
}

// Passing in a variable is the most common use case for the macro.
#[test]
fn variable() {
    let value: u8 = 221;
    let fields = splitbits!(value, "aaabbccc");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);
}

// The caller shouldn't have to specify the type of the value passed in.
#[test]
fn variable_type_inferred() {
    let value = 221;
    let fields = splitbits!(value, "aaabbccc");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);
}

// Single bit fields should result in bools, not u8s.
#[test]
fn bools() {
    let fields = splitbits!(0b11010101, "abbbcdee");
    assert_eq!(fields.a, true);
    assert_eq!(fields.b, 0b101u8);
    assert_eq!(fields.c, false);
    assert_eq!(fields.d, true);
    assert_eq!(fields.e, 0b01u8);
}

// Periods hold their place, but the bits they correspond to are ignored.
#[test]
fn periods() {
    let fields = splitbits!(0b11011101, ".aa.bb..");
    assert_eq!(fields.a, 0b10u8);
    assert_eq!(fields.b, 0b11u8);
}

// Spaces are stripped out before processing, whatever place they are in.
#[test]
fn underscores() {
    let fields = splitbits!(0b110_11101, " a aa   b bccc  ");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);
}

#[test]
fn noncontinguous() {
    let fields = splitbits!(0b1101_1101, "abadadda");
    assert_eq!(fields.a, 0b1011u8);
    assert_eq!(fields.b, true);
    assert_eq!(fields.d, 0b110u8);
}

#[test]
fn some_of_everything() {
    let fields = splitbits!(0b1111_1101, ".ad. cdd.");
    assert_eq!(fields.a, true);
    assert_eq!(fields.c, true);
    assert_eq!(fields.d, 0b110u8);
}

// Using the same template twice in the same scope should work (i.e. no struct name conflicts)
#[test]
fn duplicate() {
    let fields = splitbits!(0b11011101, "aaabbccc");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);

    let fields2 = splitbits!(0b01001100, "aaabbccc");
    assert_eq!(fields2.a, 0b010u8);
    assert_eq!(fields2.b, 0b01u8);
    assert_eq!(fields2.c, 0b100u8);
}

// LARGE FIELD TESTS

#[test]
fn u16() {
    let fields = splitbits!(
        0b1101110111111001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(fields.a, 0b110111011u16);

    assert_eq!(fields.d, 0b111u8);
    assert_eq!(fields.e, true);
    assert_eq!(fields.f, 0b001u8);
}

#[test]
fn u32() {
    let fields = splitbits!(
        0b1101_1101_1000_0100_0000_0000_1111_1001,
         "aaaa bbbb bbbb bbbb bbbb bbbi jjjj klll",
    );

    assert_eq!(fields.a, 0b1101u8);
    assert_eq!(fields.b, 0b110_1100_0010_0000_0000u32);
    assert_eq!(fields.i, false);
    assert_eq!(fields.j, 0b1111u8);
    assert_eq!(fields.k, true);
    assert_eq!(fields.l, 0b001u8);
}

#[test]
fn u64() {
    let fields = splitbits!(
        0b1101_1101_1000_0000_0000_0000_1111_0001_1101_1101_1000_0000_0000_0000_1101_0001,
         "aaaa bbbb bbbb bbbb bbbb bbbi jjjk klll mmmm nnoo pppq qrrr ssst tuuu uuvw xxxx",
    );
    assert_eq!(fields.a, 0b1101u8);
    assert_eq!(fields.b, 0b110_1100_0000_0000_0000u32);
    assert_eq!(fields.i, false);
    assert_eq!(fields.j, 0b111u8);
    assert_eq!(fields.k, 0b10u8);
    assert_eq!(fields.l, 0b001u8);
    assert_eq!(fields.m, 0b1101u8);
    assert_eq!(fields.n, 0b11u8);
    assert_eq!(fields.o, 0b01u8);
    assert_eq!(fields.p, 0b100u8);
    assert_eq!(fields.q, 0b00u8);
    assert_eq!(fields.r, 0b000u8);
    assert_eq!(fields.s, 0b000u8);
    assert_eq!(fields.t, 0b00u8);
    assert_eq!(fields.u, 0b00011u8);
    assert_eq!(fields.v, false);
    assert_eq!(fields.w, true);
    assert_eq!(fields.x, 0b0001u8);
}

#[test]
fn u128() {
    let fields = splitbits!(
        0b1101_1101_1000_0100_0001_0000_1111_0001_1101_1101_1000_0000_0100_0110_1101_0001_1101_1101_1000_0000_1001_0000_1111_0001_1101_1101_1000_0000_0000_0000_1101_0001,
         "aaaa bbcc cdde efff ffff ff.. .... gg.. ...h hiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iii. pppq qrrr ssst tuuu vvvw wxxx",
    );
    assert_eq!(fields.a, 0b1101u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b011u8);
    assert_eq!(fields.d, 0b00u8);
    assert_eq!(fields.e, 0b00u8);
    assert_eq!(fields.f, 0b1_0000_0100u16);
    assert_eq!(fields.g, 0b00u8);
    assert_eq!(fields.h, 0b11u8);
    assert_eq!(fields.i, 0b10_1100_0000_0010_0011_0110_1000_1110_1110_1100_0000_0100_1000_0111_1000_1110_1110u128);
    assert_eq!(fields.p, 0b100u8);
    assert_eq!(fields.q, 0b00u8);
    assert_eq!(fields.r, 0b000u8);
    assert_eq!(fields.s, 0b000u8);
    assert_eq!(fields.t, 0b00u8);
    assert_eq!(fields.u, 0b000u8);
    assert_eq!(fields.v, 0b110u8);
    assert_eq!(fields.w, 0b10u8);
    assert_eq!(fields.x, 0b0001u8);
}

#[test]
fn min_u8() {
    let fields = splitbits!(
        min=u8,
        0b1101110111111001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(fields.a, 0b110111011u16);

    assert_eq!(fields.d, 0b111u8);
    assert_eq!(fields.e, 0b1u8);
    assert_eq!(fields.f, 0b001u8);
}

#[test]
fn min_u16() {
    let fields = splitbits!(
        min=u16,
        0b1101110111111001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(fields.a, 0b110111011u16);

    assert_eq!(fields.d, 0b111u16);
    assert_eq!(fields.e, 0b1u16);
    assert_eq!(fields.f, 0b001u16);
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

// ALTERNATE VARIABLE SETTING METHODS

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

// HEXADECIMAL

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

// NON-STANDARD SIZE INTEGERS

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
#[should_panic(expected = "Variable a is too big for its location in the template. 165 > 127")]
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
    assert_eq!(result,       0b1010_1010_1010_1100_1111_1011_1000_1000_1000_1000_1000_1000_1000_1000_1000_1000u64);
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
#[should_panic(expected = "Variable a is too big for its location in the template. 165 > 127")]
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
fn replacehex_ux() {
    let a = u4::new(0xE);
    let c: u8 = 0x2A;
    let b = u28::new(0x90210AB);
    let result = replacehex!(0xABCD_EF01_2345_6789, "0a.. cc.b bbbb bb1D");
    assert_eq!(result,       0x0ECD_2A09_0210_AB1Du64);
}

#[test]
fn compile_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_failures/*.rs");
}
