"""Tests at every layer — including an ARCHITECTURE test that enforces the
Dependency Rule mechanically (stdlib unittest)."""

import ast
import pathlib
import unittest

from adapters import InMemoryRepo
from application import NotFoundError, TaskService
from domain import AlreadyDoneError, EmptyTitleError, Task
from main_app import App


class DomainTests(unittest.TestCase):
    """Domain rules tested with zero infrastructure — the payoff of a pure core."""

    def test_task_requires_a_title(self) -> None:
        with self.assertRaises(EmptyTitleError):
            Task.new(1, "   ")

    def test_completing_twice_is_an_error(self) -> None:
        task = Task.new(1, "write lesson")
        task.complete()
        with self.assertRaises(AlreadyDoneError):
            task.complete()


class UseCaseTests(unittest.TestCase):
    """Use cases with the in-memory adapter injected."""

    def setUp(self) -> None:
        self.service = TaskService(InMemoryRepo())

    def test_add_then_complete_roundtrip(self) -> None:
        task_id = self.service.add("ship part 2")
        self.service.complete(task_id)
        with self.assertRaises(AlreadyDoneError):
            self.service.complete(task_id)

    def test_completing_unknown_id_is_not_found(self) -> None:
        with self.assertRaises(NotFoundError):
            self.service.complete(99)


class EndToEndTests(unittest.TestCase):
    """The composition root, through user-visible messages only."""

    def test_full_flow_through_the_app(self) -> None:
        app = App()
        self.assertEqual(app.add("write lesson"), "Added task #1.")
        self.assertEqual(app.complete(1), "Task #1 done.")
        self.assertEqual(app.complete(1), "Task #1 was already done.")
        self.assertEqual(app.complete(9), "No task #9.")
        self.assertEqual(app.add("  "), "A task needs a title.")


class ArchitectureTests(unittest.TestCase):
    """The Dependency Rule as an executable check: if someone adds
    `from adapters import ...` to domain.py, THIS TEST FAILS."""

    FORBIDDEN = {
        "domain.py": {"application", "adapters", "main_app"},
        "application.py": {"adapters", "main_app"},
        "adapters.py": {"application", "main_app"},
    }

    def imports_of(self, filename: str) -> set[str]:
        source = pathlib.Path(__file__).with_name(filename).read_text()
        modules: set[str] = set()
        for node in ast.walk(ast.parse(source)):
            if isinstance(node, ast.Import):
                modules.update(alias.name.split(".")[0] for alias in node.names)
            elif isinstance(node, ast.ImportFrom) and node.module:
                modules.add(node.module.split(".")[0])
        return modules

    def test_dependencies_point_inward(self) -> None:
        for filename, forbidden in self.FORBIDDEN.items():
            with self.subTest(layer=filename):
                illegal = self.imports_of(filename) & forbidden
                self.assertEqual(
                    illegal,
                    set(),
                    f"{filename} must not import outward: {illegal}",
                )


if __name__ == "__main__":
    unittest.main()
