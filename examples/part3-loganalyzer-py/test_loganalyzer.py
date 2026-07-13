"""Tests for loganalyzer.py (stdlib unittest)."""

import unittest

from loganalyzer import Entry, Stats, parse_line, report

LINE = '203.0.113.9 - - [12/Jul/2026:10:00:00] "GET /api/users HTTP/1.1" 200 512'


class ParseTests(unittest.TestCase):
    def test_parses_valid_line(self) -> None:
        self.assertEqual(parse_line(LINE), Entry(path="/api/users", status=200))

    def test_malformed_lines(self) -> None:
        for bad in [
            "not a log line",
            'x "GET /a HTTP/1.1" banana 1',
            'x "GET /a HTTP/1.1" 999999 1',
        ]:
            with self.subTest(bad=bad):
                self.assertIsNone(parse_line(bad))


class StatsTests(unittest.TestCase):
    def test_aggregates_classes_and_malformed(self) -> None:
        s = Stats()
        s.add_line(LINE)
        s.add_line('x - - [t] "GET /a HTTP/1.1" 404 0')
        s.add_line('x - - [t] "GET /a HTTP/1.1" 500 0')
        s.add_line("garbage")
        self.assertEqual((s.total, s.malformed), (3, 1))
        self.assertEqual(s.by_class, [0, 1, 0, 1, 1])
        self.assertAlmostEqual(s.error_rate(), 66.7, delta=0.1)

    def test_top_paths_sorted_desc_then_alpha(self) -> None:
        s = Stats()
        for p in ["/b", "/a", "/b", "/c", "/a"]:
            s.add_line(f'x - - [t] "GET {p} HTTP/1.1" 200 0')
        self.assertEqual(s.top_paths(2), [("/a", 2), ("/b", 2)])


class ReportTests(unittest.TestCase):
    def test_shared_format(self) -> None:
        out = report(LINE + "\ngarbage\n")
        self.assertTrue(out.startswith("total: 1\n"))
        self.assertIn("malformed: 1\n", out)
        self.assertIn("  /api/users: 1\n", out)


if __name__ == "__main__":
    unittest.main()
