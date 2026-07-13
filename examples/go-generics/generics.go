// Package generics is the tested companion code for the Academy lesson
// "Go: Generics". Type parameters (Go 1.18+) let one function or type work for
// many element types with compile-time safety and no interface{} boxing.
//
// Run the tests with:
//
//	go test ./...
package generics

// Number is a constraint: a type set of everything we can sum. The ~ means
// "any type whose underlying type is this", so named types like `type Celsius
// float64` are included.
type Number interface {
	~int | ~int64 | ~float64
}

// Sum adds every element of a slice. One definition works for []int, []float64,
// and any named numeric type — chosen at the call site, checked at compile time.
func Sum[T Number](values []T) T {
	var total T
	for _, v := range values {
		total += v
	}
	return total
}

// Map applies f to each element, returning a new slice. It is generic over TWO
// type parameters: the input element type T and the result type U.
func Map[T, U any](s []T, f func(T) U) []U {
	out := make([]U, len(s))
	for i, v := range s {
		out[i] = f(v)
	}
	return out
}

// Filter returns the elements for which keep reports true. `any` is the
// most permissive constraint (an alias for interface{}), used when the body
// doesn't rely on any operations on T.
func Filter[T any](s []T, keep func(T) bool) []T {
	out := make([]T, 0, len(s))
	for _, v := range s {
		if keep(v) {
			out = append(out, v)
		}
	}
	return out
}

// Keys returns the keys of a map. `comparable` is the built-in constraint for
// types usable as map keys or with == (so it can be a map key here).
func Keys[K comparable, V any](m map[K]V) []K {
	out := make([]K, 0, len(m))
	for k := range m {
		out = append(out, k)
	}
	return out
}

// Max returns the larger of two values. `Ordered` is our constraint for types
// that support the < operator.
type Ordered interface {
	~int | ~int64 | ~float64 | ~string
}

// Max returns the larger of a and b (a on ties).
func Max[T Ordered](a, b T) T {
	if a >= b {
		return a
	}
	return b
}

// Stack is a generic LIFO container. The type parameter is on the type itself,
// so Stack[int] and Stack[string] are distinct, fully type-checked types.
type Stack[T any] struct {
	items []T
}

// Push adds an element to the top.
func (s *Stack[T]) Push(v T) {
	s.items = append(s.items, v)
}

// Pop removes and returns the top element, plus ok=false if the stack is empty.
func (s *Stack[T]) Pop() (T, bool) {
	var zero T
	if len(s.items) == 0 {
		return zero, false
	}
	top := s.items[len(s.items)-1]
	s.items = s.items[:len(s.items)-1]
	return top, true
}

// Len reports how many elements are on the stack.
func (s *Stack[T]) Len() int { return len(s.items) }
