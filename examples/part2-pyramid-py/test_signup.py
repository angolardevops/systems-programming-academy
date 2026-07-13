"""Tests for signup.py at all three pyramid levels (stdlib unittest)."""

import doctest
import unittest

import signup
from signup import (
    App,
    DuplicateEmailError,
    InMemoryRepo,
    InvalidEmailError,
    RecordingNotifier,
    SignupService,
    validate_email,
)


class UnitValidatorTests(unittest.TestCase):
    """UNIT level: many tiny tests on the pure validator — instant, no setup."""

    def test_accepts_a_normal_address(self) -> None:
        validate_email("ada@example.com")  # no exception

    def test_rejects_bad_inputs(self) -> None:
        bad = [
            "",
            "ada.example.com",
            "a@b@c.com",
            "ada@nodot",
            "@example.com",
            "a da@x.com",
        ]
        for email in bad:
            with self.subTest(email=email):
                with self.assertRaises(InvalidEmailError):
                    validate_email(email)


class IntegrationServiceTests(unittest.TestCase):
    """INTEGRATION level: the service with real in-memory adapters — proving
    the parts collaborate (stored AND notified), not just work alone."""

    def setUp(self) -> None:
        self.notifier = RecordingNotifier()
        self.service = SignupService(InMemoryRepo(), self.notifier)

    def test_signup_stores_and_notifies(self) -> None:
        self.service.signup("ada@example.com")
        self.assertEqual(self.notifier.sent, ["ada@example.com"])

    def test_duplicate_rejected_and_not_notified_twice(self) -> None:
        self.service.signup("ada@example.com")
        with self.assertRaises(DuplicateEmailError):
            self.service.signup("ada@example.com")
        self.assertEqual(len(self.notifier.sent), 1)  # no second welcome


class EndToEndAppTests(unittest.TestCase):
    """E2E level: one test driving the composition root like a caller would,
    asserting only the user-visible outcome."""

    def test_full_signup_flow_through_the_app(self) -> None:
        app = App()
        self.assertEqual(
            app.signup("ada@example.com"), "Welcome, ada@example.com! Check your inbox."
        )
        self.assertEqual(
            app.signup("ada@example.com"), "ada@example.com is already registered."
        )
        self.assertEqual(app.signup("nope"), "'nope' is not a valid email address.")


def load_tests(loader, tests, ignore):  # noqa: ANN001, ARG001
    tests.addTests(doctest.DocTestSuite(signup))
    return tests


if __name__ == "__main__":
    unittest.main()
