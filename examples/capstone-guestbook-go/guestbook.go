// Package guestbook is the capstone: one small app that composes the Part 5
// frameworks — validation, parameterized query building, and autoescaped
// templating — into a guestbook, then defeats both classic injection attacks.
//
// The request pipeline is: validate -> store (parameterized) -> render
// (autoescaped). Each stage mirrors a Part 5 lesson. The point is the two
// adversarial tests: submitting `'; DROP TABLE comments; --` and
// `<script>alert(1)</script>` as a real comment, and proving the other rows
// survive and the script renders as inert text. Input defence and output
// defence, the same "safe by default" principle on both sides.
//
// Dependency-free and I/O-free: an in-memory store, so the whole pipeline is
// directly testable.
package guestbook

import (
	"fmt"
	"strings"
	"unicode/utf8"
)

// ValidateSubmission returns every error at once (never bailing on the first)
// as "field: message" lines.
func ValidateSubmission(author, body string) []string {
	errors := []string{}
	author = strings.TrimSpace(author)
	body = strings.TrimSpace(body)

	switch {
	case author == "":
		errors = append(errors, "author: is required")
	case utf8.RuneCountInString(author) < 2:
		errors = append(errors, "author: must be at least 2 characters")
	case utf8.RuneCountInString(author) > 40:
		errors = append(errors, "author: must be at most 40 characters")
	}

	switch {
	case body == "":
		errors = append(errors, "body: is required")
	case utf8.RuneCountInString(body) > 500:
		errors = append(errors, "body: must be at most 500 characters")
	}

	return errors
}

// InsertSQL builds the parameterized INSERT: the SQL carries only "?"
// placeholders; the user values travel separately, bound as data, never SQL.
func InsertSQL(author, body string) (string, []string) {
	return "INSERT INTO comments (author, body) VALUES (?, ?)", []string{author, body}
}

// EscapeHTML escapes text ("&" first, to avoid double-escaping the entities the
// later replacements introduce).
func EscapeHTML(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, `"`, "&quot;")
	s = strings.ReplaceAll(s, "'", "&#39;")
	return s
}

// Comment is one stored comment.
type Comment struct {
	Author string
	Body   string
}

// Store is an in-memory comment table, standing in for a real database. It
// binds params as row data — modelling what a driver does: bind values, never
// execute them.
type Store struct {
	comments []Comment
}

// NewStore returns an empty store.
func NewStore() *Store { return &Store{} }

// ExecuteInsert executes a parameterized INSERT. It accepts only the exact
// two-placeholder comment insert, so a caller cannot smuggle values into the
// SQL string.
func (s *Store) ExecuteInsert(sql string, params []string) error {
	if sql != "INSERT INTO comments (author, body) VALUES (?, ?)" {
		return fmt.Errorf("store only accepts the parameterized comment insert")
	}
	if len(params) != 2 {
		return fmt.Errorf("expected two bound params, got %d", len(params))
	}
	// Bind params as DATA — whatever is in them is stored verbatim, never SQL.
	s.comments = append(s.comments, Comment{Author: params[0], Body: params[1]})
	return nil
}

// All returns every stored comment, oldest first.
func (s *Store) All() []Comment { return s.comments }

// Submit validates, and if clean, stores via a parameterized insert. Returns
// the (possibly empty) list of validation errors; on error the store is
// untouched.
func Submit(store *Store, author, body string) []string {
	errors := ValidateSubmission(author, body)
	if len(errors) == 0 {
		sql, params := InsertSQL(strings.TrimSpace(author), strings.TrimSpace(body))
		_ = store.ExecuteInsert(sql, params)
	}
	return errors
}

// RenderPage renders every stored comment, HTML-escaped, so untrusted content
// can never become markup.
func RenderPage(store *Store) string {
	var b strings.Builder
	b.WriteString("<ul class=\"guestbook\">\n")
	for _, c := range store.All() {
		fmt.Fprintf(&b, "  <li><strong>%s</strong>: %s</li>\n",
			EscapeHTML(c.Author), EscapeHTML(c.Body))
	}
	b.WriteString("</ul>")
	return b.String()
}
