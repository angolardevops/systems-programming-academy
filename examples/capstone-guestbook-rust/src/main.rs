//! Demo: run a guestbook through the full pipeline, including two hostile
//! submissions, and print the safely-rendered page.

use capstone_guestbook_rust::{render_page, submit, Store};

fn main() {
    let mut store = Store::new();

    submit(&mut store, "Ana", "Love this academy!");
    submit(&mut store, "Bruno", "The Rust track is superb.");

    // Two attacks, submitted as ordinary comments:
    let sqli = submit(&mut store, "Mallory", "'; DROP TABLE comments; --");
    let xss = submit(&mut store, "Eve", "<script>alert('pwned')</script>");
    println!("SQLi submission errors: {sqli:?}  (empty = accepted as data)");
    println!("XSS submission errors:  {xss:?}\n");

    // Invalid submission, rejected with accumulated errors:
    let bad = submit(&mut store, "X", "");
    println!("Invalid submission errors: {bad:?}\n");

    println!("Rendered page (all values autoescaped):");
    println!("{}", render_page(&store));
    println!(
        "\n-- {} comments stored; the table was never dropped; no live <script>. --",
        store.all().len()
    );
}
