// Package main is the capstone guestbook, promoted to a real running web app:
// a goroutine-per-connection HTTP server (the Part 4 mini-NGINX model) backed by
// a real SQLite database via database/sql.
//
// The security defences are unchanged from the capstone — parameterized inserts,
// autoescaped rendering — but now they run over real TCP sockets against a real
// SQLite engine, so the SQL-injection proof is against actual SQLite, not a
// stand-in.
//
// Routes:
//   - GET /         renders the guestbook page.
//   - POST /comment parses the form, validates, inserts (parameterized). On
//     success, 303-redirects to /; on validation failure, 400 with the errors.
//
// Uses modernc.org/sqlite — a pure-Go SQLite driver, so no cgo and no C
// toolchain needed to build.
package main

import (
	"bufio"
	"database/sql"
	"fmt"
	"net"
	"net/url"
	"os"
	"strconv"
	"strings"
	"unicode/utf8"

	_ "modernc.org/sqlite"
)

// ---------------------------------------------------------------------------
// Domain logic (validation, escaping, rendering) — same as the capstone.
// ---------------------------------------------------------------------------

func validateSubmission(author, body string) []string {
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

func escapeHTML(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, `"`, "&quot;")
	s = strings.ReplaceAll(s, "'", "&#39;")
	return s
}

// ---------------------------------------------------------------------------
// Real SQLite storage. The insert is parameterized: the driver binds the value,
// so a '; DROP TABLE ... payload is stored as data, never executed.
// ---------------------------------------------------------------------------

// OpenDB opens (or creates) the database and ensures the comments table exists.
func OpenDB(path string) (*sql.DB, error) {
	db, err := sql.Open("sqlite", path)
	if err != nil {
		return nil, err
	}
	_, err = db.Exec("CREATE TABLE IF NOT EXISTS comments (" +
		"id INTEGER PRIMARY KEY AUTOINCREMENT, author TEXT NOT NULL, body TEXT NOT NULL)")
	if err != nil {
		return nil, err
	}
	return db, nil
}

// InsertComment runs a parameterized INSERT — the values are bound, never
// spliced into the SQL text.
func InsertComment(db *sql.DB, author, body string) error {
	_, err := db.Exec("INSERT INTO comments (author, body) VALUES (?, ?)",
		strings.TrimSpace(author), strings.TrimSpace(body))
	return err
}

// AllComments returns every comment, oldest first, as [author, body] pairs.
func AllComments(db *sql.DB) ([][2]string, error) {
	rows, err := db.Query("SELECT author, body FROM comments ORDER BY id")
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out [][2]string
	for rows.Next() {
		var a, b string
		if err := rows.Scan(&a, &b); err != nil {
			return nil, err
		}
		out = append(out, [2]string{a, b})
	}
	return out, rows.Err()
}

// RenderPage renders the full HTML page, every value autoescaped.
func RenderPage(db *sql.DB) (string, error) {
	comments, err := AllComments(db)
	if err != nil {
		return "", err
	}
	var items strings.Builder
	for _, c := range comments {
		fmt.Fprintf(&items, "  <li><strong>%s</strong>: %s</li>\n",
			escapeHTML(c[0]), escapeHTML(c[1]))
	}
	return "<!doctype html>\n<html><head><title>Guestbook</title></head><body>\n" +
		"<h1>Guestbook</h1>\n" +
		"<ul class=\"guestbook\">\n" + items.String() + "</ul>\n" +
		"<form method=\"post\" action=\"/comment\">\n" +
		"  <input name=\"author\" placeholder=\"name\">\n" +
		"  <input name=\"body\" placeholder=\"message\">\n" +
		"  <button>Post</button>\n" +
		"</form>\n</body></html>", nil
}

// ---------------------------------------------------------------------------
// HTTP layer: parse request head + form body, route, build the response.
// ---------------------------------------------------------------------------

