// Package adapters holds concrete implementations of the app-layer ports. It
// depends inward on app/domain; nothing inward depends on it.
package adapters

import (
	"academy.example/hexagonal/domain"
)

// InMemoryRepo is the storage adapter (production would add Postgres here).
type InMemoryRepo struct {
	tasks map[int]domain.Task
	next  int
}

func NewInMemoryRepo() *InMemoryRepo {
	return &InMemoryRepo{tasks: make(map[int]domain.Task)}
}

func (r *InMemoryRepo) NextID() int { return r.next + 1 }

func (r *InMemoryRepo) Save(t domain.Task) {
	if t.ID > r.next {
		r.next = t.ID
	}
	r.tasks[t.ID] = t
}

func (r *InMemoryRepo) Get(id int) (domain.Task, bool) {
	t, ok := r.tasks[id]
	return t, ok
}
