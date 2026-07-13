"""Tested companion code for the Academy lesson "Python: The Data Model".

The *data model* is the set of special ("dunder") methods — ``__len__``,
``__getitem__``, ``__repr__``, ``__add__`` and friends — that let your own
objects plug into Python's syntax and built-in functions. Implement them and
``len(obj)``, ``obj[i]``, ``a + b``, ``repr(obj)`` and ``for x in obj`` all just
work.

Run the tests:

    python3 -m unittest discover -s . -p 'test_*.py'
    python3 -m doctest datamodel.py -v
"""

from __future__ import annotations

import math
from collections import namedtuple

# A tiny immutable record. namedtuple already gives it a good __repr__, __eq__,
# and iteration — itself a demonstration of the data model.
Card = namedtuple("Card", ["rank", "suit"])


class FrenchDeck:
    """A deck of 52 cards.

    By implementing just ``__len__`` and ``__getitem__``, the deck becomes a
    full sequence: it supports ``len()``, indexing, slicing, iteration,
    ``in``, and ``reversed()`` — none of which we wrote explicitly.

    >>> deck = FrenchDeck()
    >>> len(deck)
    52
    >>> deck[0]
    Card(rank='2', suit='spades')
    >>> deck[-1]
    Card(rank='A', suit='hearts')
    >>> Card('Q', 'hearts') in deck
    True
    >>> Card('Z', 'spades') in deck
    False
    """

    ranks = [str(n) for n in range(2, 11)] + list("JQKA")
    suits = ["spades", "diamonds", "clubs", "hearts"]

    def __init__(self) -> None:
        self._cards = [Card(rank, suit) for suit in self.suits for rank in self.ranks]

    def __len__(self) -> int:
        return len(self._cards)

    def __getitem__(self, position: int) -> Card:
        return self._cards[position]


class Vector:
    """A 2D vector demonstrating the arithmetic and representation dunders.

    >>> v = Vector(3, 4)
    >>> v
    Vector(3, 4)
    >>> abs(v)
    5.0
    >>> Vector(1, 2) + Vector(2, 4)
    Vector(3, 6)
    >>> Vector(2, 3) * 3
    Vector(6, 9)
    >>> bool(Vector(0, 0))
    False
    >>> Vector(1, 0) == Vector(1, 0)
    True
    """

    def __init__(self, x: float, y: float) -> None:
        self.x = x
        self.y = y

    def __repr__(self) -> str:
        # __repr__ should be unambiguous and, ideally, look like valid source.
        return f"Vector({self.x!r}, {self.y!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Vector):
            return NotImplemented
        return (self.x, self.y) == (other.x, other.y)

    def __abs__(self) -> float:
        return math.hypot(self.x, self.y)

    def __bool__(self) -> bool:
        # An object is "truthy" unless it says otherwise; here, the zero vector
        # is falsy.
        return bool(abs(self))

    def __add__(self, other: Vector) -> Vector:
        return Vector(self.x + other.x, self.y + other.y)

    def __mul__(self, scalar: float) -> Vector:
        return Vector(self.x * scalar, self.y * scalar)


def total_ranks(deck: FrenchDeck) -> int:
    """Sum the numeric ranks (2..10) in a deck, ignoring face cards and aces.

    Uses plain iteration over the deck — enabled purely by ``__getitem__``.

    >>> total_ranks(FrenchDeck())
    216
    """
    total = 0
    for card in deck:  # iteration comes for free from __getitem__
        if card.rank.isdigit():
            total += int(card.rank)
    return total
