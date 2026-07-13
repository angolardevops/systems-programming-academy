"""Health-check agent — Python implementation of the Part 3 project. All
targets are probed in parallel (ThreadPoolExecutor) with a connect timeout; the
report is deterministic and the exit code scripting-friendly.

Run the tests / the tool:

    python3 -m unittest discover -s . -p 'test_*.py'
    python3 healthcheck.py targets.conf
"""

from __future__ import annotations

import socket
import sys
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass


@dataclass(frozen=True)
class Target:
    """One named probe target."""

    name: str
    addr: str  # host:port


@dataclass(frozen=True)
class Probe:
    """The outcome of probing one target."""

    name: str
    up: bool


def parse_targets(text: str) -> list[Target]:
    """Parses `name = host:port` lines (# comments, blanks tolerated)."""
    targets: list[Target] = []
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue
        name, _, addr = line.partition("=")
        targets.append(Target(name=name.strip(), addr=addr.strip()))
    return targets


def probe(addr: str, timeout: float) -> bool:
    """TCP-probes one address: up means a connection within the timeout."""
    host, _, port_text = addr.rpartition(":")
    try:
        port = int(port_text)
    except ValueError:
        return False
    try:
        with socket.create_connection((host, port), timeout=timeout):
            return True
    except OSError:
        return False


def check_all(targets: list[Target], timeout: float) -> list[Probe]:
    """Probes every target in parallel — Part 1's thread pool doing real ops
    work: N targets cost one timeout, not N."""
    with ThreadPoolExecutor(max_workers=max(len(targets), 1)) as pool:
        probes = list(
            pool.map(lambda t: Probe(name=t.name, up=probe(t.addr, timeout)), targets)
        )
    return sorted(probes, key=lambda p: p.name)  # deterministic report order


def report(probes: list[Probe]) -> tuple[str, int]:
    """Renders the shared report format and derives the exit code."""
    lines = []
    up = 0
    for p in probes:
        status = "UP" if p.up else "DOWN"
        if p.up:
            up += 1
        lines.append(f"{status} {p.name}")
    down = len(probes) - up
    lines.append("---")
    lines.append(f"{up} up, {down} down")
    return "\n".join(lines) + "\n", (0 if down == 0 else 1)


def main() -> None:
    if len(sys.argv) < 2:
        print("usage: healthcheck <targets-file>", file=sys.stderr)
        raise SystemExit(2)
    with open(sys.argv[1], encoding="utf-8") as f:
        targets = parse_targets(f.read())
    text, code = report(check_all(targets, timeout=0.5))
    print(text, end="")
    raise SystemExit(code)


if __name__ == "__main__":
    main()
