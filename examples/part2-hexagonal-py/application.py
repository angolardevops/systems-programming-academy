"""The middle layer: use cases plus the ports they need. Depends only on
domain — the port is defined HERE, in the layer that uses it.
"""

from __future__ import annotations

from typing import Protocol

from domain import Task


class NotFoundError(Exception):
    """The use case was asked about a task that doesn't exist."""


class TaskRepo(Protocol):
    """The port the use cases need from storage."""

    def next_id(self) -> int: ...
    def save(self, task: Task) -> None: ...
    def get(self, task_id: int) -> Task | None: ...


class TaskService:
    """The use cases, depending on the port."""

    def __init__(self, repo: TaskRepo) -> None:
        self._repo = repo

    def add(self, title: str) -> int:
        task_id = self._repo.next_id()
        task = Task.new(task_id, title)  # domain rule enforced here
        self._repo.save(task)
        return task_id

    def complete(self, task_id: int) -> None:
        task = self._repo.get(task_id)
        if task is None:
            raise NotFoundError(f"task {task_id} not found")
        task.complete()  # domain rule enforced here
        self._repo.save(task)
