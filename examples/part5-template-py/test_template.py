"""Tests: the same ten scenarios as the Rust and Go twins, plus doctests."""

import doctest
import unittest

import template
from template import TemplateError, render


def load_tests(loader, tests, ignore):  # noqa: ARG001 - unittest protocol
    tests.addTests(doctest.DocTestSuite(template))
    return tests


class TemplateTest(unittest.TestCase):
    def test_substitutes_a_variable(self) -> None:
        self.assertEqual(render("Hello {{ name }}!", {"name": "Ana"}), "Hello Ana!")

    def test_autoescapes_html_by_default(self) -> None:
        ctx = {"comment": "<script>alert('xss')</script>"}
        self.assertEqual(
            render("<p>{{ comment }}</p>", ctx),
            "<p>&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;</p>",
        )

    def test_raw_filter_opts_out_of_escaping(self) -> None:
        self.assertEqual(
            render("{{ body | raw }}", {"body": "<b>bold</b>"}), "<b>bold</b>"
        )

    def test_ampersand_is_escaped_first(self) -> None:
        self.assertEqual(render("{{ x }}", {"x": "a & b < c"}), "a &amp; b &lt; c")

    def test_filters_compose_then_escape(self) -> None:
        self.assertEqual(
            render("{{ name | trim | upper }}", {"name": "  <ana>  "}), "&lt;ANA&gt;"
        )

    def test_upper_then_raw_skips_escape(self) -> None:
        self.assertEqual(render("{{ tag | upper | raw }}", {"tag": "<b>"}), "<B>")

    def test_unknown_variable_is_an_error(self) -> None:
        with self.assertRaises(TemplateError) as ctx:
            render("{{ missing }}", {})
        self.assertIn("missing", str(ctx.exception))

    def test_unknown_filter_is_an_error(self) -> None:
        with self.assertRaises(TemplateError) as ctx:
            render("{{ x | shout }}", {"x": "hi"})
        self.assertIn("shout", str(ctx.exception))

    def test_unclosed_delimiter_is_an_error(self) -> None:
        with self.assertRaises(TemplateError) as ctx:
            render("start {{ x ", {"x": "hi"})
        self.assertIn("unclosed", str(ctx.exception))

    def test_literal_text_passes_through_untouched(self) -> None:
        self.assertEqual(render("a {{ n }} b {{ n }} c", {"n": "1"}), "a 1 b 1 c")


if __name__ == "__main__":
    unittest.main()
