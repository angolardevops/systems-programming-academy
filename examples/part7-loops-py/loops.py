"""loops — the interpreter becomes imperative: statements, print, and while.

Until now every line was one expression that echoed its value. This lesson makes
the language *do* things: a program is a sequence of statements separated by
``;``, grouped with ``{ }`` blocks; ``print`` emits output; and ``while cond do
body`` loops. The printed output is what the three implementations verify
byte-for-byte.
"""

from __future__ import annotations

from dataclasses import dataclass, field


class CalcError(Exception):
    """A lexing, parsing, or evaluation error. Its message is user-facing."""


# ---------------------------------------------------------------- values & env


@dataclass
class Function:
    params: list[str]
    body: Expr
    env: Scope


class Scope:
    def __init__(self, parent: Scope | None = None) -> None:
        self.vars: dict[str, int | Function] = {}
        self.parent = parent

    def lookup(self, name: str) -> int | Function | None:
        scope: Scope | None = self
        while scope is not None:
            if name in scope.vars:
                return scope.vars[name]
            scope = scope.parent
        return None


# ---------------------------------------------------------------- lexer


@dataclass(frozen=True)
class Token:
    """Kind ``'n'`` number, ``'i'`` identifier, ``'='`` assign, ``'C'`` comparison
    (``op``), else the operator/paren/brace/semicolon character."""

    kind: str
    num: int = 0
    name: str = ""
    op: str = ""


def tokenize(src: str) -> list[Token]:
    """Tokenize the whole program. Newlines are whitespace; statements are
    separated by ``;`` and grouped by ``{ }``."""
    tokens: list[Token] = []
    i = 0
    while i < len(src):
        c = src[i]
        nxt = src[i + 1] if i + 1 < len(src) else ""
        if c in " \t\r\n":
            i += 1
        elif c in ";{},+-*/()":
            tokens.append(Token(c))
            i += 1
        elif c == "=" and nxt == "=":
            tokens.append(Token("C", op="=="))
            i += 2
        elif c == "=":
            tokens.append(Token("="))
            i += 1
        elif c in "!<>" and nxt == "=":
            tokens.append(Token("C", op=c + "="))
            i += 2
        elif c in "<>":
            tokens.append(Token("C", op=c))
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


# ---------------------------------------------------------------- ast


@dataclass(frozen=True)
class Expr:
    """Kind ``'n'`` number, ``'v'`` variable, ``'c'`` call, ``'u'`` unary neg,
    ``'C'`` comparison, ``'A'``/``'O'``/``'N'`` and/or/not, ``'I'`` if, else a
    binary operator."""

    kind: str
    num: int = 0
    name: str = ""
    op: str = ""
    args: tuple[Expr, ...] = ()
    left: Expr | None = None
    right: Expr | None = None
    cond: Expr | None = None
    then: Expr | None = None
    els: Expr | None = None


@dataclass(frozen=True)
class Stmt:
    """Kind ``'f'`` fndef, ``'a'`` assign, ``'p'`` print, ``'w'`` while, ``'b'``
    block, ``'e'`` expression."""

    kind: str
    expr: Expr | None = None
    name: str = ""
    params: tuple[str, ...] = ()
    body: Stmt | None = None
    stmts: tuple[Stmt, ...] = field(default=())


# ---------------------------------------------------------------- parser


