"""Tests: the same eight scenarios as the Rust and Go twins."""

import unittest

from validator import (
    Schema,
    error_lines,
    in_range,
    max_length,
    min_length,
    one_of,
    required,
)


def user_schema() -> Schema:
    return (
        Schema()
        .field("name", required(), min_length(2), max_length(30))
        .field("age", required(), in_range(18, 120))
        .field("role", one_of("admin", "user", "guest"))
    )


class ValidatorTest(unittest.TestCase):
    def test_valid_record_has_no_errors(self) -> None:
        data = {"name": "Ana", "age": "30", "role": "admin"}
        self.assertEqual(user_schema().validate(data), [])

    def test_missing_required_field_reports_is_required(self) -> None:
        data = {"age": "30", "role": "user"}
        self.assertEqual(
            error_lines(user_schema().validate(data)), ["name: is required"]
        )

    def test_too_short_reports_min_length(self) -> None:
        data = {"name": "A", "age": "30", "role": "user"}
        self.assertEqual(
            error_lines(user_schema().validate(data)),
            ["name: must be at least 2 characters"],
        )

    def test_accumulates_all_errors_not_just_the_first(self) -> None:
        data = {"name": "A", "age": "old", "role": "wizard"}
        self.assertEqual(
            error_lines(user_schema().validate(data)),
            [
                "name: must be at least 2 characters",
                "age: must be an integer",
                "role: must be one of admin, user, guest",
            ],
        )

    def test_range_checks_bounds(self) -> None:
        data = {"name": "Ana", "age": "150", "role": "user"}
        self.assertEqual(
            error_lines(user_schema().validate(data)),
            ["age: must be between 18 and 120"],
        )

    def test_optional_absent_field_is_skipped(self) -> None:
        schema = Schema().field("bio", max_length(100))
        self.assertEqual(schema.validate({}), [])

    def test_one_of_accepts_allowed_and_rejects_others(self) -> None:
        schema = Schema().field("role", one_of("admin", "user"))
        self.assertEqual(schema.validate({"role": "admin"}), [])
        self.assertEqual(
            error_lines(schema.validate({"role": "root"})),
            ["role: must be one of admin, user"],
        )

    def test_multibyte_length_counts_characters_not_bytes(self) -> None:
        # "José" is 4 characters but 5 bytes — min_length must count chars.
        schema = Schema().field("name", min_length(4))
        self.assertEqual(schema.validate({"name": "José"}), [])


if __name__ == "__main__":
    unittest.main()
