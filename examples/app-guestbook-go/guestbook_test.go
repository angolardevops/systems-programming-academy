package main

import (
	"database/sql"
	"io"
	"net"
	"path/filepath"
	"strconv"
	"strings"
	"testing"
)

func startServer(t *testing.T) (string, *sql.DB) {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "test.db")
	db, err := OpenDB(dbPath)
	if err != nil {
		t.Fatalf("open db: %v", err)
	}
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("listen: %v", err)
	}
	t.Cleanup(func() { ln.Close(); db.Close() })
	go Serve(ln, db)
	return ln.Addr().String(), db
}

func request(t *testing.T, addr, raw string) (int, string, string) {
	t.Helper()
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	defer conn.Close()
	io.WriteString(conn, raw)
	resp, err := io.ReadAll(conn)
	if err != nil {
		t.Fatalf("read: %v", err)
	}
	head, body, _ := strings.Cut(string(resp), "\r\n\r\n")
	status, _ := strconv.Atoi(strings.Fields(head)[1])
	return status, head, body
}

func get(t *testing.T, addr, path string) (int, string, string) {
	return request(t, addr, "GET "+path+" HTTP/1.0\r\n\r\n")
}

func post(t *testing.T, addr, path, form string) (int, string, string) {
	raw := "POST " + path + " HTTP/1.0\r\n" +
		"Content-Type: application/x-www-form-urlencoded\r\n" +
		"Content-Length: " + strconv.Itoa(len(form)) + "\r\n\r\n" + form
	return request(t, addr, raw)
}

func TestGetRootRendersEmptyGuestbook(t *testing.T) {
	addr, _ := startServer(t)
	status, _, body := get(t, addr, "/")
	if status != 200 || !strings.Contains(body, "<h1>Guestbook</h1>") {
		t.Fatalf("status=%d body=%q", status, body)
	}
}

func TestPostValidCommentRedirectsAndPersists(t *testing.T) {
	addr, db := startServer(t)
	status, head, _ := post(t, addr, "/comment", "author=Ana&body=Hello")
	if status != 303 || !strings.Contains(head, "Location: /") {
		t.Fatalf("status=%d head=%q", status, head)
	}
	rows, _ := AllComments(db)
	if len(rows) != 1 || rows[0] != [2]string{"Ana", "Hello"} {
		t.Fatalf("not persisted: %v", rows)
	}
	_, _, page := get(t, addr, "/")
	if !strings.Contains(page, "<strong>Ana</strong>: Hello") {
		t.Fatalf("not on page: %s", page)
	}
}

func TestPostInvalidCommentIs400AndPersistsNothing(t *testing.T) {
	addr, db := startServer(t)
	status, _, body := post(t, addr, "/comment", "author=A&body=")
	if status != 400 {
		t.Fatalf("status=%d", status)
	}
	if !strings.Contains(body, "author: must be at least 2 characters") ||
		!strings.Contains(body, "body: is required") {
		t.Fatalf("missing errors: %s", body)
	}
	rows, _ := AllComments(db)
	if len(rows) != 0 {
		t.Fatalf("should persist nothing, got %v", rows)
	}
}

func TestSQLInjectionAgainstRealSQLiteTableSurvives(t *testing.T) {
	addr, db := startServer(t)
	post(t, addr, "/comment", "author=Alice&body=first")
	// URL-encoded '; DROP TABLE comments; --
	payload := "%27%3B+DROP+TABLE+comments%3B+--"
	status, _, _ := post(t, addr, "/comment", "author=Mallory&body="+payload)
	if status != 303 {
		t.Fatalf("status=%d", status)
	}
	rows, err := AllComments(db) // if the table were dropped, this errors
	if err != nil {
		t.Fatalf("table appears dropped: %v", err)
	}
	if len(rows) != 2 {
		t.Fatalf("expected 2 rows, got %d", len(rows))
	}
	if rows[0] != [2]string{"Alice", "first"} {
		t.Fatalf("existing row lost: %v", rows[0])
	}
	if rows[1][1] != "'; DROP TABLE comments; --" {
		t.Fatalf("payload not stored verbatim: %q", rows[1][1])
	}
}

func TestXSSPayloadRendersInert(t *testing.T) {
	addr, _ := startServer(t)
	post(t, addr, "/comment", "author=Eve&body=%3Cscript%3Ealert(1)%3C%2Fscript%3E")
	_, _, page := get(t, addr, "/")
	if !strings.Contains(page, "&lt;script&gt;alert(1)&lt;/script&gt;") {
		t.Fatalf("script not escaped: %s", page)
	}
	if strings.Contains(page, "<script>alert(1)") {
		t.Fatalf("live script present: %s", page)
	}
}

func TestUnknownRouteIs404(t *testing.T) {
	addr, _ := startServer(t)
	status, _, _ := get(t, addr, "/nope")
	if status != 404 {
		t.Fatalf("status=%d", status)
	}
}
