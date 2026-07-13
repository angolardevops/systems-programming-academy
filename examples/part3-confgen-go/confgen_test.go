package main

import (
	"errors"
	"testing"
)

const spec = "# demo service\nname = api\ndomain = api.example.com\nport = 8080\nreplicas = 2\n"

func TestParsesFullSpec(t *testing.T) {
	got, err := ParseSpec(spec)
	if err != nil {
		t.Fatal(err)
	}
	want := Spec{Name: "api", Domain: "api.example.com", Port: 8080, Replicas: 2}
	if got != want {
		t.Errorf("ParseSpec = %+v, want %+v", got, want)
	}
}

func TestPreciseErrors(t *testing.T) {
	if _, err := ParseSpec("domain = x\nport = 1\n"); !errors.Is(err, ErrMissing) {
		t.Errorf("expected ErrMissing, got %v", err)
	}
	if _, err := ParseSpec("name = a\ndomain = x\nport = banana\n"); !errors.Is(err, ErrInvalid) {
		t.Errorf("expected ErrInvalid, got %v", err)
	}
	if _, err := ParseSpec("name = a\ndomain = x\nport = 1\nreplicas = 0\n"); !errors.Is(err, ErrInvalid) {
		t.Errorf("expected ErrInvalid for zero replicas, got %v", err)
	}
}

func TestReplicasDefaultsToOne(t *testing.T) {
	got, err := ParseSpec("name = a\ndomain = x\nport = 9000\n")
	if err != nil || got.Replicas != 1 {
		t.Errorf("Replicas = %d, %v; want 1, nil", got.Replicas, err)
	}
}

// GOLDEN TEST: exact expected artifact, byte for byte.
func TestNginxGolden(t *testing.T) {
	s, _ := ParseSpec(spec)
	expected := `upstream api {
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
}

server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
`
	if got := RenderNginx(s); got != expected {
		t.Errorf("nginx golden mismatch:\n%s", got)
	}
}

func TestSystemdGolden(t *testing.T) {
	s, _ := ParseSpec(spec)
	expected := `[Unit]
Description=api service (instance %i)
After=network.target

[Service]
ExecStart=/usr/local/bin/api --port %i
Restart=on-failure
User=api
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
`
	if got := RenderSystemd(s); got != expected {
		t.Errorf("systemd golden mismatch:\n%s", got)
	}
}
