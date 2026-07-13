"""Tests for ttlcache.py (stdlib unittest, plus its doctests)."""

import doctest
import unittest

import ttlcache
from ttlcache import TtlCache, get_user


class FakeClock:
    """A deterministic clock tests advance by hand."""

    def __init__(self) -> None:
        self.t = 0.0

    def __call__(self) -> float:
        return self.t

    def advance(self, seconds: float) -> None:
        self.t += seconds


class TtlCacheTests(unittest.TestCase):
    def setUp(self) -> None:
        self.clock = FakeClock()
        self.cache = TtlCache(ttl_seconds=60, clock=self.clock)

    def test_get_fresh_value_counts_hit(self) -> None:
        self.cache.put("k", "v")
        self.assertEqual(self.cache.get("k"), "v")
        self.assertEqual((self.cache.hits, self.cache.misses), (1, 0))

    def test_entry_expires_after_ttl(self) -> None:
        self.cache.put("k", "v")

        self.clock.advance(59)  # still fresh
        self.assertEqual(self.cache.get("k"), "v")

        self.clock.advance(1)  # TTL reached: stale
        self.assertIsNone(self.cache.get("k"))
        self.assertEqual((self.cache.hits, self.cache.misses), (1, 1))

    def test_missing_key_counts_a_miss(self) -> None:
        self.assertIsNone(self.cache.get("absent"))
        self.assertEqual(self.cache.misses, 1)


class CacheAsideTests(unittest.TestCase):
    def test_backend_called_only_on_miss(self) -> None:
        clock = FakeClock()
        cache = TtlCache(ttl_seconds=60, clock=clock)
        calls = []

        def backend(user_id: int) -> str:
            calls.append(user_id)
            return f"user-{user_id}"

        # First call: miss -> backend; second: hit -> no backend call.
        self.assertEqual(get_user(cache, 42, backend), "user-42")
        self.assertEqual(get_user(cache, 42, backend), "user-42")
        self.assertEqual(len(calls), 1)

        # After expiry the backend is consulted again.
        clock.advance(61)
        self.assertEqual(get_user(cache, 42, backend), "user-42")
        self.assertEqual(len(calls), 2)


def load_tests(loader, tests, ignore):  # noqa: ANN001, ARG001
    tests.addTests(doctest.DocTestSuite(ttlcache))
    return tests


if __name__ == "__main__":
    unittest.main()
