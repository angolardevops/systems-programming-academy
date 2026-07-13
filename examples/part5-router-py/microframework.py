"""A micro web framework: routing table, path parameters, and a middleware
chain — the abstraction the mini-NGINX server lacked.

The framework/library distinction lives here: you don't call the router in a
loop, you *register handlers* — often with a decorator — and hand it control.
It calls you back when a request matches. That inversion, "don't call us,
we'll call you", is what makes this a framework rather than a library.

Everything is pure dispatch over ``Request``/``Response`` objects, so the whole
thing is testable without a socket.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass, field


@dataclass
class Request:
    """An incoming request after parsing: method, path, and the path
    parameters captured by the matched route (e.g. ``/users/:id`` fills
    ``id``)."""

    method: str
    path: str
    params: dict[str, str] = field(default_factory=dict)


@dataclass
class Response:
    """A status code and body. Real frameworks carry headers too; this is
    the irreducible core."""

    status: int
    body: str


# A handler turns a request into a response.
Handler = Callable[[Request], Response]
# Middleware wraps the next handler — the decorator pattern. It can run code
# before (auth, logging), after (headers, timing), or short-circuit (reject)
# by not calling ``next``.
Middleware = Callable[[Request, Handler], Response]


def _split_path(path: str) -> list[str]:
    """Split a path into non-empty segments: ``/a/b/`` -> ``['a', 'b']``."""
    return [s for s in path.split("/") if s]


def _match_segments(route_seg: list[str], req_seg: list[str]) -> dict[str, str] | None:
    """Match route segments against request segments, capturing ``:params``.
    Return ``None`` on any mismatch (different length or a fixed segment that
    differs)."""
    if len(route_seg) != len(req_seg):
        return None
    params: dict[str, str] = {}
    for rseg, actual in zip(route_seg, req_seg):
        if rseg.startswith(":"):
            params[rseg[1:]] = actual
        elif rseg != actual:
            return None
    return params


class Router:
    """Register routes and middleware, then :meth:`dispatch`."""

    def __init__(self) -> None:
        self._routes: list[tuple[str, list[str], Handler]] = []
        self._middleware: list[Middleware] = []

    def route(self, method: str, pattern: str) -> Callable[[Handler], Handler]:
        """Decorator registering a handler for ``method`` + ``pattern``. A
        segment starting with ``:`` is a path parameter, e.g. ``/users/:id``.

        Usage::

            @app.route("GET", "/users/:id")
            def show(req):
                return Response(200, f"user {req.params['id']}")
        """
        segments = _split_path(pattern)

        def register(handler: Handler) -> Handler:
            self._routes.append((method, segments, handler))
            return handler  # returned unchanged, so it stays directly callable

        return register

    def use(self, mw: Middleware) -> Middleware:
        """Add a middleware (also usable as a decorator). First added runs
        outermost."""
        self._middleware.append(mw)
        return mw

    def dispatch(self, method: str, path: str) -> Response:
        """Route a request: run the middleware chain around the router core.
        Middleware wraps the ENTIRE dispatch, so it observes 404s and 405s
        too — a logging middleware must see every request, not just matched
        ones."""
        return self._run_chain(self._route_request, Request(method, path))

    def _route_request(self, req: Request) -> Response:
        """The router core, wrapped by middleware: pure matching. 404 if no
        path matches, 405 if the path matches but not the method, else the
        matched handler with path params bound."""
        req_segments = _split_path(req.path)
        path_matched = False
        for method, segments, handler in self._routes:
            params = _match_segments(segments, req_segments)
            if params is not None:
                path_matched = True
                if method == req.method:
                    return handler(Request(req.method, req.path, params))
        if path_matched:
            return Response(405, "405 Method Not Allowed")
        return Response(404, "404 Not Found")

    def _run_chain(self, core: Handler, req: Request) -> Response:
        """Fold the middleware around ``core``. Iterating from the last added
        to the first makes index 0 the outermost wrapper — "first registered
        runs first"."""
        nxt = core
        for mw in reversed(self._middleware):
            inner = nxt

            def wrapped(
                r: Request, mw: Middleware = mw, inner: Handler = inner
            ) -> Response:
                return mw(r, inner)

            nxt = wrapped
        return nxt(req)


if __name__ == "__main__":
    app = Router()

    @app.use
    def logging_mw(req: Request, nxt: Handler) -> Response:
        res = nxt(req)
        print(f"{req.method} {req.path} -> {res.status}")
        return res

    @app.use
    def auth_mw(req: Request, nxt: Handler) -> Response:
        if req.path.startswith("/admin"):
            return Response(401, "401 Unauthorized")
        return nxt(req)

    @app.route("GET", "/")
    def home(_req: Request) -> Response:
        return Response(200, "home")

    @app.route("GET", "/users/:id")
    def show_user(req: Request) -> Response:
        return Response(200, f"user profile: {req.params['id']}")

    @app.route("GET", "/admin")
    def admin(_req: Request) -> Response:
        return Response(200, "admin panel")

    for m, p in [
        ("GET", "/"),
        ("GET", "/users/7"),
        ("GET", "/admin"),
        ("GET", "/nope"),
    ]:
        result = app.dispatch(m, p)
        print(f"    => {result.status} {result.body!r}")
