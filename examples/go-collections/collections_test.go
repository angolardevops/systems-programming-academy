package collections

import (
	"reflect"
	"testing"
)

func TestWordCount(t *testing.T) {
	counts := WordCount("The cat, the CAT! the dog.")
	if counts["the"] != 3 {
		t.Errorf("the = %d, want 3", counts["the"])
	}
	if counts["cat"] != 2 {
		t.Errorf("cat = %d, want 2", counts["cat"])
	}
	if counts["dog"] != 1 {
		t.Errorf("dog = %d, want 1", counts["dog"])
	}
	if _, ok := counts["missing"]; ok {
		t.Error("missing should not be present")
	}
}

func TestTopWords(t *testing.T) {
	got := TopWords("b a a b c b", 2)
	want := []Pair{{"b", 3}, {"a", 2}}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("TopWords = %v, want %v", got, want)
	}
}

func TestTopWordsTieBreaksAlphabetically(t *testing.T) {
	got := TopWords("cherry banana apple", 3)
	want := []Pair{{"apple", 1}, {"banana", 1}, {"cherry", 1}}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("TopWords = %v, want %v", got, want)
	}
}

func TestSumEvens(t *testing.T) {
	cases := []struct {
		name string
		in   []int
		want int
	}{
		{"mixed", []int{1, 2, 3, 4, 5, 6}, 12},
		{"empty", nil, 0},
		{"all odd", []int{1, 3, 5}, 0},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if got := SumEvens(tc.in); got != tc.want {
				t.Errorf("SumEvens(%v) = %d, want %d", tc.in, got, tc.want)
			}
		})
	}
}

func TestUniqueSortedDoesNotMutateInput(t *testing.T) {
	in := []int{3, 1, 2, 3, 1, 2}
	got := UniqueSorted(in)
	want := []int{1, 2, 3}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("UniqueSorted = %v, want %v", got, want)
	}
	// The caller's slice must be untouched.
	if !reflect.DeepEqual(in, []int{3, 1, 2, 3, 1, 2}) {
		t.Errorf("input was mutated: %v", in)
	}
}

func TestRuneCountVsLen(t *testing.T) {
	s := "héllo" // é is 2 bytes in UTF-8
	if got := RuneCount(s); got != 5 {
		t.Errorf("RuneCount(%q) = %d, want 5", s, got)
	}
	if len(s) != 6 {
		t.Errorf("len(%q) = %d, want 6 bytes", s, len(s))
	}
}

func TestJoinUpper(t *testing.T) {
	if got := JoinUpper([]string{"hello", "go", "world"}); got != "HELLO GO WORLD" {
		t.Errorf("JoinUpper = %q, want %q", got, "HELLO GO WORLD")
	}
	if got := JoinUpper(nil); got != "" {
		t.Errorf("JoinUpper(nil) = %q, want empty", got)
	}
}
