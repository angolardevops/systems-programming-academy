"""Tests: echo over real sockets and the waits-overlap proof."""

import unittest

from async_demo import (
    async_sleepers,
    echo_roundtrip,
    gather_roundtrips,
    start_echo_server,
)


class EchoServerTest(unittest.IsolatedAsyncioTestCase):
    async def test_roundtrip(self) -> None:
        server, port = await start_echo_server()
        try:
            self.assertEqual(await echo_roundtrip(port, "hello"), "hello")
        finally:
            server.close()
            await server.wait_closed()

    async def test_concurrent_clients_on_one_thread(self) -> None:
        server, port = await start_echo_server()
        try:
            messages = [f"client {i}" for i in range(5)]
            self.assertEqual(await gather_roundtrips(port, messages), messages)
        finally:
            server.close()
            await server.wait_closed()


class SleepersTest(unittest.TestCase):
    def test_async_waits_overlap(self) -> None:
        # 200 concurrent 50ms sleeps = 10s of waiting; wall time ~50ms.
        # The generous bound keeps the test robust on loaded machines.
        self.assertLess(async_sleepers(200, 0.05), 0.5)

    def test_async_sleepers_returns_elapsed(self) -> None:
        elapsed = async_sleepers(1, 0.01)
        self.assertGreaterEqual(elapsed, 0.01)


if __name__ == "__main__":
    unittest.main()
