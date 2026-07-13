// Package usersdemo is the tested companion code for the Academy lesson
// "Go: Error Handling". It shows Go's model — errors are ordinary values
// returned alongside results — plus sentinel errors, custom error types,
// wrapping with %w, and errors.Is / errors.As.
//
// Run the tests with:
//
//	go test ./...
package usersdemo

import (
	"errors"
	"fmt"
	"strconv"
	"strings"
)

// ErrNotFound is a sentinel error: a package-level value callers can compare
// against with errors.Is to detect a specific condition.
var ErrNotFound = errors.New("user not found")

// ValidationError is a custom error type carrying structured detail. Because it
// has an Error() string method, it satisfies the built-in error interface.
type ValidationError struct {
	Field  string
	Reason string
}

// Error makes ValidationError satisfy the error interface.
func (e *ValidationError) Error() string {
	return fmt.Sprintf("invalid %s: %s", e.Field, e.Reason)
}

// User is a parsed record.
type User struct {
	Name string
	Age  int
}

// ParseUser parses a "name,age" line. It returns a typed *ValidationError for
// bad shape or empty name, and wraps strconv's error (with %w) for a bad age so
// callers can still inspect the underlying cause.
func ParseUser(line string) (User, error) {
	parts := strings.Split(line, ",")
	if len(parts) != 2 {
		return User{}, &ValidationError{Field: "line", Reason: "expected exactly one comma"}
	}

	name := strings.TrimSpace(parts[0])
	if name == "" {
		return User{}, &ValidationError{Field: "name", Reason: "must not be empty"}
	}

	age, err := strconv.Atoi(strings.TrimSpace(parts[1]))
	if err != nil {
		// %w wraps err so errors.Is/As can reach it; the message adds context.
		return User{}, fmt.Errorf("parsing age: %w", err)
	}

	return User{Name: name, Age: age}, nil
}

// directory is a tiny in-memory store for the lookup example.
var directory = map[string]string{
	"1": "Ada",
	"2": "Grace",
}

// Lookup returns the name for an id, or ErrNotFound (the sentinel) if absent.
func Lookup(id string) (string, error) {
	name, ok := directory[id]
	if !ok {
		// Wrap the sentinel with context; errors.Is still matches ErrNotFound.
		return "", fmt.Errorf("lookup id %q: %w", id, ErrNotFound)
	}
	return name, nil
}

// FirstValidAge parses each line and returns the first user's age, stopping at
// the first error — the fail-fast style, using early returns instead of nesting.
func FirstValidAge(lines []string) (int, error) {
	for _, line := range lines {
		u, err := ParseUser(line)
		if err != nil {
			return 0, fmt.Errorf("line %q: %w", line, err)
		}
		return u.Age, nil
	}
	return 0, errors.New("no lines provided")
}
