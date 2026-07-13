"""Config generator — Python implementation of the Part 3 project. All three
languages emit byte-identical output for the same spec.

Run the tests / the tool:

    python3 -m unittest discover -s . -p 'test_*.py'
    python3 confgen.py service.conf
"""

from __future__ import annotations

import sys
from dataclasses import dataclass


class SpecError(Exception):
    """A precise spec-file problem (missing key or invalid value)."""


@dataclass(frozen=True)
class Spec:
    """A validated service spec."""

    name: str
    domain: str
    port: int
    replicas: int


def parse_spec(text: str) -> Spec:
    """Parses the `key = value` format (# comments, blank lines ok)."""
    values: dict[str, str] = {}
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue  # tolerated: not a key=value line
        key, _, value = line.partition("=")
        values[key.strip()] = value.strip()

    for required in ("name", "domain", "port"):
        if required not in values:
            raise SpecError(f"missing required key {required}")

    def as_int(key: str, default: int | None = None) -> int:
        raw_value = values.get(key)
        if raw_value is None:
            assert default is not None
            return default
        try:
            return int(raw_value)
        except ValueError:
            raise SpecError(f"invalid value {raw_value!r} for key {key}") from None

    spec = Spec(
        name=values["name"],
        domain=values["domain"],
        port=as_int("port"),
        replicas=as_int("replicas", default=1),
    )
    if spec.replicas < 1:
        raise SpecError("invalid value '0' for key replicas")
    return spec


def render_nginx(spec: Spec) -> str:
    """Renders the upstream + server block."""
    servers = "".join(
        f"    server 127.0.0.1:{spec.port + i};\n" for i in range(spec.replicas)
    )
    return f"""upstream {spec.name} {{
{servers}}}

server {{
    listen 80;
    server_name {spec.domain};

    location / {{
        proxy_pass http://{spec.name};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }}
}}
"""


def render_systemd(spec: Spec) -> str:
    """Renders the unit template (%i = instance)."""
    return f"""[Unit]
Description={spec.name} service (instance %i)
After=network.target

[Service]
ExecStart=/usr/local/bin/{spec.name} --port %i
Restart=on-failure
User={spec.name}
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
"""


def generate(text: str) -> str:
    """Full CLI output: both artifacts with headers (shared across languages)."""
    spec = parse_spec(text)
    return (
        f"--- nginx: {spec.name}.conf\n{render_nginx(spec)}\n"
        f"--- systemd: {spec.name}@.service\n{render_systemd(spec)}"
    )


def main() -> None:
    if len(sys.argv) > 1:
        with open(sys.argv[1], encoding="utf-8") as f:
            text = f.read()
    else:
        text = sys.stdin.read()
    try:
        print(generate(text), end="")
    except SpecError as e:
        print(f"confgen: {e}", file=sys.stderr)
        raise SystemExit(1) from None


if __name__ == "__main__":
    main()
