use splitbits::*;

fn main() {
    let x: u16 = 0b1010_0101_0000_1111;
    combinebits!(a, "aaaa aaaa", taco);
    combinebits!(a, "aaaa aaaa", "aaaa aaaa");
}
