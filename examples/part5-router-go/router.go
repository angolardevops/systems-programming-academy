// Package router is a micro web framework: a routing table, path parameters,
// and a middleware chain — the abstraction the mini-NGINX server lacked.
//
// The framework/library distinction lives here: you don't call the router in
// a loop, you register handlers and hand it control. It calls you back when a
// request matches — "don't call us, we'll call you", the inversion that makes
// this a framework rather than a library.
//
// Everything is pure dispatch over Request/Response structs, so it is testable
// without a socket. Bolting it onto the Part 4 server is a matter of turning
// bytes into a Request and a Response back into bytes.
package router

import "strings"

// Request is an incoming request after parsing: method, path, and the path
// parameters captured by the matched route (e.g. /users/:id fills "id").
type Request struct {
	Method string
	Path   string
	Params map[string]string
}

// Response is a status code and body. Real frameworks carry headers too;
// this is the irreducible core.
type Response struct {
	Status int
	Body   string
}

// Handler turns a request into a response.
type Handler func(*Request) Response

// Middleware wraps the next handler, producing a new one — the decorator
// pattern. It can run code before (auth, logging), after (headers, timing),
// or short-circuit (reject) the inner handler by not calling next.
type Middleware func(req *Request, next Handler) Response

type route struct {
	method   string
	segments []string
	handler  Handler
}

// Router holds registered routes and middleware. The zero value is not
// usable; call New.
type Router struct {
	routes     []route
	middleware []Middleware
}

// New returns an empty router.
func New() *Router { return &Router{} }

// Route registers a handler for method + pattern. A segment starting with
// ":" is a path parameter, e.g. /users/:id. Returns the router for chaining.
func (r *Router) Route(method, pattern string, h Handler) *Router {
	r.routes = append(r.routes, route{
		method:   method,
		segments: splitPath(pattern),
		handler:  h,
	})
	return r
}

// Use adds a middleware. The first added runs outermost.
func (r *Router) Use(mw Middleware) *Router {
	r.middleware = append(r.middleware, mw)
	return r
}

// Dispatch routes a request: run the middleware chain around the router core.
// Middleware wraps the ENTIRE dispatch, so it observes 404s and 405s too — a
// logging middleware must see every request, not just matched ones.
func (r *Router) Dispatch(method, path string) Response {
	base := &Request{Method: method, Path: path}
	return r.runChain(r.routeRequest, base)
}

// routeRequest is the router core, wrapped by middleware: pure matching.
// 404 if no path matches, 405 if the path matches but not the method, else
// the matched handler with path params bound.
func (r *Router) routeRequest(req *Request) Response {
	reqSegments := splitPath(req.Path)
	pathMatched := false

	for _, rt := range r.routes {
		if params, ok := matchSegments(rt.segments, reqSegments); ok {
			pathMatched = true
			if rt.method == req.Method {
				matched := &Request{Method: req.Method, Path: req.Path, Params: params}
				return rt.handler(matched)
			}
		}
	}

	if pathMatched {
		return Response{Status: 405, Body: "405 Method Not Allowed"}
	}
	return Response{Status: 404, Body: "404 Not Found"}
}

// runChain folds the middleware around core. Iterating from the last added
// to the first makes index 0 the outermost wrapper — "first registered runs
// first".
func (r *Router) runChain(core Handler, req *Request) Response {
	next := core
	for i := len(r.middleware) - 1; i >= 0; i-- {
		mw := r.middleware[i]
		inner := next
		next = func(rq *Request) Response { return mw(rq, inner) }
	}
	return next(req)
}

// splitPath splits a path into non-empty segments: "/a/b/" -> ["a", "b"].
func splitPath(path string) []string {
	var out []string
	for _, s := range strings.Split(path, "/") {
		if s != "" {
			out = append(out, s)
		}
	}
	return out
}

// matchSegments matches route segments against request segments, capturing
// ":params". Returns ok=false on any mismatch (different length or a fixed
// segment that differs).
func matchSegments(routeSeg, reqSeg []string) (map[string]string, bool) {
	if len(routeSeg) != len(reqSeg) {
		return nil, false
	}
	params := map[string]string{}
	for i, rseg := range routeSeg {
		if name, ok := strings.CutPrefix(rseg, ":"); ok {
			params[name] = reqSeg[i]
		} else if rseg != reqSeg[i] {
			return nil, false
		}
	}
	return params, true
}
