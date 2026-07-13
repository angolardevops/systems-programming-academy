"""Unit tests for datamodel.py (stdlib unittest — no third-party deps).

This module also wires the doctests in datamodel.py into the unittest run, so
``python3 -m unittest`` executes both.
"""

import doctest
import unittest

import datamodel
from datamodel import Card, FrenchDeck, Vector, total_ranks


class FrenchDeckTests(unittest.TestCase):
    def setUp(self) -> None:
        self.deck = FrenchDeck()

    def test_len_is_52(self) -> None:
        self.assertEqual(len(self.deck), 52)

    def test_indexing_and_slicing(self) -> None:
        self.assertEqual(self.deck[0], Card("2", "spades"))
        # Slicing works because __getitem__ forwards to the list.
        top3 = self.deck[:3]
        self.assertEqual(len(top3), 3)

    def test_iteration_and_membership(self) -> None:
        # `in` and iteration both come from __getitem__.
        self.assertIn(Card("A", "hearts"), self.deck)
        self.assertNotIn(Card("Z", "spades"), self.deck)
        self.assertEqual(len(list(self.deck)), 52)

    def test_reversed(self) -> None:
        self.assertEqual(next(reversed(self.deck)), Card("A", "hearts"))


class VectorTests(unittest.TestCase):
    def test_repr_roundtrips(self) -> None:
        self.assertEqual(repr(Vector(3, 4)), "Vector(3, 4)")

    def test_equality(self) -> None:
        self.assertEqual(Vector(1, 2), Vector(1, 2))
        self.assertNotEqual(Vector(1, 2), Vector(2, 1))
        # Comparing to a non-Vector is False, not an error.
        self.assertNotEqual(Vector(1, 2), (1, 2))

    def test_abs(self) -> None:
        self.assertEqual(abs(Vector(3, 4)), 5.0)

    def test_add_and_mul(self) -> None:
        self.assertEqual(Vector(1, 2) + Vector(2, 4), Vector(3, 6))
        self.assertEqual(Vector(2, 3) * 3, Vector(6, 9))

    def test_bool(self) -> None:
        self.assertFalse(Vector(0, 0))
        self.assertTrue(Vector(0, 1))


class TotalRanksTests(unittest.TestCase):
    def test_total_ranks(self) -> None:
        # Each suit contributes 2+3+...+10 = 54; four suits => 216.
        self.assertEqual(total_ranks(FrenchDeck()), 216)


def load_tests(loader, tests, ignore):  # noqa: ANN001, ARG001
    """Add datamodel.py's doctests to the unittest suite."""
    tests.addTests(doctest.DocTestSuite(datamodel))
    return tests


if __name__ == "__main__":
    unittest.main()
