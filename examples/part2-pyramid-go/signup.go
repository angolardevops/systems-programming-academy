// Package signup is the Go companion for the Part 2 lesson "The Testing
// Pyramid". The same feature is implemented in Rust, Go, and Python, with
// tests at all three levels: unit (pure validator), integration (service +
// real in-memory adapters), and end-to-end (the App composition root).
//
//	go test ./...
package signup

import (
	"errors"
	"fmt"
	"strings"
)

// ---------------------------------------------------------------- validation

var (
	ErrEmpty      = errors.New("email is empty")
	ErrBadFormat  = errors.New("email format is invalid")
	ErrDuplicate  = errors.New("email already registered")
	ErrWhitespace = errors.New("email contains whitespace")
)

// ValidateEmail is pure — the base of the pyramid: no I/O, instant tests.
func ValidateEmail(email string) error {
	if email == "" {
		return ErrEmpty
	}
	if strings.ContainsAny(email, " \t\n") {
		return ErrWhitespace
	}
	parts := strings.Split(email, "@")
	if len(parts) != 2 || parts[0] == "" || parts[1] == "" || !strings.Contains(parts[1], ".") {
		return ErrBadFormat
	}
	return nil
}

// ------------------------------------------------------------------- service

// UserRepo is the storage port.
type UserRepo interface {
	Exists(email string) bool
	Save(email string)
}

// Notifier is the notification port.
type Notifier interface {
	SendWelcome(email string)
}

// InMemoryRepo is the storage adapter used in tests and this demo.
type InMemoryRepo struct{ emails map[string]bool }

func NewInMemoryRepo() *InMemoryRepo { return &InMemoryRepo{emails: make(map[string]bool)} }

func (r *InMemoryRepo) Exists(email string) bool { return r.emails[email] }
func (r *InMemoryRepo) Save(email string)        { r.emails[email] = true }

// RecordingNotifier records welcomes (a real one would talk SMTP).
type RecordingNotifier struct{ Sent []string }

func (n *RecordingNotifier) SendWelcome(email string) { n.Sent = append(n.Sent, email) }

// Service is the middle of the pyramid: logic coordinating the two ports.
type Service struct {
	repo     UserRepo
	notifier Notifier
}

func NewService(repo UserRepo, notifier Notifier) *Service {
	return &Service{repo: repo, notifier: notifier}
}

func (s *Service) Signup(email string) error {
	if err := ValidateEmail(email); err != nil {
		return err
	}
	if s.repo.Exists(email) {
		return fmt.Errorf("%s: %w", email, ErrDuplicate)
	}
	s.repo.Save(email)
	s.notifier.SendWelcome(email)
	return nil
}

// ----------------------------------------------------------------------- app

// App is the top of the pyramid: the composition root a binary would call,
// returning the user-visible message.
type App struct{ service *Service }

func NewApp() *App {
	return &App{service: NewService(NewInMemoryRepo(), &RecordingNotifier{})}
}

func (a *App) Signup(email string) string {
	err := a.service.Signup(email)
	switch {
	case err == nil:
		return fmt.Sprintf("Welcome, %s! Check your inbox.", email)
	case errors.Is(err, ErrDuplicate):
		return fmt.Sprintf("%s is already registered.", email)
	default:
		return fmt.Sprintf("'%s' is not a valid email address.", email)
	}
}
