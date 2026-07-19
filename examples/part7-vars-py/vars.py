"""vars — the expression interpreter grown into a tiny language with memory.

Lesson 1 evaluated one expression. Real languages remember things: you name a
value and use it later. That is a variable, and the thing holding the names is an
*environment* — a name->value map threaded through evaluation. Add that and the
calculator becomes a language. Still integer arithmetic, so results stay exact
and byte-identical across the three languages. The environment introduced here is
the seed of scopes and closures in the lessons to come.
"""

from __future__ import annotations

from dataclasses import dataclass

Env = dict[str, int]


class CalcError(Exception):
    """A lexing, parsing, or evaluation error. Its message is user-facing."""


# ---------------------------------------------------------------- lexer


@dataclass(frozen=True)
class Token:
    """Kind ``'n'`` number (in ``num``), ``'i'`` identifier (in ``name``),
    ``'='`` assign, else the operator/parenthesis character."""

    kind: str
    num: int = 0
    name: str = ""


def tokenize(src: str) -> list[Token]:
    """Turn one statement's source into tokens, adding identifiers and the ``=``
    assignment token to the lexer from lesson 1."""
    tokens: list[Token] = []
    i = 0
    while i < len(src):
        c = src[i]
        if c in " \t\r\n":
            i += 1
        elif c in "=+-*/()":
            tokens.append(Token(c))
            i += 1
        elif c.isdigit():
            start = i
            while i < len(src) and src[i].isdigit():
                i += 1
            tokens.append(Token("n", num=int(src[start:i])))
        elif c.isalpha() or c == "_":
            start = i
            while i < len(src) and (src[i].isalnum() or src[i] == "_"):
                i += 1
            tokens.append(Token("i", name=src[start:i]))
        else:
            raise CalcError(f"unexpected character '{c}'")
    return tokens


# ---------------------------------------------------------------- parser


@dataclass(frozen=True)
class Expr:
    """Kind ``'n'`` number (``num``), ``'v'`` variable (``name``), ``'u'`` unary
    neg (``left``), else a binary operator (``left``, ``right``)."""

    kind: str
    num: int = 0
    name: str = ""
    left: Expr | None = None
    right: Expr | None = None


@dataclass(frozen=True)
class Stmt:
    """Kind ``'a'`` assignment (``name = expr``) or ``'e'`` bare expression."""

    kind: str
    expr: Expr
    name: str = ""


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

    def expr(self) -> Expr:
        left = self.term()
        while (t := self._peek()) is not None and t.kind in "+-":
            self.pos += 1
            left = Expr(t.kind, left=left, right=self.term())
        return left

    def term(self) -> Expr:
        left = self.factor()
        while (t := self._peek()) is not None and t.kind in "*/":
            self.pos += 1
            left = Expr(t.kind, left=left, right=self.factor())
        return left

    def factor(self) -> Expr:  # factor := Num | Ident | '(' expr ')' | '-' factor
        t = self._next()
        if t is None:
            raise CalcError("unexpected end of input")
        if t.kind == "n":
            return Expr("n", num=t.num)
        if t.kind == "i":
            return Expr("v", name=t.name)
        if t.kind == "-":
            return Expr("u", left=self.factor())
        if t.kind == "(":
            inner = self.expr()
            close = self._next()
            if close is None or close.kind != ")":
                raise CalcError("expected ')'")
            return inner
        raise CalcError(f"unexpected token '{t.kind}'")


def parse_stmt(tokens: list[Token]) -> Stmt:
    """Parse one statement: ``name = expr`` is an assignment (identifier followed
    by ``=``); anything else is a bare expression."""
    is_assign = len(tokens) >= 2 and tokens[0].kind == "i" and tokens[1].kind == "="
    p = _Parser(tokens)
    if is_assign:
        name = tokens[0].name
        p.pos = 2  # skip identifier and '='
        stmt = Stmt("a", expr=p.expr(), name=name)
    else:
        stmt = Stmt("e", expr=p.expr())
    if p.pos != len(p.tokens):
        raise CalcError(f"unexpected trailing input: '{p.tokens[p.pos].kind}'")
    return stmt


# ---------------------------------------------------------------- evaluator


def evaluate(e: Expr, env: Env) -> int:
    """Evaluate an expression against the environment; an unbound variable is an
    error."""
    if e.kind == "n":
        return e.num
    if e.kind == "v":
        if e.name not in env:
            raise CalcError(f"undefined variable '{e.name}'")
        return env[e.name]
    if e.kind == "u":
        assert e.left is not None
        return -evaluate(e.left, env)
    assert e.left is not None and e.right is not None
    a, b = evaluate(e.left, env), evaluate(e.right, env)
    if e.kind == "+":
        return a + b
    if e.kind == "-":
        return a - b
    if e.kind == "*":
        return a * b
    if e.kind == "/":
        if b == 0:
            raise CalcError("division by zero")
        q = abs(a) // abs(b)  # truncate toward zero, not Python's floor //
        return -q if (a < 0) != (b < 0) else q
    raise CalcError("unreachable")  # pragma: no cover


def execute(stmt: Stmt, env: Env) -> int:
    """Run one statement, returning its value. An assignment stores the value and
    evaluates to it; a bare expression just evaluates."""
    if stmt.kind == "a":
        v = evaluate(stmt.expr, env)
        env[stmt.name] = v
        return v
    return evaluate(stmt.expr, env)


def run_program(src: str) -> list[str]:
    """Run a whole program: one statement per non-empty line, sharing a single
    environment so state persists. Returns one ``"line  =>  value"`` (or error)
    string per statement."""
    env: Env = {}
    out: list[str] = []
    for line in src.split("\n"):
        trimmed = line.strip()
        if not trimmed:
            continue
        try:
            v = execute(parse_stmt(tokenize(trimmed)), env)
            out.append(f"{trimmed}  =>  {v}")
        except CalcError as e:
            out.append(f"{trimmed}  =>  error: {e}")
    return out


if __name__ == "__main__":
    import sys

    arg = " ".join(sys.argv[1:]).strip()
    src = arg.replace(";", "\n") if arg else sys.stdin.read()
    for line in run_program(src):
        print(line)
