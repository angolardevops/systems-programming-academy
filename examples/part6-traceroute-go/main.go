package main

// traceroute — trace the path to a host by stepping the IP TTL.
//
// Usage: traceroute <host> [max_hops]   (needs root or CAP_NET_RAW)
//
// The probe building, checksum, ICMP classification, and rendering live in
// traceroute.go and are fully tested without privileges. Only this file needs a
// raw socket — to set the outgoing TTL and read the ICMP replies. Run with
// `sudo`, or grant the capability once:
// `sudo setcap cap_net_raw+ep ./part6-traceroute-go`.
//
// Run: `sudo go run . 1.1.1.1 30`

import (
	"errors"
	"fmt"
	"net"
	"os"
	"strconv"
	"syscall"
	"time"
)

const probesPerHop = 3

func main() {
	host := "127.0.0.1"
	if len(os.Args) > 1 {
		host = os.Args[1]
	}
	maxHops := 30
	if len(os.Args) > 2 {
		if n, err := strconv.Atoi(os.Args[2]); err == nil {
			maxHops = n
		}
	}
	if err := run(host, maxHops); err != nil {
		fmt.Fprintf(os.Stderr, "traceroute: %v\n", err)
		os.Exit(1)
	}
}

func run(host string, maxHops int) error {
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

	fd, err := syscall.Socket(syscall.AF_INET, syscall.SOCK_RAW, syscall.IPPROTO_ICMP)
	if err != nil {
		if errors.Is(err, syscall.EPERM) || errors.Is(err, syscall.EACCES) {
			fmt.Fprintf(os.Stderr,
				"traceroute: raw sockets need root. Try `sudo traceroute %s`, or grant\n"+
					"the capability once with `sudo setcap cap_net_raw+ep <binary>`.\n", host)
		}
		return err
	}
	defer syscall.Close(fd)

	tv := syscall.Timeval{Sec: 1, Usec: 0}
	_ = syscall.SetsockoptTimeval(fd, syscall.SOL_SOCKET, syscall.SO_RCVTIMEO, &tv)

	dest := &syscall.SockaddrInet4{Addr: [4]byte(ip4)}
	id := uint16(os.Getpid())
	fmt.Println(RenderHeader(host, ip4.String(), maxHops))

	payload := make([]byte, 32)
	for ttl := 1; ttl <= maxHops; ttl++ {
		// The one extra socket option that makes traceroute out of ping.
		_ = syscall.SetsockoptInt(fd, syscall.IPPROTO_IP, syscall.IP_TTL, ttl)

		addr := ""
		probes := make([]Probe, 0, probesPerHop)
		reached := false

		for p := 0; p < probesPerHop; p++ {
			seq := uint16(ttl*probesPerHop + p)
			packet := BuildEchoRequest(id, seq, payload)
			sent := time.Now()
			if err := syscall.Sendto(fd, packet, 0, dest); err != nil {
				probes = append(probes, Probe{OK: false})
				continue
			}
			buf := make([]byte, 1500)
			n, from, err := syscall.Recvfrom(fd, buf, 0)
			if err != nil {
				probes = append(probes, Probe{OK: false}) // timeout -> '*'
				continue
			}
			elapsed := float64(time.Since(sent).Nanoseconds()) / 1e6
			datagram := buf[:n]
			ihl := int(datagram[0]&0x0f) * 4
			reply, ok := Classify(datagram[ihl:])
			if !ok || reply.ID != id {
				probes = append(probes, Probe{OK: false})
				continue
			}
			if sa, ok := from.(*syscall.SockaddrInet4); ok && addr == "" {
				addr = net.IP(sa.Addr[:]).String()
			}
			probes = append(probes, Probe{RTT: elapsed, OK: true})
			if reply.Kind == EchoReply {
				reached = true
			}
		}

		fmt.Println(RenderHop(ttl, addr, probes))
		if reached {
			break // the destination itself answered
		}
	}
	return nil
}
