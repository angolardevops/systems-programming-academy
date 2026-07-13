package async

import (
	"fmt"
	"net"
	"testing"
	"time"
)

func startServer(t *testing.T, n int) (string, chan struct{}) {
	t.Helper()
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("listen: %v", err)
	}
	done := make(chan struct{})
	go func() {
		ServeNEchoes(ln, n)
		ln.Close()
		close(done)
	}()
	return ln.Addr().String(), done
}

func TestEchoRoundtrip(t *testing.T) {
	addr, done := startServer(t, 1)
	got, err := EchoRoundtrip(addr, "hello")
	if err != nil {
		t.Fatalf("roundtrip: %v", err)
	}
	if got != "hello" {
		t.Fatalf("EchoRoundtrip = %q, want %q", got, "hello")
	}
	<-done
}

func TestManyClients(t *testing.T) {
	const n = 5
	addr, done := startServer(t, n)
	for i := range n {
		msg := fmt.Sprintf("client %d", i)
		got, err := EchoRoundtrip(addr, msg)
		if err != nil {
			t.Fatalf("roundtrip %d: %v", i, err)
		}
		if got != msg {
			t.Fatalf("EchoRoundtrip = %q, want %q", got, msg)
		}
	}
	<-done
}

func TestConcurrentWaitsOverlap(t *testing.T) {
	// 1000 goroutines sleeping 50ms each: ~50s of sleeping, ~50ms of wall
	// time. The generous bound keeps the test robust on loaded machines.
	elapsed := ConcurrentWaits(1000, 50*time.Millisecond)
	if elapsed > 500*time.Millisecond {
		t.Fatalf("1000 concurrent 50ms waits took %v, want < 500ms", elapsed)
	}
}

func BenchmarkConcurrentWaits10k(b *testing.B) {
	for i := 0; i < b.N; i++ {
		ConcurrentWaits(10_000, 50*time.Millisecond)
	}
}

func BenchmarkConcurrentWaits100k(b *testing.B) {
	for i := 0; i < b.N; i++ {
		ConcurrentWaits(100_000, 50*time.Millisecond)
	}
}
