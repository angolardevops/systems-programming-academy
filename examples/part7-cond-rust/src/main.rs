//! cond — run a small program with conditionals, comparisons, and recursion.
//!
//! Usage:
//!   cond "fact(n) = if n <= 1 then 1 else n * fact(n - 1); fact(6)"
//!   printf 'x = 5\nif x < 3 then 1 else 2\n' | cond

use part7_cond_rust::run_program;
use std::io::Read;

fn main() {
    let arg = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let src = if arg.trim().is_empty() {
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        buf
    } else {
        arg.replace(';', "\n")
    };
    for line in run_program(&src) {
        println!("{line}");
    }
}
