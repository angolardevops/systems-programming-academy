package shapes

import (
	"math"
	"testing"
)

const eps = 1e-9

func TestRectangleAreaAndPerimeter(t *testing.T) {
	r := Rectangle{Width: 3, Height: 4}
	if got := r.Area(); math.Abs(got-12) > eps {
		t.Errorf("Area() = %v, want 12", got)
	}
	if got := r.Perimeter(); math.Abs(got-14) > eps {
		t.Errorf("Perimeter() = %v, want 14", got)
	}
}

func TestCircleAreaAndStringer(t *testing.T) {
	c := Circle{Radius: 2}
	if got := c.Area(); math.Abs(got-math.Pi*4) > eps {
		t.Errorf("Area() = %v, want ~12.566", got)
	}
	if got := c.String(); got != "Circle(r=2)" {
		t.Errorf("String() = %q, want %q", got, "Circle(r=2)")
	}
}

// Table-driven test: the idiomatic Go pattern for covering many cases compactly.
func TestTotalArea(t *testing.T) {
	cases := []struct {
		name   string
		shapes []Shape
		want   float64
	}{
		{"empty", nil, 0},
		{"one rectangle", []Shape{Rectangle{2, 3}}, 6},
		{"mixed", []Shape{Rectangle{2, 3}, Circle{Radius: 1}}, 6 + math.Pi},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if got := TotalArea(tc.shapes); math.Abs(got-tc.want) > eps {
				t.Errorf("TotalArea() = %v, want %v", got, tc.want)
			}
		})
	}
}

func TestKindOf(t *testing.T) {
	if got := KindOf(Rectangle{1, 1}); got != "rectangle" {
		t.Errorf("KindOf(Rectangle) = %q, want rectangle", got)
	}
	if got := KindOf(Circle{Radius: 1}); got != "circle" {
		t.Errorf("KindOf(Circle) = %q, want circle", got)
	}
}

func TestPointerReceiverMutates(t *testing.T) {
	var c Counter
	c.Inc()
	c.Inc()
	c.Inc()
	if got := c.Value(); got != 3 {
		t.Errorf("Value() = %d, want 3", got)
	}
}

func TestEmbeddingSatisfiesInterface(t *testing.T) {
	// Labeled embeds Shape, so it satisfies Shape itself and can go in a []Shape.
	l := Labeled{Name: "unit square", Shape: Rectangle{1, 1}}
	shapes := []Shape{l}
	if got := TotalArea(shapes); math.Abs(got-1) > eps {
		t.Errorf("TotalArea([Labeled]) = %v, want 1", got)
	}
	want := "unit square: area=1.00 perimeter=4.00"
	if got := l.Describe(); got != want {
		t.Errorf("Describe() = %q, want %q", got, want)
	}
}
