"""fns — the small language grows functions and closures.

Lesson 2 gave the language memory (an environment). This lesson gives it
abstraction: named functions you define and call. A call creates a fresh scope
for the arguments, and a function captures the environment where it was defined
(a closure); its call scope chains to that captured one, so it sees the variables
in scope where it was written — lexical scoping. Environments form a parent
chain. Values are now integers or functions.
"""

from __future__ import annotations

from dataclasses import dataclass, field


class CalcError(Exception):
    """A lexing, parsing, or evaluation error. Its message is user-facing."""


# ---------------------------------------------------------------- values & env


@dataclass
class Function:
    """Captures its parameters, body, and the environment where it was defined
    (its closure)."""

    params: list[str]
    body: Expr
    env: Scope


# A runtime value is either an int or a Function.
Value = "int | Function"


class Scope:
    """One lexical scope: its own bindings plus an optional parent to fall
    through to. The chain of parents is what makes scoping lexical."""

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
    """Kind ``'n'`` number (``num``), ``'i'`` identifier (``name``), ``'='``
    assign, ``','`` comma, else the operator/parenthesis character."""

    kind: str
    num: int = 0
    name: str = ""


def tokenize(src: str) -> list[Token]:
    """Turn one statement's source into tokens, adding the comma (for argument
    lists) to lesson 2's lexer."""
    tokens: list[Token] = []
    i = 0
    while i < len(src):
        c = src[i]
        if c in " \t\r\n":
            i += 1
        elif c in "=,+-*/()":
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
    """Kind ``'n'`` number, ``'v'`` variable, ``'c'`` call (``name``, ``args``),
    ``'u'`` unary neg (``left``), else a binary operator (``left``, ``right``)."""

    kind: str
    num: int = 0
    name: str = ""
    args: tuple[Expr, ...] = ()
    left: Expr | None = None
    right: Expr | None = None


@dataclass(frozen=True)
class Stmt:
    """Kind ``'f'`` function definition (``name``, ``params``, ``expr``), ``'a'``
    assignment (``name = expr``), or ``'e'`` bare expression."""

    kind: str
    expr: Expr
    name: str = ""
    params: tuple[str, ...] = field(default=())


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
                self.pos += 1  # consume '('
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

    def args(self) -> list[Expr]:  # (expr (',' expr)*)? ')'  — '(' already consumed
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
    """Parse one statement: a function definition (``name(params) = body``), an
    assignment (``name = expr``), or a bare expression."""
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
    i = 2  # skip name and '('
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
    i += 2  # skip ')' and '='
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


def evaluate(e: Expr, env: Scope) -> int | Function:
    """Evaluate an expression to a value in the given environment."""
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
        q = abs(a) // abs(b)  # truncate toward zero
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
    # Evaluate arguments in the CALLER's environment...
    argv = [evaluate(a, env) for a in e.args]
    # ...but run the body in a new scope chained to the function's DEFINING
    # environment (its closure) — this is lexical scoping.
    call = Scope(parent=fn.env)
    for p, v in zip(fn.params, argv):
        call.vars[p] = v
    return evaluate(fn.body, call)


def execute(stmt: Stmt, env: Scope) -> int | Function:
    """Run one statement, returning its value. A definition builds a closure; an
    assignment stores a value; a bare expression is evaluated."""
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
    """Run a whole program: one statement per non-empty line, sharing one root
    environment. Returns one ``"line  =>  value"`` (or error) string per
    statement."""
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
