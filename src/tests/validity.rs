use super::script_test;
use super::TestOutcomeTarget;
use test_generator::test_resources;

/// Autoload the Amber test files in validity, match the output in the comment
#[test_resources("src/tests/validity/*.ab")]
fn test_validity(input: &str) {
    std::env::set_var("AMBER_PATH", std::env::current_dir().unwrap().clone().join("src/tests/validity"));
    script_test(input, TestOutcomeTarget::Success)
}
