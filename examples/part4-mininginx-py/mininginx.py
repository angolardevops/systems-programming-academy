"""mininginx — usage: python3 mininginx.py <docroot> <port>

A concurrent static-file HTTP server with no http.server: raw asyncio
streams, hand parsing, one coroutine per connection multiplexed on a
single-threaded event loop (the Async lesson's model). Serves byte-identical
responses to the Rust and Go twins.
"""

from __future__ import annotations

import asyncio
import sys
from pathlib import Path

_REASONS = {
    200: "OK",
    400: "Bad Request",
    404: "Not Found",
    405: "Method Not Allowed",
}

_CONTENT_TYPES = {
    ".html": "text/html",
    ".css": "text/css",
    ".js": "application/javascript",
    ".json": "application/json",
    ".png": "image/png",
    ".txt": "text/plain",
}


def resolve(docroot: Path, url_path: str) -> Path | None:
    """Map a URL path to a file inside ``docroot``, or ``None`` if the path
    tries to escape it. Purely lexical: any ``..`` component is rejected
    outright — the request never touches the filesystem outside the root.

    >>> resolve(Path("/srv/www"), "/") == Path("/srv/www/index.html")
    True
    >>> resolve(Path("/srv/www"), "/../etc/passwd") is None
    True
    """
    if url_path == "/":
        url_path = "/index.html"
    resolved = docroot
    for component in url_path.split("/"):
        if component in ("", "."):
            continue
        if component == ".." or "\0" in component:
            return None  # traversal attempt: never leaves docroot
        resolved = resolved / component
    return resolved


def content_type(path: Path) -> str:
    """Content-Type by file extension — the tiny subset a static site needs."""
    return _CONTENT_TYPES.get(path.suffix, "application/octet-stream")


def build_response(status: int, ctype: str, body: bytes) -> bytes:
    """Serialize a full HTTP/1.0 response. The exact bytes here are the
    cross-language contract with the Rust and Go twins."""
    reason = _REASONS.get(status, "Internal Server Error")
    head = (
        f"HTTP/1.0 {status} {reason}\r\n"
        f"Content-Type: {ctype}\r\n"
        f"Content-Length: {len(body)}\r\n"
        "Connection: close\r\n\r\n"
    )
    return head.encode() + body


def error_response(status: int) -> bytes:
    bodies = {
        400: "400 Bad Request\n",
        404: "404 Not Found\n",
        405: "405 Method Not Allowed\n",
    }
    body = bodies.get(status, "500 Internal Server Error\n")
    return build_response(status, "text/plain", body.encode())


async def handle_connection(
    reader: asyncio.StreamReader, writer: asyncio.StreamWriter, docroot: Path
) -> None:
    """Parse the request head, resolve, read the file, respond, close."""
    try:
        request_line = (await reader.readline()).decode("latin-1")
        parts = request_line.split()
        if len(parts) < 3 or not parts[2].startswith("HTTP/"):
            writer.write(error_response(400))
            return
        method, url_path = parts[0], parts[1]

        # Drain the headers; we serve statelessly and ignore them all.
        while True:
            line = await reader.readline()
            if line in (b"\r\n", b"\n", b""):
                break

        if method != "GET":
            writer.write(error_response(405))
            return
        file = resolve(docroot, url_path)
        if file is None:
            writer.write(error_response(404))
            return
        try:
            body = file.read_bytes()
        except OSError:
            writer.write(error_response(404))
            return
        writer.write(build_response(200, content_type(file), body))
    finally:
        await writer.drain()
        writer.close()
        await writer.wait_closed()


async def start_server(docroot: Path, port: int = 0) -> tuple[asyncio.Server, int]:
    """Start the server; return it and the port actually bound."""

    async def handler(
        reader: asyncio.StreamReader, writer: asyncio.StreamWriter
    ) -> None:
        await handle_connection(reader, writer, docroot)

    server = await asyncio.start_server(handler, "127.0.0.1", port)
    bound_port = server.sockets[0].getsockname()[1]
    return server, bound_port


async def _main(docroot: Path, port: int) -> None:
    server, bound_port = await start_server(docroot, port)
    print(f"listening on 127.0.0.1:{bound_port}", flush=True)
    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("usage: python3 mininginx.py <docroot> <port>", file=sys.stderr)
        raise SystemExit(2)
    root = Path(sys.argv[1])
    if not root.is_dir():
        print(f"error: docroot {root} is not a directory", file=sys.stderr)
        raise SystemExit(2)
    asyncio.run(_main(root, int(sys.argv[2])))
