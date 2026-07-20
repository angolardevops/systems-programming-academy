//! loops — run an imperative program (statements, print, while loops).
//!
//! Usage:
//!   loops "i = 1; while i <= 5 do { print i; i = i + 1 }"
//!   cat program.txt | loops

use part7_loops_rust::run_program;
use std::io::Read;

fn main() {
    let arg = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let src = if arg.trim().is_empty() {
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        buf
    } else {
        arg
    };
    for line in run_program(&src) {
        println!("{line}");
    }
}
