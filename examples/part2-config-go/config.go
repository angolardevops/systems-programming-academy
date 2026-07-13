// Package appconfig is the Go companion for the Part 2 lesson "Configuration &
// Secrets". The same design is implemented in Rust, Go, and Python.
//
// Principles: parse env vars into a typed struct once at startup (fail fast);
// inject the lookup function so tests never mutate the real environment; redact
// secrets in the String representation so logs can't leak them.
//
//	go test ./...
package appconfig

import (
	"errors"
	"fmt"
	"strconv"
)

// Config is the typed application configuration.
type Config struct {
	Host   string
	Port   int
	Debug  bool
	APIKey string // secret: never printed in full
}

// ErrMissing is wrapped by Load when a required variable is absent.
var ErrMissing = errors.New("missing required env var")

// ErrInvalid is wrapped by Load when a variable cannot be parsed.
var ErrInvalid = errors.New("invalid env var value")

// String implements fmt.Stringer and redacts the secret, so %v/%s in logs are
// safe by default.
func (c Config) String() string {
	return fmt.Sprintf(
		"Config{Host:%s Port:%d Debug:%t APIKey:***REDACTED***}",
		c.Host, c.Port, c.Debug,
	)
}

// Load builds a Config from an injected lookup function (in production pass
// os.Getenv; in tests pass a map lookup). Optional vars get defaults; required
// ones fail fast with a wrapped sentinel error.
func Load(getenv func(string) string) (Config, error) {
	cfg := Config{Host: "localhost", Port: 8080} // defaults

	if host := getenv("APP_HOST"); host != "" {
		cfg.Host = host
	}

	if raw := getenv("APP_PORT"); raw != "" {
		port, err := strconv.Atoi(raw)
		if err != nil {
			return Config{}, fmt.Errorf("APP_PORT=%q: %w", raw, ErrInvalid)
		}
		cfg.Port = port
	}

	debug := getenv("APP_DEBUG")
	cfg.Debug = debug == "1" || debug == "true"

	key := getenv("APP_API_KEY")
	if key == "" {
		return Config{}, fmt.Errorf("APP_API_KEY: %w", ErrMissing)
	}
	cfg.APIKey = key

	return cfg, nil
}
