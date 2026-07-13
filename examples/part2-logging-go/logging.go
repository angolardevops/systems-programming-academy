// Package structlog is the Go companion for the Part 2 lesson "Logging &
// Observability". Go's answer is in the standard library: log/slog provides
// levelled, structured logging with pluggable handlers.
//
// Principles: structured key-value lines, level filtering, an injected
// io.Writer sink so tests capture output, and context fields bound with
// logger.With(...) that appear on every subsequent line.
//
//	go test ./...
package structlog

import (
	"io"
	"log/slog"
)

// New builds a JSON slog.Logger writing to the injected sink at the given
// minimum level. For deterministic tests we strip the time attribute; a real
// deployment keeps it.
func New(sink io.Writer, level slog.Level) *slog.Logger {
	handler := slog.NewJSONHandler(sink, &slog.HandlerOptions{
		Level: level,
		ReplaceAttr: func(groups []string, a slog.Attr) slog.Attr {
			if a.Key == slog.TimeKey {
				return slog.Attr{} // drop time for reproducible output
			}
			return a
		},
	})
	return slog.New(handler)
}

// WithRequestID binds a request_id context field: every line logged through the
// returned logger carries it automatically.
func WithRequestID(logger *slog.Logger, id string) *slog.Logger {
	return logger.With("request_id", id)
}

// HandleLogin is a tiny "business" function that logs structured events —
// used by the tests to show fields flowing through.
func HandleLogin(logger *slog.Logger, userID int, ok bool) {
	if ok {
		logger.Info("user logged in", "user_id", userID)
		return
	}
	logger.Warn("login failed", "user_id", userID)
}
