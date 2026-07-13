"""Tested companion code for the Academy lesson "Python: Type Hints".

Python is dynamically typed at runtime, but *type hints* (PEP 484 and later) let
you annotate code so tools like mypy/pyright and your editor can catch bugs before
you run it. Hints are not enforced at runtime — they are a contract for tooling and
readers. This module shows the common ones with modern 3.10+ syntax.

Run the tests:

    python3 -m unittest discover -s . -p 'test_*.py'
"""

from __future__ import annotations

from dataclasses import dataclass, field


def greet(name: str, times: int = 1) -> str:
    """Return a greeting repeated ``times`` times.

    >>> greet("Ada")
    'Hello, Ada!'
    >>> greet("Go", times=2)
    'Hello, Go! Hello, Go!'
    """
    return " ".join([f"Hello, {name}!"] * times)


def first(items: list[int]) -> int | None:
    """Return the first item, or ``None`` if the list is empty.

    The ``int | None`` return type (an Optional) documents that callers must
    handle the empty case.

    >>> first([10, 20])
    10
    >>> first([]) is None
    True
    """
    if not items:
        return None
    return items[0]


def total_price(prices: dict[str, float]) -> float:
    """Sum the values of a name -> price mapping.

    >>> total_price({"a": 1.5, "b": 2.5})
    4.0
    >>> total_price({})
    0.0
    """
    # Start at 0.0 so an empty mapping still returns a float, matching the
    # annotation (bare sum() would return int 0).
    return sum(prices.values(), 0.0)


@dataclass(frozen=True)
class Point:
    """An immutable 2D point. ``@dataclass`` generates ``__init__``, ``__repr__``,
    ``__eq__`` and (because it is frozen) ``__hash__`` from the annotations.

    >>> p = Point(1, 2)
    >>> p
    Point(x=1, y=2)
    >>> p == Point(1, 2)
    True
    >>> {p}  # frozen dataclasses are hashable
    {Point(x=1, y=2)}
    """

    x: float
    y: float


@dataclass
class Cart:
    """A mutable shopping cart. Note the ``field(default_factory=list)`` — using a
    bare ``[]`` default would share one list across all instances.

    >>> c = Cart()
    >>> c.add("apple", 1.5)
    >>> c.add("pear", 2.0)
    >>> c.total()
    3.5
    """

    items: list[tuple[str, float]] = field(default_factory=list)

    def add(self, name: str, price: float) -> None:
        self.items.append((name, price))

    def total(self) -> float:
        return sum((price for _, price in self.items), 0.0)


# A simple generic function using a type variable (PEP 695 / 3.12 syntax).
def last[T](items: list[T]) -> T | None:
    """Return the last element of any list, preserving its element type.

    >>> last([1, 2, 3])
    3
    >>> last(["a", "b"])
    'b'
    >>> last([]) is None
    True
    """
    return items[-1] if items else None
