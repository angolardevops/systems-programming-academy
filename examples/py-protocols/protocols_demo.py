"""Tested companion code for the Academy lesson "Python: Protocols & Duck Typing".

Python has always used *duck typing*: "if it walks like a duck and quacks like a
duck, it's a duck" — code cares about what an object *can do*, not what class it
is. ``typing.Protocol`` (PEP 544) makes that idea checkable: it describes a set of
methods/attributes, and any object that has them satisfies the protocol
**structurally**, with no explicit inheritance. This module shows duck typing,
Protocols, and ``@runtime_checkable``.

Run the tests:

    python3 -m unittest discover -s . -p 'test_*.py'
"""

from __future__ import annotations

from typing import Protocol, runtime_checkable


class Duck:
    def quack(self) -> str:
        return "Quack!"


class Dog:
    def quack(self) -> str:  # a Dog that happens to quack
        return "Woof-quack"


def make_it_quack(thing: SupportsQuack) -> str:
    """Call .quack() on anything that has it — duck typing in action.

    >>> make_it_quack(Duck())
    'Quack!'
    >>> make_it_quack(Dog())
    'Woof-quack'
    """
    return thing.quack()


class SupportsQuack(Protocol):
    """Structural type: anything with a ``quack() -> str`` method satisfies it,
    with no need to inherit from this class.
    """

    def quack(self) -> str: ...


@runtime_checkable
class Sized(Protocol):
    """A protocol marked ``@runtime_checkable`` so ``isinstance`` works against it
    (checking only for the presence of the methods, not their signatures).
    """

    def __len__(self) -> int: ...


def describe_size(obj: Sized) -> str:
    """Report an object's length using the Sized protocol.

    >>> describe_size([1, 2, 3])
    'has 3 items'
    >>> describe_size("hello")
    'has 5 items'
    """
    return f"has {len(obj)} items"


class Renderable(Protocol):
    """Objects that can render themselves to a string."""

    def render(self) -> str: ...


class Button:
    def __init__(self, label: str) -> None:
        self.label = label

    def render(self) -> str:
        return f"[ {self.label} ]"


class Heading:
    def __init__(self, text: str) -> None:
        self.text = text

    def render(self) -> str:
        return f"# {self.text}"


def render_page(widgets: list[Renderable]) -> str:
    """Render a list of anything with a ``render()`` method — Button and Heading
    both qualify without sharing a base class.

    >>> render_page([Heading("Home"), Button("OK")])
    '# Home\\n[ OK ]'
    """
    return "\n".join(w.render() for w in widgets)
