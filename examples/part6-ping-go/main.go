package main

// ping — send ICMP echo requests and time the replies.
//
// Usage: ping <host> [count]   (needs root or CAP_NET_RAW)
//
// The packet building, checksum, reply parsing, and statistics live in
// ping.go and are fully tested without privileges. Only this file needs a raw
// socket — the one thing the kernel guards behind root, because a raw socket
// can forge any packet. Run with `sudo`, or grant the capability once:
// `sudo setcap cap_net_raw+ep ./part6-ping-go`.
//
// Run: `sudo go run . 1.1.1.1 4`

import (
	"errors"
	"fmt"
	"net"
	"os"
	"strconv"
	"syscall"
	"time"
)

func main() {
	host := "127.0.0.1"
	if len(os.Args) > 1 {
		host = os.Args[1]
	}
	count := 4
	if len(os.Args) > 2 {
		if n, err := strconv.Atoi(os.Args[2]); err == nil {
			count = n
		}
	}
	if err := run(host, count); err != nil {
		fmt.Fprintf(os.Stderr, "ping: %v\n", err)
		os.Exit(1)
	}
}

func run(host string, count int) error {
	// Resolve the host to an IPv4 address using the standard resolver.
	addrs, err := net.LookupIP(host)
	if err != nil {
		return err
	}
	var ip4 net.IP
	for _, a := range addrs {
		if v4 := a.To4(); v4 != nil {
			ip4 = v4
			break
		}
	}
	if ip4 == nil {
		return errors.New("no IPv4 address")
	}

	// A raw ICMP socket — this is the privileged line.
	fd, err := syscall.Socket(syscall.AF_INET, syscall.SOCK_RAW, syscall.IPPROTO_ICMP)
	if err != nil {
		if errors.Is(err, syscall.EPERM) || errors.Is(err, syscall.EACCES) {
			fmt.Fprintf(os.Stderr,
				"ping: raw sockets need root. Try `sudo ping %s`, or grant the\n"+
					"capability once with `sudo setcap cap_net_raw+ep <binary>`.\n", host)
		}
		return err
	}
	defer syscall.Close(fd)

	// A 1-second receive timeout so a lost reply doesn't hang the loop.
	tv := syscall.Timeval{Sec: 1, Usec: 0}
	_ = syscall.SetsockoptTimeval(fd, syscall.SOL_SOCKET, syscall.SO_RCVTIMEO, &tv)

	dest := &syscall.SockaddrInet4{Addr: [4]byte(ip4)}
	id := uint16(os.Getpid())
	fmt.Printf("PING %s (%s): 56 data bytes\n", host, ip4)

	payload := make([]byte, 56)
	for i := range payload {
		payload[i] = 0x42
	}

	var rtts []float64
	for seq := 1; seq <= count; seq++ {
		packet := BuildEchoRequest(id, uint16(seq), payload)
		sent := time.Now()
		if err := syscall.Sendto(fd, packet, 0, dest); err != nil {
			fmt.Fprintf(os.Stderr, "ping: send failed: %v\n", err)
			continue
		}
		// The kernel hands back the whole IP datagram; skip the IP header.
		buf := make([]byte, 1500)
		n, _, err := syscall.Recvfrom(fd, buf, 0)
		if err != nil {
			fmt.Printf("Request timeout for icmp_seq %d\n", seq)
			continue
		}
		elapsed := time.Since(sent)
		datagram := buf[:n]
		ihl := int(datagram[0]&0x0f) * 4
		if reply, ok := ParseEchoReply(datagram[ihl:]); ok && reply.ID == id {
			ms := float64(elapsed.Nanoseconds()) / 1e6
			rtts = append(rtts, ms)
			fmt.Printf("64 bytes from %s: icmp_seq=%d time=%.3f ms\n", ip4, reply.Seq, ms)
		}
		if seq < count {
			time.Sleep(time.Second)
		}
	}

	fmt.Printf("\n%s\n", Summarize(host, count, rtts))
	return nil
}
