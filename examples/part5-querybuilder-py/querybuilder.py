"""A SQL query builder: a fluent, chainable API that compiles method calls
into a parameterized SQL string plus a list of bound values.

Two framework ideas live here. First, the **builder pattern**: each method
returns ``self``, so calls chain into a sentence —
``table("users").where(...).order_by(...)``. Second, and non-negotiable,
**parameterized queries**: user values NEVER get formatted into the SQL text.
They become ``?`` placeholders and travel in a separate params list, so a value
like ``"'; DROP TABLE users; --"`` is data, not code — the single most
important defence against SQL injection.

Compiling to a string means the whole thing is testable with zero database:
we assert the exact SQL and params.
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class _Condition:
    column: str
    op: str
    value: str


@dataclass
class Query:
    """A query under construction. Build with :func:`table`, chain methods,
    then call :meth:`build` to get ``(sql, params)``."""

    _table: str
    _columns: list[str] = field(default_factory=list)
    _conditions: list[_Condition] = field(default_factory=list)
    _order_by: str | None = None
    _limit: int | None = None

    def select(self, *columns: str) -> Query:
        """Restrict the selected columns. With no columns, the query selects *."""
        self._columns = list(columns)
        return self

    def where(self, column: str, op: str, value: str) -> Query:
        """Add a ``column op ?`` condition, binding ``value`` as a parameter.
        Multiple calls are joined with ``AND`` in call order."""
        self._conditions.append(_Condition(column, op, value))
        return self

    def order_by(self, column: str) -> Query:
        """Set the ORDER BY column (last call wins)."""
        self._order_by = column
        return self

    def limit(self, n: int) -> Query:
        """Set the LIMIT (last call wins)."""
        self._limit = n
        return self

    def build(self) -> tuple[str, list[str]]:
        """Compile the query into ``(sql, params)``. The SQL contains only
        ``?`` placeholders where user values go; the values themselves are in
        ``params``, positionally matching the placeholders left to right.

        >>> table("users").where("age", ">", "18").build()
        ('SELECT * FROM users WHERE age > ?', ['18'])
        """
        cols = ", ".join(self._columns) if self._columns else "*"
        sql = f"SELECT {cols} FROM {self._table}"
        params: list[str] = []

        if self._conditions:
            clauses = []
            for c in self._conditions:
                params.append(c.value)
                clauses.append(f"{c.column} {c.op} ?")
            sql += " WHERE " + " AND ".join(clauses)

        if self._order_by is not None:
            sql += f" ORDER BY {self._order_by}"
        if self._limit is not None:
            sql += f" LIMIT {self._limit}"

        return sql, params


def table(name: str) -> Query:
    """Start a query against ``name``, selecting all columns by default."""
    return Query(_table=name)


if __name__ == "__main__":
    sql, params = (
        table("users")
        .select("id", "name", "email")
        .where("age", ">=", "18")
        .where("country", "=", "AO")
        .order_by("name")
        .limit(25)
        .build()
    )
    print(f"SQL:    {sql}")
    print(f"params: {params}\n")

    evil = "'; DROP TABLE users; --"
    sql, params = table("users").where("name", "=", evil).build()
    print("Injection attempt as a value:")
    print(f"SQL:    {sql}")
    print(f"params: {params}")
    print("-> the payload is data, not SQL. The table is safe.")
