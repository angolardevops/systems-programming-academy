package generics

import (
	"reflect"
	"sort"
	"testing"
)

func TestSum(t *testing.T) {
	if got := Sum([]int{1, 2, 3, 4}); got != 10 {
		t.Errorf("Sum(ints) = %d, want 10", got)
	}
	if got := Sum([]float64{1.5, 2.5}); got != 4.0 {
		t.Errorf("Sum(floats) = %v, want 4.0", got)
	}
}

// A named type whose underlying type is float64 works because Number uses ~float64.
type Celsius float64

func TestSumNamedType(t *testing.T) {
	if got := Sum([]Celsius{20, 22}); got != 42 {
		t.Errorf("Sum(Celsius) = %v, want 42", got)
	}
}

func TestMap(t *testing.T) {
	got := Map([]int{1, 2, 3}, func(n int) int { return n * n })
	if !reflect.DeepEqual(got, []int{1, 4, 9}) {
		t.Errorf("Map squares = %v, want [1 4 9]", got)
	}
	// T and U differ: int -> string.
	strs := Map([]int{1, 2}, func(n int) string {
		if n%2 == 0 {
			return "even"
		}
		return "odd"
	})
	if !reflect.DeepEqual(strs, []string{"odd", "even"}) {
		t.Errorf("Map to strings = %v", strs)
	}
}

func TestFilter(t *testing.T) {
	got := Filter([]int{1, 2, 3, 4, 5, 6}, func(n int) bool { return n%2 == 0 })
	if !reflect.DeepEqual(got, []int{2, 4, 6}) {
		t.Errorf("Filter evens = %v, want [2 4 6]", got)
	}
}

func TestKeys(t *testing.T) {
	got := Keys(map[string]int{"a": 1, "b": 2, "c": 3})
	sort.Strings(got) // map order is random; sort for a stable assertion
	if !reflect.DeepEqual(got, []string{"a", "b", "c"}) {
		t.Errorf("Keys = %v, want [a b c]", got)
	}
}

func TestMax(t *testing.T) {
	if got := Max(3, 8); got != 8 {
		t.Errorf("Max(3, 8) = %d, want 8", got)
	}
	if got := Max("apple", "pear"); got != "pear" {
		t.Errorf("Max strings = %q, want pear", got)
	}
	if got := Max(5, 5); got != 5 {
		t.Errorf("Max(5, 5) = %d, want 5", got)
	}
}

func TestGenericStack(t *testing.T) {
	var s Stack[string]
	if _, ok := s.Pop(); ok {
		t.Error("Pop on empty stack should return ok=false")
	}
	s.Push("a")
	s.Push("b")
	if s.Len() != 2 {
		t.Errorf("Len = %d, want 2", s.Len())
	}
	top, ok := s.Pop()
	if !ok || top != "b" {
		t.Errorf("Pop = %q, %v; want b, true", top, ok)
	}
	if s.Len() != 1 {
		t.Errorf("Len after pop = %d, want 1", s.Len())
	}
}
