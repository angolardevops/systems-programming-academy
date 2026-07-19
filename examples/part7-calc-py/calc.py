"""calc — a tiny interpreter for integer arithmetic, in three stages.

Every interpreter is the same pipeline::

    source text  --lexer-->  tokens  --parser-->  a syntax tree  --eval-->  a value

The language is integer arithmetic with ``+ - * /``, parentheses, and unary
minus. ``/`` is integer division truncated toward zero. Integers keep every
result exact and identical across languages — the lexer, parser, and evaluator
are the lesson, not floating-point formatting.
"""

from __future__ import annotations

from dataclasses import dataclass


class CalcError(Exception):
    """A lexing, parsing, or evaluation error. Its message is user-facing."""


# ---------------------------------------------------------------- lexer


@dataclass(frozen=True)
class Token:
    """The smallest meaningful piece of source. ``kind`` is ``'n'`` for a number
    (value in ``num``), otherwise the operator/parenthesis character."""

    kind: str
    num: int = 0


def tokenize(src: str) -> list[Token]:
    """Turn source text into a flat list of tokens. Whitespace is skipped; any
    character that isn't a digit, operator, or parenthesis is an error."""
    tokens: list[Token] = []
    i = 0
    while i < len(src):
        c = src[i]
        if c in " \t\n\r":
            i += 1
        elif c in "+-*/()":
            tokens.append(Token(c))
            i += 1
        elif c.isdigit():
            start = i
            while i < len(src) and src[i].isdigit():
                i += 1
            tokens.append(Token("n", int(src[start:i])))
        else:
            raise CalcError(f"unexpected character '{c}'")
    return tokens


# ---------------------------------------------------------------- parser


@dataclass(frozen=True)
class Expr:
    """A node in the abstract syntax tree. ``kind`` is ``'n'`` (number, in
    ``num``), ``'u'`` (unary negation, ``left`` set), else a binary operator
    (``left`` and ``right`` set)."""

    kind: str
    num: int = 0
    left: Expr | None = None
    right: Expr | None = None


class _Parser:
    def __init__(self, tokens: list[Token]) -> None:
        self.tokens = tokens
        self.pos = 0

    def _peek(self) -> Token | None:
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    def _next(self) -> Token | None:
        t = self._peek()
        if t is not None:
            self.pos += 1
        return t

    def expr(self) -> Expr:  # expr := term (('+' | '-') term)*
        left = self.term()
        while (t := self._peek()) is not None and t.kind in "+-":
            self.pos += 1
            left = Expr(t.kind, left=left, right=self.term())
        return left

    def term(self) -> Expr:  # term := factor (('*' | '/') factor)*
        left = self.factor()
        while (t := self._peek()) is not None and t.kind in "*/":
            self.pos += 1
            left = Expr(t.kind, left=left, right=self.factor())
        return left

    def factor(self) -> Expr:  # factor := Num | '(' expr ')' | '-' factor
        t = self._next()
        if t is None:
            raise CalcError("unexpected end of input")
        if t.kind == "n":
            return Expr("n", num=t.num)
        if t.kind == "-":
            return Expr("u", left=self.factor())
        if t.kind == "(":
            inner = self.expr()
            close = self._next()
            if close is None or close.kind != ")":
                raise CalcError("expected ')'")
            return inner
        raise CalcError(f"unexpected token '{t.kind}'")


def parse(tokens: list[Token]) -> Expr:
    """Turn a token list into a syntax tree, enforcing precedence (``*`` and
    ``/`` bind tighter than ``+`` and ``-``) and rejecting trailing garbage."""
    p = _Parser(tokens)
    e = p.expr()
    if p.pos != len(p.tokens):
        raise CalcError(f"unexpected trailing input: '{p.tokens[p.pos].kind}'")
    return e


# ---------------------------------------------------------------- evaluator


def to_sexp(e: Expr) -> str:
    """Render a syntax tree as a fully-parenthesised S-expression, so precedence
    is visible: ``1 + 2 * 3`` becomes ``(+ 1 (* 2 3))``."""
    if e.kind == "n":
        return str(e.num)
    if e.kind == "u":
        assert e.left is not None
        return f"(neg {to_sexp(e.left)})"
    assert e.left is not None and e.right is not None
    return f"({e.kind} {to_sexp(e.left)} {to_sexp(e.right)})"


def _trunc_div(a: int, b: int) -> int:
    """Integer division truncated toward zero (not Python's floor //)."""
    q = abs(a) // abs(b)
    return -q if (a < 0) != (b < 0) else q


def evaluate(e: Expr) -> int:
    """Walk the tree and compute its value. Division truncates toward zero;
    dividing by zero is an error."""
    if e.kind == "n":
        return e.num
    if e.kind == "u":
        assert e.left is not None
        return -evaluate(e.left)
    assert e.left is not None and e.right is not None
    a, b = evaluate(e.left), evaluate(e.right)
    if e.kind == "+":
        return a + b
    if e.kind == "-":
        return a - b
    if e.kind == "*":
        return a * b
    if e.kind == "/":
        if b == 0:
            raise CalcError("division by zero")
        return _trunc_div(a, b)
    raise CalcError("unreachable")  # pragma: no cover


def run(src: str) -> tuple[str, int]:
    """The whole pipeline: source text to ``(s-expression, value)``. Raises
    ``CalcError`` on the first problem."""
    ast = parse(tokenize(src))
    return to_sexp(ast), evaluate(ast)


if __name__ == "__main__":
    import sys

    def _report(src: str) -> None:
        try:
            sexp, value = run(src)
            print(f"{src}  =>  {sexp}  =>  {value}")
        except CalcError as e:
            print(f"{src}  =>  error: {e}")

    arg = " ".join(sys.argv[1:]).strip()
    if arg:
        _report(arg)
    else:
        for line in sys.stdin:
            if line.strip():
                _report(line.rstrip("\n"))
