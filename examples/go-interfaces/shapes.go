// Package shapes is the tested companion code for the Academy lesson
// "Structs, Methods & Interfaces". It shows Go's type system: structs, value vs
// pointer receivers, implicitly-satisfied interfaces, composition by embedding,
// and type switches.
//
// Run the tests with:
//
//	go test ./...
package shapes

import (
	"fmt"
	"math"
)

// Shape is the behaviour shared by everything with an area and a perimeter.
// In Go, a type satisfies an interface simply by having the right methods —
// there is no "implements" keyword. This is structural, implicit satisfaction.
type Shape interface {
	Area() float64
	Perimeter() float64
}

// Rectangle is a value type. Its methods use a value receiver because they only
// read fields — no need to mutate, and copying two floats is cheap.
type Rectangle struct {
	Width, Height float64
}

// Area returns width times height.
func (r Rectangle) Area() float64 { return r.Width * r.Height }

// Perimeter returns twice the sum of the sides.
func (r Rectangle) Perimeter() float64 { return 2 * (r.Width + r.Height) }

// Circle is another Shape.
type Circle struct {
	Radius float64
}

// Area returns pi r squared.
func (c Circle) Area() float64 { return math.Pi * c.Radius * c.Radius }

// Perimeter returns the circumference.
func (c Circle) Perimeter() float64 { return 2 * math.Pi * c.Radius }

// String makes Circle satisfy fmt.Stringer, so it formats nicely with %v/%s.
func (c Circle) String() string { return fmt.Sprintf("Circle(r=%g)", c.Radius) }

// TotalArea sums the areas of any collection of shapes — the payoff of coding to
// an interface rather than concrete types.
func TotalArea(shapes []Shape) float64 {
	var total float64
	for _, s := range shapes {
		total += s.Area()
	}
	return total
}

// KindOf reports the concrete type behind a Shape using a type switch — the Go
// way to recover the dynamic type when you need it.
func KindOf(s Shape) string {
	switch s.(type) {
	case Rectangle:
		return "rectangle"
	case Circle:
		return "circle"
	default:
		return "unknown"
	}
}

// Counter demonstrates a pointer receiver. Inc must take *Counter because it
// mutates the receiver; a value receiver would modify a copy and be lost.
type Counter struct {
	n int
}

// Inc increments the counter in place.
func (c *Counter) Inc() { c.n++ }

// Value reads the current count.
func (c *Counter) Value() int { return c.n }

// Labeled shows composition by embedding: a Labeled *has a* Shape and gains its
// methods, so Labeled itself satisfies Shape without re-declaring anything.
// Go favours composition over inheritance.
type Labeled struct {
	Name string
	Shape
}

// Describe uses the embedded Shape's methods plus the label. Because Shape is
// embedded, we can call l.Area() directly.
func (l Labeled) Describe() string {
	return fmt.Sprintf("%s: area=%.2f perimeter=%.2f", l.Name, l.Area(), l.Perimeter())
}
