//! Clean/Hexagonal Architecture — Rust companion for the Part 2 lesson. The
//! same task tracker is implemented in Rust, Go, and Python.
//!
//! The layers, as modules (a real project would use directories or crates):
//! - [`domain`] — entities and business rules. Depends on NOTHING else here.
//! - [`app`] — use cases and ports. Depends only on `domain`.
//! - [`adapters`] — concrete implementations of the ports. Depends on `app`.
//! - [`composition`] — the root that wires adapters into use cases.
//!
//! The **Dependency Rule**: source dependencies point inward. `domain` has no
//! `use crate::app` or `use crate::adapters` — the module tree enforces it.
//!
//! ```text
//! cargo test
//! ```

/// Innermost layer: entities and rules. No imports from other layers.
pub mod domain {
    /// Why a domain operation was rejected.
    #[derive(Debug, PartialEq, Eq)]
    pub enum DomainError {
        EmptyTitle,
        AlreadyDone,
    }

    /// The task entity. Its invariants live here, next to its data.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Task {
        pub id: u32,
        pub title: String,
        pub done: bool,
    }

    impl Task {
        /// Business rule: a task must have a non-empty title.
        pub fn new(id: u32, title: &str) -> Result<Task, DomainError> {
            if title.trim().is_empty() {
                return Err(DomainError::EmptyTitle);
            }
            Ok(Task {
                id,
                title: title.trim().to_string(),
                done: false,
            })
        }

        /// Business rule: completing twice is an error, not a no-op.
        pub fn complete(&mut self) -> Result<(), DomainError> {
            if self.done {
                return Err(DomainError::AlreadyDone);
            }
            self.done = true;
            Ok(())
        }
    }
}

/// Middle layer: use cases + the ports they need. Depends only on `domain`.
pub mod app {
    use crate::domain::{DomainError, Task};

    /// Use-case failures: domain rules plus application-level conditions.
    #[derive(Debug, PartialEq, Eq)]
    pub enum AppError {
        Domain(DomainError),
        NotFound(u32),
    }

    /// Port: what the use cases need from storage — defined HERE, in the layer
    /// that uses it, not next to the database.
    pub trait TaskRepo {
        fn next_id(&self) -> u32;
        fn save(&mut self, task: Task);
        fn get(&self, id: u32) -> Option<Task>;
    }

    /// The use cases, generic over the port.
    pub struct TaskService<R: TaskRepo> {
        repo: R,
    }

    impl<R: TaskRepo> TaskService<R> {
        pub fn new(repo: R) -> Self {
            TaskService { repo }
        }

        /// Use case: add a task; returns its id.
        pub fn add(&mut self, title: &str) -> Result<u32, AppError> {
            let id = self.repo.next_id();
            let task = Task::new(id, title).map_err(AppError::Domain)?;
            self.repo.save(task);
            Ok(id)
        }

        /// Use case: complete a task by id.
        pub fn complete(&mut self, id: u32) -> Result<(), AppError> {
            let mut task = self.repo.get(id).ok_or(AppError::NotFound(id))?;
            task.complete().map_err(AppError::Domain)?;
            self.repo.save(task);
            Ok(())
        }
    }
}

/// Outer layer: concrete adapters for the ports. Depends on `app` (inward).
pub mod adapters {
    use crate::app::TaskRepo;
    use crate::domain::Task;
    use std::collections::HashMap;

    /// In-memory storage adapter (production would add a Postgres adapter here).
    #[derive(Default)]
    pub struct InMemoryRepo {
        tasks: HashMap<u32, Task>,
        next: u32,
    }

    impl TaskRepo for InMemoryRepo {
        fn next_id(&self) -> u32 {
            self.next + 1
        }
        fn save(&mut self, task: Task) {
            self.next = self.next.max(task.id);
            self.tasks.insert(task.id, task);
        }
        fn get(&self, id: u32) -> Option<Task> {
            self.tasks.get(&id).cloned()
        }
    }
}

/// The composition root: the only place that knows every layer, wiring
/// adapters into use cases and mapping results to user-visible strings.
pub mod composition {
    use crate::adapters::InMemoryRepo;
    use crate::app::{AppError, TaskService};
    use crate::domain::DomainError;

    pub struct App {
        service: TaskService<InMemoryRepo>,
    }

    impl App {
        pub fn new() -> Self {
            App {
                service: TaskService::new(InMemoryRepo::default()),
            }
        }

        pub fn add(&mut self, title: &str) -> String {
            match self.service.add(title) {
                Ok(id) => format!("Added task #{id}."),
                Err(AppError::Domain(DomainError::EmptyTitle)) => {
                    "A task needs a title.".to_string()
                }
                Err(e) => format!("Unexpected error: {e:?}"),
            }
        }

        pub fn complete(&mut self, id: u32) -> String {
            match self.service.complete(id) {
                Ok(()) => format!("Task #{id} done."),
                Err(AppError::NotFound(_)) => format!("No task #{id}."),
                Err(AppError::Domain(DomainError::AlreadyDone)) => {
                    format!("Task #{id} was already done.")
                }
                Err(e) => format!("Unexpected error: {e:?}"),
            }
        }
    }

    impl Default for App {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod domain_tests {
    //! Domain rules tested with zero infrastructure — the payoff of a pure core.
    use crate::domain::*;

    #[test]
    fn task_requires_a_title() {
        assert_eq!(Task::new(1, "   "), Err(DomainError::EmptyTitle));
    }

    #[test]
    fn completing_twice_is_an_error() {
        let mut t = Task::new(1, "write lesson").unwrap();
        t.complete().unwrap();
        assert_eq!(t.complete(), Err(DomainError::AlreadyDone));
    }
}

#[cfg(test)]
mod usecase_tests {
    //! Use cases with the in-memory adapter injected.
    use crate::adapters::InMemoryRepo;
    use crate::app::*;

    #[test]
    fn add_then_complete_roundtrip() {
        let mut svc = TaskService::new(InMemoryRepo::default());
        let id = svc.add("ship part 2").unwrap();
        assert_eq!(svc.complete(id), Ok(()));
        assert_eq!(
            svc.complete(id),
            Err(AppError::Domain(crate::domain::DomainError::AlreadyDone))
        );
    }

    #[test]
    fn completing_unknown_id_is_not_found() {
        let mut svc = TaskService::new(InMemoryRepo::default());
        assert_eq!(svc.complete(99), Err(AppError::NotFound(99)));
    }
}

#[cfg(test)]
mod e2e_tests {
    //! The composition root, through user-visible messages only.
    use crate::composition::App;

    #[test]
    fn full_flow_through_the_app() {
        let mut app = App::new();
        assert_eq!(app.add("write lesson"), "Added task #1.");
        assert_eq!(app.complete(1), "Task #1 done.");
        assert_eq!(app.complete(1), "Task #1 was already done.");
        assert_eq!(app.complete(9), "No task #9.");
        assert_eq!(app.add("  "), "A task needs a title.");
    }
}
