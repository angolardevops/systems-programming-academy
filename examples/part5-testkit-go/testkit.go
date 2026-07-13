// Package testkit is a tiny test framework — the tool this academy has been
// using, built from the inside.
//
// Three pieces every test framework shares:
//   - Assertions that turn a condition into a descriptive failure ("expected 4,
//     got 5"), not a bare false.
//   - A registry + runner that runs each test in isolation, so one test's
//     failure (or panic) does not stop the others.
//   - A canonical report — the "test x ... ok / FAILED" summary — deterministic
//     and byte-identical across languages.
//
// We test the framework the only honest way: feed it known-passing and
// known-failing tests and assert its report. No I/O — the report is a string.
package testkit

import "fmt"

// Check is the result of an assertion or test body: nil passed, a non-empty
// error failed with a human-readable reason.
type Check = error

// AssertTrue asserts a boolean condition, failing with message.
func AssertTrue(condition bool, message string) Check {
	if condition {
		return nil
	}
	return fmt.Errorf("%s", message)
}

// AssertEq asserts two values are equal, failing with a message showing both.
func AssertEq[T comparable](actual, expected T) Check {
	if actual == expected {
		return nil
	}
	return fmt.Errorf("expected %v, got %v", expected, actual)
}

type testCase struct {
	name string
	body func() Check
}

// TestKit is a registry of named tests. Register with Test, then Run.
type TestKit struct {
	tests []testCase
}

// New returns an empty kit.
func New() *TestKit { return &TestKit{} }

// Test registers a test under name. Chainable.
func (k *TestKit) Test(name string, body func() Check) *TestKit {
	k.tests = append(k.tests, testCase{name, body})
	return k
}

type outcome struct {
	name    string
	failed  bool
	message string
}

// Run runs every test in registration order, isolating each: a returned error
// is a failure, and a panic is recovered and turned into a failure too, so a
// crashing test cannot take down the run.
func (k *TestKit) Run() *Report {
	outcomes := make([]outcome, 0, len(k.tests))
	for _, tc := range k.tests {
		outcomes = append(outcomes, runOne(tc))
	}
	return &Report{outcomes: outcomes}
}

func runOne(tc testCase) (result outcome) {
	result.name = tc.name
	defer func() {
		if r := recover(); r != nil {
			result.failed = true
			result.message = fmt.Sprintf("panicked: %v", r)
		}
	}()
	if err := tc.body(); err != nil {
		result.failed = true
		result.message = err.Error()
	}
	return result
}

// Report is the result of a run: every outcome, plus a canonical text summary.
type Report struct {
	outcomes []outcome
}

// Passed returns the number of passing tests.
func (r *Report) Passed() int {
	n := 0
	for _, o := range r.outcomes {
		if !o.failed {
			n++
		}
	}
	return n
}

// Failed returns the number of failing tests.
func (r *Report) Failed() int { return len(r.outcomes) - r.Passed() }

// OK reports whether every test passed.
func (r *Report) OK() bool { return r.Failed() == 0 }

// Summary renders the canonical report — the exact format is the cross-language
// contract asserted by the tests.
func (r *Report) Summary() string {
	lines := []string{fmt.Sprintf("running %d tests", len(r.outcomes))}
	for _, o := range r.outcomes {
		status := "ok"
		if o.failed {
			status = "FAILED"
		}
		lines = append(lines, fmt.Sprintf("test %s ... %s", o.name, status))
	}
	lines = append(lines, "")

	var failures []outcome
	for _, o := range r.outcomes {
		if o.failed {
			failures = append(failures, o)
		}
	}
	if len(failures) > 0 {
		lines = append(lines, "failures:")
		for _, o := range failures {
			lines = append(lines, fmt.Sprintf("    %s: %s", o.name, o.message))
		}
		lines = append(lines, "")
	}

	result := "ok"
	if !r.OK() {
		result = "FAILED"
	}
	lines = append(lines, fmt.Sprintf("test result: %s. %d passed; %d failed",
		result, r.Passed(), r.Failed()))

	out := ""
	for i, l := range lines {
		if i > 0 {
			out += "\n"
		}
		out += l
	}
	return out
}
