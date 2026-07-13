//! A declarative validation framework: describe the rules a record must
//! satisfy, then validate a record against them and get back *every* error at
//! once.
//!
//! The framework idea is **declaration over imperative checking**. Instead of
//! hand-writing `if name.is_empty() { ... } if age < 18 { ... }` scattered
//! through a handler, you declare a schema — "name is required and at least 2
//! chars; age is an integer in 18..=120" — and the evaluator applies it.
//!
//! The one design decision that matters most is **error accumulation**: the
//! validator collects *all* failures and returns them together, rather than
//! bailing on the first. A form that reports one error per submit ("fix email"
//! ... "now fix password" ... "now fix age") is a terrible experience; a form
//! that reports all four at once is a good one. Fail-fast is for programming
//! errors; user input wants fail-complete.
//!
//! Rules are plain checks (no regex crate), so the framework is dependency-free
//! and the collected errors are directly assertable — no I/O anywhere.

use std::collections::HashMap;

/// One validation rule applied to a single field's value.
#[derive(Clone)]
pub enum Rule {
    /// The value must be present and non-empty.
    Required,
    /// At least `n` characters.
    MinLength(usize),
    /// At most `n` characters.
    MaxLength(usize),
    /// Must parse as an integer.
    IsInt,
    /// Must parse as an integer within `lo..=hi` (implies [`Rule::IsInt`]).
    InRange(i64, i64),
    /// Must be one of the allowed values.
    OneOf(Vec<String>),
}

impl Rule {
    /// Checks `value` against this rule, returning an error message if it
    /// fails, or `None` if it passes. Messages are stable text — they are the
    /// cross-language contract asserted by the tests.
    fn check(&self, value: &str) -> Option<String> {
        match self {
            Rule::Required => {
                // Handled by the schema before other rules; here for
                // completeness if used directly.
                if value.is_empty() {
                    Some("is required".to_string())
                } else {
                    None
                }
            }
            Rule::MinLength(n) => {
                if value.chars().count() < *n {
                    Some(format!("must be at least {n} characters"))
                } else {
                    None
                }
            }
            Rule::MaxLength(n) => {
                if value.chars().count() > *n {
                    Some(format!("must be at most {n} characters"))
                } else {
                    None
                }
            }
            Rule::IsInt => {
                if value.parse::<i64>().is_err() {
                    Some("must be an integer".to_string())
                } else {
                    None
                }
            }
            Rule::InRange(lo, hi) => match value.parse::<i64>() {
                Err(_) => Some("must be an integer".to_string()),
                Ok(n) if n < *lo || n > *hi => Some(format!("must be between {lo} and {hi}")),
                Ok(_) => None,
            },
            Rule::OneOf(options) => {
                if options.iter().any(|o| o == value) {
                    None
                } else {
                    Some(format!("must be one of {}", options.join(", ")))
                }
            }
        }
    }
}

/// A single validation failure: which field, and what was wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub field: String,
    pub message: String,
}

impl Error {
    /// Renders as `"field: message"` — the stable format the tests assert.
    pub fn to_line(&self) -> String {
        format!("{}: {}", self.field, self.message)
    }
}

/// A validation schema: an ordered list of `(field, rules)`. Order is
/// preserved in the error output, so results are deterministic and identical
/// across languages.
#[derive(Default)]
pub struct Schema {
    fields: Vec<(String, Vec<Rule>)>,
}

impl Schema {
    pub fn new() -> Self {
        Schema::default()
    }

    /// Declares the rules for `field`. Chainable.
    pub fn field(mut self, name: &str, rules: Vec<Rule>) -> Self {
        self.fields.push((name.to_string(), rules));
        self
    }

