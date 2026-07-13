"""Python companion for the Part 2 lesson "Logging & Observability". Built on
the stdlib ``logging`` module: a JSON formatter for structured lines, level
filtering, an injected stream so tests capture output, and a LoggerAdapter that
binds context fields (request_id) to every line.

Timestamps are omitted from the format so tests are deterministic; production
formats add them. Run the tests:

    python3 -m unittest discover -s . -p 'test_*.py'
"""

from __future__ import annotations

import json
import logging
from typing import IO, Any

# Attributes present on every LogRecord; anything else came from `extra=`.
_STANDARD_ATTRS = frozenset(logging.LogRecord("", 0, "", 0, "", (), None).__dict__) | {
    "message",
    "asctime",
    "taskName",
}


class JsonFormatter(logging.Formatter):
    """Formats each record as one JSON line: level, msg, plus any extra fields."""

    def format(self, record: logging.LogRecord) -> str:
        payload: dict[str, Any] = {
            "level": record.levelname,
            "msg": record.getMessage(),
        }
        for key, value in record.__dict__.items():
            if key not in _STANDARD_ATTRS:
                payload[key] = value  # fields passed via extra=
        return json.dumps(payload)


def new_logger(name: str, sink: IO[str], level: int = logging.INFO) -> logging.Logger:
    """Builds a structured logger writing JSON lines to the injected sink."""
    logger = logging.getLogger(name)
    logger.setLevel(level)
    logger.handlers.clear()  # keep tests isolated from prior configuration
    logger.propagate = False
    handler = logging.StreamHandler(sink)
    handler.setFormatter(JsonFormatter())
    logger.addHandler(handler)
    return logger


def with_request_id(
    logger: logging.Logger, request_id: str
) -> logging.LoggerAdapter[logging.Logger]:
    """Binds request_id as a context field carried by every subsequent line."""

    class _Adapter(logging.LoggerAdapter[logging.Logger]):
        def process(self, msg: str, kwargs: Any) -> tuple[str, Any]:
            extra = kwargs.get("extra") or {}
            kwargs["extra"] = {**self.extra, **extra}
            return msg, kwargs

    return _Adapter(logger, {"request_id": request_id})


def handle_login(
    logger: logging.Logger | logging.LoggerAdapter, user_id: int, ok: bool
) -> None:
    """A tiny 'business' function that logs structured events."""
    if ok:
        logger.info("user logged in", extra={"user_id": user_id})
    else:
        logger.warning("login failed", extra={"user_id": user_id})
