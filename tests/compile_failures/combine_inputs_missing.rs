use splitbits::*;

fn main() {
    let a: u8 = 0b1010_0101;
    let result = combinebits!("aaaa aaaa bbbb bbbb");
    assert_eq!(result, 0b1010_0101_0000_1111u16);
}
