#![cfg(test)]

#[test]
fn test_macro() {
    let t = trybuild::TestCases::new();
    // Test cases that should compile successfully
    t.pass("tests/ui/pass/*.rs");
    // Test cases that should fail to compile with specific errors
    t.compile_fail("tests/ui/fail/*.rs");
}
