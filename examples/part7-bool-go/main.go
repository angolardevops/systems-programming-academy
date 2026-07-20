package main

// cond — run a small program with conditionals, comparisons, and recursion.
//
// Usage:
//
//	cond "fact(n) = if n <= 1 then 1 else n * fact(n - 1); fact(6)"
//	printf 'x = 5\nif x < 3 then 1 else 2\n' | cond

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
