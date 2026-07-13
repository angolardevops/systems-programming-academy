"""Event-driven I/O with asyncio: many waits, one thread.

``asyncio`` is CPython's event loop in the standard library. ``await`` marks
the exact points where a coroutine can be suspended; between them the loop
runs other coroutines whose I/O completed. Because everything happens on ONE
thread, the GIL is irrelevant here — and there are no data races between
awaits, because only one coroutine runs at a time.
"""

from __future__ import annotations

import asyncio
import threading
import time


async def echo_handler(
    reader: asyncio.StreamReader, writer: asyncio.StreamWriter
) -> None:
    """Echo one newline-terminated message back to the client, then close."""
    data = await reader.readline()
    writer.write(data)
    await writer.drain()
    writer.close()
    await writer.wait_closed()


async def start_echo_server() -> tuple[asyncio.Server, int]:
    """Start an echo server on an ephemeral port; return it and the port."""
    server = await asyncio.start_server(echo_handler, "127.0.0.1", 0)
    port = server.sockets[0].getsockname()[1]
    return server, port


async def echo_roundtrip(port: int, message: str) -> str:
    """Send one message to the echo server and return the echoed reply."""
    reader, writer = await asyncio.open_connection("127.0.0.1", port)
    writer.write((message + "\n").encode())
    await writer.drain()
    reply = (await reader.readline()).decode().removesuffix("\n")
    writer.close()
    await writer.wait_closed()
    return reply


async def gather_roundtrips(port: int, messages: list[str]) -> list[str]:
    """Run every roundtrip CONCURRENTLY on one thread with asyncio.gather."""
    return list(await asyncio.gather(*(echo_roundtrip(port, m) for m in messages)))


def async_sleepers(n: int, pause: float) -> float:
    """Run ``n`` concurrent ``asyncio.sleep(pause)`` tasks on one thread and
    return the wall-clock elapsed time: n×pause of waiting costs ~pause."""

    async def main() -> None:
        await asyncio.gather(*(asyncio.sleep(pause) for _ in range(n)))

    start = time.perf_counter()
    asyncio.run(main())
    return time.perf_counter() - start


def thread_sleepers(n: int, pause: float) -> float:
    """The same experiment with one OS thread per wait — the baseline that
    asyncio's task-per-wait competes against."""
    threads = [threading.Thread(target=time.sleep, args=(pause,)) for _ in range(n)]
    start = time.perf_counter()
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    return time.perf_counter() - start


if __name__ == "__main__":

    def timed(label: str, fn, *args):  # type: ignore[no-untyped-def]
        start = time.perf_counter()
        fn(*args)
        print(f"{label:<36} {time.perf_counter() - start:>8.3f}s")

    print("-- n concurrent 50ms waits --")
    for n in (1_000, 10_000):
        timed(f"{n:>6} asyncio tasks", async_sleepers, n, 0.05)
    for n in (1_000, 10_000):
        timed(f"{n:>6} OS threads", thread_sleepers, n, 0.05)
