"""ping — the testable core of an ICMP echo tool.

Sending an ICMP packet needs a raw socket, which needs root (see ``__main__``
below). But everything interesting is pure and needs no privileges: building
the echo-request packet, the internet checksum (RFC 1071), parsing an echo
reply, and the round-trip-time statistics. Those are what this module provides
and what test_ping.py pins down.
"""

from __future__ import annotations

import math
from dataclasses import dataclass


def checksum(data: bytes) -> int:
    """The 16-bit internet checksum (RFC 1071): the one's-complement sum of the
    data as big-endian 16-bit words, then complemented. A valid packet sums
    back to zero, so ``checksum(valid_packet) == 0`` — exactly how a receiver
    verifies one."""
    total = 0
    for i in range(0, len(data) - 1, 2):
        total += (data[i] << 8) | data[i + 1]
    if len(data) % 2 == 1:  # odd trailing byte padded with a zero low byte
        total += data[-1] << 8
    while total >> 16:
        total = (total & 0xFFFF) + (total >> 16)
    return (~total) & 0xFFFF


def build_echo_request(id: int, seq: int, payload: bytes) -> bytes:
    """Build an ICMP echo-request packet (type 8, code 0) with the given
    identifier, sequence number, and payload, checksum filled in."""
    header = bytes([8, 0, 0, 0]) + id.to_bytes(2, "big") + seq.to_bytes(2, "big")
    packet = header + payload
    ck = checksum(packet)
    return packet[:2] + ck.to_bytes(2, "big") + packet[4:]


@dataclass(frozen=True)
class EchoReply:
    """The identifier and sequence number recovered from an echo reply."""

    id: int
    seq: int


def parse_echo_reply(data: bytes) -> EchoReply | None:
    """Parse an ICMP echo reply (type 0, code 0), returning its id and seq —
    but only if the checksum verifies. A corrupted packet, a non-reply type, or
    a runt returns ``None``."""
    if len(data) < 8 or data[0] != 0 or data[1] != 0:
        return None
    if checksum(data) != 0:
        return None
    return EchoReply(
        id=int.from_bytes(data[4:6], "big"),
        seq=int.from_bytes(data[6:8], "big"),
    )


def summarize(host: str, transmitted: int, rtts: list[float]) -> str:
    """Render the closing statistics block, byte-for-byte like ``ping(8)``.

    ``rtts`` are the successful round-trip times in milliseconds; ``transmitted``
    is how many requests went out. With zero replies the rtt line is omitted."""
    received = len(rtts)
    lost = transmitted - received
    loss_pct = 0 if transmitted == 0 else math.floor(lost / transmitted * 100 + 0.5)
    out = (
        f"--- {host} ping statistics ---\n"
        f"{transmitted} packets transmitted, {received} received, {loss_pct}% packet loss"
    )
    if received > 0:
        n = received
        rtt_min, rtt_max = min(rtts), max(rtts)
        avg = sum(rtts) / n
        mdev = math.sqrt(max(sum(r * r for r in rtts) / n - avg * avg, 0.0))
        out += f"\nrtt min/avg/max/mdev = {rtt_min:.3f}/{avg:.3f}/{rtt_max:.3f}/{mdev:.3f} ms"
    return out


if __name__ == "__main__":
    import os
    import socket
    import sys
    import time

    # ping <host> [count] — needs root or CAP_NET_RAW.
    #
    # The pure logic above is fully tested. Only this block needs a raw socket,
    # the one thing the kernel guards behind root because a raw socket can forge
    # any packet. Run with `sudo`, or grant the capability once:
    # `sudo setcap cap_net_raw+ep $(readlink -f $(which python3))` (affects that
    # interpreter — prefer sudo for a one-off).
    host = sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1"
    count = int(sys.argv[2]) if len(sys.argv) > 2 else 4

    ip = socket.gethostbyname(host)
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_RAW, socket.IPPROTO_ICMP)
    except PermissionError:
        print(
            f"ping: raw sockets need root. Try `sudo python3 ping.py {host}`, or grant\n"
            "the capability once with `sudo setcap cap_net_raw+ep <python3>`.",
            file=sys.stderr,
        )
        raise SystemExit(1) from None

    sock.settimeout(1.0)
    ident = os.getpid() & 0xFFFF
    print(f"PING {host} ({ip}): 56 data bytes")

    rtts: list[float] = []
    for seq in range(1, count + 1):
        packet = build_echo_request(ident, seq, b"\x42" * 56)
        sent = time.perf_counter()
        sock.sendto(packet, (ip, 0))
        try:
            datagram, _ = sock.recvfrom(1500)
        except socket.timeout:
            print(f"Request timeout for icmp_seq {seq}")
            continue
        elapsed = (time.perf_counter() - sent) * 1000.0
        ihl = (datagram[0] & 0x0F) * 4  # skip the IP header
        reply = parse_echo_reply(datagram[ihl:])
        if reply is not None and reply.id == ident:
            rtts.append(elapsed)
            print(f"64 bytes from {ip}: icmp_seq={reply.seq} time={elapsed:.3f} ms")
        if seq < count:
            time.sleep(1.0)

    sock.close()
    print(f"\n{summarize(host, count, rtts)}")
