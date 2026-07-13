"""Log analyzer — Python implementation of the Part 3 project. The same tool is
built in Rust, Go, and Python and benchmarked head-to-head.

Run the tests / the tool:

    python3 -m unittest discover -s . -p 'test_*.py'
    python3 loganalyzer.py access.log
"""

from __future__ import annotations

import sys
from collections import Counter
from dataclasses import dataclass, field


@dataclass(frozen=True)
class Entry:
    """One successfully parsed request line."""

    path: str
    status: int


def parse_line(line: str) -> Entry | None:
    """Parses one log line; None means malformed (counted, never fatal)."""
    parts = line.split('"', 2)
    if len(parts) != 3:
        return None
    req_fields = parts[1].split()
    suf_fields = parts[2].split()
    if len(req_fields) < 2 or not suf_fields:
        return None
    try:
        status = int(suf_fields[0])
    except ValueError:
        return None
    if not 100 <= status <= 599:
        return None
    return Entry(path=req_fields[1], status=status)


@dataclass
class Stats:
    """Aggregated statistics over a whole log."""

    total: int = 0
    malformed: int = 0
    by_class: list[int] = field(default_factory=lambda: [0] * 5)
    paths: Counter[str] = field(default_factory=Counter)

    def add_line(self, line: str) -> None:
        entry = parse_line(line)
        if entry is None:
            self.malformed += 1
            return
        self.total += 1
        self.by_class[entry.status // 100 - 1] += 1
        self.paths[entry.path] += 1

    def error_rate(self) -> float:
        if self.total == 0:
            return 0.0
        return (self.by_class[3] + self.by_class[4]) / self.total * 100

    def top_paths(self, n: int) -> list[tuple[str, int]]:
        # count desc, then path asc — deterministic across implementations.
        return sorted(self.paths.items(), key=lambda kv: (-kv[1], kv[0]))[:n]


def report(text: str) -> str:
    """Analyzes a whole log text and renders the shared output format."""
    stats = Stats()
    for line in text.splitlines():
        if line.strip():
            stats.add_line(line)
    out = [f"total: {stats.total}"]
    for i, label in enumerate(["1xx", "2xx", "3xx", "4xx", "5xx"]):
        out.append(f"{label}: {stats.by_class[i]}")
    out.append(f"malformed: {stats.malformed}")
    out.append(f"error_rate: {stats.error_rate():.1f}%")
    out.append("top paths:")
    for path, count in stats.top_paths(3):
        out.append(f"  {path}: {count}")
    return "\n".join(out) + "\n"


def main() -> None:
    if len(sys.argv) > 1:
        with open(sys.argv[1], encoding="utf-8") as f:
            text = f.read()
    else:
        text = sys.stdin.read()
    print(report(text), end="")


if __name__ == "__main__":
    main()
