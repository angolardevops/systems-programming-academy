package main

import (
	"strings"
	"testing"
	"unicode/utf8"
)

func TestParsesScanWithHostAndPorts(t *testing.T) {
	got, err := ParseCommand([]string{"scan", "example.com", "1-1024"})
	if err != nil || got != (Command{Kind: "scan", Host: "example.com", Ports: "1-1024"}) {
		t.Errorf("ParseCommand scan = %+v, %v", got, err)
	}
}

func TestParsesPingAndTraceWithDefaultsAndOverrides(t *testing.T) {
	if c, _ := ParseCommand([]string{"ping", "h"}); c.Count != 4 || c.Kind != "ping" {
		t.Errorf("ping default = %+v", c)
	}
	if c, _ := ParseCommand([]string{"ping", "h", "7"}); c.Count != 7 {
		t.Errorf("ping override = %+v", c)
	}
	if c, _ := ParseCommand([]string{"trace", "h"}); c.MaxHops != 30 || c.Kind != "trace" {
		t.Errorf("trace default = %+v", c)
	}
}

func TestRejectsUnknownAndMissingArguments(t *testing.T) {
	if _, err := ParseCommand([]string{}); err == nil {
		t.Error("empty args should error")
	}
	if _, err := ParseCommand([]string{"bogus"}); err == nil || !strings.Contains(err.Error(), "unknown command 'bogus'") {
		t.Errorf("bogus should error with unknown: %v", err)
	}
	if _, err := ParseCommand([]string{"scan", "h"}); err == nil {
		t.Error("scan missing ports should error")
	}
}

func TestUsageListsAllThreeSubcommands(t *testing.T) {
	u := Usage()
	for _, want := range []string{"netdiag scan", "netdiag ping", "netdiag trace"} {
		if !strings.Contains(u, want) {
			t.Errorf("usage missing %q", want)
		}
	}
}

func TestRendersTheBanner(t *testing.T) {
	lines := strings.Split(Banner("scan example.com"), "\n")
	if len(lines) != 3 {
		t.Fatalf("banner lines = %d, want 3", len(lines))
	}
	if lines[0] != "╔"+strings.Repeat("═", 46)+"╗" {
		t.Errorf("top border wrong: %q", lines[0])
	}
	if lines[2] != "╚"+strings.Repeat("═", 46)+"╝" {
		t.Errorf("bottom border wrong: %q", lines[2])
	}
	if !strings.HasPrefix(lines[1], "║  netdiag :: scan example.com") || !strings.HasSuffix(lines[1], "║") {
		t.Errorf("content line wrong: %q", lines[1])
	}
	if n := utf8.RuneCountInString(lines[1]); n != 48 {
		t.Errorf("content width = %d, want 48", n)
	}
}

func TestRendersASectionRule(t *testing.T) {
	rule := Section("open ports")
	if !strings.HasPrefix(rule, "── open ports ") {
		t.Errorf("prefix wrong: %q", rule)
	}
	if n := utf8.RuneCountInString(rule); n != 48 {
		t.Errorf("rule width = %d, want 48", n)
	}
	if !strings.HasSuffix(rule, "─────") {
		t.Errorf("rule tail wrong: %q", rule)
	}
}
