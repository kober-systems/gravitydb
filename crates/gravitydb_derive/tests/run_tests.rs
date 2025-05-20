#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse-simple.rs");
    t.pass("tests/02-parse-inner-structs.rs");
}
