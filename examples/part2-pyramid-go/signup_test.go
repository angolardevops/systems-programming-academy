package signup

import (
	"errors"
	"testing"
)

// UNIT level: many tiny table-driven cases on the pure validator.
func TestValidateEmail(t *testing.T) {
	cases := []struct {
		name  string
		email string
		want  error
	}{
		{"valid", "ada@example.com", nil},
		{"empty", "", ErrEmpty},
		{"missing at", "ada.example.com", ErrBadFormat},
		{"multiple at", "a@b@c.com", ErrBadFormat},
		{"no dot in domain", "ada@nodot", ErrBadFormat},
		{"empty local", "@example.com", ErrBadFormat},
		{"whitespace", "a da@example.com", ErrWhitespace},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got := ValidateEmail(tc.email)
			if !errors.Is(got, tc.want) && got != tc.want {
				t.Errorf("ValidateEmail(%q) = %v, want %v", tc.email, got, tc.want)
			}
		})
	}
}

// INTEGRATION level: the service with real in-memory adapters — proving the
// parts collaborate (stored AND notified), not just work alone.
func TestSignupStoresAndNotifies(t *testing.T) {
	notifier := &RecordingNotifier{}
	svc := NewService(NewInMemoryRepo(), notifier)

	if err := svc.Signup("ada@example.com"); err != nil {
		t.Fatal(err)
	}
	if len(notifier.Sent) != 1 || notifier.Sent[0] != "ada@example.com" {
		t.Errorf("Sent = %v, want [ada@example.com]", notifier.Sent)
	}
}

func TestDuplicateSignupRejectedAndNotNotifiedTwice(t *testing.T) {
	notifier := &RecordingNotifier{}
	svc := NewService(NewInMemoryRepo(), notifier)

	if err := svc.Signup("ada@example.com"); err != nil {
		t.Fatal(err)
	}
	if err := svc.Signup("ada@example.com"); !errors.Is(err, ErrDuplicate) {
		t.Errorf("expected ErrDuplicate, got %v", err)
	}
	if len(notifier.Sent) != 1 { // no second welcome
		t.Errorf("Sent = %v, want exactly one", notifier.Sent)
	}
}

// END-TO-END level: one test driving the composition root like a caller would,
// asserting only the user-visible outcome.
func TestFullSignupFlowThroughTheApp(t *testing.T) {
	app := NewApp()
	if got := app.Signup("ada@example.com"); got != "Welcome, ada@example.com! Check your inbox." {
		t.Errorf("first signup: %q", got)
	}
	if got := app.Signup("ada@example.com"); got != "ada@example.com is already registered." {
		t.Errorf("duplicate signup: %q", got)
	}
	if got := app.Signup("nope"); got != "'nope' is not a valid email address." {
		t.Errorf("invalid signup: %q", got)
	}
}
