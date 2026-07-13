"""Tests: correctness of the queue-based pipeline (timings live in __main__)."""

import doctest
import queue
import unittest

import channels_demo
from channels_demo import first_response, sum_squares_pool, throughput


def load_tests(loader, tests, ignore):  # noqa: ARG001 - unittest protocol
    tests.addTests(doctest.DocTestSuite(channels_demo))
    return tests


class SumSquaresPoolTest(unittest.TestCase):
    def test_matches_sequential(self) -> None:
        nums = list(range(1, 1001))
        want = sum(n * n for n in nums)
        self.assertEqual(sum_squares_pool(nums, 4), want)

    def test_one_worker(self) -> None:
        self.assertEqual(sum_squares_pool([1, 2, 3], 1), 14)

    def test_more_workers_than_jobs(self) -> None:
        self.assertEqual(sum_squares_pool([3], 16), 9)

    def test_empty_input_is_zero(self) -> None:
        self.assertEqual(sum_squares_pool([], 4), 0)

    def test_rejects_zero_workers(self) -> None:
        with self.assertRaises(ValueError):
            sum_squares_pool([1], 0)


class FirstResponseTest(unittest.TestCase):
    def test_returns_available_item(self) -> None:
        ready: queue.Queue[str] = queue.Queue()
        ready.put("fast")
        self.assertEqual(first_response(ready), "fast")

    def test_times_out_on_empty_queue(self) -> None:
        empty: queue.Queue[str] = queue.Queue()
        with self.assertRaises(queue.Empty):
            first_response(empty, timeout=0.01)


class ThroughputTest(unittest.TestCase):
    def test_unbounded_and_bounded_sum_all(self) -> None:
        want = sum(range(10_000))
        self.assertEqual(throughput(10_000), want)
        self.assertEqual(throughput(10_000, maxsize=1024), want)


if __name__ == "__main__":
    unittest.main()
