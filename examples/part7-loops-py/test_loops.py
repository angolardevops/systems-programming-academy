"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from loops import run_program


class LoopsTest(unittest.TestCase):
    def test_print_emits_values(self) -> None:
        self.assertEqual(run_program("print 42; print 7"), ["42", "7"])

    def test_while_loop_counts(self) -> None:
        self.assertEqual(
            run_program("i = 1; while i <= 5 do { print i; i = i + 1 }"),
            ["1", "2", "3", "4", "5"],
        )

    def test_loop_computes_factorial_iteratively(self) -> None:
        out = run_program(
            "n = 5; acc = 1; i = 1; while i <= n do { acc = acc * i; i = i + 1 }; print acc"
        )
        self.assertEqual(out, ["120"])

    def test_nested_loops(self) -> None:
        out = run_program(
            "i = 1; while i <= 3 do { j = 1; while j <= 3 do { print i * j; j = j + 1 }; i = i + 1 }"
        )
        self.assertEqual(out, ["1", "2", "3", "2", "4", "6", "3", "6", "9"])

    def test_loops_and_functions_together(self) -> None:
        out = run_program(
            "sq(x) = x * x; i = 1; while i <= 4 do { print sq(i); i = i + 1 }"
        )
        self.assertEqual(out, ["1", "4", "9", "16"])

    def test_fibonacci_sequence_via_loop(self) -> None:
        out = run_program(
            "a = 0; b = 1; i = 0; while i < 8 do { print a; t = a + b; a = b; b = t; i = i + 1 }"
        )
        self.assertEqual(out, ["0", "1", "1", "2", "3", "5", "8", "13"])


if __name__ == "__main__":
    unittest.main()
