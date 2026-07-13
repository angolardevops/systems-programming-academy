//! Caching — Rust companion for the Part 2 lesson. The same design is
//! implemented in Rust, Go, and Python for comparison.
//!
//! Principles demonstrated:
//! - A **TTL cache**: entries expire after a time-to-live, so stale data ages out.
//! - The clock is **injected** (a trait), so expiry tests are deterministic —
//!   no sleeping in tests.
//! - **Hit/miss counters**: a cache you can't measure can't be tuned.
//! - The **cache-aside** pattern: check the cache, on miss load from the backend
//!   and store — proven by a test counting backend calls.
//!
//! ```text
//! cargo test
//! ```

use std::collections::HashMap;

/// The injected time source. Production uses [`SystemClock`]; tests use a fake
/// they can advance by hand — the same move as injecting a repository.
pub trait Clock {
    /// Seconds from an arbitrary epoch (monotonic is fine).
    fn now(&self) -> u64;
}

/// Real clock for production use.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// A TTL cache over string keys. Values expire `ttl_seconds` after insertion.
pub struct TtlCache<V, C: Clock> {
    entries: HashMap<String, (V, u64)>, // value + expiry timestamp
    ttl_seconds: u64,
    clock: C,
    pub hits: u64,
    pub misses: u64,
}

impl<V: Clone, C: Clock> TtlCache<V, C> {
    pub fn new(ttl_seconds: u64, clock: C) -> Self {
        TtlCache {
            entries: HashMap::new(),
            ttl_seconds,
            clock,
            hits: 0,
            misses: 0,
        }
    }

    /// Stores a value, stamping its expiry from the injected clock.
    pub fn put(&mut self, key: &str, value: V) {
        let expires = self.clock.now() + self.ttl_seconds;
        self.entries.insert(key.to_string(), (value, expires));
    }

    /// Returns the value if present and not expired, updating the counters.
    pub fn get(&mut self, key: &str) -> Option<V> {
        match self.entries.get(key) {
            Some((value, expires)) if self.clock.now() < *expires => {
                self.hits += 1;
                Some(value.clone())
            }
            Some(_) => {
                // Present but stale: evict lazily and count a miss.
                self.entries.remove(key);
                self.misses += 1;
                None
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }
}

/// Cache-aside: consult the cache first; on miss, load from `backend` and store.
/// `backend` is any closure — in tests, one that counts its calls.
pub fn get_user<C: Clock>(
    cache: &mut TtlCache<String, C>,
    id: u32,
    backend: &mut impl FnMut(u32) -> String,
) -> String {
    let key = format!("user:{id}");
    if let Some(name) = cache.get(&key) {
        return name; // served from cache — no backend call
    }
    let name = backend(id);
    cache.put(&key, name.clone());
    name
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// Deterministic fake clock: tests advance it by hand.
    struct FakeClock(Cell<u64>);

    impl Clock for &FakeClock {
        fn now(&self) -> u64 {
            self.0.get()
        }
    }

    #[test]
    fn get_returns_fresh_value_and_counts_hit() {
        let clock = FakeClock(Cell::new(0));
        let mut cache = TtlCache::new(60, &clock);
        cache.put("k", "v".to_string());
        assert_eq!(cache.get("k"), Some("v".to_string()));
        assert_eq!((cache.hits, cache.misses), (1, 0));
    }

    #[test]
    fn entry_expires_after_ttl() {
        let clock = FakeClock(Cell::new(0));
        let mut cache = TtlCache::new(60, &clock);
        cache.put("k", "v".to_string());

        clock.0.set(59); // still fresh
        assert!(cache.get("k").is_some());

        clock.0.set(60); // TTL reached: stale
        assert_eq!(cache.get("k"), None);
        assert_eq!((cache.hits, cache.misses), (1, 1));
    }

    #[test]
    fn missing_key_counts_a_miss() {
        let clock = FakeClock(Cell::new(0));
        let mut cache: TtlCache<String, _> = TtlCache::new(60, &clock);
        assert_eq!(cache.get("absent"), None);
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn cache_aside_calls_backend_only_on_miss() {
        let clock = FakeClock(Cell::new(0));
        let mut cache = TtlCache::new(60, &clock);
        // Cell lets the closure count calls while we assert between uses —
        // a plain `&mut` counter would still be borrowed by the closure.
        let backend_calls = Cell::new(0u32);
        let mut backend = |id: u32| {
            backend_calls.set(backend_calls.get() + 1);
            format!("user-{id}")
        };

        // First call: miss -> backend; second: hit -> no backend call.
        assert_eq!(get_user(&mut cache, 42, &mut backend), "user-42");
        assert_eq!(get_user(&mut cache, 42, &mut backend), "user-42");
        assert_eq!(backend_calls.get(), 1);

        // After expiry the backend is consulted again.
        clock.0.set(61);
        assert_eq!(get_user(&mut cache, 42, &mut backend), "user-42");
        assert_eq!(backend_calls.get(), 2);
    }
}
