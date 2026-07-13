"""Threads & shared state in Python: locks, batching, and the GIL.

CPython threads are real OS threads, but the Global Interpreter Lock (GIL)
lets only one of them execute Python bytecode at a time. Two consequences:

* CPU-bound thread code does NOT speed up — the benchmark in ``__main__``
  measures it.
* Races still exist: the GIL protects single bytecodes, not read-modify-write
  sequences like ``counter += 1``. You still need ``threading.Lock``.
"""

from __future__ import annotations

import threading
from collections.abc import Sequence


def sum_parallel(data: Sequence[int], n_threads: int) -> int:
    """Sum ``data`` by splitting it into ``n_threads`` chunks, one thread each.

    The result is correct — but on CPython the GIL serializes the CPU-bound
    loops, so this is *concurrency without parallelism*: no speedup.

    >>> sum_parallel(range(1, 101), 4)
    5050
    """
    if n_threads < 1:
        raise ValueError("need at least one thread")
    chunk_size = max(1, -(-len(data) // n_threads))  # ceiling division
    results: list[int] = [0] * n_threads

    def work(slot: int, chunk: Sequence[int]) -> None:
        results[slot] = sum(chunk)

    threads = [
        threading.Thread(
            target=work, args=(i, data[i * chunk_size : (i + 1) * chunk_size])
        )
        for i in range(n_threads)
    ]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    return sum(results)


def counter_locked(n_threads: int, iters: int) -> int:
    """Increment a shared counter under a ``threading.Lock``.

    Always returns exactly ``n_threads * iters`` — the lock makes the
    read-modify-write atomic.
    """
    counter = 0
    lock = threading.Lock()

    def work() -> None:
        nonlocal counter
        for _ in range(iters):
            with lock:
                counter += 1

    threads = [threading.Thread(target=work) for _ in range(n_threads)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    return counter


def counter_batched(n_threads: int, iters: int) -> int:
    """Accumulate locally, take the lock once per thread.

    Same contract as :func:`counter_locked` with ``n`` lock acquisitions
    instead of ``n * iters``.
    """
    counter = 0
    lock = threading.Lock()

    def work() -> None:
        nonlocal counter
        local = 0
        for _ in range(iters):
            local += 1
        with lock:
            counter += local

    threads = [threading.Thread(target=work) for _ in range(n_threads)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    return counter


if __name__ == "__main__":
    import time

    def timed(label: str, fn, *args):  # type: ignore[no-untyped-def]
        start = time.perf_counter()
        out = fn(*args)
        print(f"{label:<28} {time.perf_counter() - start:>8.3f}s")
        return out

    data = list(range(5_000_000))
    expected = sum(data)

    print("-- parallel sum, 5M elements (GIL: expect NO speedup) --")
    assert timed("sum 1 thread", sum_parallel, data, 1) == expected
    assert timed("sum 4 threads", sum_parallel, data, 4) == expected

    n_threads, iters = 4, 250_000
    print(f"-- shared counter, {n_threads} threads x {iters} increments --")
    assert (
        timed("lock per increment", counter_locked, n_threads, iters)
        == n_threads * iters
    )
    assert (
        timed("batched (lock once)", counter_batched, n_threads, iters)
        == n_threads * iters
    )
