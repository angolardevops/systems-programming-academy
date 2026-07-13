package router

import "testing"

func helloRouter() *Router {
	r := New()
	r.Route("GET", "/", func(*Request) Response { return Response{200, "home"} })
	r.Route("GET", "/users/:id", func(req *Request) Response {
		return Response{200, "user " + req.Params["id"]}
	})
	r.Route("POST", "/users", func(*Request) Response { return Response{201, "created"} })
	return r
}

func TestDispatchesStaticRoute(t *testing.T) {
	if got := helloRouter().Dispatch("GET", "/"); got != (Response{200, "home"}) {
		t.Fatalf("Dispatch = %+v", got)
	}
}

func TestCapturesPathParameter(t *testing.T) {
	if got := helloRouter().Dispatch("GET", "/users/42"); got != (Response{200, "user 42"}) {
		t.Fatalf("Dispatch = %+v", got)
	}
}

func TestUnknownPathIs404(t *testing.T) {
	if got := helloRouter().Dispatch("GET", "/nope"); got.Status != 404 {
		t.Fatalf("status = %d, want 404", got.Status)
	}
}

func TestKnownPathWrongMethodIs405(t *testing.T) {
	// /users/:id exists for GET; DELETE should be 405, not 404.
	if got := helloRouter().Dispatch("DELETE", "/users/42"); got.Status != 405 {
		t.Fatalf("status = %d, want 405", got.Status)
	}
}

func TestMethodDisambiguatesSamePath(t *testing.T) {
	r := New()
	r.Route("GET", "/x", func(*Request) Response { return Response{200, "get"} })
	r.Route("POST", "/x", func(*Request) Response { return Response{200, "post"} })
	if got := r.Dispatch("GET", "/x"); got.Body != "get" {
		t.Fatalf("GET body = %q", got.Body)
	}
	if got := r.Dispatch("POST", "/x"); got.Body != "post" {
		t.Fatalf("POST body = %q", got.Body)
	}
}

func TestMiddlewareRunsAroundHandler(t *testing.T) {
	r := New()
	r.Use(func(req *Request, next Handler) Response {
		res := next(req)
		res.Body = "[wrapped: " + res.Body + "]"
		return res
	})
	r.Route("GET", "/", func(*Request) Response { return Response{200, "core"} })
	if got := r.Dispatch("GET", "/"); got.Body != "[wrapped: core]" {
		t.Fatalf("body = %q", got.Body)
	}
}

func TestMiddlewareCanShortCircuit(t *testing.T) {
	r := New()
	// An "auth" middleware that rejects everything without calling the handler.
	r.Use(func(*Request, Handler) Response { return Response{401, "401 Unauthorized"} })
	r.Route("GET", "/secret", func(*Request) Response { return Response{200, "TOP SECRET"} })
	got := r.Dispatch("GET", "/secret")
	if got.Status != 401 || got.Body == "TOP SECRET" {
		t.Fatalf("short-circuit failed: %+v", got)
	}
}

func TestMiddlewareOrderIsOutermostFirst(t *testing.T) {
	r := New()
	r.Use(func(req *Request, next Handler) Response {
		res := next(req)
		res.Body += "A" // outer: unwinds last
		return res
	})
	r.Use(func(req *Request, next Handler) Response {
		res := next(req)
		res.Body += "B" // inner
		return res
	})
	r.Route("GET", "/", func(*Request) Response { return Response{200, ""} })
	// Handler runs, then inner (B), then outer (A): "BA".
	if got := r.Dispatch("GET", "/"); got.Body != "BA" {
		t.Fatalf("body = %q, want BA", got.Body)
	}
}

func TestMiddlewareSeesEveryRequestOnce(t *testing.T) {
	count := 0
	r := New()
	r.Use(func(req *Request, next Handler) Response {
		count++
		return next(req)
	})
	r.Route("GET", "/", func(*Request) Response { return Response{200, "ok"} })
	r.Dispatch("GET", "/")
	r.Dispatch("GET", "/missing") // 404 must still pass through middleware
	if count != 2 {
		t.Fatalf("middleware ran %d times, want 2", count)
	}
}
