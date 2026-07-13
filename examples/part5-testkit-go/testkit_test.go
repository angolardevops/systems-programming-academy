package testkit

import (
	"strings"
	"testing"
)

func TestAssertEqPassesAndFailsWithAMessage(t *testing.T) {
	if AssertEq(2+2, 4) != nil {
		t.Fatal("2+2 == 4 should pass")
	}
	if err := AssertEq(2+2, 5); err == nil || err.Error() != "expected 5, got 4" {
		t.Fatalf("got %v", err)
	}
}

func TestAllPassingReport(t *testing.T) {
	report := New().
		Test("adds", func() Check { return AssertEq(2+2, 4) }).
		Test("truthy", func() Check { return AssertTrue(1 < 2, "1 should be < 2") }).
		Run()
	if !report.OK() {
		t.Fatal("should be ok")
	}
	want := "running 2 tests\n" +
		"test adds ... ok\n" +
		"test truthy ... ok\n" +
		"\n" +
		"test result: ok. 2 passed; 0 failed"
	if got := report.Summary(); got != want {
		t.Fatalf("summary\n got:\n%s\n want:\n%s", got, want)
	}
}

func TestMixedReportListsFailures(t *testing.T) {
	report := New().
		Test("adds", func() Check { return AssertEq(2+2, 4) }).
		Test("subtracts", func() Check { return AssertEq(5-2, 2) }).
		Test("multiplies", func() Check { return AssertEq(2*3, 6) }).
		Run()
	if report.OK() {
		t.Fatal("should not be ok")
	}
	if report.Passed() != 2 || report.Failed() != 1 {
		t.Fatalf("passed=%d failed=%d", report.Passed(), report.Failed())
	}
	want := "running 3 tests\n" +
		"test adds ... ok\n" +
		"test subtracts ... FAILED\n" +
		"test multiplies ... ok\n" +
		"\n" +
		"failures:\n" +
		"    subtracts: expected 2, got 3\n" +
		"\n" +
		"test result: FAILED. 2 passed; 1 failed"
	if got := report.Summary(); got != want {
		t.Fatalf("summary\n got:\n%s\n want:\n%s", got, want)
	}
}

func TestAPanickingTestIsCaughtNotFatal(t *testing.T) {
	report := New().
		Test("boom", func() Check { panic("kaboom") }).
		Test("after", func() Check { return AssertEq(1, 1) }).
		Run()
	if report.Failed() != 1 || report.Passed() != 1 {
		t.Fatalf("passed=%d failed=%d", report.Passed(), report.Failed())
	}
	if !strings.Contains(report.Summary(), "boom: panicked: kaboom") {
		t.Fatalf("summary missing panic: %s", report.Summary())
	}
}

func TestEmptyKitReportsZero(t *testing.T) {
	want := "running 0 tests\n\ntest result: ok. 0 passed; 0 failed"
	if got := New().Run().Summary(); got != want {
		t.Fatalf("got:\n%s", got)
	}
}
