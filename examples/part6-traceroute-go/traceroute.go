// Package main — traceroute: the testable core of a hop-by-hop path tracer.
//
// traceroute is ping with one twist: send echo requests with a deliberately
// small IP TTL, so each router along the path decrements it to zero and reports
// back an ICMP time-exceeded (type 11). Raise the TTL by one each round and you
// learn the path, hop by hop, until the destination answers with an echo reply
// (type 0).
//
// Setting the TTL and reading replies needs a raw socket (root — see main.go).
// Everything else is pure and tested in traceroute_test.go. Library and command
// share one package so `go run .` and `go test` both work.
package main

import (
	"encoding/binary"
	"fmt"
	"strings"
)

// Checksum is the 16-bit internet checksum (RFC 1071) — identical to ping's.
// A valid packet sums back to zero, so Checksum(valid) == 0.
func Checksum(data []byte) uint16 {
	var sum uint32
	for i := 0; i+1 < len(data); i += 2 {
		sum += uint32(binary.BigEndian.Uint16(data[i:]))
	}
	if len(data)%2 == 1 {
		sum += uint32(uint16(data[len(data)-1]) << 8)
	}
	for sum>>16 != 0 {
		sum = (sum & 0xffff) + (sum >> 16)
	}
	return ^uint16(sum)
}

// BuildEchoRequest builds an ICMP echo-request probe (type 8), checksum filled
// in — the packet whose TTL we vary.
func BuildEchoRequest(id, seq uint16, payload []byte) []byte {
	pkt := make([]byte, 8+len(payload))
	pkt[0] = 8
	binary.BigEndian.PutUint16(pkt[4:], id)
	binary.BigEndian.PutUint16(pkt[6:], seq)
	copy(pkt[8:], payload)
	binary.BigEndian.PutUint16(pkt[2:], Checksum(pkt))
	return pkt
}

// Kind tells what an incoming ICMP message says about a probe.
type Kind int

const (
	// EchoReply — the destination answered; this hop is the final one.
	EchoReply Kind = iota
	// TimeExceeded — a router reported the TTL expired; an intermediate hop.
	TimeExceeded
)

// Reply is a classified ICMP message with the id and seq of the probe it
// refers to.
type Reply struct {
	Kind Kind
	ID   uint16
	Seq  uint16
}

// Classify a received ICMP message, recovering the probe's id and seq. Returns
// ok == false for anything else (bad checksum, unknown type, runt).
//
// The trick is time-exceeded: its body carries the IP header and first 8 bytes
// of the packet that expired — our echo request's header, holding the id and
// seq. So we reach past the outer ICMP header and the embedded IP header.
func Classify(data []byte) (Reply, bool) {
	if len(data) < 8 || Checksum(data) != 0 {
		return Reply{}, false
	}
	switch {
	case data[0] == 0 && data[1] == 0:
		return Reply{EchoReply, binary.BigEndian.Uint16(data[4:]), binary.BigEndian.Uint16(data[6:])}, true
	case data[0] == 11:
		embedded := data[8:]
		if len(embedded) < 1 {
			return Reply{}, false
		}
		ihl := int(embedded[0]&0x0f) * 4
		if len(embedded) < ihl+8 {
			return Reply{}, false
		}
		inner := embedded[ihl:]
		return Reply{TimeExceeded, binary.BigEndian.Uint16(inner[4:]), binary.BigEndian.Uint16(inner[6:])}, true
	default:
		return Reply{}, false
	}
}

// RenderHeader is the opening line, byte-for-byte like traceroute(8).
func RenderHeader(host, ip string, maxHops int) string {
	return fmt.Sprintf("traceroute to %s (%s), %d hops max", host, ip, maxHops)
}

// Probe is one measurement: RTT in ms, or OK == false for a timeout.
type Probe struct {
	RTT float64
	OK  bool
}

// RenderHop renders one hop line. addr is the responding router ("" if no probe
// answered); probes carry one entry per probe, printed as its time or "*".
//
//	1  192.168.1.1  0.512 ms  0.489 ms  0.501 ms
//	2  10.0.0.1  4.123 ms  4.200 ms  *
//	3  * * *
func RenderHop(ttl int, addr string, probes []Probe) string {
	var b strings.Builder
	fmt.Fprintf(&b, "%2d  ", ttl)
	if addr == "" {
		stars := make([]string, len(probes))
		for i := range stars {
			stars[i] = "*"
		}
		b.WriteString(strings.Join(stars, " "))
		return b.String()
	}
	b.WriteString(addr)
	for _, p := range probes {
		if p.OK {
			fmt.Fprintf(&b, "  %.3f ms", p.RTT)
		} else {
			b.WriteString("  *")
		}
	}
	return b.String()
}
