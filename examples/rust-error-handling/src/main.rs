//! End-to-end runnable demo for the error-handling lesson.
//!
//! Run it with:
//!
//! ```text
//! cargo run
//! ```
//!
//! It shows both strategies from the library: fail-soft parsing (keep good
//! rows, report bad ones) and the `?` operator propagating an error out of
//! `main`. `main` returns `Result<(), Box<dyn Error>>`, so any error is printed
//! by the runtime and the process exits with a non-zero status.

use std::error::Error;

use error_handling::{first_adult, parse_users_lenient, ParseError, User};

fn main() -> Result<(), Box<dyn Error>> {
    // A deliberately messy input: two good rows, one bad, one blank line.
    let input = "\
Ada Lovelace, 36
Bob, not-a-number
Carol Shaw, 40
";

    println!("== Lenient parse (keep good rows) ==");
    let (users, errors) = parse_users_lenient(input);
    for u in &users {
        println!("  ok:   {} ({})", u.name, u.age);
    }
    for (line, err) in &errors {
        println!("  skip: line {line}: {err}");
    }

    match first_adult(&users) {
        Some(u) => println!("\nFirst adult: {}", u.name),
        None => println!("\nNo adults found."),
    }

    // Now show `?` propagating a typed error. `parse_one` returns a Result and
    // the `?` hands any ParseError back to `main`, which prints it and exits 1.
    println!("\n== Strict parse (propagate first error) ==");
    let user = parse_one("Grace Hopper, 45")?;
    println!("  parsed: {} ({})", user.name, user.age);

    // This line WOULD return an error if uncommented — try it:
    // let bad = parse_one("oops")?;
    // println!("{bad:?}");

    Ok(())
}

/// Thin wrapper so the `?` in `main` has something to propagate from.
fn parse_one(line: &str) -> Result<User, ParseError> {
    error_handling::parse_user(line)
}
