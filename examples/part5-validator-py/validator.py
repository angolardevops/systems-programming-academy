"""A declarative validation framework: describe the rules a record must
satisfy, then validate a record against them and get back *every* error at once.

The framework idea is **declaration over imperative checking**: instead of
scattered ``if`` statements, you declare a schema and the evaluator applies it.
The decision that matters most is **error accumulation** — collect ALL failures
and return them together, rather than bailing on the first. A form that reports
every problem at once is a good experience; one that reports them one at a time
is not. Fail-fast is for programming errors; user input wants fail-complete.

Rules are plain checks, so the framework is dependency-free and the collected
errors are directly assertable — no I/O anywhere.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

# A rule checks a value and returns an error message, or "" if it passes.
Rule = Callable[[str], str]


def required() -> Rule:
    """The value must be present and non-empty."""
    return lambda v: "is required" if v == "" else ""


def min_length(n: int) -> Rule:
    """At least ``n`` characters (counts characters, not bytes)."""
    return lambda v: f"must be at least {n} characters" if len(v) < n else ""


def max_length(n: int) -> Rule:
    """At most ``n`` characters."""
    return lambda v: f"must be at most {n} characters" if len(v) > n else ""


def is_int() -> Rule:
    """Must parse as an integer."""

    def check(v: str) -> str:
        try:
            int(v)
        except ValueError:
            return "must be an integer"
        return ""

    return check


def in_range(lo: int, hi: int) -> Rule:
    """Must parse as an integer within ``[lo, hi]`` (implies :func:`is_int`)."""

    def check(v: str) -> str:
        try:
            n = int(v)
        except ValueError:
            return "must be an integer"
        return f"must be between {lo} and {hi}" if n < lo or n > hi else ""

    return check


def one_of(*options: str) -> Rule:
    """Must be one of the allowed values."""
    return lambda v: "" if v in options else "must be one of " + ", ".join(options)


def _is_required(rule: Rule) -> bool:
    """A rule is the 'required' rule if it flags the empty string."""
    return rule("") == "is required"


@dataclass
class Error:
    """A single validation failure."""

    field: str
    message: str

    def line(self) -> str:
        """Render as ``"field: message"`` — the stable format the tests assert."""
        return f"{self.field}: {self.message}"


class Schema:
    """An ordered list of (field, rules). Order is preserved in the error
    output, so results are deterministic and identical across languages."""

    def __init__(self) -> None:
        self._fields: list[tuple[str, list[Rule]]] = []

    def field(self, name: str, *rules: Rule) -> Schema:
        """Declare the rules for a field. Chainable."""
        self._fields.append((name, list(rules)))
        return self

    def validate(self, data: dict[str, str]) -> list[Error]:
        """Return every error found, in field-declaration then rule order.

        A field carrying :func:`required` that is missing/empty yields exactly
        one "is required" error and its other rules are skipped. A field
        without ``required`` that is absent/empty is skipped — that is what
        "optional" means.
        """
        errors: list[Error] = []
        for name, rules in self._fields:
            value = data.get(name, "")
            present = value != ""
            is_req = any(_is_required(r) for r in rules)

            if not present:
                if is_req:
                    errors.append(Error(name, "is required"))
                continue

            for rule in rules:
                if _is_required(rule):
                    continue
                message = rule(value)
                if message:
                    errors.append(Error(name, message))
        return errors


def error_lines(errors: list[Error]) -> list[str]:
    """Render a list of errors as ``"field: message"`` lines."""
    return [e.line() for e in errors]


if __name__ == "__main__":
    schema = (
        Schema()
        .field("username", required(), min_length(3), max_length(20))
        .field("age", required(), in_range(18, 120))
        .field("role", one_of("admin", "user", "guest"))
    )

    bad = {"username": "ab", "age": "twelve", "role": "superadmin"}
    print("Validating a broken signup:")
    for line in error_lines(schema.validate(bad)):
        print(f"  - {line}")

    good = {"username": "walter", "age": "34", "role": "admin"}
    print(f"\nValidating a good signup: {len(schema.validate(good))} errors")
