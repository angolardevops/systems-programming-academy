package main

import (
	"strings"
	"testing"
)

func run1(t *testing.T, src string) string {
	t.Helper()
	out := RunProgram(src)
	return out[len(out)-1]
}

func TestTokenizesComparisonOperators(t *testing.T) {
	toks, err := Tokenize("a <= b == c")
	if err != nil {
		t.Fatal(err)
	}
	if len(toks) != 5 || toks[1].Kind != 'C' || toks[1].Op != "<=" || toks[3].Op != "==" {
		t.Errorf("got %+v", toks)
	}
}

func TestComparisonsYieldOneOrZero(t *testing.T) {
	cases := map[string]string{
		"3 < 5":  "3 < 5  =>  1",
		"3 > 5":  "3 > 5  =>  0",
		"4 == 4": "4 == 4  =>  1",
		"4 != 4": "4 != 4  =>  0",
		"5 >= 5": "5 >= 5  =>  1",
	}
	for src, want := range cases {
		if got := run1(t, src); got != want {
			t.Errorf("%q => %q, want %q", src, got, want)
		}
	}
}

func TestIfSelectsTheTakenBranch(t *testing.T) {
	if got := run1(t, "if 1 then 10 else 20"); got != "if 1 then 10 else 20  =>  10" {
		t.Errorf("got %q", got)
	}
	if got := run1(t, "if 0 then 10 else 20"); got != "if 0 then 10 else 20  =>  20" {
		t.Errorf("got %q", got)
	}
	if got := run1(t, "if 3 < 5 then 100 else 200"); got != "if 3 < 5 then 100 else 200  =>  100" {
		t.Errorf("got %q", got)
	}
}

func TestRecursionNowTerminates(t *testing.T) {
	got := run1(t, "fact(n) = if n <= 1 then 1 else n * fact(n - 1)\nfact(5)")
	if got != "fact(5)  =>  120" {
		t.Errorf("got %q", got)
	}
}

func TestRecursiveFibonacci(t *testing.T) {
	got := run1(t, "fib(n) = if n < 2 then n else fib(n - 1) + fib(n - 2)\nfib(10)")
	if got != "fib(10)  =>  55" {
		t.Errorf("got %q", got)
	}
}

func TestOnlyTheTakenBranchIsEvaluated(t *testing.T) {
	out := RunProgram("safe(n) = if n == 0 then 0 else 100 / n\nsafe(0)\nsafe(4)")
	if out[1] != "safe(0)  =>  0" { // no division-by-zero error
		t.Errorf("safe(0): %q", out[1])
	}
	if !strings.Contains(out[2], "25") {
		t.Errorf("safe(4): %q", out[2])
	}
}
