"""Tested companion code for the Academy lesson "Python: Iterators & Generators".

Iteration is one of Python's most powerful ideas. The *iterator protocol*
(``__iter__``/``__next__``) is what ``for`` loops actually use, and *generators*
(functions with ``yield``) build iterators with almost no boilerplate. Generators
are **lazy**: they produce values one at a time, on demand, so they can model
infinite sequences and process huge streams with tiny memory.

Run the tests:

    python3 -m unittest discover -s . -p 'test_*.py'
"""

from __future__ import annotations

from collections.abc import Iterator


class Countdown:
    """A hand-written iterator implementing the protocol directly.

    ``__iter__`` returns the iterator (here, self); ``__next__`` returns the next
    value or raises ``StopIteration`` when exhausted.

    >>> list(Countdown(3))
    [3, 2, 1]
    """

    def __init__(self, start: int) -> None:
        self.current = start

    def __iter__(self) -> Countdown:
        return self

    def __next__(self) -> int:
        if self.current <= 0:
            raise StopIteration
        self.current -= 1
        return self.current + 1


def countdown(start: int) -> Iterator[int]:
    """The same thing as a *generator* — ``yield`` does all the protocol work.

    >>> list(countdown(3))
    [3, 2, 1]
    """
    n = start
    while n > 0:
        yield n
        n -= 1


def fib() -> Iterator[int]:
    """An *infinite* generator of Fibonacci numbers. Safe because the consumer
    decides when to stop (e.g. with ``itertools.islice`` or ``take``).

    >>> take(fib(), 8)
    [0, 1, 1, 2, 3, 5, 8, 13]
    """
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b


def take(iterable: Iterator[int], n: int) -> list[int]:
    """Return the first ``n`` items of any iterable/iterator, driving it lazily.

    >>> take(countdown(100), 3)
    [100, 99, 98]
    """
    out: list[int] = []
    it = iter(iterable)
    for _ in range(n):
        try:
            out.append(next(it))
        except StopIteration:
            break
    return out


def read_even_squares(nums: list[int]) -> Iterator[int]:
    """A generator pipeline: filter to evens, square them — one lazy pass, no
    intermediate list.

    >>> list(read_even_squares([1, 2, 3, 4, 5, 6]))
    [4, 16, 36]
    """
    return (n * n for n in nums if n % 2 == 0)


def running_total(nums: list[int]) -> Iterator[int]:
    """Yield the running (cumulative) sum. Generators can carry state between
    yields, which plain comprehensions cannot.

    >>> list(running_total([1, 2, 3, 4]))
    [1, 3, 6, 10]
    """
    total = 0
    for n in nums:
        total += n
        yield total
