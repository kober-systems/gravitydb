#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse-simple.rs");
    t.pass("tests/02-parse-inner-structs.rs");
    t.pass("tests/03-schema_type_not_recursive.rs");
}
