"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from fns import Token, run_program, tokenize


class FnsTest(unittest.TestCase):
    def test_tokenizes_a_function_definition(self) -> None:
        self.assertEqual(
            tokenize("f(x, y) = x + y"),
            [
                Token("i", name="f"),
                Token("("),
                Token("i", name="x"),
                Token(","),
                Token("i", name="y"),
                Token(")"),
                Token("="),
                Token("i", name="x"),
                Token("+"),
                Token("i", name="y"),
            ],
        )

    def test_defines_and_calls_a_function(self) -> None:
        self.assertEqual(
            run_program("double(x) = x * 2\ndouble(21)"),
            ["double(x) = x * 2  =>  <fn>", "double(21)  =>  42"],
        )

    def test_handles_multiple_arguments_and_nested_calls(self) -> None:
        out = run_program("add(a, b) = a + b\nadd(add(1, 2), 3)")
        self.assertEqual(out[-1], "add(add(1, 2), 3)  =>  6")

    def test_closures_use_lexical_not_dynamic_scope(self) -> None:
        # f captures x = 10 where defined; g's own x parameter must not leak in.
        out = run_program("x = 10\nf(n) = n + x\ng(x) = f(0)\ng(999)")
        self.assertEqual(out[-1], "g(999)  =>  10")

    def test_closure_sees_later_updates_to_captured_variable(self) -> None:
        out = run_program(
            "base = 100\nshift(n) = n + base\nshift(5)\nbase = 200\nshift(5)"
        )
        self.assertEqual(out[2], "shift(5)  =>  105")
        self.assertEqual(out[4], "shift(5)  =>  205")

    def test_reports_arity_and_kind_errors(self) -> None:
        out = run_program("double(x) = x * 2\ndouble(1, 2)\nnope(3)\n5(3)")
        self.assertIn("expects 1 argument(s), got 2", out[1])
        self.assertIn("undefined function 'nope'", out[2])
        self.assertTrue("not a function" in out[3] or "trailing" in out[3])


if __name__ == "__main__":
    unittest.main()
