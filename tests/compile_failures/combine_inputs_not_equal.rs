use splitbits::*;

fn main() {
    let x: u8 = 0b1010_0101;
    let result = combinebits!(x, "aaaa aaaa bbbb bbbb");
    assert_eq!(result, 0b1010_0101_0000_1111u16);
}
