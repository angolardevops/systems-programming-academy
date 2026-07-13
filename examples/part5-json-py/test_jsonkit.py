"""Tests: the same nine scenarios as the Rust and Go twins, plus doctests."""

import doctest
import unittest

import jsonkit
from jsonkit import JSONError, decode, encode


def load_tests(loader, tests, ignore):  # noqa: ARG001 - unittest protocol
    tests.addTests(doctest.DocTestSuite(jsonkit))
    return tests


class JsonkitTest(unittest.TestCase):
    def test_encodes_primitives(self) -> None:
        self.assertEqual(encode(None), "null")
        self.assertEqual(encode(True), "true")
        self.assertEqual(encode(False), "false")
        self.assertEqual(encode(-42), "-42")
        self.assertEqual(encode("hi"), '"hi"')

    def test_encodes_nested_structure_canonically(self) -> None:
        doc = {"name": "Ana", "age": 30, "tags": ["a", "b"]}
        self.assertEqual(encode(doc), '{"name":"Ana","age":30,"tags":["a","b"]}')

    def test_escapes_json_string_grammar_not_html(self) -> None:
        # Quotes and backslashes get JSON escapes; < and > are NOT touched.
        self.assertEqual(encode('a"b\\c\nd<e>'), '"a\\"b\\\\c\\nd<e>"')

    def test_escapes_control_characters_as_unicode(self) -> None:
        self.assertEqual(encode("\x01\x1f"), '"\\u0001\\u001f"')

    def test_decodes_primitives(self) -> None:
        self.assertIsNone(decode("null"))
        self.assertIs(decode("true"), True)
        self.assertEqual(decode("-42"), -42)
        self.assertEqual(decode('  "hi"  '), "hi")

    def test_decodes_nested_structure(self) -> None:
        parsed = decode('{"a":[1,2],"b":{"c":true}}')
        self.assertEqual(parsed, {"a": [1, 2], "b": {"c": True}})

    def test_round_trips_canonical_json(self) -> None:
        canonical = '{"id":7,"items":["x","y"],"ok":false,"note":null}'
        self.assertEqual(encode(decode(canonical)), canonical)

    def test_round_trips_escaped_string(self) -> None:
        value = 'line1\nline2\t"quoted"'
        self.assertEqual(decode(encode(value)), value)

    def test_malformed_input_is_an_error(self) -> None:
        for bad in ("{", "[1,]", "nul", "true false"):
            with self.assertRaises(JSONError):
                decode(bad)


if __name__ == "__main__":
    unittest.main()