func parseForm(body string) map[string]string {
	form := map[string]string{}
	values, err := url.ParseQuery(body)
	if err != nil {
		return form
	}
	for k, v := range values {
		if len(v) > 0 {
			form[k] = v[0]
		}
	}
	return form
}

func buildResponse(status int, ctype, body, extraHeaders string) []byte {
	reasons := map[int]string{200: "OK", 303: "See Other", 400: "Bad Request", 404: "Not Found"}
	reason := reasons[status]
	if reason == "" {
		reason = "OK"
	}
	head := fmt.Sprintf("HTTP/1.0 %d %s\r\nContent-Type: %s\r\nContent-Length: %d\r\n%sConnection: close\r\n\r\n",
		status, reason, ctype, len(body), extraHeaders)
	return append([]byte(head), []byte(body)...)
}

// HandleRequest routes one request to a response. GET / renders; POST /comment
// submits.
func HandleRequest(db *sql.DB, method, path, body string) []byte {
	switch {
	case method == "GET" && path == "/":
		page, err := RenderPage(db)
		if err != nil {
			return buildResponse(500, "text/plain", "error\n", "")
		}
		return buildResponse(200, "text/html", page, "")
	case method == "POST" && path == "/comment":
		form := parseForm(body)
		errors := validateSubmission(form["author"], form["body"])
		if len(errors) > 0 {
			var b strings.Builder
			b.WriteString("<h1>Errors</h1>\n<ul>\n")
			for _, e := range errors {
				fmt.Fprintf(&b, "  <li>%s</li>\n", escapeHTML(e))
			}
			b.WriteString("</ul>")
			return buildResponse(400, "text/html", b.String(), "")
		}
		if err := InsertComment(db, form["author"], form["body"]); err != nil {
			return buildResponse(500, "text/plain", "error\n", "")
		}
		return buildResponse(303, "text/plain", "", "Location: /\r\n")
	default:
		return buildResponse(404, "text/plain", "404 Not Found\n", "")
	}
}

func handleConnection(conn net.Conn, db *sql.DB) {
	defer conn.Close()
	reader := bufio.NewReader(conn)

	requestLine, err := reader.ReadString('\n')
	if err != nil {
		return
	}
	parts := strings.Fields(requestLine)
	if len(parts) < 3 {
		return
	}
	method, path := parts[0], parts[1]

	contentLength := 0
	for {
		line, err := reader.ReadString('\n')
		if err != nil || line == "\r\n" || line == "\n" {
			break
		}
		name, value, found := strings.Cut(line, ":")
		if found && strings.EqualFold(strings.TrimSpace(name), "content-length") {
			contentLength, _ = strconv.Atoi(strings.TrimSpace(value))
		}
	}

	body := ""
	if contentLength > 0 {
		buf := make([]byte, contentLength)
		if _, err := readFull(reader, buf); err == nil {
			body = string(buf)
		}
	}

	conn.Write(HandleRequest(db, method, path, body))
}

// readFull reads exactly len(buf) bytes from the buffered reader.
func readFull(r *bufio.Reader, buf []byte) (int, error) {
	total := 0
	for total < len(buf) {
		n, err := r.Read(buf[total:])
		total += n
		if err != nil {
			return total, err
		}
	}
	return total, nil
}

// Serve accepts connections forever, one goroutine per connection.
func Serve(ln net.Listener, db *sql.DB) {
	for {
		conn, err := ln.Accept()
		if err != nil {
			return
		}
		go handleConnection(conn, db)
	}
}

func main() {
	path := "guestbook.db"
	if len(os.Args) > 1 {
		path = os.Args[1]
	}
	port := "8080"
	if len(os.Args) > 2 {
		port = os.Args[2]
	}

	db, err := OpenDB(path)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error opening db: %v\n", err)
		os.Exit(1)
	}
	defer db.Close()

	ln, err := net.Listen("tcp", "127.0.0.1:"+port)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error listening: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("guestbook listening on http://%s\n", ln.Addr())
	Serve(ln, db)
}
