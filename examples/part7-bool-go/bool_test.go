package main

import "testing"

func run1(t *testing.T, src string) string {
	t.Helper()
	out := RunProgram(src)
	return out[len(out)-1]
}

func TestAndOrTruthTables(t *testing.T) {
	cases := map[string]string{
		"1 and 1": "1 and 1  =>  1",
		"1 and 0": "1 and 0  =>  0",
		"0 and 1": "0 and 1  =>  0",
		"0 or 0":  "0 or 0  =>  0",
		"1 or 0":  "1 or 0  =>  1",
	}
	for src, want := range cases {
		if got := run1(t, src); got != want {
			t.Errorf("%q => %q, want %q", src, got, want)
		}
	}
}

func TestNotNegatesTruthiness(t *testing.T) {
	if got := run1(t, "not 0"); got != "not 0  =>  1" {
		t.Errorf("got %q", got)
	}
	if got := run1(t, "not 5"); got != "not 5  =>  0" { // any nonzero is truthy
		t.Errorf("got %q", got)
	}
	if got := run1(t, "not not 3"); got != "not not 3  =>  1" {
		t.Errorf("got %q", got)
	}
}

func TestPrecedenceNotTighterThanAndTighterThanOr(t *testing.T) {
	if got := run1(t, "1 or 0 and 0"); got != "1 or 0 and 0  =>  1" { // 1 or (0 and 0)
		t.Errorf("got %q", got)
	}
	if got := run1(t, "not 0 and 1"); got != "not 0 and 1  =>  1" { // (not 0) and 1
		t.Errorf("got %q", got)
	}
	if got := run1(t, "2 > 1 and 3 > 5"); got != "2 > 1 and 3 > 5  =>  0" { // (2>1) and (3>5)
		t.Errorf("got %q", got)
	}
}

func TestAndShortCircuitsAvoidingTheError(t *testing.T) {
	out := RunProgram("guard(x) = if x != 0 and 100 / x > 1 then 100 / x else -1\nguard(0)\nguard(50)")
	if out[1] != "guard(0)  =>  -1" { // no division-by-zero
		t.Errorf("guard(0): %q", out[1])
	}
	if out[2] != "guard(50)  =>  2" {
		t.Errorf("guard(50): %q", out[2])
	}
}

func TestOrShortCircuitsAvoidingTheError(t *testing.T) {
	out := RunProgram("check(a) = if a == 0 or 10 / a > 0 then 1 else 0\ncheck(0)")
	if last := out[len(out)-1]; last != "check(0)  =>  1" { // no division-by-zero
		t.Errorf("got %q", last)
	}
}

func TestBooleansComposeIntoRealPredicates(t *testing.T) {
	out := RunProgram("in_range(x, lo, hi) = if x >= lo and x <= hi then 1 else 0\nin_range(5, 1, 10)\nin_range(15, 1, 10)")
	if out[1] != "in_range(5, 1, 10)  =>  1" {
		t.Errorf("in-range: %q", out[1])
	}
	if out[2] != "in_range(15, 1, 10)  =>  0" {
		t.Errorf("out-of-range: %q", out[2])
	}
}
