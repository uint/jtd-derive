#[test]
fn enum_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/derive_errors/enum/*.rs");
}