    /// Validates `data`, returning every error found, in field-declaration
    /// then rule order.
    ///
    /// Semantics for absent/empty values: a field carrying [`Rule::Required`]
    /// that is missing or empty yields exactly one "is required" error and its
    /// other rules are skipped (nothing to check). A field *without*
    /// `Required` that is absent/empty is simply skipped — that is what
    /// "optional" means.
    pub fn validate(&self, data: &HashMap<String, String>) -> Vec<Error> {
        let mut errors = Vec::new();
        for (field, rules) in &self.fields {
            let value = data.get(field).map(String::as_str).unwrap_or("");
            let present = !value.is_empty();
            let required = rules.iter().any(|r| matches!(r, Rule::Required));

            if !present {
                if required {
                    errors.push(Error {
                        field: field.clone(),
                        message: "is required".to_string(),
                    });
                }
                continue; // absent value: nothing else to check
            }

            for rule in rules {
                if matches!(rule, Rule::Required) {
                    continue; // already satisfied (value is present)
                }
                if let Some(message) = rule.check(value) {
                    errors.push(Error {
                        field: field.clone(),
                        message,
                    });
                }
            }
        }
        errors
    }
}

/// Convenience: render a list of errors as sorted-by-declaration lines.
pub fn error_lines(errors: &[Error]) -> Vec<String> {
    errors.iter().map(Error::to_line).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user_schema() -> Schema {
        Schema::new()
            .field(
                "name",
                vec![Rule::Required, Rule::MinLength(2), Rule::MaxLength(30)],
            )
            .field("age", vec![Rule::Required, Rule::InRange(18, 120)])
            .field(
                "role",
                vec![Rule::OneOf(vec![
                    "admin".to_string(),
                    "user".to_string(),
                    "guest".to_string(),
                ])],
            )
    }

    fn record(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn valid_record_has_no_errors() {
        let data = record(&[("name", "Ana"), ("age", "30"), ("role", "admin")]);
        assert!(user_schema().validate(&data).is_empty());
    }

    #[test]
    fn missing_required_field_reports_is_required() {
        let data = record(&[("age", "30"), ("role", "user")]);
        let errors = error_lines(&user_schema().validate(&data));
        assert_eq!(errors, vec!["name: is required"]);
    }

    #[test]
    fn too_short_reports_min_length() {
        let data = record(&[("name", "A"), ("age", "30"), ("role", "user")]);
        let errors = error_lines(&user_schema().validate(&data));
        assert_eq!(errors, vec!["name: must be at least 2 characters"]);
    }

    #[test]
    fn accumulates_all_errors_not_just_the_first() {
        // name too short, age not an int, role not allowed — all three come
        // back together, in declaration order.
        let data = record(&[("name", "A"), ("age", "old"), ("role", "wizard")]);
        let errors = error_lines(&user_schema().validate(&data));
        assert_eq!(
            errors,
            vec![
                "name: must be at least 2 characters",
                "age: must be an integer",
                "role: must be one of admin, user, guest",
            ]
        );
    }

    #[test]
    fn range_checks_bounds() {
        let data = record(&[("name", "Ana"), ("age", "150"), ("role", "user")]);
        let errors = error_lines(&user_schema().validate(&data));
        assert_eq!(errors, vec!["age: must be between 18 and 120"]);
    }

    #[test]
    fn optional_absent_field_is_skipped() {
        // A schema with an optional bio: absent is fine, no error.
        let schema = Schema::new().field("bio", vec![Rule::MaxLength(100)]);
        assert!(schema.validate(&record(&[])).is_empty());
    }

    #[test]
    fn one_of_accepts_allowed_and_rejects_others() {
        let schema = Schema::new().field(
            "role",
            vec![Rule::OneOf(vec!["admin".to_string(), "user".to_string()])],
        );
        assert!(schema.validate(&record(&[("role", "admin")])).is_empty());
        let errors = error_lines(&schema.validate(&record(&[("role", "root")])));
        assert_eq!(errors, vec!["role: must be one of admin, user"]);
    }

    #[test]
    fn multibyte_length_counts_characters_not_bytes() {
        // "José" is 4 characters but 5 bytes — MinLength must count chars.
        let schema = Schema::new().field("name", vec![Rule::MinLength(4)]);
        assert!(schema.validate(&record(&[("name", "José")])).is_empty());
    }
}
