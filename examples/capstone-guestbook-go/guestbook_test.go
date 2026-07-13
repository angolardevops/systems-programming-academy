package guestbook

import (
	"reflect"
	"strings"
	"testing"
)

func TestValidSubmissionIsStored(t *testing.T) {
	store := NewStore()
	errs := Submit(store, "Ana", "Hello, world!")
	if len(errs) != 0 {
		t.Fatalf("unexpected errors: %v", errs)
	}
	if len(store.All()) != 1 || store.All()[0].Author != "Ana" {
		t.Fatalf("comment not stored: %v", store.All())
	}
}

func TestInvalidSubmissionAccumulatesErrorsAndStoresNothing(t *testing.T) {
	store := NewStore()
	errs := Submit(store, "A", "   ")
	want := []string{"author: must be at least 2 characters", "body: is required"}
	if !reflect.DeepEqual(errs, want) {
		t.Fatalf("errors\n got:  %v\n want: %v", errs, want)
	}
	if len(store.All()) != 0 {
		t.Fatalf("nothing should be stored on error, got %v", store.All())
	}
}

func TestInsertIsParameterizedNeverInterpolated(t *testing.T) {
	evil := "'; DROP TABLE comments; --"
	sql, params := InsertSQL("Ana", evil)
	if sql != "INSERT INTO comments (author, body) VALUES (?, ?)" {
		t.Fatalf("sql = %q", sql)
	}
	if !reflect.DeepEqual(params, []string{"Ana", evil}) {
		t.Fatalf("params = %v", params)
	}
	if strings.Contains(sql, "DROP") {
		t.Fatal("payload must never reach the SQL text")
	}
}

func TestSQLInjectionPayloadIsStoredAsInertDataTableSurvives(t *testing.T) {
	store := NewStore()
	Submit(store, "Alice", "first comment")
	errs := Submit(store, "Mallory", "'; DROP TABLE comments; --")
	if len(errs) != 0 {
		t.Fatalf("unexpected errors: %v", errs)
	}
	if len(store.All()) != 2 {
		t.Fatalf("table should still have 2 rows, got %d", len(store.All()))
	}
	if store.All()[0].Body != "first comment" {
		t.Fatalf("existing row was lost: %v", store.All()[0])
	}
	if store.All()[1].Body != "'; DROP TABLE comments; --" {
		t.Fatalf("payload not stored verbatim: %q", store.All()[1].Body)
	}
}

func TestXSSPayloadRendersAsInertText(t *testing.T) {
	store := NewStore()
	Submit(store, "Mallory", "<script>alert(document.cookie)</script>")
	page := RenderPage(store)
	if !strings.Contains(page, "&lt;script&gt;alert(document.cookie)&lt;/script&gt;") {
		t.Fatalf("script must be escaped: %s", page)
	}
	if strings.Contains(page, "<script>") {
		t.Fatalf("no live script tag may appear: %s", page)
	}
}

func TestEndToEndBothAttacksDefeated(t *testing.T) {
	store := NewStore()
	Submit(store, "Ana", "Nice site!")
	Submit(store, "Mallory", "'; DROP TABLE comments; --")
	Submit(store, "Eve", "<script>steal()</script>")

	if len(store.All()) != 3 {
		t.Fatalf("all three rows should survive, got %d", len(store.All()))
	}
	page := RenderPage(store)
	if !strings.Contains(page, "&#39;; DROP TABLE comments; --") {
		t.Fatalf("SQLi text should render escaped: %s", page)
	}
	if !strings.Contains(page, "&lt;script&gt;steal()&lt;/script&gt;") {
		t.Fatalf("XSS should render escaped: %s", page)
	}
	if strings.Contains(page, "<script>") {
		t.Fatalf("no live script: %s", page)
	}
}
