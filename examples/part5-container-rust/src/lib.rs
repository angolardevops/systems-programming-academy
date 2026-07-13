//! A dependency-injection container: register services by name with a factory,
//! then resolve them — the framework wires the graph, calling each factory and
//! feeding it whatever it asks the container to resolve.
//!
//! This is the inversion of control from Part 2's Repository & DI lesson, made
//! into a reusable framework. Three things a real container must get right and
//! this one does, all tested:
//!
//! * **Lifetimes.** A *transient* service is rebuilt on every resolve; a
//!   *singleton* is built once and cached. Getting this wrong means either
//!   shared state that should be fresh, or reconstructing an expensive object
//!   on every request.
//! * **Cycle detection.** If A needs B and B needs A, naive resolution
//!   recurses forever (stack overflow). We track the resolution stack and
//!   return a clear error instead.
//! * **Missing dependencies fail loudly**, with the name that was not found.
//!
//! Factories build `String` values here so the assembled graph is directly
//! assertable — no database, no network.

use std::cell::RefCell;
use std::collections::HashMap;

/// A factory builds a service, using `&Container` to resolve its own
/// dependencies. Returns the built value or an error string.
pub type Factory = Box<dyn Fn(&Container) -> Result<String, String>>;

struct Registration {
    factory: Factory,
    singleton: bool,
}

/// The container: register services, then [`resolve`](Container::resolve) them.
#[derive(Default)]
pub struct Container {
    registrations: HashMap<String, Registration>,
    cache: RefCell<HashMap<String, String>>,
    resolving: RefCell<Vec<String>>,
}

impl Container {
    pub fn new() -> Self {
        Container::default()
    }

    /// Registers a **transient** service: its factory runs on every resolve,
    /// producing a fresh value each time.
    pub fn register(
        &mut self,
        name: &str,
        factory: impl Fn(&Container) -> Result<String, String> + 'static,
    ) {
        self.registrations.insert(
            name.to_string(),
            Registration {
                factory: Box::new(factory),
                singleton: false,
            },
        );
    }

    /// Registers a **singleton** service: its factory runs at most once; the
    /// result is cached and returned on every later resolve.
    pub fn register_singleton(
        &mut self,
        name: &str,
        factory: impl Fn(&Container) -> Result<String, String> + 'static,
    ) {
        self.registrations.insert(
            name.to_string(),
            Registration {
                factory: Box::new(factory),
                singleton: true,
            },
        );
    }

    /// Resolves `name`: returns the cached singleton if present, otherwise
    /// runs its factory (which may resolve further dependencies), caching the
    /// result if it is a singleton.
    ///
    /// Errors if the name is not registered, or if resolving it would form a
    /// cycle (A -> B -> A).
    pub fn resolve(&self, name: &str) -> Result<String, String> {
        // Serve a cached singleton without touching the resolution stack.
        if let Some(value) = self.cache.borrow().get(name) {
            return Ok(value.clone());
        }

        // Cycle detection: is this name already on the resolution stack?
        {
            let stack = self.resolving.borrow();
            if stack.iter().any(|n| n == name) {
                let mut chain = stack.clone();
                chain.push(name.to_string());
                return Err(format!("dependency cycle: {}", chain.join(" -> ")));
            }
        }

        let registration = self
            .registrations
            .get(name)
            .ok_or_else(|| format!("service not registered: {name}"))?;

        self.resolving.borrow_mut().push(name.to_string());
        let result = (registration.factory)(self);
        self.resolving.borrow_mut().pop();

        let value = result?;
        if registration.singleton {
            self.cache
                .borrow_mut()
                .insert(name.to_string(), value.clone());
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn resolves_a_leaf_service() {
        let mut c = Container::new();
        c.register("config", |_| Ok("Config(db=memory)".to_string()));
        assert_eq!(c.resolve("config").unwrap(), "Config(db=memory)");
    }

    #[test]
    fn resolves_a_dependency_chain() {
        let mut c = Container::new();
        c.register("config", |_| Ok("Config".to_string()));
        c.register("repo", |c| {
            Ok(format!("Repo(uses {})", c.resolve("config")?))
        });
        c.register("service", |c| {
            Ok(format!("Service(uses {})", c.resolve("repo")?))
        });
        assert_eq!(
            c.resolve("service").unwrap(),
            "Service(uses Repo(uses Config))"
        );
    }

    #[test]
    fn unknown_service_errors_with_its_name() {
        let c = Container::new();
        let err = c.resolve("nope").unwrap_err();
        assert!(err.contains("nope"), "error should name the service: {err}");
    }

    #[test]
    fn transient_rebuilds_every_resolve() {
        let counter = Rc::new(RefCell::new(0));
        let mut c = Container::new();
        let counter_clone = Rc::clone(&counter);
        c.register("id", move |_| {
            *counter_clone.borrow_mut() += 1;
            Ok(format!("instance-{}", counter_clone.borrow()))
        });
        assert_eq!(c.resolve("id").unwrap(), "instance-1");
        assert_eq!(c.resolve("id").unwrap(), "instance-2");
        assert_eq!(c.resolve("id").unwrap(), "instance-3");
    }

    #[test]
    fn singleton_builds_once_and_caches() {
        let counter = Rc::new(RefCell::new(0));
        let mut c = Container::new();
        let counter_clone = Rc::clone(&counter);
        c.register_singleton("id", move |_| {
            *counter_clone.borrow_mut() += 1;
            Ok(format!("instance-{}", counter_clone.borrow()))
        });
        assert_eq!(c.resolve("id").unwrap(), "instance-1");
        assert_eq!(c.resolve("id").unwrap(), "instance-1"); // cached, not rebuilt
        assert_eq!(*counter.borrow(), 1, "factory must run exactly once");
    }

    #[test]
    fn direct_cycle_is_detected() {
        let mut c = Container::new();
        c.register("a", |c| Ok(format!("A({})", c.resolve("b")?)));
        c.register("b", |c| Ok(format!("B({})", c.resolve("a")?)));
        let err = c.resolve("a").unwrap_err();
        assert!(err.contains("cycle"), "expected cycle error, got: {err}");
        assert!(
            err.contains("a -> b -> a"),
            "error should show the chain: {err}"
        );
    }

    #[test]
    fn self_cycle_is_detected() {
        let mut c = Container::new();
        c.register("loop", |c| c.resolve("loop"));
        assert!(c.resolve("loop").unwrap_err().contains("cycle"));
    }

    #[test]
    fn singleton_dependency_is_shared_across_consumers() {
        let counter = Rc::new(RefCell::new(0));
        let mut c = Container::new();
        let cc = Rc::clone(&counter);
        c.register_singleton("db", move |_| {
            *cc.borrow_mut() += 1;
            Ok(format!("DB#{}", cc.borrow()))
        });
        c.register("users", |c| Ok(format!("Users({})", c.resolve("db")?)));
        c.register("orders", |c| Ok(format!("Orders({})", c.resolve("db")?)));
        // Two different consumers, but they share the one singleton DB.
        assert_eq!(c.resolve("users").unwrap(), "Users(DB#1)");
        assert_eq!(c.resolve("orders").unwrap(), "Orders(DB#1)");
        assert_eq!(*counter.borrow(), 1);
    }
}
