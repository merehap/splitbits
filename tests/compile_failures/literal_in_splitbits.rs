use splitbits::*;

fn main() {
    splitbits!(0b1111_0000, "aaa. 0bbb");
    splitbits!(0b1111_0000, "aaa. 1bbb");
    splitbits!(0b1111_0000, "aaa. Abbb");
    splitbits!(0b1111_0000, "aaa. Zbbb");
}
