"""Python companion for the Part 2 lesson "Configuration & Secrets". The same
design is implemented in Rust, Go, and Python.

Principles: parse env vars into a typed, frozen dataclass once at startup (fail
fast); take the environment as an injected mapping so tests never mutate
``os.environ``; redact secrets in ``repr`` so logs can't leak them.

Run the tests:

    python3 -m unittest discover -s . -p 'test_*.py'
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field


class MissingConfigError(Exception):
    """A required environment variable is absent."""


class InvalidConfigError(Exception):
    """An environment variable is present but not parseable."""


@dataclass(frozen=True)
class Config:
    """Typed application configuration. ``api_key`` is excluded from the
    generated ``repr`` (``repr=False``) so printing a Config never leaks it.

    >>> cfg = Config.from_env({"APP_API_KEY": "s3cret"})
    >>> cfg.port
    8080
    >>> "s3cret" in repr(cfg)
    False
    """

    host: str
    port: int
    debug: bool
    api_key: str = field(repr=False)  # secret: excluded from repr

    @classmethod
    def from_env(cls, env: Mapping[str, str]) -> Config:
        """Build a Config from an injected mapping (pass ``os.environ`` in
        production, a plain dict in tests). Optional vars get defaults; required
        ones fail fast with a typed exception.
        """
        host = env.get("APP_HOST", "localhost")

        raw_port = env.get("APP_PORT", "8080")
        try:
            port = int(raw_port)
        except ValueError:
            raise InvalidConfigError(
                f"invalid value {raw_port!r} for env var APP_PORT"
            ) from None

        debug = env.get("APP_DEBUG", "") in ("1", "true")

        api_key = env.get("APP_API_KEY")
        if api_key is None:
            raise MissingConfigError("missing required env var APP_API_KEY")

        return cls(host=host, port=port, debug=debug, api_key=api_key)
