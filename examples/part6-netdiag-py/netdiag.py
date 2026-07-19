"""netdiag — the Part 6 capstone: one CLI over the five tools.

Every tool in Part 6 grew its own entry point. A real network engineer wants them
under one command with a consistent look: ``netdiag scan``, ``netdiag ping``,
``netdiag trace``. This module is the new part unifying them requires — the
command layer: parse a subcommand and its arguments into a ``Command``, and
render every tool's output through one shared report format (a boxed banner and
section headers). Both are pure and byte-identical across the three languages.
"""

from __future__ import annotations

from dataclasses import dataclass

WIDTH = 46


@dataclass(frozen=True)
class Command:
    """A parsed subcommand and its arguments. ``kind`` is scan/ping/trace."""

    kind: str
    host: str
    ports: str = ""
    count: int = 0
    max_hops: int = 0


def usage() -> str:
    """The help text — one line per subcommand."""
    return (
        "netdiag — network diagnostics\n"
        "\n"
        "usage:\n"
        "  netdiag scan  <host> <ports>     TCP connect scan (e.g. 1-1024)\n"
        "  netdiag ping  <host> [count]     ICMP echo, default 4 probes\n"
        "  netdiag trace <host> [max_hops]  path trace, default 30 hops"
    )


class UsageError(ValueError):
    """Raised on an unknown subcommand or missing argument; str() is the help."""


def parse_command(args: list[str]) -> Command:
    """Parse argv-after-the-program-name into a ``Command``, or raise
    ``UsageError`` carrying the usage text."""
    if not args:
        raise UsageError(usage())
    sub = args[0]
    if sub == "scan":
        if len(args) < 3:
            raise UsageError(usage())
        return Command("scan", host=args[1], ports=args[2])
    if sub == "ping":
        if len(args) < 2:
            raise UsageError(usage())
        return Command("ping", host=args[1], count=_int_arg(args, 2, 4))
    if sub == "trace":
        if len(args) < 2:
            raise UsageError(usage())
        return Command("trace", host=args[1], max_hops=_int_arg(args, 2, 30))
    raise UsageError(f"netdiag: unknown command '{sub}'\n\n{usage()}")


def _int_arg(args: list[str], i: int, fallback: int) -> int:
    if i < len(args):
        try:
            return int(args[i])
        except ValueError:
            pass
    return fallback


def banner(title: str) -> str:
    """A boxed banner atop each report."""
    bar = "═" * WIDTH
    content = f"  netdiag :: {title}"
    pad = max(0, WIDTH - len(content))
    return f"╔{bar}╗\n║{content}{' ' * pad}║\n╚{bar}╝"


def section(title: str) -> str:
    """A section header rule: ``── open ports ─────…`` to a fixed width."""
    prefix = f"── {title} "
    fill = max(0, (WIDTH + 2) - len(prefix))
    return prefix + "─" * fill


if __name__ == "__main__":
    import socket
    import sys

    # netdiag <scan|ping|trace> <host> [args]
    #
    # The parsing and report format above are the tested library. scan runs here
    # (a TCP connect scan needs no privilege); ping/trace need a raw socket, so
    # this capstone reports the plan and points at the dedicated tools.
    try:
        cmd = parse_command(sys.argv[1:])
    except UsageError as e:
        print(e, file=sys.stderr)
        raise SystemExit(2) from None

    def _delegates_note(tool: str) -> None:
        print(section("note"))
        print(f"  {tool} needs a raw socket (root / CAP_NET_RAW).")
        print(f"  Run the dedicated `{tool}` tool from its lesson with sudo.")

    def _parse_ports(spec: str) -> list[int]:
        ports: set[int] = set()
        for item in spec.split(","):
            item = item.strip()
            if "-" in item:
                lo, _, hi = item.partition("-")
                try:
                    a, b = int(lo), int(hi)
                except ValueError:
                    continue
                ports.update(range(min(a, b), max(a, b) + 1))
            else:
                try:
                    ports.add(int(item))
                except ValueError:
                    continue
        return sorted(ports)

    if cmd.kind == "scan":
        print(banner(f"scan {cmd.host}"))
        print(section("open ports"))
        found = 0
        for port in _parse_ports(cmd.ports):
            sock = socket.socket()
            sock.settimeout(0.3)
            try:
                sock.connect((cmd.host, port))
                print(f"  {port}/tcp open")
                found += 1
            except OSError:
                pass
            finally:
                sock.close()
        if found == 0:
            print("  (none)")
    elif cmd.kind == "ping":
        print(banner(f"ping {cmd.host}"))
        print(section("plan"))
        print(f"  {cmd.count} ICMP echo probes to {cmd.host}")
        _delegates_note("ping")
    else:  # trace
        print(banner(f"trace {cmd.host}"))
        print(section("plan"))
        print(f"  up to {cmd.max_hops} hops to {cmd.host}")
        _delegates_note("traceroute")
