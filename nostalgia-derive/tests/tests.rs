#[cfg(test)]
mod tests {
    use trybuild;

    #[test]
    fn test_that_we_can_test_the_macro() {
        let t = trybuild::TestCases::new();
        t.pass("tests/ui/key-tests-assign-id-pass.rs");
        t.pass("tests/ui/key-tests-argument-id-pass.rs");
    }
}
