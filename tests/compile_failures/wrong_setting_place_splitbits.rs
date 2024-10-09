use splitbits::*;

fn main() {
    splitbits!(0b11011101, min=u8, "aaabbccc");
    splitbits!(0b11011101, "aaabbccc", min=u8);
}
