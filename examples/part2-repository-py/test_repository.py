"""Tests for repository.py (stdlib unittest, plus its doctests)."""

import doctest
import unittest

import repository
from repository import (
    DuplicateIdError,
    InMemoryUserRepository,
    User,
    UserService,
)


def new_service() -> UserService:
    # DI: inject the in-memory adapter — no database, deterministic.
    return UserService(InMemoryUserRepository())


class ServiceTests(unittest.TestCase):
    def test_registers_and_lists_sorted(self) -> None:
        svc = new_service()
        svc.register(2, "Grace")
        svc.register(1, "Ada")
        self.assertEqual(svc.list_names(), ["Ada", "Grace"])

    def test_rejects_duplicate_id(self) -> None:
        svc = new_service()
        svc.register(1, "Ada")
        with self.assertRaises(DuplicateIdError):
            svc.register(1, "Someone")


class RepositoryTests(unittest.TestCase):
    def test_get_and_all(self) -> None:
        repo = InMemoryUserRepository()
        repo.add(User(id=1, name="Ada"))
        self.assertEqual(repo.get(1), User(id=1, name="Ada"))
        self.assertIsNone(repo.get(2))
        self.assertEqual(len(repo.all()), 1)


def load_tests(loader, tests, ignore):  # noqa: ANN001, ARG001
    tests.addTests(doctest.DocTestSuite(repository))
    return tests


if __name__ == "__main__":
    unittest.main()
