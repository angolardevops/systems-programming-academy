// Package main — ping: the testable core of an ICMP echo tool.
//
// Sending an ICMP packet needs a raw socket, which needs root (see main.go).
// But everything interesting is pure and needs no privileges: building the
// echo-request packet, the internet checksum (RFC 1071), parsing an echo
// reply, and the round-trip-time statistics. Those are what this file provides
// and what ping_test.go pins down. Library and command share one package so
// `go run .` and `go test` both work.
package main

import (
	"encoding/binary"
	"fmt"
	"math"
	"strings"
)

// Checksum is the 16-bit internet checksum (RFC 1071): the one's-complement
// sum of the data as big-endian 16-bit words, then complemented. A valid
// packet sums back to zero, so Checksum(validPacket) == 0 — exactly how a
// receiver verifies one.
func Checksum(data []byte) uint16 {
	var sum uint32
	for i := 0; i+1 < len(data); i += 2 {
		sum += uint32(binary.BigEndian.Uint16(data[i:]))
	}
	if len(data)%2 == 1 { // odd trailing byte padded with a zero low byte
		sum += uint32(uint16(data[len(data)-1]) << 8)
	}
	for sum>>16 != 0 {
		sum = (sum & 0xffff) + (sum >> 16)
	}
	return ^uint16(sum)
}

// BuildEchoRequest builds an ICMP echo-request packet (type 8, code 0) with
// the given identifier, sequence number, and payload, checksum filled in.
func BuildEchoRequest(id, seq uint16, payload []byte) []byte {
	pkt := make([]byte, 8+len(payload))
	pkt[0] = 8 // type: echo request
	pkt[1] = 0 // code
	binary.BigEndian.PutUint16(pkt[4:], id)
	binary.BigEndian.PutUint16(pkt[6:], seq)
	copy(pkt[8:], payload)
	binary.BigEndian.PutUint16(pkt[2:], Checksum(pkt))
	return pkt
}

// EchoReply holds the identifier and sequence number from an echo reply.
type EchoReply struct {
	ID  uint16
	Seq uint16
}

// ParseEchoReply parses an ICMP echo reply (type 0, code 0), returning its id
// and seq — but only if the checksum verifies. A corrupted packet, a non-reply
// type, or a runt returns ok == false.
func ParseEchoReply(data []byte) (EchoReply, bool) {
	if len(data) < 8 || data[0] != 0 || data[1] != 0 {
		return EchoReply{}, false
	}
	if Checksum(data) != 0 {
		return EchoReply{}, false
	}
	return EchoReply{
		ID:  binary.BigEndian.Uint16(data[4:]),
		Seq: binary.BigEndian.Uint16(data[6:]),
	}, true
}

// Summarize renders the closing statistics block, byte-for-byte like ping(8).
// rtts are the successful round-trip times in milliseconds; transmitted is how
// many requests went out. With zero replies the rtt line is omitted.
func Summarize(host string, transmitted int, rtts []float64) string {
	received := len(rtts)
	lost := transmitted - received
	lossPct := 0
	if transmitted > 0 {
		lossPct = int(math.Floor(float64(lost)/float64(transmitted)*100 + 0.5))
	}
	var b strings.Builder
	fmt.Fprintf(&b, "--- %s ping statistics ---\n", host)
	fmt.Fprintf(&b, "%d packets transmitted, %d received, %d%% packet loss", transmitted, received, lossPct)
	if received > 0 {
		n := float64(received)
		min, max, sum, sq := rtts[0], rtts[0], 0.0, 0.0
		for _, r := range rtts {
			if r < min {
				min = r
			}
			if r > max {
				max = r
			}
			sum += r
			sq += r * r
		}
		avg := sum / n
		mdev := math.Sqrt(math.Max(sq/n-avg*avg, 0))
		fmt.Fprintf(&b, "\nrtt min/avg/max/mdev = %.3f/%.3f/%.3f/%.3f ms", min, avg, max, mdev)
	}
	return b.String()
}
