package app

import (
	"errors"
	"testing"

	"academy.example/hexagonal/domain"
)

// A fake repo defined IN THE TEST — the app layer is tested without adapters.
type fakeRepo struct {
	tasks map[int]domain.Task
	next  int
}

func newFakeRepo() *fakeRepo { return &fakeRepo{tasks: make(map[int]domain.Task)} }

func (f *fakeRepo) NextID() int { return f.next + 1 }
func (f *fakeRepo) Save(t domain.Task) {
	if t.ID > f.next {
		f.next = t.ID
	}
	f.tasks[t.ID] = t
}
func (f *fakeRepo) Get(id int) (domain.Task, bool) { t, ok := f.tasks[id]; return t, ok }

func TestAddThenCompleteRoundtrip(t *testing.T) {
	svc := NewService(newFakeRepo())
	id, err := svc.Add("ship part 2")
	if err != nil {
		t.Fatal(err)
	}
	if err := svc.Complete(id); err != nil {
		t.Fatal(err)
	}
	if err := svc.Complete(id); !errors.Is(err, domain.ErrAlreadyDone) {
		t.Errorf("expected ErrAlreadyDone, got %v", err)
	}
}

func TestCompletingUnknownIDIsNotFound(t *testing.T) {
	svc := NewService(newFakeRepo())
	if err := svc.Complete(99); !errors.Is(err, ErrNotFound) {
		t.Errorf("expected ErrNotFound, got %v", err)
	}
}
