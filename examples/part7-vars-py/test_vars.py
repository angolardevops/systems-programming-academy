"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from vars import Env, Token, execute, parse_stmt, run_program, tokenize


class VarsTest(unittest.TestCase):
    def test_tokenizes_identifiers_and_assignment(self) -> None:
        self.assertEqual(
            tokenize("x = 5"),
            [Token("i", name="x"), Token("="), Token("n", num=5)],
        )

    def test_assignment_binds_and_reference_reads(self) -> None:
        env: Env = {}
        self.assertEqual(execute(parse_stmt(tokenize("x = 40")), env), 40)
        self.assertEqual(execute(parse_stmt(tokenize("x + 2")), env), 42)

    def test_state_persists_across_statements(self) -> None:
        self.assertEqual(
            run_program("x = 5\ny = x * 2 + 1\ny - x"),
            ["x = 5  =>  5", "y = x * 2 + 1  =>  11", "y - x  =>  6"],
        )

    def test_undefined_variable_is_an_error(self) -> None:
        self.assertEqual(
            run_program("z + 1"),
            ["z + 1  =>  error: undefined variable 'z'"],
        )

    def test_reassignment_updates_using_the_old_value(self) -> None:
        self.assertEqual(
            run_program("x = 1\nx = x + 10\nx"),
            ["x = 1  =>  1", "x = x + 10  =>  11", "x  =>  11"],
        )

    def test_arithmetic_errors_still_reported(self) -> None:
        out = run_program("10 / 0\nfoo bar")
        self.assertIn("division by zero", out[0])
        self.assertIn("trailing", out[1])


if __name__ == "__main__":
    unittest.main()
