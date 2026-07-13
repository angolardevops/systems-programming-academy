//! Micro-benchmark: **static dispatch (generics) vs dynamic dispatch (`dyn`)**.
//!
//! Run it in release mode for representative numbers:
//!
//! ```text
//! cargo run --release --bin dispatch
//! ```
//!
//! Both loops sum the areas of the same shapes many times. The static version
//! monomorphizes and inlines `area()`; the dynamic version goes through a
//! vtable on every call. The gap is what the lesson's discussion refers to.

use std::time::Instant;

use traits::{Circle, Rectangle, Shape, Triangle};

const ITERS: u32 = 20_000;

fn time_it(label: &str, mut f: impl FnMut() -> f64) {
    // Warm up so we measure steady state.
    let mut acc = 0.0;
    for _ in 0..100 {
        acc += f();
    }
    let start = Instant::now();
    for _ in 0..ITERS {
        acc += f();
    }
    let per_iter = start.elapsed().as_nanos() as f64 / ITERS as f64;
    // Print acc so the optimizer cannot delete the loop as dead code.
    println!("{label:<26} {per_iter:>10.1} ns/iter   (checksum {acc:.1})");
}

// Static dispatch: generic over the concrete shape type.
fn sum_static<S: Shape>(shapes: &[S]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

// Dynamic dispatch: each element is a trait object behind a pointer.
fn sum_dynamic(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

fn main() {
    // Homogeneous data for the static path (all circles).
    let circles: Vec<Circle> = (0..1_000).map(|i| Circle { radius: i as f64 }).collect();

    // Heterogeneous data for the dynamic path (mixed shapes).
    let mixed: Vec<Box<dyn Shape>> = (0..1_000)
        .map(|i| -> Box<dyn Shape> {
            match i % 3 {
                0 => Box::new(Circle { radius: i as f64 }),
                1 => Box::new(Rectangle {
                    width: i as f64,
                    height: 2.0,
                }),
                _ => Box::new(Triangle {
                    base: i as f64,
                    height: 3.0,
                }),
            }
        })
        .collect();

    println!("Summing areas of 1,000 shapes, {ITERS} iterations:\n");
    time_it("static dispatch (generic)", || sum_static(&circles));
    time_it("dynamic dispatch (dyn)", || sum_dynamic(&mixed));

    println!(
        "\nStatic dispatch inlines area(); dynamic goes through a vtable.\n\
         Reach for `dyn` when you need heterogeneous collections; reach for\n\
         generics when the type is fixed and the hot path matters."
    );
}
