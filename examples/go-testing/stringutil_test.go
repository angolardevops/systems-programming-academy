package stringutil

import (
	"fmt"
	"testing"
)

// A plain test: the simplest form. Name it TestXxx, take *testing.T, and fail
// with t.Errorf (keeps going) or t.Fatalf (stops this test).
func TestReverseASCII(t *testing.T) {
	if got := Reverse("Hello"); got != "olleH" {
		t.Errorf("Reverse(\"Hello\") = %q, want %q", got, "olleH")
	}
}

// Reverse must be rune-aware, not byte-aware.
func TestReverseUnicode(t *testing.T) {
	if got := Reverse("héllo"); got != "olléh" {
		t.Errorf("Reverse(\"héllo\") = %q, want %q", got, "olléh")
	}
}

// Table-driven test with subtests: the idiomatic Go pattern. Each case is a row;
// t.Run gives each its own name in the output and lets you run one with
// `go test -run TestIsPalindrome/phrase`.
func TestIsPalindrome(t *testing.T) {
	cases := []struct {
		name string
		in   string
		want bool
	}{
		{"empty", "", true},
		{"single", "x", true},
		{"simple", "level", true},
		{"mixed case", "RaceCar", true},
		{"phrase", "A man, a plan, a canal: Panama", true},
		{"not", "hello", false},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if got := IsPalindrome(tc.in); got != tc.want {
				t.Errorf("IsPalindrome(%q) = %v, want %v", tc.in, got, tc.want)
			}
		})
	}
}

func TestCountVowels(t *testing.T) {
	cases := []struct {
		in   string
		want int
	}{
		{"", 0},
		{"xyz", 0},
		{"hello", 2},
		{"AEIOU", 5},
	}
	for _, tc := range cases {
		t.Run(tc.in, func(t *testing.T) {
			if got := CountVowels(tc.in); got != tc.want {
				t.Errorf("CountVowels(%q) = %d, want %d", tc.in, got, tc.want)
			}
		})
	}
}

// A test helper: t.Helper() makes failure line numbers point at the caller, not
// at this function — so the failure shows the assertion, not the helper.
func assertEqual(t *testing.T, got, want string) {
	t.Helper()
	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestReverseTwiceIsIdentity(t *testing.T) {
	in := "testing"
	assertEqual(t, Reverse(Reverse(in)), in)
}

// An Example function is compiled, run, and its stdout compared to the
// // Output: comment — so it doubles as documentation AND a test.
func ExampleReverse() {
	fmt.Println(Reverse("Go"))
	// Output: oG
}

func ExampleIsPalindrome() {
	fmt.Println(IsPalindrome("RaceCar"))
	fmt.Println(IsPalindrome("hello"))
	// Output:
	// true
	// false
}

// A benchmark: run with `go test -bench .`. The loop runs b.N times, chosen by
// the framework to get a stable measurement.
func BenchmarkReverse(b *testing.B) {
	s := "the quick brown fox jumps over the lazy dog"
	for i := 0; i < b.N; i++ {
		_ = Reverse(s)
	}
}
