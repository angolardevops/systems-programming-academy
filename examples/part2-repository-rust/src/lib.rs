//! Repository Pattern & Dependency Injection — Rust companion for the Part 2
//! lesson. The same tiny domain is implemented in Rust, Go, and Python so the
//! three can be compared directly.
//!
//! Layers:
//! - **Domain**: [`User`] — plain data.
//! - **Port**: the [`UserRepository`] trait — an abstraction the service depends on.
//! - **Adapter**: [`InMemoryUserRepository`] — one concrete implementation.
//! - **Service**: [`UserService`] — business logic, generic over the repository,
//!   so a test can inject a fake without touching a database.
//!
//! ```text
//! cargo test
//! ```

use std::collections::HashMap;

/// A user record (the domain entity).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: u32,
    pub name: String,
}

/// Errors the repository can report.
#[derive(Debug, PartialEq, Eq)]
pub enum RepoError {
    /// A user with this id already exists.
    DuplicateId(u32),
}

/// The **port**: what the service needs from storage, expressed as behaviour, not
/// a concrete database. Any type implementing this can back the service.
pub trait UserRepository {
    fn add(&mut self, user: User) -> Result<(), RepoError>;
    fn get(&self, id: u32) -> Option<User>;
    fn all(&self) -> Vec<User>;
}

/// An **adapter**: an in-memory implementation, ideal for tests and demos.
#[derive(Default)]
pub struct InMemoryUserRepository {
    users: HashMap<u32, User>,
}

impl UserRepository for InMemoryUserRepository {
    fn add(&mut self, user: User) -> Result<(), RepoError> {
        if self.users.contains_key(&user.id) {
            return Err(RepoError::DuplicateId(user.id));
        }
        self.users.insert(user.id, user);
        Ok(())
    }

    fn get(&self, id: u32) -> Option<User> {
        self.users.get(&id).cloned()
    }

    fn all(&self) -> Vec<User> {
        self.users.values().cloned().collect()
    }
}

/// The **service** holds the business logic and depends on the *trait*, not a
/// concrete repository. `R` is injected at construction — compile-time
/// dependency injection via generics (zero runtime cost).
pub struct UserService<R: UserRepository> {
    repo: R,
}

impl<R: UserRepository> UserService<R> {
    /// Injects the repository dependency.
    pub fn new(repo: R) -> Self {
        UserService { repo }
    }

    /// Registers a user, rejecting a duplicate id.
    pub fn register(&mut self, id: u32, name: &str) -> Result<(), RepoError> {
        self.repo.add(User {
            id,
            name: name.to_string(),
        })
    }

    /// Returns all user names, sorted for a deterministic result.
    pub fn list_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.repo.all().into_iter().map(|u| u.name).collect();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Because the service is generic over UserRepository, the test injects the
    // in-memory adapter — no database, fully deterministic.
    fn service() -> UserService<InMemoryUserRepository> {
        UserService::new(InMemoryUserRepository::default())
    }

    #[test]
    fn registers_and_lists_sorted() {
        let mut svc = service();
        svc.register(2, "Grace").unwrap();
        svc.register(1, "Ada").unwrap();
        assert_eq!(
            svc.list_names(),
            vec!["Ada".to_string(), "Grace".to_string()]
        );
    }

    #[test]
    fn rejects_duplicate_id() {
        let mut svc = service();
        svc.register(1, "Ada").unwrap();
        assert_eq!(svc.register(1, "Someone"), Err(RepoError::DuplicateId(1)));
    }

    #[test]
    fn repository_get_and_all() {
        let mut repo = InMemoryUserRepository::default();
        repo.add(User {
            id: 1,
            name: "Ada".into(),
        })
        .unwrap();
        assert_eq!(
            repo.get(1),
            Some(User {
                id: 1,
                name: "Ada".into()
            })
        );
        assert_eq!(repo.get(2), None);
        assert_eq!(repo.all().len(), 1);
    }
}
