"""The composition root: the only module that knows every layer, wiring
adapters into use cases and mapping errors to user-visible messages.
"""

from __future__ import annotations

from adapters import InMemoryRepo
from application import NotFoundError, TaskService
from domain import AlreadyDoneError, EmptyTitleError


class App:
    def __init__(self) -> None:
        self._service = TaskService(InMemoryRepo())

    def add(self, title: str) -> str:
        try:
            task_id = self._service.add(title)
        except EmptyTitleError:
            return "A task needs a title."
        return f"Added task #{task_id}."

    def complete(self, task_id: int) -> str:
        try:
            self._service.complete(task_id)
        except NotFoundError:
            return f"No task #{task_id}."
        except AlreadyDoneError:
            return f"Task #{task_id} was already done."
        return f"Task #{task_id} done."
