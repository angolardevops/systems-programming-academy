"""Python companion for the Part 2 lesson "The Testing Pyramid". The same
feature is implemented in Rust, Go, and Python, with tests at all three levels:
unit (pure validator), integration (service + real in-memory adapters), and
end-to-end (the App composition root).

Run the tests:

    python3 -m unittest discover -s . -p 'test_*.py'
"""

from __future__ import annotations

from typing import Protocol

# ---------------------------------------------------------------- validation


class InvalidEmailError(Exception):
    """The email failed validation."""


class DuplicateEmailError(Exception):
    """The email is already registered."""


def validate_email(email: str) -> None:
    """Pure validation — the base of the pyramid: no I/O, instant tests.

    >>> validate_email("ada@example.com")  # no exception means valid
    """
    if not email:
        raise InvalidEmailError("email is empty")
    if any(c.isspace() for c in email):
        raise InvalidEmailError("email contains whitespace")
    parts = email.split("@")
    if len(parts) != 2 or not parts[0] or not parts[1] or "." not in parts[1]:
        raise InvalidEmailError("email format is invalid")


# ------------------------------------------------------------------- service


class UserRepo(Protocol):
    """The storage port."""

    def exists(self, email: str) -> bool: ...
    def save(self, email: str) -> None: ...


class Notifier(Protocol):
    """The notification port."""

    def send_welcome(self, email: str) -> None: ...


class InMemoryRepo:
    """The storage adapter used in tests and this demo."""

    def __init__(self) -> None:
        self._emails: set[str] = set()

    def exists(self, email: str) -> bool:
        return email in self._emails

    def save(self, email: str) -> None:
        self._emails.add(email)


class RecordingNotifier:
    """Records welcomes (a real one would talk SMTP)."""

    def __init__(self) -> None:
        self.sent: list[str] = []

    def send_welcome(self, email: str) -> None:
        self.sent.append(email)


class SignupService:
    """The middle of the pyramid: logic coordinating the two ports."""

    def __init__(self, repo: UserRepo, notifier: Notifier) -> None:
        self._repo = repo
        self._notifier = notifier

    def signup(self, email: str) -> None:
        validate_email(email)
        if self._repo.exists(email):
            raise DuplicateEmailError(f"{email} already registered")
        self._repo.save(email)
        self._notifier.send_welcome(email)


# ----------------------------------------------------------------------- app


class App:
    """The top of the pyramid: the composition root a CLI/web layer would call,
    returning the user-visible message.
    """

    def __init__(self) -> None:
        self._service = SignupService(InMemoryRepo(), RecordingNotifier())

    def signup(self, email: str) -> str:
        try:
            self._service.signup(email)
        except DuplicateEmailError:
            return f"{email} is already registered."
        except InvalidEmailError:
            return f"'{email}' is not a valid email address."
        return f"Welcome, {email}! Check your inbox."
