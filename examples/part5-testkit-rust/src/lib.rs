//! A tiny test framework — the tool this whole academy has been using, now
//! built from the inside.
//!
//! Three pieces, which every test framework (cargo test, `go test`, pytest,
//! JUnit) shares:
//!
//! * **Assertions** that turn a condition into a descriptive failure — not just
//!   "false", but "expected 4, got 5".
//! * **A registry + runner** that runs each test in **isolation**, so one
//!   test's failure (or panic) does not stop the others — the property that
//!   makes a test suite useful.
//! * **A canonical report** — the `test x ... ok` / `FAILED` summary you have
//!   read after every lesson — deterministic and byte-identical across
//!   languages.
//!
//! We test the framework the only honest way: feed it known-passing and
//! known-failing tests and assert its report. No I/O — the report is a string.

use std::panic::{self, AssertUnwindSafe};

/// The result of one assertion or test body: `Ok(())` passed, `Err(message)`
/// failed with a human-readable reason.
pub type Check = Result<(), String>;

/// Asserts a boolean condition, failing with `message`.
pub fn assert_true(condition: bool, message: &str) -> Check {
    if condition {
        Ok(())
    } else {
        Err(message.to_string())
    }
}

/// Asserts two values are equal, failing with a message that shows both — the
/// difference between a useful assertion and a bare `false`.
pub fn assert_eq<T: PartialEq + std::fmt::Debug>(actual: T, expected: T) -> Check {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("expected {expected:?}, got {actual:?}"))
    }
}

type TestFn = Box<dyn Fn() -> Check>;

/// A registry of named tests. Register with [`test`](TestKit::test), then
/// [`run`](TestKit::run).
#[derive(Default)]
pub struct TestKit {
    tests: Vec<(String, TestFn)>,
}

impl TestKit {
    pub fn new() -> Self {
        TestKit::default()
    }

    /// Registers a test under `name`. Chainable.
    pub fn test(&mut self, name: &str, body: impl Fn() -> Check + 'static) -> &mut Self {
        self.tests.push((name.to_string(), Box::new(body)));
        self
    }

    /// Runs every test in registration order, isolating each: a returned
    /// `Err` is a failure, and a **panic** is caught and turned into a failure
    /// too, so a crashing test cannot take down the run.
    pub fn run(&self) -> Report {
        let mut outcomes = Vec::new();
        for (name, body) in &self.tests {
            let result = panic::catch_unwind(AssertUnwindSafe(body));
            let outcome = match result {
                Ok(Ok(())) => Outcome::Pass,
                Ok(Err(message)) => Outcome::Fail(message),
                Err(payload) => {
                    let msg = payload
                        .downcast_ref::<&str>()
                        .map(|s| s.to_string())
                        .or_else(|| payload.downcast_ref::<String>().cloned())
                        .unwrap_or_else(|| "unknown panic".to_string());
                    Outcome::Fail(format!("panicked: {msg}"))
                }
            };
            outcomes.push((name.clone(), outcome));
        }
        Report { outcomes }
    }
}

enum Outcome {
    Pass,
    Fail(String),
}

/// The result of a run: every test's outcome, and a canonical text summary.
pub struct Report {
    outcomes: Vec<(String, Outcome)>,
}

impl Report {
    pub fn passed(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|(_, o)| matches!(o, Outcome::Pass))
            .count()
    }

    pub fn failed(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|(_, o)| matches!(o, Outcome::Fail(_)))
            .count()
    }

    /// True if every test passed.
    pub fn ok(&self) -> bool {
        self.failed() == 0
    }

    /// Renders the canonical report — the exact format is the cross-language
    /// contract asserted by the tests.
    pub fn summary(&self) -> String {
        let mut lines = vec![format!("running {} tests", self.outcomes.len())];
        for (name, outcome) in &self.outcomes {
            let status = match outcome {
                Outcome::Pass => "ok",
                Outcome::Fail(_) => "FAILED",
            };
            lines.push(format!("test {name} ... {status}"));
        }
        lines.push(String::new());

        let failures: Vec<&(String, Outcome)> = self
            .outcomes
            .iter()
            .filter(|(_, o)| matches!(o, Outcome::Fail(_)))
            .collect();
        if !failures.is_empty() {
            lines.push("failures:".to_string());
            for (name, outcome) in failures {
                if let Outcome::Fail(message) = outcome {
                    lines.push(format!("    {name}: {message}"));
                }
            }
            lines.push(String::new());
        }

        let result = if self.ok() { "ok" } else { "FAILED" };
        lines.push(format!(
            "test result: {result}. {} passed; {} failed",
            self.passed(),
            self.failed()
        ));
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_eq_passes_and_fails_with_a_message() {
        assert!(assert_eq(2 + 2, 4).is_ok());
        assert_eq!(assert_eq(2 + 2, 5).unwrap_err(), "expected 5, got 4");
    }

    #[test]
    fn all_passing_report() {
        let mut kit = TestKit::new();
        kit.test("adds", || assert_eq(2 + 2, 4))
            .test("truthy", || assert_true(1 < 2, "1 should be < 2"));
        let report = kit.run();
        assert!(report.ok());
        assert_eq!(
            report.summary(),
            "running 2 tests\n\
             test adds ... ok\n\
             test truthy ... ok\n\
             \n\
             test result: ok. 2 passed; 0 failed"
        );
    }

    #[test]
    fn mixed_report_lists_failures() {
        let mut kit = TestKit::new();
        kit.test("adds", || assert_eq(2 + 2, 4))
            .test("subtracts", || assert_eq(5 - 2, 2))
            .test("multiplies", || assert_eq(2 * 3, 6));
        let report = kit.run();
        assert!(!report.ok());
        assert_eq!(report.passed(), 2);
        assert_eq!(report.failed(), 1);
        assert_eq!(
            report.summary(),
            "running 3 tests\n\
             test adds ... ok\n\
             test subtracts ... FAILED\n\
             test multiplies ... ok\n\
             \n\
             failures:\n\
             \x20   subtracts: expected 2, got 3\n\
             \n\
             test result: FAILED. 2 passed; 1 failed"
        );
    }

    #[test]
    fn a_panicking_test_is_caught_not_fatal() {
        let mut kit = TestKit::new();
        kit.test("boom", || panic!("kaboom"))
            .test("after", || assert_eq(1, 1));
        let report = kit.run();
        // The panic became a failure; the run continued to the next test.
        assert_eq!(report.failed(), 1);
        assert_eq!(report.passed(), 1);
        assert!(report.summary().contains("boom: panicked: kaboom"));
    }

    #[test]
    fn empty_kit_reports_zero() {
        let report = TestKit::new().run();
        assert_eq!(
            report.summary(),
            "running 0 tests\n\ntest result: ok. 0 passed; 0 failed"
        );
    }
}
