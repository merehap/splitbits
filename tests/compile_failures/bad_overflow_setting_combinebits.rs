use splitbits::*;

fn main() {
    let a: u8 = 0b1010_0101;
    combinebits!(overflow=explode, "aaaa aaaa");
}
