package main

// fns — run a small program with functions and closures.
//
// Usage:
//
//	fns "double(x) = x * 2; double(21)"       statements split on ';'
//	printf 'inc(n) = n + 1\ninc(41)\n' | fns   or one statement per stdin line

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
		src = strings.ReplaceAll(arg, ";", "\n")
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
