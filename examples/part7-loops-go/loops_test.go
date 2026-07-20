package main

import (
	"reflect"
	"testing"
)

func eq(t *testing.T, got, want []string) {
	t.Helper()
	if !reflect.DeepEqual(got, want) {
		t.Errorf("got %v, want %v", got, want)
	}
}

func TestPrintEmitsValues(t *testing.T) {
	eq(t, RunProgram("print 42; print 7"), []string{"42", "7"})
}

func TestWhileLoopCounts(t *testing.T) {
	eq(t, RunProgram("i = 1; while i <= 5 do { print i; i = i + 1 }"), []string{"1", "2", "3", "4", "5"})
}

func TestLoopComputesFactorialIteratively(t *testing.T) {
	eq(t, RunProgram("n = 5; acc = 1; i = 1; while i <= n do { acc = acc * i; i = i + 1 }; print acc"), []string{"120"})
}

func TestNestedLoops(t *testing.T) {
	eq(t, RunProgram("i = 1; while i <= 3 do { j = 1; while j <= 3 do { print i * j; j = j + 1 }; i = i + 1 }"),
		[]string{"1", "2", "3", "2", "4", "6", "3", "6", "9"})
}

func TestLoopsAndFunctionsTogether(t *testing.T) {
	eq(t, RunProgram("sq(x) = x * x; i = 1; while i <= 4 do { print sq(i); i = i + 1 }"), []string{"1", "4", "9", "16"})
}

func TestFibonacciSequenceViaLoop(t *testing.T) {
	eq(t, RunProgram("a = 0; b = 1; i = 0; while i < 8 do { print a; t = a + b; a = b; b = t; i = i + 1 }"),
		[]string{"0", "1", "1", "2", "3", "5", "8", "13"})
}
