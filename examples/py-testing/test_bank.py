"""Tests for bank.py, showcasing the unittest toolkit (stdlib — no deps).

Demonstrates: setUp fixtures, assertRaises, subtests (the pythonic
"table-driven" pattern), mocking with unittest.mock, and wiring in doctests.
"""

import doctest
import unittest
from unittest.mock import Mock

import bank
from bank import Account, InsufficientFundsError, stamp, transfer


class AccountTests(unittest.TestCase):
    def setUp(self) -> None:
        # setUp runs before every test method — a fresh account each time.
        self.acc = Account(100)

    def test_deposit_increases_balance(self) -> None:
        self.assertEqual(self.acc.deposit(50), 150)
        self.assertEqual(self.acc.balance, 150)

    def test_withdraw_reduces_balance(self) -> None:
        self.assertEqual(self.acc.withdraw(30), 70)

    def test_overdraw_raises_and_leaves_balance(self) -> None:
        with self.assertRaises(InsufficientFundsError):
            self.acc.withdraw(150)
        self.assertEqual(self.acc.balance, 100)  # invariant held

    def test_rejects_nonpositive_amounts(self) -> None:
        # Subtests: many cases, each reported separately, in one method.
        for amount in (0, -5):
            with self.subTest(amount=amount):
                with self.assertRaises(ValueError):
                    self.acc.deposit(amount)
                with self.assertRaises(ValueError):
                    self.acc.withdraw(amount)

    def test_negative_initial_balance_rejected(self) -> None:
        with self.assertRaises(ValueError):
            Account(-1)


class TransferTests(unittest.TestCase):
    def test_successful_transfer(self) -> None:
        a, b = Account(100), Account(0)
        transfer(a, b, 40)
        self.assertEqual((a.balance, b.balance), (60, 40))

    def test_failed_transfer_does_not_credit_destination(self) -> None:
        a, b = Account(30), Account(0)
        with self.assertRaises(InsufficientFundsError):
            transfer(a, b, 100)
        # The destination must be untouched when the source can't pay.
        self.assertEqual((a.balance, b.balance), (30, 0))


class MockingTests(unittest.TestCase):
    def test_stamp_uses_injected_clock(self) -> None:
        # A Mock stands in for a real Clock so the result is deterministic.
        clock = Mock()
        clock.now.return_value = "2024-01-01T00:00"
        self.assertEqual(stamp(clock, "hello"), "[2024-01-01T00:00] hello")
        clock.now.assert_called_once()


def load_tests(loader, tests, ignore):  # noqa: ANN001, ARG001
    tests.addTests(doctest.DocTestSuite(bank))
    return tests


if __name__ == "__main__":
    unittest.main()
