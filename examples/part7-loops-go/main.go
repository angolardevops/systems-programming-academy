package main

// loops — run an imperative program (statements, print, while loops).
//
// Usage:
//
//	loops "i = 1; while i <= 5 do { print i; i = i + 1 }"
//	cat program.txt | loops

import (
	"fmt"
	"io"
	"os"
	"strings"
)

func main() {
	arg := strings.TrimSpace(strings.Join(os.Args[1:], " "))
	src := arg
	if src == "" {
		b, _ := io.ReadAll(os.Stdin)
		src = string(b)
	}
	for _, line := range RunProgram(src) {
		fmt.Println(line)
	}
}
