"""Tests: the same six scenarios as the Rust and Go twins, scan tested against
real loopback sockets."""

import socket
import unittest

from portscan import State, parse_ports, render_table, scan_all, scan_port, service_name


class PortscanTest(unittest.TestCase):
    def test_parses_ports_and_ranges_sorted_unique(self) -> None:
        self.assertEqual(parse_ports("80"), [80])
        self.assertEqual(parse_ports("22,80,443"), [22, 80, 443])
        self.assertEqual(parse_ports("1-3"), [1, 2, 3])
        self.assertEqual(parse_ports("3-1, 2, 80"), [1, 2, 3, 80])
        self.assertEqual(parse_ports("22, oops, 90000, 443"), [22, 443])

    def test_looks_up_well_known_services(self) -> None:
        self.assertEqual(service_name(22), "ssh")
        self.assertEqual(service_name(443), "https")
        self.assertEqual(service_name(6379), "redis")
        self.assertEqual(service_name(12345), "unknown")

    def test_open_port_detected_against_real_listener(self) -> None:
        ln = socket.socket()
        ln.bind(("127.0.0.1", 0))
        ln.listen()
        port = ln.getsockname()[1]
        try:
            self.assertEqual(scan_port("127.0.0.1", port, 1.0), State.OPEN)
        finally:
            ln.close()

    def test_closed_port_detected_bind_then_close(self) -> None:
        ln = socket.socket()
        ln.bind(("127.0.0.1", 0))
        port = ln.getsockname()[1]
        ln.close()  # nothing listening now -> refused
        self.assertEqual(scan_port("127.0.0.1", port, 1.0), State.CLOSED)

    def test_scan_all_finds_open_ports_among_closed(self) -> None:
        listeners = []
        open_ports = []
        for _ in range(2):
            ln = socket.socket()
            ln.bind(("127.0.0.1", 0))
            ln.listen()
            listeners.append(ln)
            open_ports.append(ln.getsockname()[1])
        cl = socket.socket()
        cl.bind(("127.0.0.1", 0))
        closed = cl.getsockname()[1]
        cl.close()

        try:
            results = scan_all("127.0.0.1", [*open_ports, closed], 1.0, 8)
            got_open = {p for p, s in results if s is State.OPEN}
            self.assertEqual(got_open, set(open_ports))
            self.assertIn((closed, State.CLOSED), results)
        finally:
            for ln in listeners:
                ln.close()

    def test_renders_the_table(self) -> None:
        rows = [(22, State.OPEN), (80, State.OPEN), (443, State.OPEN)]
        expected = (
            "PORT      STATE     SERVICE\n"
            "22/tcp    open      ssh\n"
            "80/tcp    open      http\n"
            "443/tcp   open      https"
        )
        self.assertEqual(render_table(rows), expected)


if __name__ == "__main__":
    unittest.main()
