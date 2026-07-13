"""Tests: the same seven scenarios as the Rust and Go twins, plus a doctest."""

import doctest
import unittest

import querybuilder
from querybuilder import table


def load_tests(loader, tests, ignore):  # noqa: ARG001 - unittest protocol
    tests.addTests(doctest.DocTestSuite(querybuilder))
    return tests


class QueryBuilderTest(unittest.TestCase):
    def test_select_all_from_table(self) -> None:
        sql, params = table("users").build()
        self.assertEqual(sql, "SELECT * FROM users")
        self.assertEqual(params, [])

    def test_select_specific_columns(self) -> None:
        sql, _ = table("users").select("id", "name").build()
        self.assertEqual(sql, "SELECT id, name FROM users")

    def test_single_where_becomes_placeholder(self) -> None:
        sql, params = table("users").where("age", ">", "18").build()
        self.assertEqual(sql, "SELECT * FROM users WHERE age > ?")
        self.assertEqual(params, ["18"])

    def test_multiple_where_joined_with_and(self) -> None:
        sql, params = (
            table("users").where("age", ">", "18").where("country", "=", "AO").build()
        )
        self.assertEqual(sql, "SELECT * FROM users WHERE age > ? AND country = ?")
        self.assertEqual(params, ["18", "AO"])

    def test_full_query_all_clauses(self) -> None:
        sql, params = (
            table("orders")
            .select("id", "total")
            .where("status", "=", "paid")
            .order_by("total")
            .limit(10)
            .build()
        )
        self.assertEqual(
            sql, "SELECT id, total FROM orders WHERE status = ? ORDER BY total LIMIT 10"
        )
        self.assertEqual(params, ["paid"])

    def test_injection_attempt_is_a_parameter_not_sql(self) -> None:
        evil = "'; DROP TABLE users; --"
        sql, params = table("users").where("name", "=", evil).build()
        self.assertEqual(sql, "SELECT * FROM users WHERE name = ?")
        self.assertEqual(params, [evil])
        self.assertNotIn("DROP", sql)

    def test_order_and_limit_optional_last_wins(self) -> None:
        sql, _ = table("t").limit(5).limit(20).build()
        self.assertEqual(sql, "SELECT * FROM t LIMIT 20")


if __name__ == "__main__":
    unittest.main()
