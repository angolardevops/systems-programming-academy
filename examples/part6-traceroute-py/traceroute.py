"""traceroute — the testable core of a hop-by-hop path tracer.

traceroute is ping with one twist: send echo requests with a deliberately small
IP TTL, so each router along the path decrements it to zero and reports back an
ICMP time-exceeded (type 11). Raise the TTL by one each round and you learn the
path, hop by hop, until the destination answers with an echo reply (type 0).

Setting the TTL and reading replies needs a raw socket (root — see ``__main__``).
Everything else is pure and tested in test_traceroute.py.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


def checksum(data: bytes) -> int:
    """The 16-bit internet checksum (RFC 1071) — identical to ping's. A valid
    packet sums back to zero, so ``checksum(valid) == 0``."""
    total = 0
    for i in range(0, len(data) - 1, 2):
        total += (data[i] << 8) | data[i + 1]
    if len(data) % 2 == 1:
        total += data[-1] << 8
    while total >> 16:
        total = (total & 0xFFFF) + (total >> 16)
    return (~total) & 0xFFFF


def build_echo_request(id: int, seq: int, payload: bytes) -> bytes:
    """Build an ICMP echo-request probe (type 8), checksum filled in — the
    packet whose TTL we vary."""
    header = bytes([8, 0, 0, 0]) + id.to_bytes(2, "big") + seq.to_bytes(2, "big")
    packet = header + payload
    ck = checksum(packet)
    return packet[:2] + ck.to_bytes(2, "big") + packet[4:]


class Kind(Enum):
    """What an incoming ICMP message says about a probe."""

    ECHO_REPLY = "echo_reply"  # the destination answered; the final hop
    TIME_EXCEEDED = "time_exceeded"  # a router reported the TTL expired


@dataclass(frozen=True)
class Reply:
    """A classified ICMP message with the probe's id and seq."""

    kind: Kind
    id: int
    seq: int


def classify(data: bytes) -> Reply | None:
    """Classify a received ICMP message, recovering the probe's id and seq.
    Returns ``None`` for anything else (bad checksum, unknown type, runt).

    The trick is time-exceeded: its body carries the IP header and first 8 bytes
    of the packet that expired — our echo request's header, holding the id and
    seq. So we reach past the outer ICMP header and the embedded IP header."""
    if len(data) < 8 or checksum(data) != 0:
        return None
    if data[0] == 0 and data[1] == 0:
        return Reply(
            Kind.ECHO_REPLY,
            int.from_bytes(data[4:6], "big"),
            int.from_bytes(data[6:8], "big"),
        )
    if data[0] == 11:
        embedded = data[8:]
        if not embedded:
            return None
        ihl = (embedded[0] & 0x0F) * 4
        inner = embedded[ihl:]
        if len(inner) < 8:
            return None
        return Reply(
            Kind.TIME_EXCEEDED,
            int.from_bytes(inner[4:6], "big"),
            int.from_bytes(inner[6:8], "big"),
        )
    return None


def render_header(host: str, ip: str, max_hops: int) -> str:
    """The opening line, byte-for-byte like ``traceroute(8)``."""
    return f"traceroute to {host} ({ip}), {max_hops} hops max"


def render_hop(ttl: int, addr: str | None, rtts: list[float | None]) -> str:
    """Render one hop line. ``addr`` is the responding router (None if no probe
    answered); ``rtts`` holds one entry per probe: a time in ms, or None for a
    timeout, printed as ``*``.

    ::

         1  192.168.1.1  0.512 ms  0.489 ms  0.501 ms
         2  10.0.0.1  4.123 ms  4.200 ms  *
         3  * * *
    """
    line = f"{ttl:2d}  "
    if addr is None:
        return line + " ".join("*" for _ in rtts)
    line += addr
    for ms in rtts:
        line += "  *" if ms is None else f"  {ms:.3f} ms"
    return line


if __name__ == "__main__":
    import os
    import socket
    import sys
    import time

    # traceroute <host> [max_hops] — needs root or CAP_NET_RAW.
    #
    # The pure logic above is fully tested. Only this block needs a raw socket,
    # to set the outgoing TTL and read the ICMP replies.
    host = sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1"
    max_hops = int(sys.argv[2]) if len(sys.argv) > 2 else 30
    probes_per_hop = 3

    ip = socket.gethostbyname(host)
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_RAW, socket.IPPROTO_ICMP)
    except PermissionError:
        print(
            f"traceroute: raw sockets need root. Try `sudo python3 traceroute.py {host}`,\n"
            "or grant the capability once with `sudo setcap cap_net_raw+ep <python3>`.",
            file=sys.stderr,
        )
        raise SystemExit(1) from None

    sock.settimeout(1.0)
    ident = os.getpid() & 0xFFFF
    print(render_header(host, ip, max_hops))

    for ttl in range(1, max_hops + 1):
        # The one extra socket option that makes traceroute out of ping.
        sock.setsockopt(socket.IPPROTO_IP, socket.IP_TTL, ttl)

        addr: str | None = None
        rtts: list[float | None] = []
        reached = False

        for probe in range(probes_per_hop):
            seq = ttl * probes_per_hop + probe
            sock.sendto(build_echo_request(ident, seq, b"\x42" * 32), (ip, 0))
            sent = time.perf_counter()
            try:
                datagram, src = sock.recvfrom(1500)
            except socket.timeout:
                rtts.append(None)  # timeout -> '*'
                continue
            elapsed = (time.perf_counter() - sent) * 1000.0
            ihl = (datagram[0] & 0x0F) * 4
            reply = classify(datagram[ihl:])
            if reply is None or reply.id != ident:
                rtts.append(None)
                continue
            if addr is None:
                addr = src[0]
            rtts.append(elapsed)
            if reply.kind is Kind.ECHO_REPLY:
                reached = True

        print(render_hop(ttl, addr, rtts))
        if reached:
            break  # the destination itself answered

    sock.close()
