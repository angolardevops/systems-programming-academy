package main

import (
	"strings"
	"testing"
)

func TestTokenizesNumbersAndOperators(t *testing.T) {
	toks, err := Tokenize("12 + 3")
	if err != nil {
		t.Fatal(err)
	}
	want := []Token{{Kind: 'n', Num: 12}, {Kind: '+'}, {Kind: 'n', Num: 3}}
	if len(toks) != len(want) {
		t.Fatalf("got %d tokens, want %d", len(toks), len(want))
	}
	for i := range want {
		if toks[i] != want[i] {
			t.Errorf("token %d = %+v, want %+v", i, toks[i], want[i])
		}
	}
}

func run(t *testing.T, src string) (string, int64) {
	t.Helper()
	sexp, v, err := Run(src)
	if err != nil {
		t.Fatalf("Run(%q) errored: %v", src, err)
	}
	return sexp, v
}

func TestPrecedenceBindsStarTighterThanPlus(t *testing.T) {
	if sexp, v := run(t, "1 + 2 * 3"); sexp != "(+ 1 (* 2 3))" || v != 7 {
		t.Errorf("got %q => %d", sexp, v)
	}
}

func TestParenthesesOverridePrecedence(t *testing.T) {
	if sexp, v := run(t, "(1 + 2) * 3"); sexp != "(* (+ 1 2) 3)" || v != 9 {
		t.Errorf("got %q => %d", sexp, v)
	}
}

func TestUnaryMinusAndTruncatingDivision(t *testing.T) {
	if sexp, v := run(t, "-7 / 2"); sexp != "(/ (neg 7) 2)" || v != -3 {
		t.Errorf("got %q => %d, want (/ (neg 7) 2) => -3", sexp, v)
	}
}

func TestEvaluatesALongerExpression(t *testing.T) {
	if sexp, v := run(t, "2 * (3 + 4) - 10 / 3"); sexp != "(- (* 2 (+ 3 4)) (/ 10 3))" || v != 11 {
		t.Errorf("got %q => %d", sexp, v)
	}
}

func TestReportsErrorsWithoutPanicking(t *testing.T) {
	cases := map[string]string{
		"1 / 0":  "division by zero",
		"1 +":    "unexpected end of input",
		"1 @ 2":  "unexpected character",
		"(1 + 2": "expected ')'",
		"1 2":    "trailing",
	}
	for src, want := range cases {
		if _, _, err := Run(src); err == nil || !strings.Contains(err.Error(), want) {
			t.Errorf("Run(%q) err = %v, want containing %q", src, err, want)
		}
	}
}
