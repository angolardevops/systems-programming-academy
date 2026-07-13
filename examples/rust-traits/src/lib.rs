//! Companion library for the lesson **Types & Traits**.
//!
//! A trait is Rust's way to describe *shared behaviour* — the closest thing to
//! an interface, but resolved either at compile time (generics) or at run time
//! (`dyn Trait`). The running example is a set of shapes that all know how to
//! report their `area`.
//!
//! Every public item is referenced from the lesson and covered by the tests at
//! the bottom of this file:
//!
//! ```text
//! cargo test
//! ```

/// Shared behaviour for anything with a computable area.
///
/// `name` and `area` are *required* methods (each implementor must provide
/// them). `describe` is a *default* method built from the required ones — an
/// implementor gets it for free but may override it.
pub trait Shape {
    /// A human-readable name for the shape kind.
    fn name(&self) -> &str;

    /// The shape's area in square units.
    fn area(&self) -> f64;

    /// Default method: a one-line summary. Implementors need not write this.
    fn describe(&self) -> String {
        format!("{} with area {:.2}", self.name(), self.area())
    }
}

/// A circle, defined by its radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub radius: f64,
}

impl Shape for Circle {
    fn name(&self) -> &str {
        "circle"
    }
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

/// An axis-aligned rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

impl Shape for Rectangle {
    fn name(&self) -> &str {
        "rectangle"
    }
    fn area(&self) -> f64 {
        self.width * self.height
    }
    // Override the default to prove it can be specialised.
    fn describe(&self) -> String {
        format!(
            "{}x{} rectangle, area {:.2}",
            self.width,
            self.height,
            self.area()
        )
    }
}

/// A triangle, defined by base and height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    pub base: f64,
    pub height: f64,
}

impl Shape for Triangle {
    fn name(&self) -> &str {
        "triangle"
    }
    fn area(&self) -> f64 {
        0.5 * self.base * self.height
    }
}

/// **Static dispatch** (generics + trait bound). The compiler generates a
/// specialised copy of this function per concrete `S` — no vtable, fully
/// inlinable. Use when the type is known at compile time.
pub fn area_of<S: Shape>(shape: &S) -> f64 {
    shape.area()
}

/// **Dynamic dispatch** (trait object). One function handles a *heterogeneous*
/// collection of shapes; the concrete method is looked up at run time through a
/// vtable. Use when you need to mix different types in one container.
pub fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

/// Returns the shape with the largest area, or `None` if the slice is empty.
/// Works over trait objects, so the shapes may be of different kinds.
pub fn largest(shapes: &[Box<dyn Shape>]) -> Option<&dyn Shape> {
    shapes
        .iter()
        .map(|b| b.as_ref())
        .max_by(|a, b| a.area().partial_cmp(&b.area()).unwrap())
}

/// `impl Trait` in return position: hand back *some* concrete `Shape` without
/// naming the type. Here it is always a `Circle`, but callers only see `Shape`.
pub fn unit_circle() -> impl Shape {
    Circle { radius: 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A small helper so the assertions read cleanly despite float imprecision.
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn circle_area_uses_pi_r_squared() {
        let c = Circle { radius: 2.0 };
        assert!(close(c.area(), std::f64::consts::PI * 4.0));
        assert_eq!(c.name(), "circle");
    }

    #[test]
    fn rectangle_area_is_width_times_height() {
        let r = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        assert!(close(r.area(), 12.0));
    }

    #[test]
    fn triangle_area_is_half_base_height() {
        let t = Triangle {
            base: 6.0,
            height: 4.0,
        };
        assert!(close(t.area(), 12.0));
    }

    #[test]
    fn default_describe_is_used_when_not_overridden() {
        let c = Circle { radius: 1.0 };
        assert_eq!(c.describe(), "circle with area 3.14");
    }

    #[test]
    fn describe_can_be_overridden() {
        let r = Rectangle {
            width: 2.0,
            height: 5.0,
        };
        assert_eq!(r.describe(), "2x5 rectangle, area 10.00");
    }

    #[test]
    fn static_dispatch_returns_area() {
        let t = Triangle {
            base: 10.0,
            height: 2.0,
        };
        assert!(close(area_of(&t), 10.0));
    }

    #[test]
    fn dynamic_dispatch_sums_heterogeneous_shapes() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Circle { radius: 1.0 }), // ~3.14159
            Box::new(Rectangle {
                width: 2.0,
                height: 3.0,
            }), // 6.0
            Box::new(Triangle {
                base: 4.0,
                height: 2.0,
            }), // 4.0
        ];
        assert!(close(total_area(&shapes), std::f64::consts::PI + 10.0));
    }

    #[test]
    fn largest_finds_biggest_and_handles_empty() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Rectangle {
                width: 2.0,
                height: 2.0,
            }), // 4.0
            Box::new(Circle { radius: 3.0 }), // ~28.27
        ];
        assert_eq!(largest(&shapes).unwrap().name(), "circle");

        let empty: Vec<Box<dyn Shape>> = Vec::new();
        assert!(largest(&empty).is_none());
    }

    #[test]
    fn impl_trait_return_is_a_real_shape() {
        let c = unit_circle();
        assert!(close(c.area(), std::f64::consts::PI));
        assert_eq!(c.name(), "circle");
    }
}
