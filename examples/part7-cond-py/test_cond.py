"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from cond import run_program, tokenize


def _last(src: str) -> str:
    return run_program(src)[-1]


class CondTest(unittest.TestCase):
    def test_tokenizes_comparison_operators(self) -> None:
        toks = tokenize("a <= b == c")
        self.assertEqual(toks[1].kind, "C")
        self.assertEqual(toks[1].op, "<=")
        self.assertEqual(toks[3].op, "==")

    def test_comparisons_yield_one_or_zero(self) -> None:
        self.assertEqual(_last("3 < 5"), "3 < 5  =>  1")
        self.assertEqual(_last("3 > 5"), "3 > 5  =>  0")
        self.assertEqual(_last("4 == 4"), "4 == 4  =>  1")
        self.assertEqual(_last("4 != 4"), "4 != 4  =>  0")
        self.assertEqual(_last("5 >= 5"), "5 >= 5  =>  1")

    def test_if_selects_the_taken_branch(self) -> None:
        self.assertEqual(_last("if 1 then 10 else 20"), "if 1 then 10 else 20  =>  10")
        self.assertEqual(_last("if 0 then 10 else 20"), "if 0 then 10 else 20  =>  20")
        self.assertEqual(
            _last("if 3 < 5 then 100 else 200"),
            "if 3 < 5 then 100 else 200  =>  100",
        )

    def test_recursion_now_terminates(self) -> None:
        out = run_program("fact(n) = if n <= 1 then 1 else n * fact(n - 1)\nfact(5)")
        self.assertEqual(out[-1], "fact(5)  =>  120")

    def test_recursive_fibonacci(self) -> None:
        out = run_program(
            "fib(n) = if n < 2 then n else fib(n - 1) + fib(n - 2)\nfib(10)"
        )
        self.assertEqual(out[-1], "fib(10)  =>  55")

    def test_only_the_taken_branch_is_evaluated(self) -> None:
        # The else branch would divide by zero, but it is never taken.
        out = run_program("safe(n) = if n == 0 then 0 else 100 / n\nsafe(0)\nsafe(4)")
        self.assertEqual(out[1], "safe(0)  =>  0")
        self.assertEqual(out[2], "safe(4)  =>  25")


if __name__ == "__main__":
    unittest.main()
