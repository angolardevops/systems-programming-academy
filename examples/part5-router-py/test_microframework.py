"""Tests: the same nine scenarios as the Rust and Go twins, plus a check that
the @route decorator leaves the handler directly callable."""

import unittest

from microframework import Handler, Request, Response, Router


def hello_router() -> Router:
    app = Router()

    @app.route("GET", "/")
    def _home(_req: Request) -> Response:
        return Response(200, "home")

    @app.route("GET", "/users/:id")
    def _show(req: Request) -> Response:
        return Response(200, f"user {req.params['id']}")

    @app.route("POST", "/users")
    def _create(_req: Request) -> Response:
        return Response(201, "created")

    return app


class RoutingTest(unittest.TestCase):
    def test_dispatches_static_route(self) -> None:
        self.assertEqual(hello_router().dispatch("GET", "/"), Response(200, "home"))

    def test_captures_path_parameter(self) -> None:
        self.assertEqual(
            hello_router().dispatch("GET", "/users/42"), Response(200, "user 42")
        )

    def test_unknown_path_is_404(self) -> None:
        self.assertEqual(hello_router().dispatch("GET", "/nope").status, 404)

    def test_known_path_wrong_method_is_405(self) -> None:
        # /users/:id exists for GET; DELETE should be 405, not 404.
        self.assertEqual(hello_router().dispatch("DELETE", "/users/42").status, 405)

    def test_method_disambiguates_same_path(self) -> None:
        app = Router()
        app.route("GET", "/x")(lambda _r: Response(200, "get"))
        app.route("POST", "/x")(lambda _r: Response(200, "post"))
        self.assertEqual(app.dispatch("GET", "/x").body, "get")
        self.assertEqual(app.dispatch("POST", "/x").body, "post")


class MiddlewareTest(unittest.TestCase):
    def test_runs_around_handler(self) -> None:
        app = Router()

        @app.use
        def _wrap(req: Request, nxt: Handler) -> Response:
            res = nxt(req)
            res.body = f"[wrapped: {res.body}]"
            return res

        app.route("GET", "/")(lambda _r: Response(200, "core"))
        self.assertEqual(app.dispatch("GET", "/").body, "[wrapped: core]")

    def test_can_short_circuit(self) -> None:
        app = Router()
        app.use(lambda _req, _nxt: Response(401, "401 Unauthorized"))
        app.route("GET", "/secret")(lambda _r: Response(200, "TOP SECRET"))
        res = app.dispatch("GET", "/secret")
        self.assertEqual(res.status, 401)
        self.assertNotIn("SECRET", res.body)

    def test_order_is_outermost_first(self) -> None:
        app = Router()

        @app.use
        def _outer(req: Request, nxt: Handler) -> Response:
            res = nxt(req)
            res.body += "A"  # unwinds last
            return res

        @app.use
        def _inner(req: Request, nxt: Handler) -> Response:
            res = nxt(req)
            res.body += "B"
            return res

        app.route("GET", "/")(lambda _r: Response(200, ""))
        # Handler, then inner (B), then outer (A): "BA".
        self.assertEqual(app.dispatch("GET", "/").body, "BA")

    def test_sees_every_request_once(self) -> None:
        count = 0
        app = Router()

        @app.use
        def _counter(req: Request, nxt: Handler) -> Response:
            nonlocal count
            count += 1
            return nxt(req)

        app.route("GET", "/")(lambda _r: Response(200, "ok"))
        app.dispatch("GET", "/")
        app.dispatch("GET", "/missing")  # 404 must still pass through middleware
        self.assertEqual(count, 2)


class DecoratorTest(unittest.TestCase):
    def test_route_returns_handler_unchanged(self) -> None:
        app = Router()

        @app.route("GET", "/ping")
        def ping(_req: Request) -> Response:
            return Response(200, "pong")

        # The decorator must return the handler itself so it stays callable.
        self.assertEqual(ping(Request("GET", "/ping")), Response(200, "pong"))


if __name__ == "__main__":
    unittest.main()
