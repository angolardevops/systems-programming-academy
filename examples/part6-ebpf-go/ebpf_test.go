package main

import (
	"reflect"
	"strings"
	"testing"
)

func TestBucketsFollowPowersOfTwo(t *testing.T) {
	cases := map[uint64]int{0: 0, 1: 0, 2: 1, 3: 1, 4: 2, 7: 2, 8: 3, 1023: 9, 1024: 10}
	for v, want := range cases {
		if got := Bucket(v); got != want {
			t.Errorf("Bucket(%d) = %d, want %d", v, got, want)
		}
	}
}

func TestBucketRangesAreInclusivePowersOfTwo(t *testing.T) {
	cases := []struct {
		k         int
		low, high uint64
	}{{0, 0, 1}, {1, 2, 3}, {2, 4, 7}, {4, 16, 31}}
	for _, c := range cases {
		if low, high := BucketRange(c.k); low != c.low || high != c.high {
			t.Errorf("BucketRange(%d) = (%d,%d), want (%d,%d)", c.k, low, high, c.low, c.high)
		}
	}
}

func TestTallyBinsSamplesByBucket(t *testing.T) {
	got := Tally([]uint64{1, 2, 3, 5, 5, 6, 9})
	want := []uint64{1, 2, 3, 1}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("Tally = %v, want %v", got, want)
	}
}

func TestRendersTheHistogram(t *testing.T) {
	counts := []uint64{0, 2, 3, 4, 1}
	want := "             usecs : count    distribution\n" +
		"            0 -> 1 : 0        |                                        |\n" +
		"            2 -> 3 : 2        |********************                    |\n" +
		"            4 -> 7 : 3        |******************************          |\n" +
		"           8 -> 15 : 4        |****************************************|\n" +
		"          16 -> 31 : 1        |**********                              |"
	if got := RenderHistogram(counts, "usecs"); got != want {
		t.Errorf("histogram\n got:\n%s\nwant:\n%s", got, want)
	}
}

func TestRendersHeaderOnlyWhenEmpty(t *testing.T) {
	got := RenderHistogram([]uint64{0, 0, 0}, "usecs")
	want := "             usecs : count    distribution"
	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestTrailingEmptyBucketsAreTrimmed(t *testing.T) {
	out := RenderHistogram([]uint64{3, 1, 0, 0}, "nsecs")
	if n := strings.Count(out, "\n") + 1; n != 3 {
		t.Errorf("line count = %d, want 3", n)
	}
	if !strings.Contains(out, "2 -> 3 : 1") {
		t.Error("expected the 2->3 row")
	}
	if strings.Contains(out, "4 -> 7") {
		t.Error("trailing empty bucket 4->7 should be trimmed")
	}
}
