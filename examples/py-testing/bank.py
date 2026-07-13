"""Tested companion code for the Academy lesson "Python: Testing".

A tiny bank-account library — a happy path, error cases, and an invariant worth
protecting — so the tests are the real subject. The test file demonstrates
``unittest`` (subtests, exceptions, mocking with ``unittest.mock``) plus doctests.

Run the tests:

    python3 -m unittest discover -s . -p 'test_*.py'
"""

from __future__ import annotations


class InsufficientFundsError(Exception):
    """Raised when a withdrawal exceeds the available balance."""


class Account:
    """A minimal account whose balance can never go negative.

    >>> acc = Account(100)
    >>> acc.deposit(50)
    150
    >>> acc.withdraw(30)
    120
    >>> acc.balance
    120
    """

    def __init__(self, balance: int = 0) -> None:
        if balance < 0:
            raise ValueError("initial balance must not be negative")
        self.balance = balance

    def deposit(self, amount: int) -> int:
        if amount <= 0:
            raise ValueError("deposit amount must be positive")
        self.balance += amount
        return self.balance

    def withdraw(self, amount: int) -> int:
        if amount <= 0:
            raise ValueError("withdrawal amount must be positive")
        if amount > self.balance:
            raise InsufficientFundsError(
                f"cannot withdraw {amount} from balance {self.balance}"
            )
        self.balance -= amount
        return self.balance


def transfer(src: Account, dst: Account, amount: int) -> None:
    """Move ``amount`` from ``src`` to ``dst`` atomically-ish: if the withdrawal
    fails, the deposit never happens.

    >>> a, b = Account(100), Account(0)
    >>> transfer(a, b, 40)
    >>> (a.balance, b.balance)
    (60, 40)
    """
    src.withdraw(amount)  # raises before dst is touched if funds are short
    dst.deposit(amount)


class Clock:
    """A trivial dependency we will *mock* in a test — pretend it reads a real
    wall clock. Keeping it injectable is what makes the code testable.
    """

    def now(self) -> str:  # pragma: no cover - replaced by a mock in tests
        raise NotImplementedError


def stamp(clock: Clock, message: str) -> str:
    """Prefix ``message`` with the clock's current time.

    In tests we inject a fake/mocked clock so the output is deterministic.
    """
    return f"[{clock.now()}] {message}"
