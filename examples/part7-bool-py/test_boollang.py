"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from boollang import run_program


def _last(src: str) -> str:
    return run_program(src)[-1]


class BoolTest(unittest.TestCase):
    def test_and_or_truth_tables(self) -> None:
        self.assertEqual(_last("1 and 1"), "1 and 1  =>  1")
        self.assertEqual(_last("1 and 0"), "1 and 0  =>  0")
        self.assertEqual(_last("0 and 1"), "0 and 1  =>  0")
        self.assertEqual(_last("0 or 0"), "0 or 0  =>  0")
        self.assertEqual(_last("1 or 0"), "1 or 0  =>  1")

    def test_not_negates_truthiness(self) -> None:
        self.assertEqual(_last("not 0"), "not 0  =>  1")
        self.assertEqual(_last("not 5"), "not 5  =>  0")  # any nonzero is truthy
        self.assertEqual(_last("not not 3"), "not not 3  =>  1")

    def test_precedence_not_tighter_than_and_tighter_than_or(self) -> None:
        self.assertEqual(_last("1 or 0 and 0"), "1 or 0 and 0  =>  1")  # 1 or (0 and 0)
        self.assertEqual(_last("not 0 and 1"), "not 0 and 1  =>  1")  # (not 0) and 1
        self.assertEqual(_last("2 > 1 and 3 > 5"), "2 > 1 and 3 > 5  =>  0")

    def test_and_short_circuits_avoiding_the_error(self) -> None:
        out = run_program(
            "guard(x) = if x != 0 and 100 / x > 1 then 100 / x else -1\nguard(0)\nguard(50)"
        )
        self.assertEqual(out[1], "guard(0)  =>  -1")  # no division-by-zero
        self.assertEqual(out[2], "guard(50)  =>  2")

    def test_or_short_circuits_avoiding_the_error(self) -> None:
        out = run_program("check(a) = if a == 0 or 10 / a > 0 then 1 else 0\ncheck(0)")
        self.assertEqual(out[-1], "check(0)  =>  1")  # no division-by-zero

    def test_booleans_compose_into_real_predicates(self) -> None:
        out = run_program(
            "in_range(x, lo, hi) = if x >= lo and x <= hi then 1 else 0\n"
            "in_range(5, 1, 10)\nin_range(15, 1, 10)"
        )
        self.assertEqual(out[1], "in_range(5, 1, 10)  =>  1")
        self.assertEqual(out[2], "in_range(15, 1, 10)  =>  0")


if __name__ == "__main__":
    unittest.main()
