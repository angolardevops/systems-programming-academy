"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from calc import CalcError, Token, run, tokenize


class CalcTest(unittest.TestCase):
    def test_tokenizes_numbers_and_operators(self) -> None:
        self.assertEqual(
            tokenize("12 + 3"),
            [Token("n", 12), Token("+"), Token("n", 3)],
        )

    def test_precedence_binds_star_tighter_than_plus(self) -> None:
        self.assertEqual(run("1 + 2 * 3"), ("(+ 1 (* 2 3))", 7))

    def test_parentheses_override_precedence(self) -> None:
        self.assertEqual(run("(1 + 2) * 3"), ("(* (+ 1 2) 3)", 9))

    def test_unary_minus_and_truncating_division(self) -> None:
        # truncates toward zero, not floor(-3.5) = -4
        self.assertEqual(run("-7 / 2"), ("(/ (neg 7) 2)", -3))

    def test_evaluates_a_longer_expression(self) -> None:
        self.assertEqual(
            run("2 * (3 + 4) - 10 / 3"),
            ("(- (* 2 (+ 3 4)) (/ 10 3))", 11),
        )

    def test_reports_errors_without_crashing(self) -> None:
        cases = {
            "1 / 0": "division by zero",
            "1 +": "unexpected end of input",
            "1 @ 2": "unexpected character",
            "(1 + 2": "expected ')'",
            "1 2": "trailing",
        }
        for src, want in cases.items():
            with self.assertRaises(CalcError) as ctx:
                run(src)
            self.assertIn(want, str(ctx.exception), src)


if __name__ == "__main__":
    unittest.main()
