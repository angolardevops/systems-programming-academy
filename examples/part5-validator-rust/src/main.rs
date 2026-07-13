//! Demo: validate a broken signup record and print every error at once —
//! the fail-complete behaviour a real form wants.

use part5_validator_rust::{error_lines, Rule, Schema};
use std::collections::HashMap;

fn main() {
    let schema = Schema::new()
        .field(
            "username",
            vec![Rule::Required, Rule::MinLength(3), Rule::MaxLength(20)],
        )
        .field("age", vec![Rule::Required, Rule::InRange(18, 120)])
        .field(
            "role",
            vec![Rule::OneOf(vec![
                "admin".to_string(),
                "user".to_string(),
                "guest".to_string(),
            ])],
        );

    let mut bad = HashMap::new();
    bad.insert("username".to_string(), "ab".to_string()); // too short
    bad.insert("age".to_string(), "twelve".to_string()); // not an int
    bad.insert("role".to_string(), "superadmin".to_string()); // not allowed

    println!("Validating a broken signup:");
    for line in error_lines(&schema.validate(&bad)) {
        println!("  - {line}");
    }

    let mut good = HashMap::new();
    good.insert("username".to_string(), "walter".to_string());
    good.insert("age".to_string(), "34".to_string());
    good.insert("role".to_string(), "admin".to_string());
    let errors = schema.validate(&good);
    println!("\nValidating a good signup: {} errors", errors.len());
}
