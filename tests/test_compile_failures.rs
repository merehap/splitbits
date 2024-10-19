#[test]
// Different compiler versions give different error messages, so we shoudln't run this by default.
// To include these in a test run, use "cargo test -- --ignored".
#[ignore]
fn compile_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_failures/*.rs");
}
