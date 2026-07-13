"""Tests for exporter.py (stdlib unittest)."""

import unittest

from exporter import Registry, demo_registry, format_value, label_string


class RegistryTests(unittest.TestCase):
    def test_counter_accumulates_per_series(self) -> None:
        r = Registry()
        r.inc_counter("hits", "Hits.", {"path": "/"}, 1)
        r.inc_counter("hits", "Hits.", {"path": "/"}, 2)
        r.inc_counter("hits", "Hits.", {"path": "/a"}, 5)
        out = r.render()
        self.assertIn('hits{path="/"} 3\n', out)
        self.assertIn('hits{path="/a"} 5\n', out)

    def test_gauge_overwrites(self) -> None:
        r = Registry()
        r.set_gauge("depth", "Depth.", None, 9)
        r.set_gauge("depth", "Depth.", None, 3)
        self.assertIn("depth 3\n", r.render())


class FormatTests(unittest.TestCase):
    def test_labels_render_sorted(self) -> None:
        self.assertEqual(label_string({"z": "1", "a": "2"}), '{a="2",z="1"}')
        self.assertEqual(label_string(None), "")

    def test_value_formatting(self) -> None:
        self.assertEqual(format_value(42), "42")
        self.assertEqual(format_value(0.5), "0.5")


class DemoTests(unittest.TestCase):
    def test_demo_renders_shared_exposition(self) -> None:
        out = demo_registry().render()
        self.assertTrue(out.startswith("# HELP cpu_load 1-minute load average.\n"))
        self.assertIn('http_requests_total{method="GET",path="/"} 42\n', out)
        self.assertIn("queue_depth 3\n", out)


if __name__ == "__main__":
    unittest.main()
