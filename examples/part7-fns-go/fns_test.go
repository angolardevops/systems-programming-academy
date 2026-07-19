package main

import (
	"reflect"
	"strings"
	"testing"
)

func TestTokenizesAFunctionDefinition(t *testing.T) {
	toks, err := Tokenize("f(x, y) = x + y")
	if err != nil {
		t.Fatal(err)
	}
	want := []Token{
		{Kind: 'i', Name: "f"}, {Kind: '('}, {Kind: 'i', Name: "x"}, {Kind: ','},
		{Kind: 'i', Name: "y"}, {Kind: ')'}, {Kind: '='},
		{Kind: 'i', Name: "x"}, {Kind: '+'}, {Kind: 'i', Name: "y"},
	}
	if !reflect.DeepEqual(toks, want) {
		t.Errorf("got %+v", toks)
	}
}

func TestDefinesAndCallsAFunction(t *testing.T) {
	got := RunProgram("double(x) = x * 2\ndouble(21)")
	want := []string{"double(x) = x * 2  =>  <fn>", "double(21)  =>  42"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("got %v", got)
	}
}

func TestHandlesMultipleArgumentsAndNestedCalls(t *testing.T) {
	got := RunProgram("add(a, b) = a + b\nadd(add(1, 2), 3)")
	if last := got[len(got)-1]; last != "add(add(1, 2), 3)  =>  6" {
		t.Errorf("got %q", last)
	}
}

func TestClosuresUseLexicalNotDynamicScope(t *testing.T) {
	// f captures x = 10 where it is defined; g's own x parameter must not
	// leak in when f is called from inside g.
	got := RunProgram("x = 10\nf(n) = n + x\ng(x) = f(0)\ng(999)")
	if last := got[len(got)-1]; last != "g(999)  =>  10" {
		t.Errorf("got %q, want lexical scope => 10", last)
	}
}

func TestClosureSeesLaterUpdatesToCapturedVariable(t *testing.T) {
	got := RunProgram("base = 100\nshift(n) = n + base\nshift(5)\nbase = 200\nshift(5)")
	if got[2] != "shift(5)  =>  105" || got[4] != "shift(5)  =>  205" {
		t.Errorf("got %v", got)
	}
}

func TestReportsArityAndKindErrors(t *testing.T) {
	got := RunProgram("double(x) = x * 2\ndouble(1, 2)\nnope(3)\n5(3)")
	if !strings.Contains(got[1], "expects 1 argument(s), got 2") {
		t.Errorf("arity: %q", got[1])
	}
	if !strings.Contains(got[2], "undefined function 'nope'") {
		t.Errorf("undefined: %q", got[2])
	}
	if !strings.Contains(got[3], "not a function") && !strings.Contains(got[3], "trailing") {
		t.Errorf("kind: %q", got[3])
	}
}