class _Parser:
    def __init__(self, tokens: list[Token]) -> None:
        self.tokens = tokens
        self.pos = 0

    def _peek(self) -> Token | None:
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    def _at(self, k: int) -> Token | None:
        j = self.pos + k
        return self.tokens[j] if j < len(self.tokens) else None

    def _next(self) -> Token | None:
        t = self._peek()
        if t is not None:
            self.pos += 1
        return t

    def _is_kw(self, kw: str) -> bool:
        t = self._peek()
        return t is not None and t.kind == "i" and t.name == kw

    def _expect_kw(self, kw: str) -> None:
        if not self._is_kw(kw):
            raise CalcError(f"expected '{kw}'")
        self.pos += 1

    def _kind_is(self, k: str) -> bool:
        t = self._peek()
        return t is not None and t.kind == k

    # program := stmt (';' stmt)* ';'?
    def program(self) -> list[Stmt]:
        stmts: list[Stmt] = []
        while True:
            while self._kind_is(";"):
                self.pos += 1
            if self._peek() is None:
                return stmts
            stmts.append(self.stmt())
            t = self._peek()
            if t is not None and t.kind != ";":
                raise CalcError("expected ';' between statements")

    def stmt(self) -> Stmt:
        if self._is_kw("print"):
            self.pos += 1
            return Stmt("p", expr=self.expr())
        if self._is_kw("while"):
            self.pos += 1
            cond = self.expr()
            self._expect_kw("do")
            return Stmt("w", expr=cond, body=self.stmt())
        if self._kind_is("{"):
            self.pos += 1
            stmts: list[Stmt] = []
            while True:
                while self._kind_is(";"):
                    self.pos += 1
                if self._kind_is("}"):
                    self.pos += 1
                    break
                if self._peek() is None:
                    raise CalcError("unclosed '{'")
                stmts.append(self.stmt())
                t = self._peek()
                if t is not None and t.kind not in ";}":
                    raise CalcError("expected ';' or '}'")
            return Stmt("b", stmts=tuple(stmts))
        if self._is_fn_def_here():
            return self._fn_def()
        t = self._peek()
        if t is not None and t.kind == "i":
            nt = self._at(1)
            if nt is not None and nt.kind == "=":
                self.pos += 2  # name and '='
                return Stmt("a", expr=self.expr(), name=t.name)
        return Stmt("e", expr=self.expr())

    def _is_fn_def_here(self) -> bool:
        t, nt = self._peek(), self._at(1)
        if t is None or t.kind != "i" or nt is None or nt.kind != "(":
            return False
        depth, k = 0, 1
        while (tk := self._at(k)) is not None:
            if tk.kind == "(":
                depth += 1
            elif tk.kind == ")":
                depth -= 1
                if depth == 0:
                    a = self._at(k + 1)
                    return a is not None and a.kind == "="
            k += 1
        return False

    def _fn_def(self) -> Stmt:
        name = self._next().name  # type: ignore[union-attr]
        self.pos += 1  # '('
        params: list[str] = []
        if not self._kind_is(")"):
            while True:
                t = self._next()
                if t is None or t.kind != "i":
                    raise CalcError("expected a parameter name")
                params.append(t.name)
                sep = self._next()
                if sep is not None and sep.kind == ",":
                    continue
                if sep is not None and sep.kind == ")":
                    break
                raise CalcError("expected ',' or ')' in parameter list")
        else:
            self.pos += 1  # ')'
        eq = self._next()
        if eq is None or eq.kind != "=":
            raise CalcError("expected '='")
        return Stmt("f", expr=self.expr(), name=name, params=tuple(params))

    # ---- expressions (same as the boolean-logic lesson) ----

    def expr(self) -> Expr:
        if self._is_kw("if"):
            self.pos += 1
            cond = self.expr()
            self._expect_kw("then")
            then = self.expr()
            self._expect_kw("else")
            els = self.expr()
            return Expr("I", cond=cond, then=then, els=els)
        return self.or_expr()

    def or_expr(self) -> Expr:
        left = self.and_expr()
        while self._is_kw("or"):
            self.pos += 1
            left = Expr("O", left=left, right=self.and_expr())
        return left

    def and_expr(self) -> Expr:
        left = self.not_expr()
        while self._is_kw("and"):
            self.pos += 1
            left = Expr("A", left=left, right=self.not_expr())
        return left

    def not_expr(self) -> Expr:
        if self._is_kw("not"):
            self.pos += 1
            return Expr("N", left=self.not_expr())
        return self.comparison()

    def comparison(self) -> Expr:
        left = self.add()
        t = self._peek()
        if t is not None and t.kind == "C":
            self.pos += 1
            return Expr("C", op=t.op, left=left, right=self.add())
        return left

    def add(self) -> Expr:
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

    def factor(self) -> Expr:
        t = self._next()
        if t is None:
            raise CalcError("unexpected end of input")
        if t.kind == "n":
            return Expr("n", num=t.num)
        if t.kind == "i":
            nxt = self._peek()
            if nxt is not None and nxt.kind == "(":
                self.pos += 1
                return Expr("c", name=t.name, args=tuple(self.args()))
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

    def args(self) -> list[Expr]:
        out: list[Expr] = []
        t = self._peek()
        if t is not None and t.kind == ")":
            self.pos += 1
            return out
        while True:
            out.append(self.expr())
            t = self._next()
            if t is None or t.kind not in ",)":
                raise CalcError("expected ',' or ')' in argument list")
            if t.kind == ")":
                return out


def parse(tokens: list[Token]) -> list[Stmt]:
    """Parse a whole program into a list of top-level statements."""
    return _Parser(tokens).program()


# ---------------------------------------------------------------- evaluator


