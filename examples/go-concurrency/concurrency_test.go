package concurrency

import (
	"reflect"
	"testing"
)

func TestSquareAll(t *testing.T) {
	got := SquareAll([]int{1, 2, 3, 4})
	want := []int{1, 4, 9, 16}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("SquareAll = %v, want %v", got, want)
	}
	if got := SquareAll(nil); len(got) != 0 {
		t.Errorf("SquareAll(nil) = %v, want empty", got)
	}
}

func TestPipeline(t *testing.T) {
	got := Pipeline([]int{1, 2, 3, 4, 5})
	want := []int{1, 4, 9, 16, 25} // pipeline preserves order
	if !reflect.DeepEqual(got, want) {
		t.Errorf("Pipeline = %v, want %v", got, want)
	}
}

func TestSharedCounterHasNoLostUpdates(t *testing.T) {
	// 8 workers * 1000 increments = 8000, every single run (test with -race).
	if got := SharedCounter(8, 1000); got != 8000 {
		t.Errorf("SharedCounter(8, 1000) = %d, want 8000", got)
	}
}

func TestSharedCounterEdges(t *testing.T) {
	if got := SharedCounter(0, 100); got != 0 {
		t.Errorf("SharedCounter(0, 100) = %d, want 0", got)
	}
	if got := SharedCounter(5, 0); got != 0 {
		t.Errorf("SharedCounter(5, 0) = %d, want 0", got)
	}
}

func TestFanInSum(t *testing.T) {
	nums := make([]int, 100)
	for i := range nums {
		nums[i] = i + 1 // 1..100, sum = 5050
	}
	for _, workers := range []int{1, 3, 4, 7, 100} {
		if got := FanInSum(nums, workers); got != 5050 {
			t.Errorf("FanInSum(1..100, %d workers) = %d, want 5050", workers, got)
		}
	}
}
