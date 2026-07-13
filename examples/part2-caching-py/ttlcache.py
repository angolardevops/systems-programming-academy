"""Python companion for the Part 2 lesson "Caching". The same design is
implemented in Rust, Go, and Python.

Principles: a TTL cache whose clock is an injected callable (deterministic
expiry tests — no sleeping), hit/miss counters, and the cache-aside pattern
proven by a call-counting fake backend.

Run the tests:

    python3 -m unittest discover -s . -p 'test_*.py'
"""

from __future__ import annotations

import time
from collections.abc import Callable
from typing import Any


class TtlCache:
    """A TTL cache over string keys. The clock is injected — pass
    ``time.monotonic`` in production, a fake in tests.

    >>> t = 0.0
    >>> cache = TtlCache(ttl_seconds=60, clock=lambda: t)
    >>> cache.put("k", "v")
    >>> cache.get("k")
    'v'
    >>> cache.hits, cache.misses
    (1, 0)
    """

    def __init__(
        self,
        ttl_seconds: float,
        clock: Callable[[], float] = time.monotonic,
    ) -> None:
        self._entries: dict[str, tuple[Any, float]] = {}
        self._ttl = ttl_seconds
        self._clock = clock
        self.hits = 0
        self.misses = 0

    def put(self, key: str, value: Any) -> None:
        """Stores a value, stamping its expiry from the injected clock."""
        self._entries[key] = (value, self._clock() + self._ttl)

    def get(self, key: str) -> Any | None:
        """Returns the value if present and fresh, updating the counters."""
        entry = self._entries.get(key)
        if entry is None:
            self.misses += 1
            return None
        value, expires = entry
        if self._clock() >= expires:
            del self._entries[key]  # lazy eviction of the stale entry
            self.misses += 1
            return None
        self.hits += 1
        return value


def get_user(cache: TtlCache, user_id: int, backend: Callable[[int], str]) -> str:
    """Cache-aside: consult the cache first; on miss, load from the backend and
    store. ``backend`` is any callable — in tests, one that counts its calls.
    """
    key = f"user:{user_id}"
    name = cache.get(key)
    if name is not None:
        return name  # served from cache — no backend call
    name = backend(user_id)
    cache.put(key, name)
    return name
