// mininginx — usage: mininginx <docroot> <port>
//
// A concurrent static-file HTTP server with NO net/http: raw TCP, hand
// parsing, and one goroutine per connection. The runtime's netpoller (Async
// lesson) is the event loop; goroutines are the concurrency dial — unbounded
// here, which is Go's default posture and a deliberate contrast with the
// Rust twin's fixed thread pool.
//
// Serves byte-identical responses to the Rust and Python twins.
package main

import (
	"bufio"
	"fmt"
	"net"
	"os"
	"path/filepath"
	"strings"
)

// resolve maps a URL path to a file inside docroot, or returns ok=false if
// the path tries to escape it. Purely lexical: any ".." component is
// rejected outright — the request never touches the filesystem outside the
// root.
func resolve(docroot, urlPath string) (string, bool) {
	if urlPath == "/" {
		urlPath = "/index.html"
	}
	resolved := docroot
	for _, component := range strings.Split(urlPath, "/") {
		switch {
		case component == "" || component == ".":
			continue
		case component == "..":
			return "", false // traversal attempt: never leaves docroot
		case strings.ContainsRune(component, 0):
			return "", false
		default:
			resolved = filepath.Join(resolved, component)
		}
	}
	return resolved, true
}

// contentType by file extension — the tiny subset a static site needs.
func contentType(path string) string {
	switch filepath.Ext(path) {
	case ".html":
		return "text/html"
	case ".css":
		return "text/css"
	case ".js":
		return "application/javascript"
	case ".json":
		return "application/json"
	case ".png":
		return "image/png"
	case ".txt":
		return "text/plain"
	default:
		return "application/octet-stream"
	}
}

// buildResponse serializes a full HTTP/1.0 response. The exact bytes here
// are the cross-language contract with the Rust and Python twins.
func buildResponse(status int, ctype string, body []byte) []byte {
	reasons := map[int]string{
		200: "OK",
		400: "Bad Request",
		404: "Not Found",
		405: "Method Not Allowed",
	}
	reason, ok := reasons[status]
	if !ok {
		reason = "Internal Server Error"
	}
	head := fmt.Sprintf(
		"HTTP/1.0 %d %s\r\nContent-Type: %s\r\nContent-Length: %d\r\nConnection: close\r\n\r\n",
		status, reason, ctype, len(body))
	return append([]byte(head), body...)
}

func errorResponse(status int) []byte {
	bodies := map[int]string{
		400: "400 Bad Request\n",
		404: "404 Not Found\n",
		405: "405 Method Not Allowed\n",
	}
	body, ok := bodies[status]
	if !ok {
		body = "500 Internal Server Error\n"
	}
	return buildResponse(status, "text/plain", []byte(body))
}

// handleConnection: parse the request head, resolve, read the file,
// respond, close. One goroutine runs this per connection.
func handleConnection(conn net.Conn, docroot string) {
	defer conn.Close()
	reader := bufio.NewReader(conn)

	requestLine, err := reader.ReadString('\n')
	if err != nil {
		conn.Write(errorResponse(400))
		return
	}
	parts := strings.Fields(requestLine)
	if len(parts) < 3 || !strings.HasPrefix(parts[2], "HTTP/") {
		conn.Write(errorResponse(400))
		return
	}
	method, urlPath := parts[0], parts[1]

	// Drain the headers; we serve statelessly and ignore them all.
	for {
		line, err := reader.ReadString('\n')
		if err != nil || line == "\r\n" || line == "\n" {
			break
		}
	}

	if method != "GET" {
		conn.Write(errorResponse(405))
		return
	}
	file, ok := resolve(docroot, urlPath)
	if !ok {
		conn.Write(errorResponse(404))
		return
	}
	body, err := os.ReadFile(file)
	if err != nil {
		conn.Write(errorResponse(404))
		return
	}
	conn.Write(buildResponse(200, contentType(file), body))
}

// serve accepts connections forever, one goroutine each.
func serve(ln net.Listener, docroot string) {
	for {
		conn, err := ln.Accept()
		if err != nil {
			return
		}
		go handleConnection(conn, docroot)
	}
}

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintln(os.Stderr, "usage: mininginx <docroot> <port>")
		os.Exit(2)
	}
	docroot := os.Args[1]
	if info, err := os.Stat(docroot); err != nil || !info.IsDir() {
		fmt.Fprintf(os.Stderr, "error: docroot %s is not a directory\n", docroot)
		os.Exit(2)
	}
	ln, err := net.Listen("tcp", "127.0.0.1:"+os.Args[2])
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: bind failed: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("listening on %s\n", ln.Addr())
	serve(ln, docroot)
}
