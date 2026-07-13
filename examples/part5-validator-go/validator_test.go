package validator

import (
	"reflect"
	"testing"
)

func userSchema() *Schema {
	return New().
		Field("name", Required(), MinLength(2), MaxLength(30)).
		Field("age", Required(), InRange(18, 120)).
		Field("role", OneOf("admin", "user", "guest"))
}

func assertLines(t *testing.T, got, want []string) {
	t.Helper()
	if len(want) == 0 && len(got) == 0 {
		return
	}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("errors\n got:  %v\n want: %v", got, want)
	}
}

func TestValidRecordHasNoErrors(t *testing.T) {
	data := map[string]string{"name": "Ana", "age": "30", "role": "admin"}
	if errs := userSchema().Validate(data); len(errs) != 0 {
		t.Fatalf("expected no errors, got %v", Lines(errs))
	}
}

func TestMissingRequiredFieldReportsIsRequired(t *testing.T) {
	data := map[string]string{"age": "30", "role": "user"}
	assertLines(t, Lines(userSchema().Validate(data)), []string{"name: is required"})
}

func TestTooShortReportsMinLength(t *testing.T) {
	data := map[string]string{"name": "A", "age": "30", "role": "user"}
	assertLines(t, Lines(userSchema().Validate(data)), []string{"name: must be at least 2 characters"})
}

func TestAccumulatesAllErrorsNotJustTheFirst(t *testing.T) {
	data := map[string]string{"name": "A", "age": "old", "role": "wizard"}
	assertLines(t, Lines(userSchema().Validate(data)), []string{
		"name: must be at least 2 characters",
		"age: must be an integer",
		"role: must be one of admin, user, guest",
	})
}

func TestRangeChecksBounds(t *testing.T) {
	data := map[string]string{"name": "Ana", "age": "150", "role": "user"}
	assertLines(t, Lines(userSchema().Validate(data)), []string{"age: must be between 18 and 120"})
}

func TestOptionalAbsentFieldIsSkipped(t *testing.T) {
	schema := New().Field("bio", MaxLength(100))
	if errs := schema.Validate(map[string]string{}); len(errs) != 0 {
		t.Fatalf("expected no errors, got %v", Lines(errs))
	}
}

func TestOneOfAcceptsAllowedAndRejectsOthers(t *testing.T) {
	schema := New().Field("role", OneOf("admin", "user"))
	if errs := schema.Validate(map[string]string{"role": "admin"}); len(errs) != 0 {
		t.Fatalf("admin should be valid, got %v", Lines(errs))
	}
	assertLines(t, Lines(schema.Validate(map[string]string{"role": "root"})),
		[]string{"role: must be one of admin, user"})
}

func TestMultibyteLengthCountsCharactersNotBytes(t *testing.T) {
	// "José" is 4 characters but 5 bytes — MinLength must count runes.
	schema := New().Field("name", MinLength(4))
	if errs := schema.Validate(map[string]string{"name": "José"}); len(errs) != 0 {
		t.Fatalf("José has 4 runes, should pass MinLength(4), got %v", Lines(errs))
	}
}
