"""Tests for appconfig.py (stdlib unittest, plus its doctests)."""

import doctest
import unittest

import appconfig
from appconfig import Config, InvalidConfigError, MissingConfigError


class LoadTests(unittest.TestCase):
    def test_loads_with_defaults(self) -> None:
        cfg = Config.from_env({"APP_API_KEY": "s3cret"})
        self.assertEqual((cfg.host, cfg.port, cfg.debug), ("localhost", 8080, False))
        self.assertEqual(cfg.api_key, "s3cret")

    def test_loads_explicit_values(self) -> None:
        cfg = Config.from_env(
            {
                "APP_HOST": "0.0.0.0",
                "APP_PORT": "9000",
                "APP_DEBUG": "true",
                "APP_API_KEY": "k",
            }
        )
        self.assertEqual((cfg.host, cfg.port, cfg.debug), ("0.0.0.0", 9000, True))

    def test_missing_secret_fails_fast(self) -> None:
        with self.assertRaises(MissingConfigError):
            Config.from_env({})

    def test_invalid_port_is_typed_error(self) -> None:
        with self.assertRaises(InvalidConfigError):
            Config.from_env({"APP_PORT": "nope", "APP_API_KEY": "k"})


class RedactionTests(unittest.TestCase):
    def test_repr_excludes_the_secret(self) -> None:
        cfg = Config.from_env({"APP_API_KEY": "hunter2"})
        printed = repr(cfg)
        self.assertNotIn("hunter2", printed)  # the secret never appears
        self.assertIn("host='localhost'", printed)  # non-secrets still shown


def load_tests(loader, tests, ignore):  # noqa: ANN001, ARG001
    tests.addTests(doctest.DocTestSuite(appconfig))
    return tests


if __name__ == "__main__":
    unittest.main()
