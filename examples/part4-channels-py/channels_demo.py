"""Message passing in Python: ``queue.Queue`` as the channel.

The stdlib has no channel type, but ``queue.Queue`` is the same idea: a
thread-safe FIFO that transfers *values* between threads so no state is
shared. Python has no ``close()`` on a queue — the idiomatic shutdown signal
is a **sentinel** (here ``None``) put once per worker.
"""

from __future__ import annotations

import queue
import threading
from collections.abc import Sequence

_SENTINEL = None


def sum_squares_pool(nums: Sequence[int], n_workers: int) -> int:
    """Square every number using ``n_workers`` threads fed by a jobs queue.

    Results travel back on a results queue; one ``None`` sentinel per worker
    replaces Go's ``close(jobs)``.

    >>> sum_squares_pool([1, 2, 3], 2)
    14
    """
    if n_workers < 1:
        raise ValueError("need at least one worker")
    jobs: queue.Queue[int | None] = queue.Queue()
    results: queue.Queue[int] = queue.Queue()

    def worker() -> None:
        while True:
            n = jobs.get()
            if n is _SENTINEL:
                break
            results.put(n * n)

    threads = [threading.Thread(target=worker) for _ in range(n_workers)]
    for t in threads:
        t.start()

    for n in nums:
        jobs.put(n)
    for _ in threads:
        jobs.put(_SENTINEL)  # one shutdown signal per worker
    for t in threads:
        t.join()

    total = 0
    while not results.empty():  # safe: all producers have exited
        total += results.get()
    return total


def first_response(ready: queue.Queue[str], timeout: float = 1.0) -> str:
    """Take the first available item — ``get(timeout=...)`` is the closest
    stdlib analogue to Go's ``select`` with a timeout arm.

    Raises ``queue.Empty`` if nothing arrives in time, so callers must
    decide what a missed deadline means instead of blocking forever.
    """
    return ready.get(timeout=timeout)


def throughput(n: int, maxsize: int = 0) -> int:
    """Push ``n`` integers from a producer thread to the calling consumer.

    ``maxsize=0`` means unbounded; a positive value adds backpressure
    (``put`` blocks when full), like a buffered Go channel.
    """
    ch: queue.Queue[int | None] = queue.Queue(maxsize=maxsize)

    def producer() -> None:
        for i in range(n):
            ch.put(i)
        ch.put(_SENTINEL)

    t = threading.Thread(target=producer)
    t.start()
    total = 0
    while True:
        v = ch.get()
        if v is _SENTINEL:
            break
        total += v
    t.join()
    return total


if __name__ == "__main__":
    import time

    def timed(label: str, fn, *args):  # type: ignore[no-untyped-def]
        start = time.perf_counter()
        out = fn(*args)
        print(f"{label:<32} {time.perf_counter() - start:>8.3f}s")
        return out

    n = 1_000_000
    expected = n * (n - 1) // 2
    print("-- 1M integers, one producer -> one consumer --")
    assert timed("queue.Queue() unbounded", throughput, n) == expected
    assert timed("queue.Queue(maxsize=1024)", throughput, n, 1024) == expected
