"""Tests: the same five scenarios as the Rust and Go twins."""

import unittest

from testkit import AssertionFail, TestKit, assert_eq, assert_true


class TestkitTest(unittest.TestCase):
    def test_assert_eq_passes_and_fails_with_a_message(self) -> None:
        assert_eq(2 + 2, 4)  # no raise
        with self.assertRaises(AssertionFail) as ctx:
            assert_eq(2 + 2, 5)
        self.assertEqual(str(ctx.exception), "expected 5, got 4")

    def test_all_passing_report(self) -> None:
        report = (
            TestKit()
            .test("adds", lambda: assert_eq(2 + 2, 4))
            .test("truthy", lambda: assert_true(1 < 2, "1 should be < 2"))
            .run()
        )
        self.assertTrue(report.ok())
        self.assertEqual(
            report.summary(),
            "running 2 tests\n"
            "test adds ... ok\n"
            "test truthy ... ok\n"
            "\n"
            "test result: ok. 2 passed; 0 failed",
        )

    def test_mixed_report_lists_failures(self) -> None:
        report = (
            TestKit()
            .test("adds", lambda: assert_eq(2 + 2, 4))
            .test("subtracts", lambda: assert_eq(5 - 2, 2))
            .test("multiplies", lambda: assert_eq(2 * 3, 6))
            .run()
        )
        self.assertFalse(report.ok())
        self.assertEqual(report.passed(), 2)
        self.assertEqual(report.failed(), 1)
        self.assertEqual(
            report.summary(),
            "running 3 tests\n"
            "test adds ... ok\n"
            "test subtracts ... FAILED\n"
            "test multiplies ... ok\n"
            "\n"
            "failures:\n"
            "    subtracts: expected 2, got 3\n"
            "\n"
            "test result: FAILED. 2 passed; 1 failed",
        )

    def test_a_raising_test_is_caught_not_fatal(self) -> None:
        def boom() -> None:
            raise ValueError("kaboom")

        report = (
            TestKit().test("boom", boom).test("after", lambda: assert_eq(1, 1)).run()
        )
        self.assertEqual(report.failed(), 1)
        self.assertEqual(report.passed(), 1)
        self.assertIn("boom: raised: ValueError('kaboom')", report.summary())

    def test_empty_kit_reports_zero(self) -> None:
        self.assertEqual(
            TestKit().run().summary(),
            "running 0 tests\n\ntest result: ok. 0 passed; 0 failed",
        )


if __name__ == "__main__":
    unittest.main()
