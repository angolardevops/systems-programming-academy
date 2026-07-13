"""Tests for confgen.py (stdlib unittest) — including golden tests."""

import unittest

from confgen import Spec, SpecError, parse_spec, render_nginx, render_systemd

SPEC = (
    "# demo service\nname = api\ndomain = api.example.com\nport = 8080\nreplicas = 2\n"
)


class ParseTests(unittest.TestCase):
    def test_parses_a_full_spec(self) -> None:
        self.assertEqual(
            parse_spec(SPEC),
            Spec(name="api", domain="api.example.com", port=8080, replicas=2),
        )

    def test_precise_errors(self) -> None:
        for bad in [
            "domain = x\nport = 1\n",  # missing name
            "name = a\ndomain = x\nport = banana\n",  # bad port
            "name = a\ndomain = x\nport = 1\nreplicas = 0\n",  # zero replicas
        ]:
            with self.subTest(bad=bad):
                with self.assertRaises(SpecError):
                    parse_spec(bad)

    def test_replicas_defaults_to_one(self) -> None:
        self.assertEqual(parse_spec("name = a\ndomain = x\nport = 9000\n").replicas, 1)


class GoldenTests(unittest.TestCase):
    """GOLDEN TESTS: exact expected artifacts, byte for byte."""

    def test_nginx_golden(self) -> None:
        expected = """upstream api {
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
"""
        self.assertEqual(render_nginx(parse_spec(SPEC)), expected)

    def test_systemd_golden(self) -> None:
        expected = """[Unit]
Description=api service (instance %i)
After=network.target

[Service]
ExecStart=/usr/local/bin/api --port %i
Restart=on-failure
User=api
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
"""
        self.assertEqual(render_systemd(parse_spec(SPEC)), expected)


if __name__ == "__main__":
    unittest.main()
