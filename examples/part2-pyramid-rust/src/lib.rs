//! Testing Pyramid — Rust companion for the Part 2 lesson. The same signup
//! feature is implemented in Rust, Go, and Python, with tests at all three
//! levels of the pyramid:
//!
//! - **Unit**: `validate_email`, a pure function — many tiny, instant tests.
//! - **Integration**: `SignupService` wired to real in-memory adapters — fewer
//!   tests, proving the parts collaborate (user stored AND welcome sent).
//! - **End-to-end**: `App`, the composition root — one test driving the public
//!   entry point and asserting the user-visible outcome.
//!
//! ```text
//! cargo test
//! ```

use std::collections::HashSet;

// ---------------------------------------------------------------- validation

/// Why an email was rejected.
#[derive(Debug, PartialEq, Eq)]
pub enum EmailError {
    Empty,
    MissingAt,
    MultipleAt,
    BadDomain,
    HasWhitespace,
}

/// Pure validation — the base of the pyramid: no I/O, no state, instant tests.
pub fn validate_email(email: &str) -> Result<(), EmailError> {
    if email.is_empty() {
        return Err(EmailError::Empty);
    }
    if email.chars().any(char::is_whitespace) {
        return Err(EmailError::HasWhitespace);
    }
    let parts: Vec<&str> = email.split('@').collect();
    match parts.as_slice() {
        [_local] => Err(EmailError::MissingAt),
        [local, domain] => {
            if local.is_empty() || domain.is_empty() || !domain.contains('.') {
                return Err(EmailError::BadDomain);
            }
            Ok(())
        }
        _ => Err(EmailError::MultipleAt),
    }
}

// ------------------------------------------------------------------- service

/// Signup failures the service can report.
#[derive(Debug, PartialEq, Eq)]
pub enum SignupError {
    Invalid(EmailError),
    AlreadyExists,
}

/// Port: user storage.
pub trait UserRepo {
    fn exists(&self, email: &str) -> bool;
    fn save(&mut self, email: &str);
}

/// Port: welcome notifications.
pub trait Notifier {
    fn send_welcome(&mut self, email: &str);
}

/// Adapter: in-memory storage.
#[derive(Default)]
pub struct InMemoryRepo {
    emails: HashSet<String>,
}

impl UserRepo for InMemoryRepo {
    fn exists(&self, email: &str) -> bool {
        self.emails.contains(email)
    }
    fn save(&mut self, email: &str) {
        self.emails.insert(email.to_string());
    }
}

/// Adapter: records sent notifications (a real one would talk SMTP).
#[derive(Default)]
pub struct RecordingNotifier {
    pub sent: Vec<String>,
}

impl Notifier for RecordingNotifier {
    fn send_welcome(&mut self, email: &str) {
        self.sent.push(email.to_string());
    }
}

/// The middle of the pyramid: business logic coordinating the two ports.
pub struct SignupService<R: UserRepo, N: Notifier> {
    repo: R,
    notifier: N,
}

impl<R: UserRepo, N: Notifier> SignupService<R, N> {
    pub fn new(repo: R, notifier: N) -> Self {
        SignupService { repo, notifier }
    }

    pub fn signup(&mut self, email: &str) -> Result<(), SignupError> {
        validate_email(email).map_err(SignupError::Invalid)?;
        if self.repo.exists(email) {
            return Err(SignupError::AlreadyExists);
        }
        self.repo.save(email);
        self.notifier.send_welcome(email);
        Ok(())
    }

    /// Test hook: expose the notifier for collaboration assertions.
    pub fn notifier(&self) -> &N {
        &self.notifier
    }
}

// ----------------------------------------------------------------------- app

/// The top of the pyramid: the composition root a user-facing binary would
/// call. It wires real (in-memory here; Postgres/SMTP in production) parts and
/// returns the user-visible message.
pub struct App {
    service: SignupService<InMemoryRepo, RecordingNotifier>,
}

impl App {
    pub fn new() -> Self {
        App {
            service: SignupService::new(InMemoryRepo::default(), RecordingNotifier::default()),
        }
    }

    pub fn signup(&mut self, email: &str) -> String {
        match self.service.signup(email) {
            Ok(()) => format!("Welcome, {email}! Check your inbox."),
            Err(SignupError::AlreadyExists) => format!("{email} is already registered."),
            Err(SignupError::Invalid(_)) => format!("'{email}' is not a valid email address."),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod unit_tests {
    //! UNIT level: many tiny tests on the pure validator. Instant, no setup.
    use super::*;

    #[test]
    fn accepts_a_normal_address() {
        assert_eq!(validate_email("ada@example.com"), Ok(()));
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(validate_email(""), Err(EmailError::Empty));
    }

    #[test]
    fn rejects_missing_at() {
        assert_eq!(
            validate_email("ada.example.com"),
            Err(EmailError::MissingAt)
        );
    }

    #[test]
    fn rejects_multiple_at() {
        assert_eq!(validate_email("a@b@c.com"), Err(EmailError::MultipleAt));
    }

    #[test]
    fn rejects_bad_domain() {
        assert_eq!(validate_email("ada@nodot"), Err(EmailError::BadDomain));
        assert_eq!(validate_email("@example.com"), Err(EmailError::BadDomain));
    }

    #[test]
    fn rejects_whitespace() {
        assert_eq!(
            validate_email("a da@example.com"),
            Err(EmailError::HasWhitespace)
        );
    }
}

#[cfg(test)]
mod integration_tests {
    //! INTEGRATION level: the service with real in-memory adapters — proving
    //! the parts collaborate (stored AND notified), not just work alone.
    use super::*;

    #[test]
    fn signup_stores_and_notifies() {
        let mut svc = SignupService::new(InMemoryRepo::default(), RecordingNotifier::default());
        svc.signup("ada@example.com").unwrap();
        assert_eq!(svc.notifier().sent, vec!["ada@example.com".to_string()]);
    }

    #[test]
    fn duplicate_signup_is_rejected_and_not_notified_twice() {
        let mut svc = SignupService::new(InMemoryRepo::default(), RecordingNotifier::default());
        svc.signup("ada@example.com").unwrap();
        assert_eq!(
            svc.signup("ada@example.com"),
            Err(SignupError::AlreadyExists)
        );
        assert_eq!(svc.notifier().sent.len(), 1); // no second welcome
    }
}

#[cfg(test)]
mod e2e_tests {
    //! END-TO-END level: one test driving the composition root exactly as a
    //! user-facing caller would, asserting only the visible outcome.
    use super::*;

    #[test]
    fn full_signup_flow_through_the_app() {
        let mut app = App::new();
        assert_eq!(
            app.signup("ada@example.com"),
            "Welcome, ada@example.com! Check your inbox."
        );
        assert_eq!(
            app.signup("ada@example.com"),
            "ada@example.com is already registered."
        );
        assert_eq!(app.signup("nope"), "'nope' is not a valid email address.");
    }
}