def _as_int(v: int | Function) -> int:
    if isinstance(v, int):
        return v
    raise CalcError("cannot do arithmetic on a function")


_CMP = {
    "<": lambda a, b: a < b,
    "<=": lambda a, b: a <= b,
    ">": lambda a, b: a > b,
    ">=": lambda a, b: a >= b,
    "==": lambda a, b: a == b,
    "!=": lambda a, b: a != b,
}


def evaluate(e: Expr, env: Scope) -> int | Function:
    if e.kind == "n":
        return e.num
    if e.kind == "v":
        v = env.lookup(e.name)
        if v is None:
            raise CalcError(f"undefined variable '{e.name}'")
        return v
    if e.kind == "u":
        assert e.left is not None
        return -_as_int(evaluate(e.left, env))
    if e.kind == "C":
        assert e.left is not None and e.right is not None
        a, b = _as_int(evaluate(e.left, env)), _as_int(evaluate(e.right, env))
        return 1 if _CMP[e.op](a, b) else 0
    if e.kind == "N":
        assert e.left is not None
        return 1 if _as_int(evaluate(e.left, env)) == 0 else 0
    if e.kind in ("A", "O"):
        assert e.left is not None and e.right is not None
        left = _as_int(evaluate(e.left, env))
        if e.kind == "A" and left == 0:
            return 0
        if e.kind == "O" and left != 0:
            return 1
        return 1 if _as_int(evaluate(e.right, env)) != 0 else 0
    if e.kind == "I":
        assert e.cond is not None and e.then is not None and e.els is not None
        if _as_int(evaluate(e.cond, env)) != 0:
            return evaluate(e.then, env)
        return evaluate(e.els, env)
    if e.kind == "c":
        return _eval_call(e, env)
    assert e.left is not None and e.right is not None
    a, b = _as_int(evaluate(e.left, env)), _as_int(evaluate(e.right, env))
    if e.kind == "+":
        return a + b
    if e.kind == "-":
        return a - b
    if e.kind == "*":
        return a * b
    if e.kind == "/":
        if b == 0:
            raise CalcError("division by zero")
        q = abs(a) // abs(b)
        return -q if (a < 0) != (b < 0) else q
    raise CalcError("unreachable")  # pragma: no cover


def _eval_call(e: Expr, env: Scope) -> int | Function:
    fn = env.lookup(e.name)
    if fn is None:
        raise CalcError(f"undefined function '{e.name}'")
    if not isinstance(fn, Function):
        raise CalcError(f"'{e.name}' is not a function")
    if len(e.args) != len(fn.params):
        raise CalcError(
            f"'{e.name}' expects {len(fn.params)} argument(s), got {len(e.args)}"
        )
    argv = [evaluate(a, env) for a in e.args]
    call = Scope(parent=fn.env)
    for p, v in zip(fn.params, argv):
        call.vars[p] = v
    return evaluate(fn.body, call)


def _show(v: int | Function) -> str:
    return str(v) if isinstance(v, int) else "<fn>"


def execute(stmt: Stmt, env: Scope, out: list[str]) -> None:
    """Execute one statement, appending any ``print`` output to ``out``."""
    if stmt.kind == "f":
        assert stmt.expr is not None
        env.vars[stmt.name] = Function(
            params=list(stmt.params), body=stmt.expr, env=env
        )
    elif stmt.kind == "a":
        assert stmt.expr is not None
        env.vars[stmt.name] = evaluate(stmt.expr, env)
    elif stmt.kind == "p":
        assert stmt.expr is not None
        out.append(_show(evaluate(stmt.expr, env)))
    elif stmt.kind == "w":
        assert stmt.expr is not None and stmt.body is not None
        while _as_int(evaluate(stmt.expr, env)) != 0:
            execute(stmt.body, env, out)
    elif stmt.kind == "b":
        for s in stmt.stmts:
            execute(s, env, out)
    else:  # 'e'
        assert stmt.expr is not None
        evaluate(stmt.expr, env)  # for effect; value discarded


def run_program(src: str) -> list[str]:
    """Run a whole program, returning everything it printed (one string per
    ``print``). Any error yields a single ``"error: ..."`` line."""
    env = Scope()
    out: list[str] = []
    try:
        for stmt in parse(tokenize(src)):
            execute(stmt, env, out)
    except CalcError as e:
        out.append(f"error: {e}")
    return out


if __name__ == "__main__":
    import sys

    arg = " ".join(sys.argv[1:]).strip()
    src = arg if arg else sys.stdin.read()
    for line in run_program(src):
        print(line)
