// Package stringutil is the tested companion code for the Academy lesson
// "Go: Testing & Documentation". The functions are deliberately small so the
// spotlight is on the tests: table-driven cases, subtests, runnable examples,
// and benchmarks.
//
// Run everything with:
//
//	go test -v ./...
//	go test -bench . ./...
package stringutil

import (
	"strings"
	"unicode"
)

// Reverse returns s with its characters in reverse order. It reverses by rune,
// not by byte, so multi-byte UTF-8 characters survive intact.
func Reverse(s string) string {
	runes := []rune(s)
	for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
		runes[i], runes[j] = runes[j], runes[i]
	}
	return string(runes)
}

// IsPalindrome reports whether s reads the same forwards and backwards, ignoring
// case and any non-letter characters (so "A man, a plan, a canal: Panama" is a
// palindrome).
func IsPalindrome(s string) bool {
	var letters []rune
	for _, r := range s {
		if unicode.IsLetter(r) {
			letters = append(letters, unicode.ToLower(r))
		}
	}
	for i, j := 0, len(letters)-1; i < j; i, j = i+1, j-1 {
		if letters[i] != letters[j] {
			return false
		}
	}
	return true
}

// CountVowels returns how many ASCII vowels (a, e, i, o, u, case-insensitive)
// appear in s.
func CountVowels(s string) int {
	count := 0
	for _, r := range strings.ToLower(s) {
		switch r {
		case 'a', 'e', 'i', 'o', 'u':
			count++
		}
	}
	return count
}
