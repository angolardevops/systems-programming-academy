"""Python companion for the Part 2 lesson "Repository Pattern & Dependency
Injection". The same domain is implemented in Rust, Go, and Python for direct
comparison.

Layers: User (domain), UserRepository (port, a Protocol), InMemoryUserRepository
(adapter), UserService (business logic depending on the Protocol).

Run the tests:

    python3 -m unittest discover -s . -p 'test_*.py'
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol


class DuplicateIdError(Exception):
    """Raised when adding a user whose id already exists."""


@dataclass(frozen=True)
class User:
    """The domain entity."""

    id: int
    name: str


class UserRepository(Protocol):
    """The port: what the service needs from storage, as a structural Protocol.
    Any object with these methods satisfies it — no inheritance required.
    """

    def add(self, user: User) -> None: ...
    def get(self, user_id: int) -> User | None: ...
    def all(self) -> list[User]: ...


class InMemoryUserRepository:
    """An adapter: a dict-backed implementation for tests and demos. It does not
    inherit from UserRepository — it satisfies it structurally.
    """

    def __init__(self) -> None:
        self._users: dict[int, User] = {}

    def add(self, user: User) -> None:
        if user.id in self._users:
            raise DuplicateIdError(f"duplicate user id {user.id}")
        self._users[user.id] = user

    def get(self, user_id: int) -> User | None:
        return self._users.get(user_id)

    def all(self) -> list[User]:
        return list(self._users.values())


class UserService:
    """Business logic depending on the UserRepository Protocol — dependency
    injection through ``__init__``. Tests inject the in-memory adapter.

    >>> svc = UserService(InMemoryUserRepository())
    >>> svc.register(2, "Grace")
    >>> svc.register(1, "Ada")
    >>> svc.list_names()
    ['Ada', 'Grace']
    """

    def __init__(self, repo: UserRepository) -> None:
        self._repo = repo

    def register(self, user_id: int, name: str) -> None:
        self._repo.add(User(id=user_id, name=name))

    def list_names(self) -> list[str]:
        return sorted(u.name for u in self._repo.all())
