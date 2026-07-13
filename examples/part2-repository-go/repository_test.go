package repository

import (
	"errors"
	"reflect"
	"testing"
)

// The service depends on the interface, so the test injects the in-memory
// adapter — no database, fully deterministic.
func newService() *UserService {
	return NewUserService(NewInMemoryUserRepo())
}

func TestRegistersAndListsSorted(t *testing.T) {
	svc := newService()
	if err := svc.Register(2, "Grace"); err != nil {
		t.Fatal(err)
	}
	if err := svc.Register(1, "Ada"); err != nil {
		t.Fatal(err)
	}
	got := svc.ListNames()
	want := []string{"Ada", "Grace"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("ListNames() = %v, want %v", got, want)
	}
}

func TestRejectsDuplicateID(t *testing.T) {
	svc := newService()
	if err := svc.Register(1, "Ada"); err != nil {
		t.Fatal(err)
	}
	err := svc.Register(1, "Someone")
	if !errors.Is(err, ErrDuplicateID) {
		t.Errorf("expected ErrDuplicateID, got %v", err)
	}
}

func TestRepositoryGetAndAll(t *testing.T) {
	repo := NewInMemoryUserRepo()
	if err := repo.Add(User{ID: 1, Name: "Ada"}); err != nil {
		t.Fatal(err)
	}
	if u, ok := repo.Get(1); !ok || u.Name != "Ada" {
		t.Errorf("Get(1) = %v, %v; want Ada, true", u, ok)
	}
	if _, ok := repo.Get(2); ok {
		t.Error("Get(2) should be false")
	}
	if len(repo.All()) != 1 {
		t.Errorf("All() len = %d, want 1", len(repo.All()))
	}
}
