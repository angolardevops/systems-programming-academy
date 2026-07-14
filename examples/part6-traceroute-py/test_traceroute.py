"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from traceroute import (
    Kind,
    Reply,
    build_echo_request,
    checksum,
    classify,
    render_header,
    render_hop,
)


def time_exceeded(inner: bytes) -> bytes:
    """Wrap an echo request in an ICMP time-exceeded message the way a router
    does: outer header [11,0,cksum,unused(4)] then a minimal IP header and the
    original ICMP."""
    ip = bytes([0x45]) + bytes(19)  # version 4, IHL 5, then zeros
    pkt = bytearray(bytes([11, 0, 0, 0, 0, 0, 0, 0]) + ip + inner)
    pkt[2:4] = checksum(bytes(pkt)).to_bytes(2, "big")
    return bytes(pkt)


class TracerouteTest(unittest.TestCase):
    def test_builds_probe_with_valid_checksum(self) -> None:
        pkt = build_echo_request(0x1234, 1, b"abcd")
        self.assertEqual(pkt[0], 8)
        self.assertEqual(checksum(pkt), 0)

    def test_classifies_a_destination_echo_reply(self) -> None:
        reply = bytearray(build_echo_request(0x00AA, 5, b""))
        reply[0] = 0
        reply[2:4] = b"\x00\x00"
        reply[2:4] = checksum(bytes(reply)).to_bytes(2, "big")
        self.assertEqual(classify(bytes(reply)), Reply(Kind.ECHO_REPLY, 0x00AA, 5))

    def test_classifies_a_router_time_exceeded(self) -> None:
        te = time_exceeded(build_echo_request(0xBEEF, 3, b""))
        self.assertEqual(classify(te), Reply(Kind.TIME_EXCEEDED, 0xBEEF, 3))

    def test_rejects_corruption_and_unknown_types(self) -> None:
        te = bytearray(time_exceeded(build_echo_request(1, 1, b"")))
        self.assertIsNotNone(classify(bytes(te)))
        te[10] ^= 0xFF  # corrupt a byte -> checksum fails
        self.assertIsNone(classify(bytes(te)))
        other = bytearray([3, 0, 0, 0, 0, 0, 0, 0])  # destination unreachable
        other[2:4] = checksum(bytes(other)).to_bytes(2, "big")
        self.assertIsNone(classify(bytes(other)))

    def test_renders_the_header(self) -> None:
        self.assertEqual(
            render_header("example.com", "93.184.216.34", 30),
            "traceroute to example.com (93.184.216.34), 30 hops max",
        )

    def test_renders_reply_partial_and_timeout_hops(self) -> None:
        full = render_hop(1, "192.168.1.1", [0.512, 0.489, 0.501])
        self.assertEqual(full, " 1  192.168.1.1  0.512 ms  0.489 ms  0.501 ms")
        partial = render_hop(2, "10.0.0.1", [4.123, 4.200, None])
        self.assertEqual(partial, " 2  10.0.0.1  4.123 ms  4.200 ms  *")
        gone = render_hop(3, None, [None, None, None])
        self.assertEqual(gone, " 3  * * *")


if __name__ == "__main__":
    unittest.main()
