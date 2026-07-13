// Package domain is the innermost layer: entities and business rules. It
// imports NOTHING from the outer layers — the Dependency Rule, enforced by Go's
// package system (an inward import of app/adapters would be a compile error
// once those import domain: import cycles are forbidden).
package domain

import (
	"errors"
	"strings"
)

var (
	// ErrEmptyTitle: a task must have a non-empty title.
	ErrEmptyTitle = errors.New("a task needs a title")
	// ErrAlreadyDone: completing twice is an error, not a no-op.
	ErrAlreadyDone = errors.New("task already done")
)

// Task is the entity; its invariants live here, next to its data.
type Task struct {
	ID    int
	Title string
	Done  bool
}

// NewTask enforces the title rule at construction.
func NewTask(id int, title string) (Task, error) {
	title = strings.TrimSpace(title)
	if title == "" {
		return Task{}, ErrEmptyTitle
	}
	return Task{ID: id, Title: title}, nil
}

// Complete enforces the no-double-completion rule.
func (t *Task) Complete() error {
	if t.Done {
		return ErrAlreadyDone
	}
	t.Done = true
	return nil
}
