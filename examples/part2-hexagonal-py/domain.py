"""The innermost layer: entities and business rules. Imports NOTHING from the
outer layers — test_architecture.py enforces this Dependency Rule with a test.
"""

from __future__ import annotations

from dataclasses import dataclass


class EmptyTitleError(Exception):
    """A task must have a non-empty title."""


class AlreadyDoneError(Exception):
    """Completing twice is an error, not a no-op."""


@dataclass
class Task:
    """The entity; its invariants live here, next to its data."""

    id: int
    title: str
    done: bool = False

    @classmethod
    def new(cls, task_id: int, title: str) -> Task:
        title = title.strip()
        if not title:
            raise EmptyTitleError("a task needs a title")
        return cls(id=task_id, title=title)

    def complete(self) -> None:
        if self.done:
            raise AlreadyDoneError(f"task {self.id} already done")
        self.done = True
