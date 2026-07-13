// Package async shows Go's answer to event-driven I/O: there is no async
// syntax at all. You write blocking code, one goroutine per connection, and
// the runtime's netpoller multiplexes every network wait onto epoll under
// the hood. Goroutines are cheap enough (a few KB of stack) that
// one-per-connection scales where one-OS-thread-per-connection cannot.
package async

import (
	"bufio"
	"io"
	"net"
	"strings"
	"sync"
	"time"
)

// ConcurrentWaits launches n goroutines that each sleep for d, waits for all
// of them, and returns the wall-clock elapsed time — the proof that waits
// overlap: n×d of sleeping costs ~d of wall time.
func ConcurrentWaits(n int, d time.Duration) time.Duration {
	start := time.Now()
	var wg sync.WaitGroup
	for range n {
		wg.Add(1)
		go func() {
			defer wg.Done()
			time.Sleep(d)
		}()
	}
	wg.Wait()
	return time.Since(start)
}

// ServeNEchoes accepts connections on ln, echoing one newline-terminated
// message per connection, until n messages have been served. One goroutine
// per connection — the reads LOOK blocking, but the netpoller parks the
// goroutine on epoll, so idle connections cost no thread.
func ServeNEchoes(ln net.Listener, n int) {
	var wg sync.WaitGroup
	for range n {
		conn, err := ln.Accept()
		if err != nil {
			break
		}
		wg.Add(1)
		go func() {
			defer wg.Done()
			defer conn.Close()
			line, err := bufio.NewReader(conn).ReadString('\n')
			if err != nil {
				return
			}
			io.WriteString(conn, line)
		}()
	}
	wg.Wait()
}

// EchoRoundtrip connects to addr, sends msg (newline-terminated), and
// returns the echoed reply without the newline.
func EchoRoundtrip(addr, msg string) (string, error) {
	conn, err := net.DialTimeout("tcp", addr, 5*time.Second)
	if err != nil {
		return "", err
	}
	defer conn.Close()
	if err := conn.SetDeadline(time.Now().Add(5 * time.Second)); err != nil {
		return "", err
	}
	if _, err := io.WriteString(conn, msg+"\n"); err != nil {
		return "", err
	}
	reply, err := bufio.NewReader(conn).ReadString('\n')
	if err != nil {
		return "", err
	}
	return strings.TrimSuffix(reply, "\n"), nil
}
