package main

// calc — a REPL/one-shot for the integer arithmetic interpreter.
//
// Usage:
//
//	calc "1 + 2 * 3"          evaluate one expression from the argument
//	echo "2 * (3+4)" | calc   evaluate each line read from stdin
//
// For each expression it prints the parsed syntax tree (as an S-expression) and
// the value — or a clear error, never a crash.

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func main() {
	arg := strings.TrimSpace(strings.Join(os.Args[1:], " "))
	if arg != "" {
		report(arg)
		return
	}
	sc := bufio.NewScanner(os.Stdin)
	for sc.Scan() {
		if line := sc.Text(); strings.TrimSpace(line) != "" {
			report(line)
		}
	}
}

func report(src string) {
	sexp, value, err := Run(src)
	if err != nil {
		fmt.Printf("%s  =>  error: %s\n", src, err)
		return
	}
	fmt.Printf("%s  =>  %s  =>  %d\n", src, sexp, value)
}
