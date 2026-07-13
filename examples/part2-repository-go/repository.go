// Package repository is the Go companion for the Part 2 lesson "Repository
// Pattern & Dependency Injection". The same domain is implemented in Rust, Go,
// and Python for direct comparison.
//
// Layers: User (domain), UserRepository (port, an interface), InMemoryUserRepo
// (adapter), UserService (business logic depending on the interface).
//
//	go test ./...
package repository

import (
	"errors"
	"fmt"
	"sort"
)

// User is the domain entity.
type User struct {
	ID   int
	Name string
}

// ErrDuplicateID is returned when adding a user whose id already exists.
var ErrDuplicateID = errors.New("duplicate user id")

// UserRepository is the port: what the service needs from storage, as an
// interface. Any type with these methods can back the service (duck typing at
// the type level).
type UserRepository interface {
	Add(u User) error
	Get(id int) (User, bool)
	All() []User
}

// InMemoryUserRepo is an adapter — a map-backed implementation for tests/demos.
type InMemoryUserRepo struct {
	users map[int]User
}

// NewInMemoryUserRepo constructs an empty in-memory repository.
func NewInMemoryUserRepo() *InMemoryUserRepo {
	return &InMemoryUserRepo{users: make(map[int]User)}
}

func (r *InMemoryUserRepo) Add(u User) error {
	if _, exists := r.users[u.ID]; exists {
		return fmt.Errorf("add user %d: %w", u.ID, ErrDuplicateID)
	}
	r.users[u.ID] = u
	return nil
}

func (r *InMemoryUserRepo) Get(id int) (User, bool) {
	u, ok := r.users[id]
	return u, ok
}

func (r *InMemoryUserRepo) All() []User {
	out := make([]User, 0, len(r.users))
	for _, u := range r.users {
		out = append(out, u)
	}
	return out
}

// UserService holds the business logic and depends on the UserRepository
// interface — dependency injection through the constructor. A test injects the
// in-memory adapter; production would inject a Postgres one.
type UserService struct {
	repo UserRepository
}

// NewUserService injects the repository dependency.
func NewUserService(repo UserRepository) *UserService {
	return &UserService{repo: repo}
}

// Register adds a user, rejecting a duplicate id.
func (s *UserService) Register(id int, name string) error {
	return s.repo.Add(User{ID: id, Name: name})
}

// ListNames returns all names, sorted for a deterministic result.
func (s *UserService) ListNames() []string {
	names := make([]string, 0)
	for _, u := range s.repo.All() {
		names = append(names, u.Name)
	}
	sort.Strings(names)
	return names
}
