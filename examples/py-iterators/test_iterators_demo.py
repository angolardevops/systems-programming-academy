"""Unit tests for iterators_demo.py (stdlib unittest, plus its doctests)."""

import doctest
import unittest

import iterators_demo
from iterators_demo import (
    Countdown,
    countdown,
    fib,
    read_even_squares,
    running_total,
    take,
)


class CountdownTests(unittest.TestCase):
    def test_iterator_class(self) -> None:
        self.assertEqual(list(Countdown(3)), [3, 2, 1])
        self.assertEqual(list(Countdown(0)), [])

    def test_generator_matches_class(self) -> None:
        self.assertEqual(list(countdown(3)), list(Countdown(3)))


class FibTests(unittest.TestCase):
    def test_first_eight(self) -> None:
        self.assertEqual(take(fib(), 8), [0, 1, 1, 2, 3, 5, 8, 13])

    def test_infinite_is_safe_when_bounded(self) -> None:
        # Only 5 values are ever produced from the infinite generator.
        self.assertEqual(take(fib(), 5), [0, 1, 1, 2, 3])


class TakeTests(unittest.TestCase):
    def test_take_more_than_available(self) -> None:
        # take stops early instead of raising when the source is exhausted.
        self.assertEqual(take(countdown(2), 5), [2, 1])


class PipelineTests(unittest.TestCase):
    def test_even_squares(self) -> None:
        self.assertEqual(list(read_even_squares([1, 2, 3, 4, 5, 6])), [4, 16, 36])

    def test_even_squares_lazy_object(self) -> None:
        # A generator expression is itself an iterator, produced lazily.
        gen = read_even_squares([2, 4])
        self.assertEqual(next(gen), 4)
        self.assertEqual(next(gen), 16)
        with self.assertRaises(StopIteration):
            next(gen)


class RunningTotalTests(unittest.TestCase):
    def test_running_total(self) -> None:
        self.assertEqual(list(running_total([1, 2, 3, 4])), [1, 3, 6, 10])
        self.assertEqual(list(running_total([])), [])


def load_tests(loader, tests, ignore):  # noqa: ANN001, ARG001
    tests.addTests(doctest.DocTestSuite(iterators_demo))
    return tests


if __name__ == "__main__":
    unittest.main()
