package ttlcache

import (
	"testing"
	"time"
)

// fakeClock is a deterministic clock tests advance by hand.
type fakeClock struct{ t time.Time }

func (f *fakeClock) now() time.Time          { return f.t }
func (f *fakeClock) advance(d time.Duration) { f.t = f.t.Add(d) }
func newFakeClock() *fakeClock               { return &fakeClock{t: time.Unix(0, 0)} }

func TestGetFreshValueCountsHit(t *testing.T) {
	clock := newFakeClock()
	cache := New[string](time.Minute, clock.now)
	cache.Put("k", "v")

	v, ok := cache.Get("k")
	if !ok || v != "v" {
		t.Fatalf("Get = %q, %v; want v, true", v, ok)
	}
	if cache.Hits != 1 || cache.Misses != 0 {
		t.Errorf("counters = %d/%d, want 1/0", cache.Hits, cache.Misses)
	}
}

func TestEntryExpiresAfterTTL(t *testing.T) {
	clock := newFakeClock()
	cache := New[string](time.Minute, clock.now)
	cache.Put("k", "v")

	clock.advance(59 * time.Second) // still fresh
	if _, ok := cache.Get("k"); !ok {
		t.Fatal("expected fresh entry at 59s")
	}

	clock.advance(1 * time.Second) // TTL reached: stale
	if _, ok := cache.Get("k"); ok {
		t.Fatal("expected expiry at 60s")
	}
	if cache.Hits != 1 || cache.Misses != 1 {
		t.Errorf("counters = %d/%d, want 1/1", cache.Hits, cache.Misses)
	}
}

func TestCacheAsideCallsBackendOnlyOnMiss(t *testing.T) {
	clock := newFakeClock()
	cache := New[string](time.Minute, clock.now)
	backendCalls := 0
	backend := func(id int) string {
		backendCalls++
		return "user-42"
	}

	// First call: miss -> backend; second: hit -> no backend call.
	GetUser(cache, 42, backend)
	GetUser(cache, 42, backend)
	if backendCalls != 1 {
		t.Errorf("backendCalls = %d, want 1", backendCalls)
	}

	// After expiry the backend is consulted again.
	clock.advance(61 * time.Second)
	GetUser(cache, 42, backend)
	if backendCalls != 2 {
		t.Errorf("backendCalls = %d, want 2", backendCalls)
	}
}
