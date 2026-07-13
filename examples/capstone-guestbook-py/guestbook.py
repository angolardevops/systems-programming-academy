"""Capstone: a guestbook that composes the Part 5 frameworks end to end and then
defeats both classic injection attacks.

The request pipeline is: validate -> store (parameterized) -> render
(autoescaped). Each stage mirrors a Part 5 lesson. The point is the two
adversarial tests: submitting ``'; DROP TABLE comments; --`` and
``<script>alert(1)</script>`` as a real comment, and proving the other rows
survive and the script renders as inert text. Input defence and output defence,
the same "safe by default" principle on both sides.

Dependency-free and I/O-free: an in-memory store, so the whole pipeline is
directly testable.
"""

from __future__ import annotations

from dataclasses import dataclass


def validate_submission(author: str, body: str) -> list[str]:
    """Return every error at once (never bailing on the first) as
    ``"field: message"`` lines."""
    errors: list[str] = []
    author = author.strip()
    body = body.strip()

    if author == "":
        errors.append("author: is required")
    elif len(author) < 2:
        errors.append("author: must be at least 2 characters")
    elif len(author) > 40:
        errors.append("author: must be at most 40 characters")

    if body == "":
        errors.append("body: is required")
    elif len(body) > 500:
        errors.append("body: must be at most 500 characters")

    return errors


def insert_sql(author: str, body: str) -> tuple[str, list[str]]:
    """Build the parameterized INSERT: the SQL carries only ``?`` placeholders;
    the user values travel separately, bound as data, never SQL."""
    return "INSERT INTO comments (author, body) VALUES (?, ?)", [author, body]


def escape_html(s: str) -> str:
    """Escape text (``&`` first, to avoid double-escaping later entities)."""
    return (
        s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
    )


@dataclass
class Comment:
    author: str
    body: str


class Store:
    """An in-memory comment table, standing in for a real database. It binds
    params as row data — modelling what a driver does: bind values, never
    execute them."""

    def __init__(self) -> None:
        self._comments: list[Comment] = []

    def execute_insert(self, sql: str, params: list[str]) -> None:
        """Execute a parameterized INSERT. Accepts only the exact
        two-placeholder comment insert, so a caller cannot smuggle values into
        the SQL string."""
        if sql != "INSERT INTO comments (author, body) VALUES (?, ?)":
            raise ValueError("store only accepts the parameterized comment insert")
        if len(params) != 2:
            raise ValueError(f"expected two bound params, got {len(params)}")
        # Bind params as DATA — whatever is in them is stored verbatim, never SQL.
        self._comments.append(Comment(author=params[0], body=params[1]))

    def all(self) -> list[Comment]:
        return self._comments


def submit(store: Store, author: str, body: str) -> list[str]:
    """Validate, and if clean, store via a parameterized insert. Returns the
    (possibly empty) list of validation errors; on error the store is
    untouched."""
    errors = validate_submission(author, body)
    if not errors:
        sql, params = insert_sql(author.strip(), body.strip())
        store.execute_insert(sql, params)
    return errors


def render_page(store: Store) -> str:
    """Render every stored comment, HTML-escaped, so untrusted content can never
    become markup."""
    lines = ['<ul class="guestbook">']
    for c in store.all():
        lines.append(
            f"  <li><strong>{escape_html(c.author)}</strong>: {escape_html(c.body)}</li>"
        )
    lines.append("</ul>")
    return "\n".join(lines)


if __name__ == "__main__":
    store = Store()
    submit(store, "Ana", "Love this academy!")
    submit(store, "Bruno", "The Rust track is superb.")
    sqli = submit(store, "Mallory", "'; DROP TABLE comments; --")
    xss = submit(store, "Eve", "<script>alert('pwned')</script>")
    print(f"SQLi submission errors: {sqli}  (empty = accepted as data)")
    print(f"XSS submission errors:  {xss}\n")

    bad = submit(store, "X", "")
    print(f"Invalid submission errors: {bad}\n")

    print("Rendered page (all values autoescaped):")
    print(render_page(store))
    print(
        f"\n-- {len(store.all())} comments stored; the table was never dropped; "
        "no live <script>. --"
    )
