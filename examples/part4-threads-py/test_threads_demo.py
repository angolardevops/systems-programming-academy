"""Tests: correctness of the threaded helpers (timings live in __main__)."""

import doctest
import unittest

import threads_demo
from threads_demo import counter_batched, counter_locked, sum_parallel


def load_tests(loader, tests, ignore):  # noqa: ARG001 - unittest protocol
    tests.addTests(doctest.DocTestSuite(threads_demo))
    return tests


class SumParallelTest(unittest.TestCase):
    def test_matches_sequential(self) -> None:
        data = list(range(1, 10_001))
        self.assertEqual(sum_parallel(data, 4), sum(data))

    def test_more_threads_than_elements(self) -> None:
        self.assertEqual(sum_parallel([1, 2, 3], 16), 6)

    def test_empty_sequence_is_zero(self) -> None:
        self.assertEqual(sum_parallel([], 4), 0)

    def test_rejects_zero_threads(self) -> None:
        with self.assertRaises(ValueError):
            sum_parallel([1], 0)


class CounterTest(unittest.TestCase):
    def test_locked_is_exact(self) -> None:
        self.assertEqual(counter_locked(8, 10_000), 80_000)

    def test_batched_is_exact(self) -> None:
        self.assertEqual(counter_batched(8, 10_000), 80_000)


if __name__ == "__main__":
    unittest.main()
