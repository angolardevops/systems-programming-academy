"""A concurrent TCP port scanner — a nmap-lite — that probes a host's ports and
prints an elegant results table.

It reuses the fan-out/fan-in concurrency of the Part 3 health-check agent: a
thread pool probes ports and tries to connect. The TCP connect scan here is
completely unprivileged. Pure parts (port-spec parsing, service lookup, table
rendering) are tested directly; the scan is tested against real loopback
sockets. A real nmap also does a raw-socket SYN scan, which needs root — the
connect scan trades stealth for portability.
"""

from __future__ import annotations

import errno
import socket
from concurrent.futures import ThreadPoolExecutor
from enum import Enum

_SERVICES = {
    21: "ftp",
    22: "ssh",
    23: "telnet",
    25: "smtp",
    53: "dns",
    80: "http",
    110: "pop3",
    143: "imap",
    443: "https",
    3306: "mysql",
    5432: "postgres",
    6379: "redis",
    8080: "http-alt",
}


class State(Enum):
    OPEN = "open"
    CLOSED = "closed"
    FILTERED = "filtered"

    @property
    def label(self) -> str:
        return self.value


def parse_ports(spec: str) -> list[int]:
    """Parse a comma-separated spec of ports and inclusive ranges
    (``"22,80,1-1024"``) into a sorted, de-duplicated list. Invalid items are
    skipped."""
    ports: set[int] = set()
    for item in spec.split(","):
        item = item.strip()
        if "-" in item:
            lo, _, hi = item.partition("-")
            try:
                a, b = int(lo.strip()), int(hi.strip())
            except ValueError:
                continue
            if _valid(a) and _valid(b):
                for p in range(min(a, b), max(a, b) + 1):
                    ports.add(p)
        else:
            try:
                p = int(item)
            except ValueError:
                continue
            if _valid(p):
                ports.add(p)
    return sorted(ports)


def _valid(p: int) -> bool:
    return 0 <= p <= 65535


def service_name(port: int) -> str:
    """Return the well-known service name for a port, or ``"unknown"``."""
    return _SERVICES.get(port, "unknown")


def scan_port(host: str, port: int, timeout: float) -> State:
    """Probe host:port with a connect timeout. Connected -> OPEN, refused ->
    CLOSED, timeout/unreachable -> FILTERED."""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(timeout)
    try:
        sock.connect((host, port))
        return State.OPEN
    except ConnectionRefusedError:
        return State.CLOSED
    except OSError as e:
        if e.errno == errno.ECONNREFUSED:
            return State.CLOSED
        return State.FILTERED
    finally:
        sock.close()


def scan_all(
    host: str, ports: list[int], timeout: float, workers: int
) -> list[tuple[int, State]]:
    """Scan every port of host concurrently with a thread pool, returning
    ``(port, state)`` pairs sorted by port."""
    with ThreadPoolExecutor(max_workers=max(1, workers)) as pool:
        states = pool.map(lambda p: (p, scan_port(host, p, timeout)), ports)
    return sorted(states, key=lambda r: r[0])


def render_table(rows: list[tuple[int, State]]) -> str:
    """Render a results table for the given rows. The exact bytes are the
    cross-language contract."""
    lines = [f"{'PORT':<10}{'STATE':<10}SERVICE"]
    for port, state in rows:
        lines.append(f"{f'{port}/tcp':<10}{state.label:<10}{service_name(port)}")
    return "\n".join(lines)


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print(
            "usage: portscan <host> [ports]   e.g. portscan 127.0.0.1 1-1024",
            file=sys.stderr,
        )
        raise SystemExit(2)
    host = sys.argv[1]
    spec = (
        sys.argv[2]
        if len(sys.argv) > 2
        else "21,22,23,25,53,80,110,143,443,3306,5432,6379,8080"
    )
    ports = parse_ports(spec)

    print(f"scanning {host} ({len(ports)} ports)…\n")
    results = scan_all(host, ports, 0.5, 128)
    open_rows = [r for r in results if r[1] is State.OPEN]
    if not open_rows:
        print("no open ports found")
    else:
        print(render_table(open_rows))
