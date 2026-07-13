//! Companion library for the lesson **Error Handling with `Result`, `Option`
//! and the `?` operator**.
//!
//! The running example is a tiny parser: it turns lines of `"name,age"` into
//! [`User`] records, reporting precise, typed errors when a line is malformed.
//! Every public item is referenced from the lesson and covered by the tests at
//! the bottom of this file. Run them with:
//!
//! ```text
//! cargo test
//! ```

use std::fmt;

/// A parsed user record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub name: String,
    pub age: u8,
}

/// Everything that can go wrong while parsing a single line.
///
/// Making the error an `enum` (rather than a `String`) means callers can match
/// on the *kind* of failure and react differently — the whole point of typed
/// errors.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The line did not contain exactly one comma (so not exactly two fields).
    WrongFieldCount { found: usize },
    /// The name field was empty after trimming.
    EmptyName,
    /// The age field was not a valid `u8`. Carries the offending text so the
    /// message can quote exactly what failed.
    InvalidAge { value: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::WrongFieldCount { found } => {
                write!(f, "expected 2 comma-separated fields, found {found}")
            }
            ParseError::EmptyName => write!(f, "name must not be empty"),
            ParseError::InvalidAge { value } => {
                write!(f, "age '{value}' is not a valid number in 0..=255")
            }
        }
    }
}

// Implementing the standard `Error` trait lets `ParseError` interoperate with
// `Box<dyn Error>`, `?` in `main`, and the wider error ecosystem.
impl std::error::Error for ParseError {}

/// Parses a single `"name,age"` line into a [`User`].
///
/// Demonstrates the `?` operator: each fallible step either yields a value or
/// returns early with a [`ParseError`].
pub fn parse_user(line: &str) -> Result<User, ParseError> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() != 2 {
        return Err(ParseError::WrongFieldCount { found: parts.len() });
    }

    let name = parts[0].trim();
    if name.is_empty() {
        return Err(ParseError::EmptyName);
    }

    let age_text = parts[1].trim();
    let age: u8 = age_text.parse().map_err(|_| ParseError::InvalidAge {
        value: age_text.to_string(),
    })?;

    Ok(User {
        name: name.to_string(),
        age,
    })
}

/// Parses many lines, stopping at the **first** error (fail-fast).
///
/// `collect()` into a `Result<Vec<_>, _>` is the idiomatic way to turn an
/// iterator of `Result`s into a single `Result`: it short-circuits on the first
/// `Err` and otherwise gives you the full `Vec`.
pub fn parse_users(input: &str) -> Result<Vec<User>, ParseError> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(parse_user)
        .collect()
}

/// Parses many lines, keeping the good rows and collecting the errors instead of
/// stopping (fail-soft). Returns `(users, errors_with_line_numbers)`.
pub fn parse_users_lenient(input: &str) -> (Vec<User>, Vec<(usize, ParseError)>) {
    let mut users = Vec::new();
    let mut errors = Vec::new();
    for (i, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match parse_user(line) {
            Ok(u) => users.push(u),
            Err(e) => errors.push((i + 1, e)), // 1-based line number
        }
    }
    (users, errors)
}

/// Returns the first adult (age >= 18), or `None` if there is none.
///
/// Demonstrates `Option`: absence is a value, not an error, and the type system
/// forces the caller to handle the "nobody" case.
pub fn first_adult(users: &[User]) -> Option<&User> {
    users.iter().find(|u| u.age >= 18)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_line() {
        assert_eq!(
            parse_user("Ada, 36"),
            Ok(User {
                name: "Ada".to_string(),
                age: 36
            })
        );
    }

    #[test]
    fn rejects_wrong_field_count() {
        assert_eq!(
            parse_user("no comma here"),
            Err(ParseError::WrongFieldCount { found: 1 })
        );
        assert_eq!(
            parse_user("a,b,c"),
            Err(ParseError::WrongFieldCount { found: 3 })
        );
    }

    #[test]
    fn rejects_empty_name() {
        assert_eq!(parse_user("  , 20"), Err(ParseError::EmptyName));
    }

    #[test]
    fn rejects_invalid_age() {
        assert_eq!(
            parse_user("Bob, twelve"),
            Err(ParseError::InvalidAge {
                value: "twelve".to_string()
            })
        );
        // 300 does not fit in a u8, so parse() fails too.
        assert_eq!(
            parse_user("Bob, 300"),
            Err(ParseError::InvalidAge {
                value: "300".to_string()
            })
        );
    }

    #[test]
    fn error_messages_are_human_readable() {
        assert_eq!(
            ParseError::WrongFieldCount { found: 3 }.to_string(),
            "expected 2 comma-separated fields, found 3"
        );
        assert_eq!(ParseError::EmptyName.to_string(), "name must not be empty");
        assert_eq!(
            ParseError::InvalidAge { value: "x".into() }.to_string(),
            "age 'x' is not a valid number in 0..=255"
        );
    }

    #[test]
    fn fail_fast_stops_at_first_error() {
        let input = "Ada, 36\nBob, oops\nCarol, 40";
        assert_eq!(
            parse_users(input),
            Err(ParseError::InvalidAge {
                value: "oops".into()
            })
        );
    }

    #[test]
    fn fail_fast_parses_all_when_clean() {
        let input = "Ada, 36\nCarol, 40\n";
        let users = parse_users(input).expect("all lines valid");
        assert_eq!(users.len(), 2);
        assert_eq!(
            users[1],
            User {
                name: "Carol".into(),
                age: 40
            }
        );
    }

    #[test]
    fn lenient_keeps_good_rows_and_reports_bad_ones() {
        let input = "Ada, 36\nBob, oops\n\nCarol, 40";
        let (users, errors) = parse_users_lenient(input);
        assert_eq!(users.len(), 2); // Ada and Carol
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, 2); // Bob is on line 2
        assert_eq!(
            errors[0].1,
            ParseError::InvalidAge {
                value: "oops".into()
            }
        );
    }

    #[test]
    fn first_adult_finds_or_returns_none() {
        let adults = vec![
            User {
                name: "Kid".into(),
                age: 10,
            },
            User {
                name: "Grown".into(),
                age: 21,
            },
        ];
        assert_eq!(first_adult(&adults), Some(&adults[1]));

        let children = vec![User {
            name: "Kid".into(),
            age: 10,
        }];
        assert_eq!(first_adult(&children), None);
    }
}
