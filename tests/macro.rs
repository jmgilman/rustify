#[test]
fn test_macro() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/macro/*.rs");
}
