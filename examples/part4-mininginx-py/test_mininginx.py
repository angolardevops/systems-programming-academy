"""Tests: the same eight scenarios as the Rust and Go twins, over real
sockets, plus doctests for the pure resolver."""

import asyncio
import doctest
import tempfile
import unittest
from pathlib import Path

import mininginx
from mininginx import build_response, resolve, start_server


def load_tests(loader, tests, ignore):  # noqa: ARG001 - unittest protocol
    tests.addTests(doctest.DocTestSuite(mininginx))
    return tests


def setup_docroot(base: Path) -> Path:
    """index.html and style.css inside the docroot — and secret.txt one
    level OUTSIDE it, the traversal target that must never be served."""
    docroot = base / "public"
    docroot.mkdir()
    (docroot / "index.html").write_text("<h1>home</h1>\n")
    (docroot / "style.css").write_text("body{}\n")
    (base / "secret.txt").write_text("TOP SECRET\n")
    return docroot


async def raw_request(port: int, raw: str) -> tuple[int, str, bytes]:
    reader, writer = await asyncio.open_connection("127.0.0.1", port)
    writer.write(raw.encode())
    await writer.drain()
    response = await reader.read()
    writer.close()
    await writer.wait_closed()
    head, _, body = response.partition(b"\r\n\r\n")
    status = int(head.split()[1])
    return status, head.decode(), body


class MininginxTest(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        docroot = setup_docroot(Path(self._tmp.name))
        self.server, self.port = await start_server(docroot)

    async def asyncTearDown(self) -> None:
        self.server.close()
        await self.server.wait_closed()
        self._tmp.cleanup()

    async def test_serves_index_for_root(self) -> None:
        status, head, body = await raw_request(self.port, "GET / HTTP/1.0\r\n\r\n")
        self.assertEqual(status, 200)
        self.assertIn("Content-Type: text/html", head)
        self.assertEqual(body, b"<h1>home</h1>\n")

    async def test_serves_css_with_content_type_and_length(self) -> None:
        status, head, body = await raw_request(
            self.port, "GET /style.css HTTP/1.0\r\n\r\n"
        )
        self.assertEqual(status, 200)
        self.assertIn("Content-Type: text/css", head)
        self.assertIn(f"Content-Length: {len(body)}", head)

    async def test_missing_file_is_404(self) -> None:
        status, _, body = await raw_request(
            self.port, "GET /nope.html HTTP/1.0\r\n\r\n"
        )
        self.assertEqual(status, 404)
        self.assertEqual(body, b"404 Not Found\n")

    async def test_post_is_405(self) -> None:
        status, _, _ = await raw_request(self.port, "POST / HTTP/1.0\r\n\r\n")
        self.assertEqual(status, 405)

    async def test_traversal_never_escapes_docroot(self) -> None:
        status, _, body = await raw_request(
            self.port, "GET /../secret.txt HTTP/1.0\r\n\r\n"
        )
        self.assertEqual(status, 404, "traversal must be rejected")
        self.assertNotIn(b"SECRET", body)

    async def test_garbage_is_400(self) -> None:
        status, _, _ = await raw_request(self.port, "NOT-HTTP\r\n\r\n")
        self.assertEqual(status, 400)

    async def test_concurrent_clients_all_succeed(self) -> None:
        results = await asyncio.gather(
            *(raw_request(self.port, "GET / HTTP/1.0\r\n\r\n") for _ in range(16))
        )
        for status, _, body in results:
            self.assertEqual(status, 200)
            self.assertEqual(body, b"<h1>home</h1>\n")


class PureFunctionTest(unittest.TestCase):
    def test_resolve_rejects_nested_dotdot(self) -> None:
        self.assertIsNone(resolve(Path("/srv/www"), "/a/../../etc/passwd"))

    def test_build_response_content_length_counts_bytes(self) -> None:
        response = build_response(200, "text/plain", "olá\n".encode())
        self.assertIn(b"Content-Length: 5", response)  # á is 2 bytes in UTF-8


if __name__ == "__main__":
    unittest.main()
