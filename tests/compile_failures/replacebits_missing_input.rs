use splitbits::*;

fn main() {
    let a = 0b101u16;
    let b = 0b00001u8;
    let c = 0b0101u128;
    let d = false;
    replacebits!(overflow=truncate, "aaab bbbb .d.. cccc")
}
