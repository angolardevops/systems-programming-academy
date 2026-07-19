// Package main — netdiag: the Part 6 capstone, one CLI over the five tools.
//
// Every tool in Part 6 grew its own main. A real network engineer wants them
// under one command with a consistent look: `netdiag scan`, `netdiag ping`,
// `netdiag trace`. This file is the new part unifying them requires — the
// command layer: parse a subcommand and its arguments into a Command, and render
// every tool's output through one shared report format (a boxed banner and
// section headers). Both are pure and byte-identical across the three languages.
// Library and command share one package so `go run .` and `go test` both work.
package main

import (
	"fmt"
	"strings"
	"unicode/utf8"
)

// Command is a parsed subcommand and its arguments. Kind is "scan", "ping", or
// "trace".
type Command struct {
	Kind    string
	Host    string
	Ports   string
	Count   int
	MaxHops int
}

// Usage is the help text — one line per subcommand.
func Usage() string {
	return `netdiag — network diagnostics

usage:
  netdiag scan  <host> <ports>     TCP connect scan (e.g. 1-1024)
  netdiag ping  <host> [count]     ICMP echo, default 4 probes
  netdiag trace <host> [max_hops]  path trace, default 30 hops`
}

// ParseCommand parses argv-after-the-program-name into a Command, or returns an
// error carrying the usage text on an unknown subcommand or missing argument.
func ParseCommand(args []string) (Command, error) {
	if len(args) == 0 {
		return Command{}, fmt.Errorf("%s", Usage())
	}
	switch args[0] {
	case "scan":
		if len(args) < 3 {
			return Command{}, fmt.Errorf("%s", Usage())
		}
		return Command{Kind: "scan", Host: args[1], Ports: args[2]}, nil
	case "ping":
		if len(args) < 2 {
			return Command{}, fmt.Errorf("%s", Usage())
		}
		return Command{Kind: "ping", Host: args[1], Count: intArg(args, 2, 4)}, nil
	case "trace":
		if len(args) < 2 {
			return Command{}, fmt.Errorf("%s", Usage())
		}
		return Command{Kind: "trace", Host: args[1], MaxHops: intArg(args, 2, 30)}, nil
	default:
		return Command{}, fmt.Errorf("netdiag: unknown command '%s'\n\n%s", args[0], Usage())
	}
}

func intArg(args []string, i, fallback int) int {
	if i < len(args) {
		var n int
		if _, err := fmt.Sscanf(args[i], "%d", &n); err == nil {
			return n
		}
	}
	return fallback
}

const width = 46

// Banner is a boxed banner atop each report.
func Banner(title string) string {
	bar := strings.Repeat("═", width)
	content := "  netdiag :: " + title
	pad := width - utf8.RuneCountInString(content)
	if pad < 0 {
		pad = 0
	}
	return fmt.Sprintf("╔%s╗\n║%s%s║\n╚%s╝", bar, content, strings.Repeat(" ", pad), bar)
}

// Section is a section header rule: `── open ports ─────…` to a fixed width.
func Section(title string) string {
	prefix := "── " + title + " "
	fill := (width + 2) - utf8.RuneCountInString(prefix)
	if fill < 0 {
		fill = 0
	}
	return prefix + strings.Repeat("─", fill)
}
