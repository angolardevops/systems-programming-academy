package domain

import (
	"errors"
	"testing"
)

// Domain rules tested with zero infrastructure — the payoff of a pure core.

func TestTaskRequiresTitle(t *testing.T) {
	if _, err := NewTask(1, "   "); !errors.Is(err, ErrEmptyTitle) {
		t.Errorf("expected ErrEmptyTitle, got %v", err)
	}
}

func TestCompletingTwiceIsAnError(t *testing.T) {
	task, _ := NewTask(1, "write lesson")
	if err := task.Complete(); err != nil {
		t.Fatal(err)
	}
	if err := task.Complete(); !errors.Is(err, ErrAlreadyDone) {
		t.Errorf("expected ErrAlreadyDone, got %v", err)
	}
}
