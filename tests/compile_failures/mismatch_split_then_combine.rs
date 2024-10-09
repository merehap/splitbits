use splitbits::*;

fn main() {
    let _ = splitbits_then_combine!(0b1001_1010, "aaab bbbb", "bbba aaaa");
}
