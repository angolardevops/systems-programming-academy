package appconfig

import (
	"errors"
	"strings"
	"testing"
)

// mapEnv turns a map into the lookup function Load expects — tests never touch
// the real process environment, so they are deterministic and parallel-safe.
func mapEnv(m map[string]string) func(string) string {
	return func(key string) string { return m[key] }
}

func TestLoadsWithDefaults(t *testing.T) {
	cfg, err := Load(mapEnv(map[string]string{"APP_API_KEY": "s3cret"}))
	if err != nil {
		t.Fatal(err)
	}
	if cfg.Host != "localhost" || cfg.Port != 8080 || cfg.Debug {
		t.Errorf("defaults wrong: %+v", cfg)
	}
}

func TestLoadsExplicitValues(t *testing.T) {
	cfg, err := Load(mapEnv(map[string]string{
		"APP_HOST":    "0.0.0.0",
		"APP_PORT":    "9000",
		"APP_DEBUG":   "true",
		"APP_API_KEY": "k",
	}))
	if err != nil {
		t.Fatal(err)
	}
	if cfg.Host != "0.0.0.0" || cfg.Port != 9000 || !cfg.Debug {
		t.Errorf("explicit values wrong: %+v", cfg)
	}
}

func TestMissingSecretFailsFast(t *testing.T) {
	_, err := Load(mapEnv(map[string]string{}))
	if !errors.Is(err, ErrMissing) {
		t.Errorf("expected ErrMissing, got %v", err)
	}
}

func TestInvalidPortIsTypedError(t *testing.T) {
	_, err := Load(mapEnv(map[string]string{"APP_PORT": "nope", "APP_API_KEY": "k"}))
	if !errors.Is(err, ErrInvalid) {
		t.Errorf("expected ErrInvalid, got %v", err)
	}
}

func TestStringRedactsSecret(t *testing.T) {
	cfg, err := Load(mapEnv(map[string]string{"APP_API_KEY": "hunter2"}))
	if err != nil {
		t.Fatal(err)
	}
	printed := cfg.String()
	if !strings.Contains(printed, "***REDACTED***") {
		t.Errorf("expected redaction marker in %q", printed)
	}
	if strings.Contains(printed, "hunter2") {
		t.Errorf("secret leaked in %q", printed)
	}
}
