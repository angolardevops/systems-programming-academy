"""A tiny test framework — the tool this academy has been using, built from the
inside.

Three pieces every test framework (pytest, unittest, JUnit) shares:

* **Assertions** that turn a condition into a descriptive failure ("expected 4,
  got 5"), not a bare ``False``.
* **A registry + runner** that runs each test in **isolation**, so one test's
  failure (or exception) does not stop the others.
* **A canonical report** — the "test x ... ok / FAILED" summary — deterministic
  and byte-identical across languages.

We test the framework the only honest way: feed it known-passing and
known-failing tests and assert its report. No I/O — the report is a string.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass


class AssertionFail(Exception):
    """Raised by an assertion helper to signal a test failure with a message."""


def assert_true(condition: bool, message: str) -> None:
    """Assert a boolean condition, failing with ``message``."""
    if not condition:
        raise AssertionFail(message)


def assert_eq(actual: object, expected: object) -> None:
    """Assert two values are equal, failing with a message showing both."""
    if actual != expected:
        raise AssertionFail(f"expected {expected!r}, got {actual!r}")


@dataclass
class _Outcome:
    name: str
    failed: bool
    message: str


class TestKit:
    """A registry of named tests. Register with :meth:`test`, then
    :meth:`run`."""

    def __init__(self) -> None:
        self._tests: list[tuple[str, Callable[[], None]]] = []

    def test(self, name: str, body: Callable[[], None]) -> TestKit:
        """Register a test under ``name``. Chainable."""
        self._tests.append((name, body))
        return self

    def run(self) -> Report:
        """Run every test in registration order, isolating each: an
        :class:`AssertionFail` is a failure, and *any* other exception is caught
        and turned into a failure too, so a crashing test cannot take down the
        run."""
        outcomes: list[_Outcome] = []
        for name, body in self._tests:
            try:
                body()
                outcomes.append(_Outcome(name, False, ""))
            except AssertionFail as e:
                outcomes.append(_Outcome(name, True, str(e)))
            except Exception as e:  # noqa: BLE001 - test isolation is the point
                outcomes.append(_Outcome(name, True, f"raised: {e!r}"))
        return Report(outcomes)


class Report:
    """The result of a run: every outcome, plus a canonical text summary."""

    def __init__(self, outcomes: list[_Outcome]) -> None:
        self._outcomes = outcomes

    def passed(self) -> int:
        return sum(1 for o in self._outcomes if not o.failed)

    def failed(self) -> int:
        return sum(1 for o in self._outcomes if o.failed)

    def ok(self) -> bool:
        return self.failed() == 0

    def summary(self) -> str:
        """Render the canonical report — the exact format is the cross-language
        contract asserted by the tests."""
        lines = [f"running {len(self._outcomes)} tests"]
        for o in self._outcomes:
            status = "FAILED" if o.failed else "ok"
            lines.append(f"test {o.name} ... {status}")
        lines.append("")

        failures = [o for o in self._outcomes if o.failed]
        if failures:
            lines.append("failures:")
            for o in failures:
                lines.append(f"    {o.name}: {o.message}")
            lines.append("")

        result = "ok" if self.ok() else "FAILED"
        lines.append(
            f"test result: {result}. {self.passed()} passed; {self.failed()} failed"
        )
        return "\n".join(lines)


if __name__ == "__main__":
    import sys

    kit = (
        TestKit()
        .test("addition works", lambda: assert_eq(2 + 2, 4))
        .test("string upper", lambda: assert_eq("hi".upper(), "HI"))
        .test("deliberate failure", lambda: assert_eq(10 // 2, 4))
        .test("a truthy check", lambda: assert_true(3 > 1, "3 should be > 1"))
    )
    report = kit.run()
    print(report.summary())
    sys.exit(0 if report.ok() else 1)
