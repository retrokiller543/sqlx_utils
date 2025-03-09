#![cfg(test)]

#[test]
#[cfg(all(not(feature = "try-parse"), not(feature = "nightly")))]
fn test_macro() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/pass/*.rs");
    t.compile_fail("tests/ui/fail/*.rs");
}

#[test]
#[cfg(all(feature = "try-parse", not(feature = "nightly")))]
fn test_macro() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/pass/*.rs");
    t.compile_fail("tests/ui/fail-try-parse/*.rs");
}

#[test]
#[cfg(all(feature = "try-parse", feature = "nightly"))]
fn test_macro() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/pass/*.rs");
    t.compile_fail("tests/ui/fail-try-parse-nightly/*.rs");
}
