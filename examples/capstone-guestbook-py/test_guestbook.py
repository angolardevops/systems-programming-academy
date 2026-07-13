"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from guestbook import Store, insert_sql, render_page, submit


class GuestbookTest(unittest.TestCase):
    def test_valid_submission_is_stored(self) -> None:
        store = Store()
        errors = submit(store, "Ana", "Hello, world!")
        self.assertEqual(errors, [])
        self.assertEqual(len(store.all()), 1)
        self.assertEqual(store.all()[0].author, "Ana")

    def test_invalid_submission_accumulates_errors_and_stores_nothing(self) -> None:
        store = Store()
        errors = submit(store, "A", "   ")
        self.assertEqual(
            errors,
            ["author: must be at least 2 characters", "body: is required"],
        )
        self.assertEqual(store.all(), [])

    def test_insert_is_parameterized_never_interpolated(self) -> None:
        evil = "'; DROP TABLE comments; --"
        sql, params = insert_sql("Ana", evil)
        self.assertEqual(sql, "INSERT INTO comments (author, body) VALUES (?, ?)")
        self.assertEqual(params, ["Ana", evil])
        self.assertNotIn("DROP", sql)

    def test_sql_injection_payload_is_stored_as_inert_data_table_survives(self) -> None:
        store = Store()
        submit(store, "Alice", "first comment")
        errors = submit(store, "Mallory", "'; DROP TABLE comments; --")
        self.assertEqual(errors, [])
        self.assertEqual(len(store.all()), 2)
        self.assertEqual(store.all()[0].body, "first comment")
        self.assertEqual(store.all()[1].body, "'; DROP TABLE comments; --")

    def test_xss_payload_renders_as_inert_text(self) -> None:
        store = Store()
        submit(store, "Mallory", "<script>alert(document.cookie)</script>")
        page = render_page(store)
        self.assertIn("&lt;script&gt;alert(document.cookie)&lt;/script&gt;", page)
        self.assertNotIn("<script>", page)

    def test_end_to_end_both_attacks_defeated(self) -> None:
        store = Store()
        submit(store, "Ana", "Nice site!")
        submit(store, "Mallory", "'; DROP TABLE comments; --")
        submit(store, "Eve", "<script>steal()</script>")

        self.assertEqual(len(store.all()), 3)
        page = render_page(store)
        self.assertIn("&#39;; DROP TABLE comments; --", page)
        self.assertIn("&lt;script&gt;steal()&lt;/script&gt;", page)
        self.assertNotIn("<script>", page)


if __name__ == "__main__":
    unittest.main()
