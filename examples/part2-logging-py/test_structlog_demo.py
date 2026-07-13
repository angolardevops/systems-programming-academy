"""Tests for structlog_demo.py (stdlib unittest)."""

import io
import json
import logging
import unittest

from structlog_demo import handle_login, new_logger, with_request_id


def lines_of(sink: io.StringIO) -> list[dict]:
    return [json.loads(line) for line in sink.getvalue().strip().splitlines()]


class StructuredLoggingTests(unittest.TestCase):
    def test_emits_structured_json(self) -> None:
        sink = io.StringIO()
        logger = new_logger("t1", sink)

        handle_login(logger, 42, ok=True)

        (line,) = lines_of(sink)
        self.assertEqual(line["level"], "INFO")
        self.assertEqual(line["msg"], "user logged in")
        self.assertEqual(line["user_id"], 42)

    def test_level_filtering(self) -> None:
        sink = io.StringIO()
        logger = new_logger("t2", sink, level=logging.WARNING)

        logger.info("noise")
        logger.warning("kept")

        lines = lines_of(sink)
        self.assertEqual(len(lines), 1)
        self.assertEqual(lines[0]["msg"], "kept")

    def test_context_field_on_every_line(self) -> None:
        sink = io.StringIO()
        logger = with_request_id(new_logger("t3", sink), "abc-123")

        logger.info("start")
        logger.warning("slow query", extra={"ms": 250})

        lines = lines_of(sink)
        self.assertEqual(len(lines), 2)
        for line in lines:
            self.assertEqual(line["request_id"], "abc-123")
        self.assertEqual(lines[1]["ms"], 250)

    def test_failed_login_logged_as_warning(self) -> None:
        sink = io.StringIO()
        logger = new_logger("t4", sink)

        handle_login(logger, 7, ok=False)

        (line,) = lines_of(sink)
        self.assertEqual((line["level"], line["user_id"]), ("WARNING", 7))


if __name__ == "__main__":
    unittest.main()
