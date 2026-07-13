"""The capstone guestbook, now a real running web app: asyncio HTTP server +
real SQLite storage.

This promotes the in-memory capstone to a running application. The security
defences are unchanged — parameterized inserts, autoescaped rendering — but now
they run over real TCP sockets against a real SQLite database, so the
SQL-injection proof is against an actual SQL engine, not a stand-in.

Routes:
* ``GET /``        — render the guestbook page.
* ``POST /comment``— parse the form, validate, insert (parameterized). On
  success, 303-redirect to ``/`` (Post/Redirect/Get); on validation failure,
  400 with the error list.

Python needs no external dependency: ``asyncio`` and ``sqlite3`` are both in the
standard library.
"""

from __future__ import annotations

import asyncio
import sqlite3
from urllib.parse import parse_qsl

# ---------------------------------------------------------------------------
# Domain logic (validation, escaping, rendering) — same as the capstone.
# ---------------------------------------------------------------------------


def validate_submission(author: str, body: str) -> list[str]:
    """Return every error at once as ``"field: message"`` lines."""
    errors: list[str] = []
    author, body = author.strip(), body.strip()
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


def escape_html(s: str) -> str:
    """Escape text (``&`` first, to avoid double-escaping later entities)."""
    return (
        s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
    )


# ---------------------------------------------------------------------------
# Real SQLite storage. The insert is parameterized: sqlite3 binds the value,
# so a '; DROP TABLE ... payload is stored as data, never executed.
# ---------------------------------------------------------------------------


def open_db(path: str) -> sqlite3.Connection:
    """Open (or create) the database and ensure the comments table exists."""
    conn = sqlite3.connect(path, check_same_thread=False)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS comments ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT, author TEXT NOT NULL, body TEXT NOT NULL)"
    )
    conn.commit()
    return conn


def insert_comment(conn: sqlite3.Connection, author: str, body: str) -> None:
    """Parameterized INSERT — the values are bound, never spliced into SQL."""
    conn.execute(
        "INSERT INTO comments (author, body) VALUES (?, ?)",
        (author.strip(), body.strip()),
    )
    conn.commit()


def all_comments(conn: sqlite3.Connection) -> list[tuple[str, str]]:
    """Every comment, oldest first, as (author, body) pairs."""
    rows = conn.execute("SELECT author, body FROM comments ORDER BY id").fetchall()
    return [(r[0], r[1]) for r in rows]


def render_page(conn: sqlite3.Connection) -> str:
    """Render the full HTML page, every value autoescaped."""
    items = "\n".join(
        f"  <li><strong>{escape_html(a)}</strong>: {escape_html(b)}</li>"
        for a, b in all_comments(conn)
    )
    return (
        "<!doctype html>\n<html><head><title>Guestbook</title></head><body>\n"
        "<h1>Guestbook</h1>\n"
        f'<ul class="guestbook">\n{items}\n</ul>\n'
        '<form method="post" action="/comment">\n'
        '  <input name="author" placeholder="name">\n'
        '  <input name="body" placeholder="message">\n'
        "  <button>Post</button>\n"
        "</form>\n</body></html>"
    )


# ---------------------------------------------------------------------------
# HTTP layer: parse the request head + form body, route, build the response.
# ---------------------------------------------------------------------------


def parse_form(body: str) -> dict[str, str]:
    """Parse an application/x-www-form-urlencoded body into a dict."""
    return dict(parse_qsl(body, keep_blank_values=True))


def build_response(
    status: int, ctype: str, body: str, extra_headers: str = ""
) -> bytes:
    reason = {200: "OK", 303: "See Other", 400: "Bad Request", 404: "Not Found"}.get(
        status, "OK"
    )
    encoded = body.encode()
    head = (
        f"HTTP/1.0 {status} {reason}\r\n"
        f"Content-Type: {ctype}\r\n"
        f"Content-Length: {len(encoded)}\r\n"
        f"{extra_headers}"
        "Connection: close\r\n\r\n"
    )
    return head.encode() + encoded


def handle_request(
    conn: sqlite3.Connection, method: str, path: str, body: str
) -> bytes:
    """Route one request to a response. GET / renders; POST /comment submits."""
    if method == "GET" and path == "/":
        return build_response(200, "text/html", render_page(conn))
    if method == "POST" and path == "/comment":
        form = parse_form(body)
        author, body_field = form.get("author", ""), form.get("body", "")
        errors = validate_submission(author, body_field)
        if errors:
            page = (
                "<h1>Errors</h1>\n<ul>\n"
                + "\n".join(f"  <li>{escape_html(e)}</li>" for e in errors)
                + "\n</ul>"
            )
            return build_response(400, "text/html", page)
        insert_comment(conn, author, body_field)
        return build_response(303, "text/plain", "", extra_headers="Location: /\r\n")
    return build_response(404, "text/plain", "404 Not Found\n")


async def _handle_connection(
    reader: asyncio.StreamReader,
    writer: asyncio.StreamWriter,
    conn: sqlite3.Connection,
) -> None:
    request_line = (await reader.readline()).decode("latin-1")
    parts = request_line.split()
    if len(parts) < 3:
        writer.close()
        return
    method, path = parts[0], parts[1]

    # Read headers, note Content-Length for the body.
    content_length = 0
    while True:
        line = (await reader.readline()).decode("latin-1")
        if line in ("\r\n", "\n", ""):
            break
        name, _, value = line.partition(":")
        if name.strip().lower() == "content-length":
            content_length = int(value.strip() or "0")

    body = ""
    if content_length > 0:
        body = (await reader.readexactly(content_length)).decode("latin-1")

    writer.write(handle_request(conn, method, path, body))
    await writer.drain()
    writer.close()
    await writer.wait_closed()


async def start_server(
    conn: sqlite3.Connection, port: int = 0
) -> tuple[asyncio.Server, int]:
    """Start the guestbook server on an ephemeral port; return it and the port."""

    async def handler(r: asyncio.StreamReader, w: asyncio.StreamWriter) -> None:
        await _handle_connection(r, w, conn)

    server = await asyncio.start_server(handler, "127.0.0.1", port)
    bound_port = server.sockets[0].getsockname()[1]
    return server, bound_port


async def _main(db_path: str, port: int) -> None:
    conn = open_db(db_path)
    server, bound_port = await start_server(conn, port)
    print(f"guestbook listening on http://127.0.0.1:{bound_port}", flush=True)
    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    import sys

    path = sys.argv[1] if len(sys.argv) > 1 else "guestbook.db"
    port = int(sys.argv[2]) if len(sys.argv) > 2 else 8080
    asyncio.run(_main(path, port))
