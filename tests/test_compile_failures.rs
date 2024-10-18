#[test]
fn compile_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_failures/*.rs");
}
