package main

import (
	"net"
	"reflect"
	"testing"
	"time"
)

func TestParsesPortsAndRangesSortedUnique(t *testing.T) {
	cases := map[string][]int{
		"80":                   {80},
		"22,80,443":            {22, 80, 443},
		"1-3":                  {1, 2, 3},
		"3-1, 2, 80":           {1, 2, 3, 80},
		"22, oops, 90000, 443": {22, 443},
	}
	for spec, want := range cases {
		if got := ParsePorts(spec); !reflect.DeepEqual(got, want) {
			t.Errorf("ParsePorts(%q) = %v, want %v", spec, got, want)
		}
	}
}

func TestLooksUpWellKnownServices(t *testing.T) {
	for port, want := range map[int]string{22: "ssh", 443: "https", 6379: "redis", 12345: "unknown"} {
		if got := ServiceName(port); got != want {
			t.Errorf("ServiceName(%d) = %q, want %q", port, got, want)
		}
	}
}

func TestOpenPortDetectedAgainstRealListener(t *testing.T) {
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	defer ln.Close()
	port := ln.Addr().(*net.TCPAddr).Port
	if s := ScanPort("127.0.0.1", port, time.Second); s != Open {
		t.Fatalf("expected Open, got %v", s.Label())
	}
}

func TestClosedPortDetectedBindThenClose(t *testing.T) {
	ln, _ := net.Listen("tcp", "127.0.0.1:0")
	port := ln.Addr().(*net.TCPAddr).Port
	ln.Close() // nothing listening now -> connection refused
	if s := ScanPort("127.0.0.1", port, time.Second); s != Closed {
		t.Fatalf("expected Closed, got %v", s.Label())
	}
}

func TestScanAllFindsOpenPortsAmongClosed(t *testing.T) {
	l1, _ := net.Listen("tcp", "127.0.0.1:0")
	l2, _ := net.Listen("tcp", "127.0.0.1:0")
	defer l1.Close()
	defer l2.Close()
	p1 := l1.Addr().(*net.TCPAddr).Port
	p2 := l2.Addr().(*net.TCPAddr).Port
	lc, _ := net.Listen("tcp", "127.0.0.1:0")
	closed := lc.Addr().(*net.TCPAddr).Port
	lc.Close()

	results := ScanAll("127.0.0.1", []int{p1, p2, closed}, time.Second, 8)
	openSet := map[int]bool{}
	closedSeen := false
	for _, r := range results {
		if r.State == Open {
			openSet[r.Port] = true
		}
		if r.Port == closed && r.State == Closed {
			closedSeen = true
		}
	}
	if !openSet[p1] || !openSet[p2] || len(openSet) != 2 {
		t.Fatalf("open set wrong: %v", openSet)
	}
	if !closedSeen {
		t.Fatal("closed port not reported closed")
	}
}

func TestRendersTheTable(t *testing.T) {
	rows := []Result{{22, Open}, {80, Open}, {443, Open}}
	got := RenderTable(rows)
	want := "PORT      STATE     SERVICE\n" +
		"22/tcp    open      ssh\n" +
		"80/tcp    open      http\n" +
		"443/tcp   open      https"
	if got != want {
		t.Fatalf("table\n got:\n%s\n want:\n%s", got, want)
	}
}
