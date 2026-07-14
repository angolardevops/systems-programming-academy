"""Tests: the same six scenarios as the Rust and Go twins."""

import unittest

from ping import (
    EchoReply,
    build_echo_request,
    checksum,
    parse_echo_reply,
    summarize,
)


class PingTest(unittest.TestCase):
    def test_checksum_of_a_known_packet(self) -> None:
        # type 8, code 0, checksum 0, id 1, seq 1.
        data = bytes([0x08, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01])
        self.assertEqual(checksum(data), 0xF7FD)

    def test_builds_echo_request_with_correct_checksum(self) -> None:
        pkt = build_echo_request(1, 1, b"")
        self.assertEqual(pkt, bytes([0x08, 0x00, 0xF7, 0xFD, 0x00, 0x01, 0x00, 0x01]))
        # A packet with a valid checksum sums back to zero.
        self.assertEqual(checksum(pkt), 0)

    def test_build_and_parse_round_trip(self) -> None:
        # Flip the request's type to 0 and recompute the checksum, then parse.
        reply = bytearray(build_echo_request(0x1234, 7, b"payload"))
        reply[0] = 0
        reply[2:4] = b"\x00\x00"
        reply[2:4] = checksum(bytes(reply)).to_bytes(2, "big")
        self.assertEqual(parse_echo_reply(bytes(reply)), EchoReply(id=0x1234, seq=7))

    def test_rejects_non_replies_runts_and_corruption(self) -> None:
        self.assertIsNone(parse_echo_reply(build_echo_request(1, 1, b"")))
        self.assertIsNone(parse_echo_reply(b"\x00\x00\x00\x00"))
        reply = bytearray([0x00, 0x00, 0xFF, 0xEF, 0x00, 0x07, 0x00, 0x09])
        self.assertEqual(parse_echo_reply(bytes(reply)), EchoReply(id=7, seq=9))
        reply[5] ^= 0xFF  # corrupt one byte
        self.assertIsNone(parse_echo_reply(bytes(reply)))

    def test_summarizes_a_clean_run(self) -> None:
        got = summarize("example.com", 3, [10.0, 20.0, 30.0])
        want = (
            "--- example.com ping statistics ---\n"
            "3 packets transmitted, 3 received, 0% packet loss\n"
            "rtt min/avg/max/mdev = 10.000/20.000/30.000/8.165 ms"
        )
        self.assertEqual(got, want)

    def test_summarizes_loss_and_total_loss(self) -> None:
        half = summarize("example.com", 4, [10.0, 30.0])
        self.assertEqual(
            half,
            "--- example.com ping statistics ---\n"
            "4 packets transmitted, 2 received, 50% packet loss\n"
            "rtt min/avg/max/mdev = 10.000/20.000/30.000/10.000 ms",
        )
        none = summarize("example.com", 3, [])
        self.assertEqual(
            none,
            "--- example.com ping statistics ---\n"
            "3 packets transmitted, 0 received, 100% packet loss",
        )


if __name__ == "__main__":
    unittest.main()
