// Package validator is a declarative validation framework: describe the rules
// a record must satisfy, then validate a record against them and get back
// every error at once.
//
// The framework idea is declaration over imperative checking: instead of
// scattered if-statements, you declare a schema and the evaluator applies it.
// The decision that matters most is error accumulation — collect ALL failures
// and return them together, rather than bailing on the first. A form that
// reports every problem at once is a good experience; one that reports them one
// at a time is not. Fail-fast is for programming errors; user input wants
// fail-complete.
//
// Rules are plain checks, so the framework is dependency-free and the collected
// errors are directly assertable — no I/O anywhere.
package validator

import (
	"fmt"
	"strconv"
	"strings"
	"unicode/utf8"
)

// Rule is one validation rule applied to a single field's value. Construct
// them with the helper functions below.
type Rule struct {
	kind    string
	n       int
	lo, hi  int
	options []string
}

// Required: the value must be present and non-empty.
func Required() Rule { return Rule{kind: "required"} }

// MinLength: at least n characters.
func MinLength(n int) Rule { return Rule{kind: "min", n: n} }

// MaxLength: at most n characters.
func MaxLength(n int) Rule { return Rule{kind: "max", n: n} }

// IsInt: must parse as an integer.
func IsInt() Rule { return Rule{kind: "int"} }

// InRange: must parse as an integer within [lo, hi] (implies IsInt).
func InRange(lo, hi int) Rule { return Rule{kind: "range", lo: lo, hi: hi} }

// OneOf: must be one of the allowed values.
func OneOf(options ...string) Rule { return Rule{kind: "oneof", options: options} }

func (r Rule) isRequired() bool { return r.kind == "required" }

// check returns an error message if value fails this rule, or "" if it passes.
// Messages are stable text — the cross-language contract asserted by the tests.
func (r Rule) check(value string) string {
	switch r.kind {
	case "required":
		if value == "" {
			return "is required"
		}
	case "min":
		if utf8.RuneCountInString(value) < r.n {
			return fmt.Sprintf("must be at least %d characters", r.n)
		}
	case "max":
		if utf8.RuneCountInString(value) > r.n {
			return fmt.Sprintf("must be at most %d characters", r.n)
		}
	case "int":
		if _, err := strconv.Atoi(value); err != nil {
			return "must be an integer"
		}
	case "range":
		n, err := strconv.Atoi(value)
		if err != nil {
			return "must be an integer"
		}
		if n < r.lo || n > r.hi {
			return fmt.Sprintf("must be between %d and %d", r.lo, r.hi)
		}
	case "oneof":
		for _, o := range r.options {
			if o == value {
				return ""
			}
		}
		return "must be one of " + strings.Join(r.options, ", ")
	}
	return ""
}

// Error is a single validation failure.
type Error struct {
	Field   string
	Message string
}

// Line renders as "field: message" — the stable format the tests assert.
func (e Error) Line() string { return e.Field + ": " + e.Message }

type fieldRules struct {
	name  string
	rules []Rule
}

// Schema is an ordered list of (field, rules). Order is preserved in the error
// output, so results are deterministic and identical across languages.
type Schema struct {
	fields []fieldRules
}

// New returns an empty schema.
func New() *Schema { return &Schema{} }

// Field declares the rules for a field. Chainable.
func (s *Schema) Field(name string, rules ...Rule) *Schema {
	s.fields = append(s.fields, fieldRules{name, rules})
	return s
}

// Validate returns every error found, in field-declaration then rule order.
// A field carrying Required that is missing/empty yields exactly one
// "is required" error and its other rules are skipped. A field without Required
// that is absent/empty is skipped — that is what "optional" means.
func (s *Schema) Validate(data map[string]string) []Error {
	errors := []Error{}
	for _, f := range s.fields {
		value := data[f.name]
		present := value != ""
		required := false
		for _, r := range f.rules {
			if r.isRequired() {
				required = true
				break
			}
		}

		if !present {
			if required {
				errors = append(errors, Error{f.name, "is required"})
			}
			continue
		}

		for _, r := range f.rules {
			if r.isRequired() {
				continue
			}
			if msg := r.check(value); msg != "" {
				errors = append(errors, Error{f.name, msg})
			}
		}
	}
	return errors
}

// Lines renders a list of errors as "field: message" lines.
func Lines(errors []Error) []string {
	lines := make([]string, len(errors))
	for i, e := range errors {
		lines[i] = e.Line()
	}
	return lines
}
