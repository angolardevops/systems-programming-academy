//! Configuration & Secrets — Rust companion for the Part 2 lesson. The same
//! design is implemented in Rust, Go, and Python for comparison.
//!
//! Principles demonstrated:
//! - Parse env vars into a **typed** config once, at startup (fail fast).
//! - Take the environment as an **injected map**, not global state, so tests are
//!   deterministic and parallel-safe.
//! - **Redact secrets** in the `Debug` representation so logs can't leak them.
//!
//! ```text
//! cargo test
//! ```

use std::collections::HashMap;
use std::fmt;

/// Typed application configuration.
#[derive(PartialEq, Eq)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub debug: bool,
    pub api_key: String, // secret: never printed in full
}

/// Everything that can go wrong while loading config. Typed, so callers (and
/// tests) can match on the exact failure.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// A required variable is absent.
    Missing(&'static str),
    /// A variable is present but not parseable as its type.
    Invalid { key: &'static str, value: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(key) => write!(f, "missing required env var {key}"),
            ConfigError::Invalid { key, value } => {
                write!(f, "invalid value {value:?} for env var {key}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

// Redact the secret: Debug output shows everything EXCEPT the api key.
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("debug", &self.debug)
            .field("api_key", &"***REDACTED***")
            .finish()
    }
}

impl Config {
    /// Loads config from an injected map (in production, collect
    /// `std::env::vars()` into a map first). Optional vars get defaults;
    /// required ones fail fast with a typed error.
    pub fn from_map(env: &HashMap<String, String>) -> Result<Config, ConfigError> {
        let host = env
            .get("APP_HOST")
            .cloned()
            .unwrap_or_else(|| "localhost".to_string());

        let port = match env.get("APP_PORT") {
            None => 8080, // default
            Some(raw) => raw.parse().map_err(|_| ConfigError::Invalid {
                key: "APP_PORT",
                value: raw.clone(),
            })?,
        };

        let debug = matches!(
            env.get("APP_DEBUG").map(String::as_str),
            Some("1") | Some("true")
        );

        let api_key = env
            .get("APP_API_KEY")
            .cloned()
            .ok_or(ConfigError::Missing("APP_API_KEY"))?;

        Ok(Config {
            host,
            port,
            debug,
            api_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn loads_with_defaults() {
        let cfg = Config::from_map(&env(&[("APP_API_KEY", "s3cret")])).unwrap();
        assert_eq!(cfg.host, "localhost");
        assert_eq!(cfg.port, 8080);
        assert!(!cfg.debug);
        assert_eq!(cfg.api_key, "s3cret");
    }

    #[test]
    fn loads_explicit_values() {
        let cfg = Config::from_map(&env(&[
            ("APP_HOST", "0.0.0.0"),
            ("APP_PORT", "9000"),
            ("APP_DEBUG", "true"),
            ("APP_API_KEY", "k"),
        ]))
        .unwrap();
        assert_eq!(
            (cfg.host.as_str(), cfg.port, cfg.debug),
            ("0.0.0.0", 9000, true)
        );
    }

    #[test]
    fn missing_secret_fails_fast() {
        assert_eq!(
            Config::from_map(&env(&[])),
            Err(ConfigError::Missing("APP_API_KEY"))
        );
    }

    #[test]
    fn invalid_port_is_a_typed_error() {
        let err =
            Config::from_map(&env(&[("APP_PORT", "nope"), ("APP_API_KEY", "k")])).unwrap_err();
        assert_eq!(
            err,
            ConfigError::Invalid {
                key: "APP_PORT",
                value: "nope".into()
            }
        );
        assert_eq!(
            err.to_string(),
            "invalid value \"nope\" for env var APP_PORT"
        );
    }

    #[test]
    fn debug_output_redacts_the_secret() {
        let cfg = Config::from_map(&env(&[("APP_API_KEY", "hunter2")])).unwrap();
        let printed = format!("{cfg:?}");
        assert!(printed.contains("***REDACTED***"));
        assert!(!printed.contains("hunter2")); // the secret never appears
    }
}
