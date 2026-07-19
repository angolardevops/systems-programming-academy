//! vars — run a small program with variables.
//!
//! Usage:
//!   vars "x = 5; y = x * 2; y - x"      one program, statements split on ';'
//!   printf 'x = 5\ny = x * 2\n' | vars   or one statement per stdin line
//!
//! State persists across statements, so later lines can read what earlier lines
//! bound. Each statement prints its value, or a clear error.

use part7_vars_rust::run_program;
use std::io::Read;

fn main() {
    let arg = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let src = if arg.trim().is_empty() {
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        buf
    } else {
        // Allow ';' as a statement separator on the command line.
        arg.replace(';', "\n")
    };
    for line in run_program(&src) {
        println!("{line}");
    }
}
