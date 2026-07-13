package main

import (
	"bytes"
	"io"
	"net"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"testing"
)

// setupDocroot creates a docroot with index.html and style.css inside it —
// and secret.txt one level OUTSIDE it, the traversal target that must never
// be served.
func setupDocroot(t *testing.T) string {
	t.Helper()
	base := t.TempDir()
	docroot := filepath.Join(base, "public")
	if err := os.Mkdir(docroot, 0o755); err != nil {
		t.Fatal(err)
	}
	must := func(err error) {
		if err != nil {
			t.Fatal(err)
		}
	}
	must(os.WriteFile(filepath.Join(docroot, "index.html"), []byte("<h1>home</h1>\n"), 0o644))
	must(os.WriteFile(filepath.Join(docroot, "style.css"), []byte("body{}\n"), 0o644))
	must(os.WriteFile(filepath.Join(base, "secret.txt"), []byte("TOP SECRET\n"), 0o644))
	return docroot
}

func startServer(t *testing.T) string {
	t.Helper()
	docroot := setupDocroot(t)
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { ln.Close() })
	go serve(ln, docroot)
	return ln.Addr().String()
}

// request sends raw bytes and returns (status, head, body).
func request(t *testing.T, addr, raw string) (int, string, []byte) {
	t.Helper()
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		t.Fatal(err)
	}
	defer conn.Close()
	if _, err := io.WriteString(conn, raw); err != nil {
		t.Fatal(err)
	}
	response, err := io.ReadAll(conn)
	if err != nil {
		t.Fatal(err)
	}
	head, body, found := bytes.Cut(response, []byte("\r\n\r\n"))
	if !found {
		t.Fatalf("no header terminator in %q", response)
	}
	fields := strings.Fields(string(head))
	status, err := strconv.Atoi(fields[1])
	if err != nil {
		t.Fatalf("bad status line %q", head)
	}
	return status, string(head), body
}

func TestServesIndexForRoot(t *testing.T) {
	addr := startServer(t)
	status, head, body := request(t, addr, "GET / HTTP/1.0\r\n\r\n")
	if status != 200 {
		t.Fatalf("status = %d, want 200", status)
	}
	if !strings.Contains(head, "Content-Type: text/html") {
		t.Fatalf("missing content type in %q", head)
	}
	if string(body) != "<h1>home</h1>\n" {
		t.Fatalf("body = %q", body)
	}
}

func TestServesCSSWithContentTypeAndLength(t *testing.T) {
	addr := startServer(t)
	status, head, body := request(t, addr, "GET /style.css HTTP/1.0\r\n\r\n")
	if status != 200 {
		t.Fatalf("status = %d, want 200", status)
	}
	if !strings.Contains(head, "Content-Type: text/css") {
		t.Fatalf("missing content type in %q", head)
	}
	if !strings.Contains(head, "Content-Length: "+strconv.Itoa(len(body))) {
		t.Fatalf("wrong content length in %q", head)
	}
}

func TestMissingFileIs404(t *testing.T) {
	addr := startServer(t)
	status, _, body := request(t, addr, "GET /nope.html HTTP/1.0\r\n\r\n")
	if status != 404 || string(body) != "404 Not Found\n" {
		t.Fatalf("got %d %q", status, body)
	}
}

func TestPostIs405(t *testing.T) {
	addr := startServer(t)
	status, _, _ := request(t, addr, "POST / HTTP/1.0\r\n\r\n")
	if status != 405 {
		t.Fatalf("status = %d, want 405", status)
	}
}

func TestTraversalNeverEscapesDocroot(t *testing.T) {
	addr := startServer(t)
	status, _, body := request(t, addr, "GET /../secret.txt HTTP/1.0\r\n\r\n")
	if status != 404 {
		t.Fatalf("traversal must be rejected, got %d", status)
	}
	if bytes.Contains(body, []byte("SECRET")) {
		t.Fatal("secret leaked!")
	}
}

func TestGarbageIs400(t *testing.T) {
	addr := startServer(t)
	status, _, _ := request(t, addr, "NOT-HTTP\r\n\r\n")
	if status != 400 {
		t.Fatalf("status = %d, want 400", status)
	}
}

func TestConcurrentClientsAllSucceed(t *testing.T) {
	addr := startServer(t)
	var wg sync.WaitGroup
	errs := make(chan string, 16)
	for range 16 {
		wg.Add(1)
		go func() {
			defer wg.Done()
			status, _, body := request(t, addr, "GET / HTTP/1.0\r\n\r\n")
			if status != 200 || string(body) != "<h1>home</h1>\n" {
				errs <- "bad response"
			}
		}()
	}
	wg.Wait()
	close(errs)
	for e := range errs {
		t.Fatal(e)
	}
}

func TestResolveRejectsDotdotAndMapsRoot(t *testing.T) {
	root := "/srv/www"
	if got, ok := resolve(root, "/"); !ok || got != "/srv/www/index.html" {
		t.Fatalf("resolve(/) = %q, %v", got, ok)
	}
	if got, ok := resolve(root, "/a/b.txt"); !ok || got != "/srv/www/a/b.txt" {
		t.Fatalf("resolve(/a/b.txt) = %q, %v", got, ok)
	}
	if _, ok := resolve(root, "/../etc/passwd"); ok {
		t.Fatal("must reject /../")
	}
	if _, ok := resolve(root, "/a/../../etc/passwd"); ok {
		t.Fatal("must reject nested ..")
	}
}
