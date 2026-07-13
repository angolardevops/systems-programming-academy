//! Demo: register a small suite with one deliberate failure and print the
//! canonical report — the same shape you have read after every lesson.

use part5_testkit_rust::{assert_eq, assert_true, TestKit};

fn main() {
    let mut kit = TestKit::new();
    kit.test("addition works", || assert_eq(2 + 2, 4))
        .test("string upper", || {
            assert_eq("hi".to_uppercase(), "HI".to_string())
        })
        .test("deliberate failure", || assert_eq(10 / 2, 4))
        .test("a truthy check", || assert_true(3 > 1, "3 should be > 1"));

    let report = kit.run();
    println!("{}", report.summary());
    std::process::exit(if report.ok() { 0 } else { 1 });
}
