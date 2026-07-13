"""The outer layer: concrete adapters for the app-layer ports. Depends inward
on domain; nothing inward depends on it.
"""

from __future__ import annotations

from domain import Task


class InMemoryRepo:
    """The storage adapter (production would add a Postgres adapter here)."""

    def __init__(self) -> None:
        self._tasks: dict[int, Task] = {}
        self._next = 0

    def next_id(self) -> int:
        return self._next + 1

    def save(self, task: Task) -> None:
        self._next = max(self._next, task.id)
        self._tasks[task.id] = task

    def get(self, task_id: int) -> Task | None:
        return self._tasks.get(task_id)
