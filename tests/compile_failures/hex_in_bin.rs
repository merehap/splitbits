use splitbits::*;

fn main() {
    let a = 0b11;
    let b = true;
    let c = false;
    let d = 0b01;
    let e = 0b1111_0000;
    combinebits!("ddAB cCbb bDEF aa01 2345 6789 eeee eeee");
    combinebits!("ddAB cCbb bDEF aa01 0000 0000 eeee eeee");
    combinebits!("dd00 c0bb b000 aa01 2345 6789 eeee eeee");
}
