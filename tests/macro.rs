#[rustversion::attr(not(nightly), ignore)]
#[test]
fn test_macro() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/macro/*.rs");
}
