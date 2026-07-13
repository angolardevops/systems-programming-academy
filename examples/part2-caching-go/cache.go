// Package ttlcache is the Go companion for the Part 2 lesson "Caching". The
// same design is implemented in Rust, Go, and Python.
//
// Principles: a TTL cache whose clock is an injected func (deterministic expiry
// tests — no sleeping), hit/miss counters, and the cache-aside pattern proven by
// a call-counting fake backend.
//
//	go test ./...
package ttlcache

import (
	"fmt"
	"time"
)

type entry[V any] struct {
	value   V
	expires time.Time
}

// Cache is a TTL cache over string keys.
type Cache[V any] struct {
	entries map[string]entry[V]
	ttl     time.Duration
	now     func() time.Time // injected clock: time.Now in prod, a fake in tests
	Hits    int
	Misses  int
}

// New builds a cache with the given TTL and clock. Pass time.Now in production.
func New[V any](ttl time.Duration, now func() time.Time) *Cache[V] {
	return &Cache[V]{
		entries: make(map[string]entry[V]),
		ttl:     ttl,
		now:     now,
	}
}

// Put stores a value, stamping its expiry from the injected clock.
func (c *Cache[V]) Put(key string, value V) {
	c.entries[key] = entry[V]{value: value, expires: c.now().Add(c.ttl)}
}

// Get returns the value if present and fresh, updating the counters.
func (c *Cache[V]) Get(key string) (V, bool) {
	var zero V
	e, ok := c.entries[key]
	if !ok {
		c.Misses++
		return zero, false
	}
	if !c.now().Before(e.expires) {
		delete(c.entries, key) // lazy eviction of stale entry
		c.Misses++
		return zero, false
	}
	c.Hits++
	return e.value, true
}

// GetUser is cache-aside: consult the cache first; on miss, load from backend
// and store. backend is any func — in tests, one that counts its calls.
func GetUser(cache *Cache[string], id int, backend func(int) string) string {
	key := fmt.Sprintf("user:%d", id)
	if name, ok := cache.Get(key); ok {
		return name // served from cache — no backend call
	}
	name := backend(id)
	cache.Put(key, name)
	return name
}
