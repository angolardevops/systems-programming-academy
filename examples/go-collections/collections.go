// Package collections is the tested companion code for the Academy lesson
// "Go: Slices, Maps & Strings". It shows the three workhorse types: slices
// (a view over a backing array), maps (hash tables), and strings (immutable
// UTF-8), plus the idioms that keep their sharp edges safe.
//
// Run the tests with:
//
//	go test ./...
package collections

import (
	"sort"
	"strings"
	"unicode"
)

// Pair is a word and its count, used for deterministic sorted output.
type Pair struct {
	Word  string
	Count int
}

// WordCount counts words case-insensitively, ignoring surrounding punctuation.
// Demonstrates the map "comma ok" pattern via direct indexing: reading a missing
// key yields the zero value (0), so counts[w]++ just works.
func WordCount(text string) map[string]int {
	counts := make(map[string]int)
	for _, raw := range strings.Fields(text) {
		word := strings.ToLower(strings.TrimFunc(raw, func(r rune) bool {
			return !unicode.IsLetter(r) && !unicode.IsNumber(r)
		}))
		if word == "" {
			continue
		}
		counts[word]++
	}
	return counts
}

// TopWords returns the n most frequent words, most-frequent first, breaking ties
// alphabetically so the result is deterministic (map iteration order is not).
func TopWords(text string, n int) []Pair {
	pairs := make([]Pair, 0, len(WordCount(text)))
	for w, c := range WordCount(text) {
		pairs = append(pairs, Pair{Word: w, Count: c})
	}
	sort.Slice(pairs, func(i, j int) bool {
		if pairs[i].Count != pairs[j].Count {
			return pairs[i].Count > pairs[j].Count // higher count first
		}
		return pairs[i].Word < pairs[j].Word // then alphabetical
	})
	if n > len(pairs) {
		n = len(pairs)
	}
	return pairs[:n]
}

// SumEvens sums the even numbers in a slice. Ranging over a slice yields index
// and value; the blank identifier _ discards the index.
func SumEvens(nums []int) int {
	total := 0
	for _, n := range nums {
		if n%2 == 0 {
			total += n
		}
	}
	return total
}

// UniqueSorted returns the input sorted with duplicates removed. It copies first
// so the caller's slice is never mutated — sort.Ints sorts in place.
func UniqueSorted(nums []int) []int {
	cp := make([]int, len(nums))
	copy(cp, nums)
	sort.Ints(cp)

	out := cp[:0] // reuse the backing array: filter in place
	for i, n := range cp {
		if i == 0 || n != cp[i-1] {
			out = append(out, n)
		}
	}
	return out
}

// RuneCount returns the number of Unicode characters (runes), which differs from
// len(s) — the byte count — for any non-ASCII text.
func RuneCount(s string) int {
	count := 0
	for range s { // ranging a string yields runes, not bytes
		count++
	}
	return count
}

// JoinUpper upper-cases each word and joins them with a space, using
// strings.Builder to avoid the quadratic cost of repeated string concatenation.
func JoinUpper(words []string) string {
	var b strings.Builder
	for i, w := range words {
		if i > 0 {
			b.WriteByte(' ')
		}
		b.WriteString(strings.ToUpper(w))
	}
	return b.String()
}
