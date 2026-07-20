"""boollang — the language gains short-circuit boolean logic.

Lesson 4 made the language Turing-complete with conditionals. This lesson adds
``and``, ``or``, and ``not`` — with short-circuit evaluation, the same laziness as
``if``: ``and`` skips its right side when the left is false, ``or`` skips it when
the left is true. Booleans are integers (1 = true, 0 = false; any nonzero is
truthy). The operators are keywords the parser recognizes by name, below
comparison in precedence: or < and < not < comparison.
"""

from __future__ import annotations

from dataclasses import dataclass


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
    """Kind ``'n'`` number, ``'i'`` identifier, ``'='`` assign, ``','`` comma,
    ``'C'`` comparison (operator in ``op``), else the operator/parenthesis
    character."""

    kind: str
    num: int = 0
    name: str = ""
    op: str = ""


def tokenize(src: str) -> list[Token]:
    """Turn one statement's source into tokens, adding comparison operators
    (single- and two-char) to lesson 3's lexer."""
    tokens: list[Token] = []
    i = 0
    while i < len(src):
        c = src[i]
        nxt = src[i + 1] if i + 1 < len(src) else ""
        if c in " \t\r\n":
            i += 1
        elif c in ",+-*/()":
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


# ---------------------------------------------------------------- parser


@dataclass(frozen=True)
class Expr:
    """Kind ``'n'`` number, ``'v'`` variable, ``'c'`` call, ``'u'`` unary neg,
    ``'C'`` comparison (``op``, ``left``, ``right``), ``'I'`` if (``cond``,
    ``then``, ``els``), else a binary operator (``left``, ``right``)."""

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
    kind: str
    expr: Expr
    name: str = ""
    params: tuple[str, ...] = ()


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

    def _is_kw(self, kw: str) -> bool:
        t = self._peek()
        return t is not None and t.kind == "i" and t.name == kw

    def _expect_kw(self, kw: str) -> None:
        if not self._is_kw(kw):
            raise CalcError(f"expected '{kw}'")
        self.pos += 1

    def expr(self) -> Expr:  # 'if' expr 'then' expr 'else' expr | comparison
        if self._is_kw("if"):
            self.pos += 1
            cond = self.expr()
            self._expect_kw("then")
            then = self.expr()
            self._expect_kw("else")
            els = self.expr()
            return Expr("I", cond=cond, then=then, els=els)
        return self.or_expr()

    def or_expr(self) -> Expr:  # and_expr ('or' and_expr)*  — loosest boolean level
        left = self.and_expr()
        while self._is_kw("or"):
            self.pos += 1
            left = Expr("O", left=left, right=self.and_expr())
        return left

    def and_expr(self) -> Expr:  # not_expr ('and' not_expr)*
        left = self.not_expr()
        while self._is_kw("and"):
            self.pos += 1
            left = Expr("A", left=left, right=self.not_expr())
        return left

    def not_expr(self) -> Expr:  # 'not' not_expr | comparison
        if self._is_kw("not"):
            self.pos += 1
            return Expr("N", left=self.not_expr())
        return self.comparison()

    def comparison(self) -> Expr:  # add (cmpop add)?
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

    def factor(
        self,
    ) -> Expr:  # Num | Ident '(' args ')' | Ident | '(' expr ')' | '-' factor
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


def _is_fn_def(tokens: list[Token]) -> bool:
    if len(tokens) < 2 or tokens[0].kind != "i" or tokens[1].kind != "(":
        return False
    depth = 0
    for i in range(1, len(tokens)):
        if tokens[i].kind == "(":
            depth += 1
        elif tokens[i].kind == ")":
            depth -= 1
            if depth == 0:
                return i + 1 < len(tokens) and tokens[i + 1].kind == "="
    return False


def parse_stmt(tokens: list[Token]) -> Stmt:
    if _is_fn_def(tokens):
        return _parse_fn_def(tokens)
    is_assign = len(tokens) >= 2 and tokens[0].kind == "i" and tokens[1].kind == "="
    p = _Parser(tokens)
    if is_assign:
        name = tokens[0].name
        p.pos = 2
        stmt = Stmt("a", expr=p.expr(), name=name)
    else:
        stmt = Stmt("e", expr=p.expr())
    if p.pos != len(p.tokens):
        raise CalcError(f"unexpected trailing input: '{p.tokens[p.pos].kind}'")
    return stmt


def _parse_fn_def(tokens: list[Token]) -> Stmt:
    name = tokens[0].name
    params: list[str] = []
    i = 2
    if tokens[i].kind != ")":
        while True:
            if tokens[i].kind != "i":
                raise CalcError("expected a parameter name")
            params.append(tokens[i].name)
            i += 1
            if tokens[i].kind == ",":
                i += 1
                continue
            if tokens[i].kind == ")":
                break
            raise CalcError("expected ',' or ')' in parameter list")
    i += 2
    p = _Parser(tokens[i:])
    body = p.expr()
    if p.pos != len(p.tokens):
        raise CalcError("unexpected trailing input in function body")
    return Stmt("f", expr=body, name=name, params=tuple(params))


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
        return 1 if _CMP[e.op](a, b) else 0  # booleans are integers
    if e.kind == "N":  # not: negate truthiness
        assert e.left is not None
        return 1 if _as_int(evaluate(e.left, env)) == 0 else 0
    if e.kind in ("A", "O"):
        assert e.left is not None and e.right is not None
        left = _as_int(evaluate(e.left, env))
        # Short-circuit: `and` stops on a false left, `or` stops on a true left.
        if e.kind == "A" and left == 0:
            return 0
        if e.kind == "O" and left != 0:
            return 1
        return 1 if _as_int(evaluate(e.right, env)) != 0 else 0
    if e.kind == "I":
        assert e.cond is not None and e.then is not None and e.els is not None
        # Lazy: evaluate the condition, then ONLY the taken branch.
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


def execute(stmt: Stmt, env: Scope) -> int | Function:
    if stmt.kind == "f":
        f = Function(params=list(stmt.params), body=stmt.expr, env=env)
        env.vars[stmt.name] = f
        return f
    if stmt.kind == "a":
        v = evaluate(stmt.expr, env)
        env.vars[stmt.name] = v
        return v
    return evaluate(stmt.expr, env)


def _show(v: int | Function) -> str:
    return str(v) if isinstance(v, int) else "<fn>"


def run_program(src: str) -> list[str]:
    env = Scope()
    out: list[str] = []
    for line in src.split("\n"):
        trimmed = line.strip()
        if not trimmed:
            continue
        try:
            v = execute(parse_stmt(tokenize(trimmed)), env)
            out.append(f"{trimmed}  =>  {_show(v)}")
        except CalcError as e:
            out.append(f"{trimmed}  =>  error: {e}")
    return out


if __name__ == "__main__":
    import sys

    arg = " ".join(sys.argv[1:]).strip()
    src = arg.replace(";", "\n") if arg else sys.stdin.read()
    for line in run_program(src):
        print(line)
