// Package main is the Go implementation of the Part 3 config-generator
// project. All three languages emit byte-identical output for the same spec.
//
//	go test ./...
//	go build && ./confgen service.conf
package main

import (
	"errors"
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"
)

// Spec is a validated service spec.
type Spec struct {
	Name     string
	Domain   string
	Port     int
	Replicas int
}

// ErrMissing and ErrInvalid wrap precise spec-file problems.
var (
	ErrMissing = errors.New("missing required key")
	ErrInvalid = errors.New("invalid value")
)

// ParseSpec parses the `key = value` format (# comments, blank lines ok).
func ParseSpec(text string) (Spec, error) {
	spec := Spec{Replicas: 1} // default
	var haveName, haveDomain, havePort bool

	for _, raw := range strings.Split(text, "\n") {
		line := strings.TrimSpace(raw)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		key, value, found := strings.Cut(line, "=")
		if !found {
			continue // tolerated: not a key=value line
		}
		key, value = strings.TrimSpace(key), strings.TrimSpace(value)
		switch key {
		case "name":
			spec.Name, haveName = value, true
		case "domain":
			spec.Domain, haveDomain = value, true
		case "port":
			p, err := strconv.Atoi(value)
			if err != nil {
				return Spec{}, fmt.Errorf("port=%q: %w", value, ErrInvalid)
			}
			spec.Port, havePort = p, true
		case "replicas":
			r, err := strconv.Atoi(value)
			if err != nil {
				return Spec{}, fmt.Errorf("replicas=%q: %w", value, ErrInvalid)
			}
			spec.Replicas = r
		}
	}

	switch {
	case !haveName:
		return Spec{}, fmt.Errorf("name: %w", ErrMissing)
	case !haveDomain:
		return Spec{}, fmt.Errorf("domain: %w", ErrMissing)
	case !havePort:
		return Spec{}, fmt.Errorf("port: %w", ErrMissing)
	case spec.Replicas < 1:
		return Spec{}, fmt.Errorf("replicas=%d: %w", spec.Replicas, ErrInvalid)
	}
	return spec, nil
}

// RenderNginx renders the upstream + server block.
func RenderNginx(spec Spec) string {
	var servers strings.Builder
	for i := 0; i < spec.Replicas; i++ {
		fmt.Fprintf(&servers, "    server 127.0.0.1:%d;\n", spec.Port+i)
	}
	return fmt.Sprintf(`upstream %[1]s {
%[3]s}

server {
    listen 80;
    server_name %[2]s;

    location / {
        proxy_pass http://%[1]s;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
`, spec.Name, spec.Domain, servers.String())
}

// RenderSystemd renders the unit template (%i = instance).
func RenderSystemd(spec Spec) string {
	return fmt.Sprintf(`[Unit]
Description=%[1]s service (instance %%i)
After=network.target

[Service]
ExecStart=/usr/local/bin/%[1]s --port %%i
Restart=on-failure
User=%[1]s
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
`, spec.Name)
}

// Generate produces the full CLI output shared across the languages.
func Generate(text string) (string, error) {
	spec, err := ParseSpec(text)
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("--- nginx: %s.conf\n%s\n--- systemd: %s@.service\n%s",
		spec.Name, RenderNginx(spec), spec.Name, RenderSystemd(spec)), nil
}

func main() {
	var data []byte
	var err error
	if len(os.Args) > 1 {
		data, err = os.ReadFile(os.Args[1])
	} else {
		data, err = io.ReadAll(os.Stdin)
	}
	if err == nil {
		var out string
		out, err = Generate(string(data))
		if err == nil {
			fmt.Print(out)
			return
		}
	}
	fmt.Fprintln(os.Stderr, "confgen:", err)
	os.Exit(1)
}
