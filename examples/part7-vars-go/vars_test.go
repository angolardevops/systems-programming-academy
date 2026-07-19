package main

import (
	"reflect"
	"strings"
	"testing"
)

func TestTokenizesIdentifiersAndAssignment(t *testing.T) {
	toks, err := Tokenize("x = 5")
	if err != nil {
		t.Fatal(err)
	}
	want := []Token{{Kind: 'i', Name: "x"}, {Kind: '='}, {Kind: 'n', Num: 5}}
	if !reflect.DeepEqual(toks, want) {
		t.Errorf("got %+v, want %+v", toks, want)
	}
}

func TestAssignmentBindsAndReferenceReads(t *testing.T) {
	env := Env{}
	st, _ := ParseStmt(mustTok(t, "x = 40"))
	if v, _ := Exec(st, env); v != 40 {
		t.Fatalf("assign returned %d", v)
	}
	st, _ = ParseStmt(mustTok(t, "x + 2"))
	if v, _ := Exec(st, env); v != 42 {
		t.Fatalf("x + 2 = %d, want 42", v)
	}
}

func mustTok(t *testing.T, s string) []Token {
	t.Helper()
	toks, err := Tokenize(s)
	if err != nil {
		t.Fatal(err)
	}
	return toks
}

func TestStatePersistsAcrossStatements(t *testing.T) {
	got := RunProgram("x = 5\ny = x * 2 + 1\ny - x")
	want := []string{"x = 5  =>  5", "y = x * 2 + 1  =>  11", "y - x  =>  6"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("got %v, want %v", got, want)
	}
}

func TestUndefinedVariableIsAnError(t *testing.T) {
	got := RunProgram("z + 1")
	if len(got) != 1 || !strings.Contains(got[0], "undefined variable 'z'") {
		t.Errorf("got %v", got)
	}
}

func TestReassignmentUpdatesUsingTheOldValue(t *testing.T) {
	got := RunProgram("x = 1\nx = x + 10\nx")
	want := []string{"x = 1  =>  1", "x = x + 10  =>  11", "x  =>  11"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("got %v, want %v", got, want)
	}
}

func TestArithmeticErrorsStillReported(t *testing.T) {
	got := RunProgram("10 / 0\nfoo bar")
	if !strings.Contains(got[0], "division by zero") {
		t.Errorf("line 0: %q", got[0])
	}
	if !strings.Contains(got[1], "trailing") {
		t.Errorf("line 1: %q", got[1])
	}
}
