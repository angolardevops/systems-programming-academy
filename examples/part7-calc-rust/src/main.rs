//! calc — a REPL/one-shot for the integer arithmetic interpreter.
//!
//! Usage:
//!   calc "1 + 2 * 3"      evaluate one expression from the argument
//!   echo "2 * (3+4)" | calc   evaluate each line read from stdin
//!
//! For each expression it prints the parsed syntax tree (as an S-expression) and
//! the value — or a clear error, never a crash.

use part7_calc_rust::run;
use std::io::BufRead;

fn main() {
    let arg = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    if !arg.trim().is_empty() {
        report(&arg);
        return;
    }
    // No argument: act as a filter over stdin, one expression per line.
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if !line.trim().is_empty() {
            report(&line);
        }
    }
}

fn report(src: &str) {
    match run(src) {
        Ok((sexp, value)) => println!("{src}  =>  {sexp}  =>  {value}"),
        Err(e) => println!("{src}  =>  error: {e}"),
    }
}
