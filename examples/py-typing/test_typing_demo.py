"""Unit tests for typing_demo.py (stdlib unittest, plus its doctests)."""

import doctest
import unittest

import typing_demo
from typing_demo import Cart, Point, first, greet, last, total_price


class GreetTests(unittest.TestCase):
    def test_default(self) -> None:
        self.assertEqual(greet("Ada"), "Hello, Ada!")

    def test_repeated(self) -> None:
        self.assertEqual(greet("Go", times=2), "Hello, Go! Hello, Go!")


class OptionalTests(unittest.TestCase):
    def test_first_returns_value(self) -> None:
        self.assertEqual(first([10, 20]), 10)

    def test_first_returns_none(self) -> None:
        self.assertIsNone(first([]))


class TotalPriceTests(unittest.TestCase):
    def test_sum(self) -> None:
        self.assertEqual(total_price({"a": 1.5, "b": 2.5}), 4.0)

    def test_empty(self) -> None:
        self.assertEqual(total_price({}), 0.0)


class PointTests(unittest.TestCase):
    def test_repr_and_eq(self) -> None:
        self.assertEqual(repr(Point(1, 2)), "Point(x=1, y=2)")
        self.assertEqual(Point(1, 2), Point(1, 2))

    def test_frozen_is_hashable(self) -> None:
        self.assertIn(Point(1, 2), {Point(1, 2)})

    def test_frozen_is_immutable(self) -> None:
        p = Point(1, 2)
        with self.assertRaises(Exception):
            p.x = 5  # type: ignore[misc]


class CartTests(unittest.TestCase):
    def test_independent_default_lists(self) -> None:
        # The default_factory ensures each Cart gets its own list.
        a, b = Cart(), Cart()
        a.add("apple", 1.5)
        self.assertEqual(a.total(), 1.5)
        self.assertEqual(b.total(), 0.0)


class GenericTests(unittest.TestCase):
    def test_last_preserves_type(self) -> None:
        self.assertEqual(last([1, 2, 3]), 3)
        self.assertEqual(last(["a", "b"]), "b")
        self.assertIsNone(last([]))


def load_tests(loader, tests, ignore):  # noqa: ANN001, ARG001
    tests.addTests(doctest.DocTestSuite(typing_demo))
    return tests


if __name__ == "__main__":
    unittest.main()
