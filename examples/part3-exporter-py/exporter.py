"""Prometheus-style exporter — Python implementation of the Part 3 project. All
three languages serve byte-identical /metrics for the same registry.

Run the tests / the server:

    python3 -m unittest discover -s . -p 'test_*.py'
    python3 exporter.py 9102   # then: curl localhost:9102/metrics
"""

from __future__ import annotations

import sys
from dataclasses import dataclass, field
from http.server import BaseHTTPRequestHandler, HTTPServer


@dataclass
class _Metric:
    help: str
    kind: str  # "counter" | "gauge"
    series: dict[str, float] = field(default_factory=dict)


def label_string(labels: dict[str, str] | None) -> str:
    """Renders {k="v",...} with keys sorted; empty -> ""."""
    if not labels:
        return ""
    inner = ",".join(f'{k}="{labels[k]}"' for k in sorted(labels))
    return "{" + inner + "}"


def format_value(v: float) -> str:
    """Whole numbers as integers, others shortest — keeps output identical
    across the three implementations."""
    if v == int(v):
        return str(int(v))
    return repr(v)


class Registry:
    """Accumulates metrics and renders the text exposition format."""

    def __init__(self) -> None:
        self._metrics: dict[str, _Metric] = {}

    def _metric(self, name: str, help_text: str, kind: str) -> _Metric:
        if name not in self._metrics:
            self._metrics[name] = _Metric(help=help_text, kind=kind)
        return self._metrics[name]

    def inc_counter(
        self, name: str, help_text: str, labels: dict[str, str] | None, delta: float
    ) -> None:
        m = self._metric(name, help_text, "counter")
        key = label_string(labels)
        m.series[key] = m.series.get(key, 0.0) + delta

    def set_gauge(
        self, name: str, help_text: str, labels: dict[str, str] | None, value: float
    ) -> None:
        self._metric(name, help_text, "gauge").series[label_string(labels)] = value

    def render(self) -> str:
        out: list[str] = []
        for name in sorted(self._metrics):
            m = self._metrics[name]
            out.append(f"# HELP {name} {m.help}")
            out.append(f"# TYPE {name} {m.kind}")
            for key in sorted(m.series):
                out.append(f"{name}{key} {format_value(m.series[key])}")
        return "\n".join(out) + "\n"


def demo_registry() -> Registry:
    """Seeds the shared demo data so /metrics can be diffed across languages."""
    r = Registry()
    r.inc_counter(
        "http_requests_total",
        "Total HTTP requests.",
        {"method": "GET", "path": "/"},
        42,
    )
    r.inc_counter(
        "http_requests_total",
        "Total HTTP requests.",
        {"method": "POST", "path": "/api"},
        7,
    )
    r.set_gauge("queue_depth", "Jobs waiting in the queue.", None, 3)
    r.set_gauge("cpu_load", "1-minute load average.", {"core": "0"}, 0.5)
    return r


class _Handler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:  # noqa: N802 (stdlib naming)
        if self.path != "/metrics":
            self.send_error(404)
            return
        body = demo_registry().render().encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/plain; version=0.0.4")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args: object) -> None:  # keep test output quiet
        pass


def main() -> None:
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 9100
    print(f"exporter listening on :{port}", file=sys.stderr)
    HTTPServer(("127.0.0.1", port), _Handler).serve_forever()


if __name__ == "__main__":
    main()
