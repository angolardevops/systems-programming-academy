// Package app is the middle layer: use cases plus the ports they need. It
// depends only on domain — the port is defined HERE, in the layer that uses
// it, not next to the database.
package app

import (
	"errors"
	"fmt"

	"academy.example/hexagonal/domain"
)

// ErrNotFound: the use case was asked about a task that doesn't exist.
var ErrNotFound = errors.New("task not found")

// TaskRepo is the port the use cases need from storage.
type TaskRepo interface {
	NextID() int
	Save(t domain.Task)
	Get(id int) (domain.Task, bool)
}

// Service holds the use cases, depending on the port.
type Service struct {
	repo TaskRepo
}

func NewService(repo TaskRepo) *Service { return &Service{repo: repo} }

// Add is the use case: create a task; returns its id.
func (s *Service) Add(title string) (int, error) {
	id := s.repo.NextID()
	task, err := domain.NewTask(id, title)
	if err != nil {
		return 0, err
	}
	s.repo.Save(task)
	return id, nil
}

// Complete is the use case: mark a task done by id.
func (s *Service) Complete(id int) error {
	task, ok := s.repo.Get(id)
	if !ok {
		return fmt.Errorf("task %d: %w", id, ErrNotFound)
	}
	if err := task.Complete(); err != nil {
		return err
	}
	s.repo.Save(task)
	return nil
}
