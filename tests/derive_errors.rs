#[test]
fn enum_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/derive_errors/enum/*.rs");
}

#[test]
fn struct_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/derive_errors/struct/*.rs");
}

#[test]
fn union_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/derive_errors/union.rs");
}
