package main

import (
	"net"
	"testing"
	"time"
)

// upAddr starts a real listener and returns its address (deterministic "up").
func upAddr(t *testing.T) string {
	t.Helper()
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { l.Close() })
	go func() { // accept and close, keeps the listener serving
		for {
			c, err := l.Accept()
			if err != nil {
				return
			}
			c.Close()
		}
	}()
	return l.Addr().String()
}

// downAddr binds then closes a listener: a deterministic refused port.
func downAddr(t *testing.T) string {
	t.Helper()
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	addr := l.Addr().String()
	l.Close()
	return addr
}

func TestParseTargets(t *testing.T) {
	targets := ParseTargets("# fleet\napi = 127.0.0.1:8080\n\nweb = 10.0.0.2:80\n")
	if len(targets) != 2 || targets[0].Name != "api" || targets[0].Addr != "127.0.0.1:8080" {
		t.Errorf("ParseTargets = %+v", targets)
	}
}

func TestProbeUpAndDown(t *testing.T) {
	if !ProbeAddr(upAddr(t), 500*time.Millisecond) {
		t.Error("expected up for real listener")
	}
	if ProbeAddr(downAddr(t), 500*time.Millisecond) {
		t.Error("expected down for closed port")
	}
}

func TestCheckAllParallelAndSorted(t *testing.T) {
	up := upAddr(t)
	targets := []Target{
		{Name: "web", Addr: up},
		{Name: "api", Addr: up},
		{Name: "cache", Addr: downAddr(t)},
	}
	probes := CheckAll(targets, 500*time.Millisecond)
	if probes[0].Name != "api" || probes[1].Name != "cache" || probes[2].Name != "web" {
		t.Errorf("not sorted: %+v", probes)
	}
	if !probes[0].Up || probes[1].Up || !probes[2].Up {
		t.Errorf("wrong statuses: %+v", probes)
	}
}

func TestReportAndExitCode(t *testing.T) {
	text, code := Report([]Probe{{Name: "api", Up: true}, {Name: "cache", Up: false}})
	if text != "UP api\nDOWN cache\n---\n1 up, 1 down\n" || code != 1 {
		t.Errorf("Report = %q, %d", text, code)
	}
	if _, allUp := Report([]Probe{{Name: "api", Up: true}}); allUp != 0 {
		t.Errorf("all-up exit code = %d, want 0", allUp)
	}
}
