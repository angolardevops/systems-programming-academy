package main

// vars — run a small program with variables.
//
// Usage:
//
//	vars "x = 5; y = x * 2; y - x"       one program, statements split on ';'
//	printf 'x = 5\ny = x * 2\n' | vars    or one statement per stdin line
//
// State persists across statements, so later lines can read what earlier lines
// bound. Each statement prints its value, or a clear error.

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func main() {
	arg := strings.TrimSpace(strings.Join(os.Args[1:], " "))
	var src string
	if arg != "" {
		src = strings.ReplaceAll(arg, ";", "\n") // ';' separates statements on the CLI
	} else {
		var b strings.Builder
		sc := bufio.NewScanner(os.Stdin)
		for sc.Scan() {
			b.WriteString(sc.Text())
			b.WriteByte('\n')
		}
		src = b.String()
	}
	for _, line := range RunProgram(src) {
		fmt.Println(line)
	}
}
