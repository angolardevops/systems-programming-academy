"""Unit tests for protocols_demo.py (stdlib unittest, plus its doctests)."""

import doctest
import unittest

import protocols_demo
from protocols_demo import (
    Button,
    Dog,
    Duck,
    Heading,
    Sized,
    describe_size,
    make_it_quack,
    render_page,
)


class DuckTypingTests(unittest.TestCase):
    def test_duck_and_dog_both_quack(self) -> None:
        self.assertEqual(make_it_quack(Duck()), "Quack!")
        self.assertEqual(make_it_quack(Dog()), "Woof-quack")


class RuntimeCheckableTests(unittest.TestCase):
    def test_isinstance_against_protocol(self) -> None:
        # @runtime_checkable lets isinstance check for __len__.
        self.assertIsInstance([1, 2], Sized)
        self.assertIsInstance("abc", Sized)
        self.assertNotIsInstance(42, Sized)  # int has no __len__

    def test_describe_size(self) -> None:
        self.assertEqual(describe_size([1, 2, 3]), "has 3 items")
        self.assertEqual(describe_size("hello"), "has 5 items")


class RenderableTests(unittest.TestCase):
    def test_heterogeneous_widgets(self) -> None:
        # Button and Heading share no base class, yet both are Renderable.
        page = render_page([Heading("Home"), Button("OK")])
        self.assertEqual(page, "# Home\n[ OK ]")

    def test_single_widget(self) -> None:
        self.assertEqual(render_page([Button("Go")]), "[ Go ]")


def load_tests(loader, tests, ignore):  # noqa: ANN001, ARG001
    tests.addTests(doctest.DocTestSuite(protocols_demo))
    return tests


if __name__ == "__main__":
    unittest.main()
