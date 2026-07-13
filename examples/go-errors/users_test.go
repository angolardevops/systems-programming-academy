package usersdemo

import (
	"errors"
	"strconv"
	"testing"
)

func TestParseUserValid(t *testing.T) {
	u, err := ParseUser("Ada, 36")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if u.Name != "Ada" || u.Age != 36 {
		t.Errorf("got %+v, want {Ada 36}", u)
	}
}

// errors.As unwraps the chain looking for a specific error TYPE.
func TestParseUserValidationErrorViaAs(t *testing.T) {
	_, err := ParseUser("no comma here")
	var ve *ValidationError
	if !errors.As(err, &ve) {
		t.Fatalf("expected *ValidationError, got %v", err)
	}
	if ve.Field != "line" {
		t.Errorf("Field = %q, want line", ve.Field)
	}
}

func TestParseUserEmptyName(t *testing.T) {
	_, err := ParseUser("  , 20")
	var ve *ValidationError
	if !errors.As(err, &ve) || ve.Field != "name" {
		t.Fatalf("expected name ValidationError, got %v", err)
	}
}

// errors.Is walks the wrapped chain looking for a specific sentinel VALUE.
func TestParseUserWrapsStrconvError(t *testing.T) {
	_, err := ParseUser("Bob, twelve")
	if !errors.Is(err, strconv.ErrSyntax) {
		t.Fatalf("expected wrapped strconv.ErrSyntax, got %v", err)
	}
}

func TestLookupFound(t *testing.T) {
	name, err := Lookup("1")
	if err != nil || name != "Ada" {
		t.Fatalf("Lookup(1) = %q, %v; want Ada, nil", name, err)
	}
}

func TestLookupNotFoundMatchesSentinel(t *testing.T) {
	_, err := Lookup("999")
	if !errors.Is(err, ErrNotFound) {
		t.Fatalf("expected ErrNotFound, got %v", err)
	}
}

func TestFirstValidAge(t *testing.T) {
	age, err := FirstValidAge([]string{"Ada, 36", "Grace, 45"})
	if err != nil || age != 36 {
		t.Fatalf("FirstValidAge = %d, %v; want 36, nil", age, err)
	}
}

func TestFirstValidAgePropagatesError(t *testing.T) {
	_, err := FirstValidAge([]string{"bad line"})
	var ve *ValidationError
	if !errors.As(err, &ve) {
		t.Fatalf("expected wrapped *ValidationError, got %v", err)
	}
}
