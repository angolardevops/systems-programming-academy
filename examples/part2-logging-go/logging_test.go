package structlog

import (
	"bytes"
	"encoding/json"
	"log/slog"
	"strings"
	"testing"
)

// decode parses one JSON log line into a map for assertions.
func decode(t *testing.T, line string) map[string]any {
	t.Helper()
	var m map[string]any
	if err := json.Unmarshal([]byte(line), &m); err != nil {
		t.Fatalf("bad JSON line %q: %v", line, err)
	}
	return m
}

func TestEmitsStructuredJSON(t *testing.T) {
	var buf bytes.Buffer
	logger := New(&buf, slog.LevelInfo)

	HandleLogin(logger, 42, true)

	m := decode(t, strings.TrimSpace(buf.String()))
	if m["msg"] != "user logged in" || m["level"] != "INFO" {
		t.Errorf("unexpected line: %v", m)
	}
	if m["user_id"] != float64(42) { // JSON numbers decode as float64
		t.Errorf("user_id = %v, want 42", m["user_id"])
	}
}

func TestLevelFiltering(t *testing.T) {
	var buf bytes.Buffer
	logger := New(&buf, slog.LevelWarn) // Info is below the threshold

	logger.Info("noise")
	logger.Warn("kept")

	lines := strings.Split(strings.TrimSpace(buf.String()), "\n")
	if len(lines) != 1 {
		t.Fatalf("expected 1 line, got %d: %v", len(lines), lines)
	}
	if m := decode(t, lines[0]); m["msg"] != "kept" {
		t.Errorf("wrong surviving line: %v", m)
	}
}

func TestContextFieldOnEveryLine(t *testing.T) {
	var buf bytes.Buffer
	logger := WithRequestID(New(&buf, slog.LevelInfo), "abc-123")

	logger.Info("start")
	logger.Warn("slow query", "ms", 250)

	lines := strings.Split(strings.TrimSpace(buf.String()), "\n")
	if len(lines) != 2 {
		t.Fatalf("expected 2 lines, got %d", len(lines))
	}
	for _, line := range lines {
		if m := decode(t, line); m["request_id"] != "abc-123" {
			t.Errorf("line missing request_id: %v", m)
		}
	}
	if m := decode(t, lines[1]); m["ms"] != float64(250) {
		t.Errorf("ms = %v, want 250", m["ms"])
	}
}
