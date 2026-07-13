"""End-to-end tests: a real asyncio server over real sockets, backed by a real
temp-file SQLite database. The SQL-injection proof runs against actual SQLite."""

import asyncio
import tempfile
import unittest
from pathlib import Path

from guestbook_app import all_comments, open_db, start_server


async def raw_request(port: int, raw: bytes) -> tuple[int, str, str]:
    reader, writer = await asyncio.open_connection("127.0.0.1", port)
    writer.write(raw)
    await writer.drain()
    response = (await reader.read()).decode("latin-1")
    writer.close()
    await writer.wait_closed()
    head, _, body = response.partition("\r\n\r\n")
    status = int(head.split()[1])
    return status, head, body


class GuestbookAppTest(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.db_path = str(Path(self._tmp.name) / "test.db")
        self.conn = open_db(self.db_path)
        self.server, self.port = await start_server(self.conn)

    async def asyncTearDown(self) -> None:
        self.server.close()
        await self.server.wait_closed()
        self.conn.close()
        self._tmp.cleanup()

    async def _get(self, path: str) -> tuple[int, str, str]:
        return await raw_request(self.port, f"GET {path} HTTP/1.0\r\n\r\n".encode())

    async def _post(self, path: str, form: str) -> tuple[int, str, str]:
        raw = (
            f"POST {path} HTTP/1.0\r\n"
            f"Content-Type: application/x-www-form-urlencoded\r\n"
            f"Content-Length: {len(form)}\r\n\r\n{form}"
        ).encode()
        return await raw_request(self.port, raw)

    async def test_get_root_renders_empty_guestbook(self) -> None:
        status, _, body = await self._get("/")
        self.assertEqual(status, 200)
        self.assertIn("<h1>Guestbook</h1>", body)

    async def test_post_valid_comment_redirects_and_persists(self) -> None:
        status, head, _ = await self._post("/comment", "author=Ana&body=Hello")
        self.assertEqual(status, 303)
        self.assertIn("Location: /", head)
        # It really landed in SQLite.
        self.assertEqual(all_comments(self.conn), [("Ana", "Hello")])
        # And it shows on the page, autoescaped.
        _, _, page = await self._get("/")
        self.assertIn("<strong>Ana</strong>: Hello", page)

    async def test_post_invalid_comment_is_400_and_persists_nothing(self) -> None:
        status, _, body = await self._post("/comment", "author=A&body=")
        self.assertEqual(status, 400)
        self.assertIn("author: must be at least 2 characters", body)
        self.assertIn("body: is required", body)
        self.assertEqual(all_comments(self.conn), [])

    async def test_sql_injection_against_real_sqlite_table_survives(self) -> None:
        await self._post("/comment", "author=Alice&body=first")
        # URL-encode the payload as a browser would.
        payload = "%27%3B+DROP+TABLE+comments%3B+--"
        status, _, _ = await self._post("/comment", f"author=Mallory&body={payload}")
        self.assertEqual(status, 303)
        # The REAL table still exists with both rows.
        rows = all_comments(self.conn)
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[0], ("Alice", "first"))
        self.assertEqual(rows[1][1], "'; DROP TABLE comments; --")

    async def test_xss_payload_renders_inert(self) -> None:
        await self._post(
            "/comment", "author=Eve&body=%3Cscript%3Ealert(1)%3C%2Fscript%3E"
        )
        _, _, page = await self._get("/")
        self.assertIn("&lt;script&gt;alert(1)&lt;/script&gt;", page)
        self.assertNotIn("<script>alert(1)", page)

    async def test_unknown_route_is_404(self) -> None:
        status, _, _ = await self._get("/nope")
        self.assertEqual(status, 404)


if __name__ == "__main__":
    unittest.main()
