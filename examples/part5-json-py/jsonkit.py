"""A JSON serialization framework: a value model, a canonical encoder, and a
recursive-descent decoder — the round trip every API speaks.

This is the serialization side of the "encoding for a grammar" theme in Part 5.
The query builder kept values out of SQL, the template engine kept them out of
HTML; here the encoder puts values *into* JSON, escaping them for the JSON
grammar (``"``, ``\\``, control characters) — a different grammar with different
dangerous characters. Reusing an HTML escaper here would be wrong.

Two framework ideas: an ordinary Python value model (``None``/``bool``/``int``/
``str``/``list``/``dict``) and canonical output (object keys in insertion order,
no incidental whitespace) so the encoding is deterministic and byte-identical
across languages. Pure string work — no I/O, and NOT the ``json`` module (we
build it, to understand it).
"""

from __future__ import annotations

# A JSON value is None, bool, int, str, list, or dict (insertion-ordered).
Json = "None | bool | int | str | list | dict"


class JSONError(Exception):
    """Raised on malformed input during decoding."""


def escape_json_string(s: str) -> str:
    """Escape a string for a JSON string literal: the two structural characters
    (``"`` and ``\\``) plus the control characters JSON forbids raw."""
    out = ['"']
    for ch in s:
        if ch == '"':
            out.append('\\"')
        elif ch == "\\":
            out.append("\\\\")
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\r":
            out.append("\\r")
        elif ch == "\t":
            out.append("\\t")
        elif ord(ch) < 0x20:
            out.append(f"\\u{ord(ch):04x}")
        else:
            out.append(ch)
    out.append('"')
    return "".join(out)


def encode(value: object) -> str:
    """Encode a value to canonical JSON: no incidental whitespace, dict keys in
    insertion order. The exact bytes are the cross-language contract.

    >>> encode({"a": [1, True, None]})
    '{"a":[1,true,null]}'
    """
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, str):
        return escape_json_string(value)
    if isinstance(value, list):
        return "[" + ",".join(encode(v) for v in value) + "]"
    if isinstance(value, dict):
        parts = (f"{escape_json_string(str(k))}:{encode(v)}" for k, v in value.items())
        return "{" + ",".join(parts) + "}"
    raise TypeError(f"cannot encode {type(value).__name__}")


def decode(text: str) -> object:
    """Parse JSON text into a Python value, or raise :class:`JSONError`. A
    hand-written recursive-descent parser — the mirror of the encoder."""
    parser = _Parser(text)
    parser.skip_ws()
    value = parser.parse_value()
    parser.skip_ws()
    if parser.pos != len(parser.text):
        raise JSONError(f"trailing characters at position {parser.pos}")
    return value


class _Parser:
    def __init__(self, text: str) -> None:
        self.text = text
        self.pos = 0

    def peek(self) -> str | None:
        return self.text[self.pos] if self.pos < len(self.text) else None

    def skip_ws(self) -> None:
        while self.peek() in (" ", "\t", "\n", "\r"):
            self.pos += 1

    def expect(self, want: str) -> None:
        if self.peek() != want:
            raise JSONError(f"expected {want!r} at position {self.pos}")
        self.pos += 1

    def parse_value(self) -> object:
        self.skip_ws()
        ch = self.peek()
        if ch == "n":
            return self._literal("null", None)
        if ch == "t":
            return self._literal("true", True)
        if ch == "f":
            return self._literal("false", False)
        if ch == '"':
            return self.parse_string()
        if ch == "[":
            return self.parse_array()
        if ch == "{":
            return self.parse_object()
        if ch is not None and (ch == "-" or ch.isdigit()):
            return self.parse_int()
        raise JSONError(f"unexpected input at position {self.pos}")

    def _literal(self, word: str, value: object) -> object:
        for want in word:
            self.expect(want)
        return value

    def parse_int(self) -> int:
        start = self.pos
        if self.peek() == "-":
            self.pos += 1
        while self.peek() is not None and self.peek().isdigit():
            self.pos += 1
        text = self.text[start : self.pos]
        try:
            return int(text)
        except ValueError:
            raise JSONError(f"invalid integer {text!r}") from None

    def parse_string(self) -> str:
        self.expect('"')
        out: list[str] = []
        while True:
            ch = self.peek()
            if ch is None:
                raise JSONError("unterminated string")
            if ch == '"':
                self.pos += 1
                return "".join(out)
            if ch == "\\":
                self.pos += 1
                esc = self.peek()
                simple = {
                    '"': '"',
                    "\\": "\\",
                    "/": "/",
                    "n": "\n",
                    "r": "\r",
                    "t": "\t",
                }
                if esc in simple:
                    out.append(simple[esc])
                elif esc == "u":
                    hexcode = self.text[self.pos + 1 : self.pos + 5]
                    if len(hexcode) != 4:
                        raise JSONError("invalid \\u escape")
                    try:
                        out.append(chr(int(hexcode, 16)))
                    except ValueError:
                        raise JSONError("invalid \\u escape") from None
                    self.pos += 4
                else:
                    raise JSONError("invalid escape")
                self.pos += 1
            else:
                out.append(ch)
                self.pos += 1

    def parse_array(self) -> list:
        self.expect("[")
        items: list = []
        self.skip_ws()
        if self.peek() == "]":
            self.pos += 1
            return items
        while True:
            items.append(self.parse_value())
            self.skip_ws()
            ch = self.peek()
            if ch == ",":
                self.pos += 1
            elif ch == "]":
                self.pos += 1
                return items
            else:
                raise JSONError(f"expected ',' or ']' at position {self.pos}")

    def parse_object(self) -> dict:
        self.expect("{")
        obj: dict = {}
        self.skip_ws()
        if self.peek() == "}":
            self.pos += 1
            return obj
        while True:
            self.skip_ws()
            key = self.parse_string()
            self.skip_ws()
            self.expect(":")
            obj[key] = self.parse_value()
            self.skip_ws()
            ch = self.peek()
            if ch == ",":
                self.pos += 1
            elif ch == "}":
                self.pos += 1
                return obj
            else:
                raise JSONError(f"expected ',' or '}}' at position {self.pos}")


if __name__ == "__main__":
    doc = {
        "user": "Ana",
        "age": 34,
        "admin": True,
        "note": 'has "quotes" and a\nnewline',
        "tags": ["python", None],
    }
    j = encode(doc)
    print("encoded:   ", j)
    print("re-encoded:", encode(decode(j)))
    print("round-trip exact:", encode(decode(j)) == j)
