package main

import (
	"math"
	"strings"
	"testing"
)

const line = `203.0.113.9 - - [12/Jul/2026:10:00:00] "GET /api/users HTTP/1.1" 200 512`

func TestParsesValidLine(t *testing.T) {
	entry, ok := ParseLine(line)
	if !ok || entry.Path != "/api/users" || entry.Status != 200 {
		t.Fatalf("ParseLine = %+v, %v", entry, ok)
	}
}

func TestMalformedLines(t *testing.T) {
	for _, bad := range []string{
		"not a log line",
		`x "GET /a HTTP/1.1" banana 1`,
		`x "GET /a HTTP/1.1" 999999 1`,
	} {
		if _, ok := ParseLine(bad); ok {
			t.Errorf("expected malformed: %q", bad)
		}
	}
}

func TestStatsAggregate(t *testing.T) {
	s := NewStats()
	s.AddLine(line)
	s.AddLine(`x - - [t] "GET /a HTTP/1.1" 404 0`)
	s.AddLine(`x - - [t] "GET /a HTTP/1.1" 500 0`)
	s.AddLine("garbage")
	if s.Total != 3 || s.Malformed != 1 {
		t.Errorf("total/malformed = %d/%d, want 3/1", s.Total, s.Malformed)
	}
	if s.ByClass != [5]uint64{0, 1, 0, 1, 1} {
		t.Errorf("ByClass = %v", s.ByClass)
	}
	if math.Abs(s.ErrorRate()-66.7) > 0.1 {
		t.Errorf("ErrorRate = %v", s.ErrorRate())
	}
}

func TestTopPathsSortedDescThenAlpha(t *testing.T) {
	s := NewStats()
	for _, p := range []string{"/b", "/a", "/b", "/c", "/a"} {
		s.AddLine(`x - - [t] "GET ` + p + ` HTTP/1.1" 200 0`)
	}
	top := s.TopPaths(2)
	if top[0].Path != "/a" || top[1].Path != "/b" {
		t.Errorf("TopPaths = %v", top)
	}
}

func TestReportSharedFormat(t *testing.T) {
	out := Report(line + "\ngarbage\n")
	if !strings.HasPrefix(out, "total: 1\n") ||
		!strings.Contains(out, "malformed: 1\n") ||
		!strings.Contains(out, "  /api/users: 1\n") {
		t.Errorf("unexpected report:\n%s", out)
	}
}
