//! fns — run a small program with functions and closures.
//!
//! Usage:
//!   fns "double(x) = x * 2; double(21)"       statements split on ';'
//!   printf 'inc(n) = n + 1\ninc(41)\n' | fns   or one statement per stdin line

use part7_fns_rust::run_program;
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
