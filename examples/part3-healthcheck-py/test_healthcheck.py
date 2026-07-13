"""Tests for healthcheck.py — probing real local listeners (stdlib unittest)."""

import socket
import unittest

from healthcheck import Probe, Target, check_all, parse_targets, probe, report


def up_addr(case: unittest.TestCase) -> str:
    """Starts a real listener on an ephemeral port (deterministic 'up')."""
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen()
    case.addCleanup(listener.close)
    host, port = listener.getsockname()
    return f"{host}:{port}"


def down_addr() -> str:
    """Binds then closes a socket: a deterministic refused port."""
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    host, port = s.getsockname()
    s.close()
    return f"{host}:{port}"


class ParseTests(unittest.TestCase):
    def test_parses_targets_with_comments(self) -> None:
        targets = parse_targets("# fleet\napi = 127.0.0.1:8080\n\nweb = 10.0.0.2:80\n")
        self.assertEqual(targets[0], Target(name="api", addr="127.0.0.1:8080"))
        self.assertEqual(len(targets), 2)


class ProbeTests(unittest.TestCase):
    def test_up_for_real_listener(self) -> None:
        self.assertTrue(probe(up_addr(self), timeout=0.5))

    def test_down_for_closed_port(self) -> None:
        self.assertFalse(probe(down_addr(), timeout=0.5))


class CheckAllTests(unittest.TestCase):
    def test_parallel_and_sorted(self) -> None:
        up = up_addr(self)
        targets = [
            Target(name="web", addr=up),
            Target(name="api", addr=up),
            Target(name="cache", addr=down_addr()),
        ]
        probes = check_all(targets, timeout=0.5)
        self.assertEqual([p.name for p in probes], ["api", "cache", "web"])
        self.assertEqual([p.up for p in probes], [True, False, True])


class ReportTests(unittest.TestCase):
    def test_report_and_exit_code(self) -> None:
        text, code = report([Probe(name="api", up=True), Probe(name="cache", up=False)])
        self.assertEqual(text, "UP api\nDOWN cache\n---\n1 up, 1 down\n")
        self.assertEqual(code, 1)
        _, all_up = report([Probe(name="api", up=True)])
        self.assertEqual(all_up, 0)


if __name__ == "__main__":
    unittest.main()
