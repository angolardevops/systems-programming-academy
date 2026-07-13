//! A micro web framework: routing table, path parameters, and a middleware
//! chain — the abstraction the mini-NGINX server lacked.
//!
//! The framework/library distinction lives here: you don't call the router in
//! a loop, you *register handlers* and hand it control. It calls you back when
//! a request matches. That inversion — "don't call us, we'll call you" — is
//! what makes this a framework.
//!
//! Everything is pure dispatch over `Request`/`Response` structs, so the whole
//! thing is testable without a socket. Bolting it onto the Part 4 server is a
//! matter of turning bytes into a `Request` and a `Response` back into bytes.

use std::collections::HashMap;

/// An incoming request after parsing: method, path, and the path parameters
/// captured by the matched route (e.g. `/users/:id` fills `id`).
#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub params: HashMap<String, String>,
}

impl Request {
    pub fn new(method: &str, path: &str) -> Self {
        Request {
            method: method.to_string(),
            path: path.to_string(),
            params: HashMap::new(),
        }
    }
}

/// A response: status code and body. Real frameworks carry headers too; this
/// is the irreducible core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

impl Response {
    pub fn new(status: u16, body: impl Into<String>) -> Self {
        Response {
            status,
            body: body.into(),
        }
    }
}

/// A handler is any function from a request to a response. Boxed so we can
/// store handlers of different concrete types in one routing table.
pub type Handler = Box<dyn Fn(&Request) -> Response + Send + Sync>;

/// Middleware wraps a handler, producing a new handler — the decorator
/// pattern. It can run code before (auth, logging), after (headers,
/// timing), or short-circuit (reject) the inner handler.
pub type Middleware =
    Box<dyn Fn(&Request, &dyn Fn(&Request) -> Response) -> Response + Send + Sync>;

/// One registered route: the method, the pattern split into segments, and
/// the handler to call on a match.
struct Route {
    method: String,
    segments: Vec<String>,
    handler: Handler,
}

/// The router: register routes and middleware, then `dispatch`. Middleware
/// applies outermost-first (the first added is the outermost wrapper).
#[derive(Default)]
pub struct Router {
    routes: Vec<Route>,
    middleware: Vec<Middleware>,
}

impl Router {
    pub fn new() -> Self {
        Router::default()
    }

    /// Registers a handler for `method` + `pattern`. A segment starting with
    /// `:` is a path parameter, e.g. `/users/:id`.
    pub fn route(
        &mut self,
        method: &str,
        pattern: &str,
        handler: impl Fn(&Request) -> Response + Send + Sync + 'static,
    ) -> &mut Self {
        self.routes.push(Route {
            method: method.to_string(),
            segments: split_path(pattern),
            handler: Box::new(handler),
        });
        self
    }

    /// Adds a middleware to the chain. The first added runs outermost.
    pub fn use_middleware(
        &mut self,
        mw: impl Fn(&Request, &dyn Fn(&Request) -> Response) -> Response + Send + Sync + 'static,
    ) -> &mut Self {
        self.middleware.push(Box::new(mw));
        self
    }

    /// Routes a request: run the middleware chain around the router core.
    /// Middleware wraps the *entire* dispatch, so it observes 404s and 405s
    /// too — a logging middleware must see every request, not just matched
    /// ones. Inside the core: 404 if no path matches, 405 if the path
    /// matches but not the method, else the matched handler with path params
    /// bound.
    pub fn dispatch(&self, method: &str, path: &str) -> Response {
        let core = move |req: &Request| self.route_request(req);
        let base = Request::new(method, path);
        self.run_chain(&core, &base)
    }

    /// The router core, wrapped by middleware: pure matching, no chain.
    fn route_request(&self, req: &Request) -> Response {
        let request_segments = split_path(&req.path);
        let mut path_matched = false;

        for route in &self.routes {
            if let Some(params) = match_segments(&route.segments, &request_segments) {
                path_matched = true;
                if route.method == req.method {
                    let mut matched = req.clone();
                    matched.params = params;
                    return (route.handler)(&matched);
                }
            }
        }

        if path_matched {
            Response::new(405, "405 Method Not Allowed")
        } else {
            Response::new(404, "404 Not Found")
        }
    }

    /// Folds the middleware around `core`. Building from the innermost out
    /// means index 0 ends up outermost, matching every framework's "first
    /// registered runs first" convention.
    fn run_chain(&self, core: &dyn Fn(&Request) -> Response, req: &Request) -> Response {
        let mut next: Box<dyn Fn(&Request) -> Response + '_> = Box::new(move |r| core(r));
        for mw in self.middleware.iter().rev() {
            let inner = next;
            next = Box::new(move |r| mw(r, &*inner));
        }
        next(req)
    }
}

/// Splits a path into non-empty segments: `/a/b/` -> `["a", "b"]`.
fn split_path(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Matches route segments against request segments, capturing `:params`.
/// Returns `None` on any mismatch (different length or a fixed segment that
/// differs).
fn match_segments(route: &[String], request: &[String]) -> Option<HashMap<String, String>> {
    if route.len() != request.len() {
        return None;
    }
    let mut params = HashMap::new();
    for (r, actual) in route.iter().zip(request.iter()) {
        if let Some(name) = r.strip_prefix(':') {
            params.insert(name.to_string(), actual.clone());
        } else if r != actual {
            return None;
        }
    }
    Some(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    fn hello_router() -> Router {
        let mut r = Router::new();
        r.route("GET", "/", |_| Response::new(200, "home"));
        r.route("GET", "/users/:id", |req| {
            Response::new(200, format!("user {}", req.params["id"]))
        });
        r.route("POST", "/users", |_| Response::new(201, "created"));
        r
    }

    #[test]
    fn dispatches_static_route() {
        let r = hello_router();
        assert_eq!(r.dispatch("GET", "/"), Response::new(200, "home"));
    }

    #[test]
    fn captures_path_parameter() {
        let r = hello_router();
        assert_eq!(
            r.dispatch("GET", "/users/42"),
            Response::new(200, "user 42")
        );
    }

    #[test]
    fn unknown_path_is_404() {
        let r = hello_router();
        assert_eq!(
            r.dispatch("GET", "/nope"),
            Response::new(404, "404 Not Found")
        );
    }

    #[test]
    fn known_path_wrong_method_is_405() {
        let r = hello_router();
        // /users/:id exists for GET; DELETE should be 405, not 404.
        assert_eq!(
            r.dispatch("DELETE", "/users/42"),
            Response::new(405, "405 Method Not Allowed")
        );
    }

    #[test]
    fn method_disambiguates_same_path() {
        let mut r = Router::new();
        r.route("GET", "/x", |_| Response::new(200, "get"));
        r.route("POST", "/x", |_| Response::new(200, "post"));
        assert_eq!(r.dispatch("GET", "/x").body, "get");
        assert_eq!(r.dispatch("POST", "/x").body, "post");
    }

    #[test]
    fn middleware_runs_around_handler() {
        let mut r = Router::new();
        r.use_middleware(|req, next| {
            let mut res = next(req);
            res.body = format!("[wrapped: {}]", res.body);
            res
        });
        r.route("GET", "/", |_| Response::new(200, "core"));
        assert_eq!(r.dispatch("GET", "/").body, "[wrapped: core]");
    }

    #[test]
    fn middleware_can_short_circuit() {
        let mut r = Router::new();
        // An "auth" middleware that rejects everything without ever calling
        // the handler.
        r.use_middleware(|_req, _next| Response::new(401, "401 Unauthorized"));
        r.route("GET", "/secret", |_| Response::new(200, "TOP SECRET"));
        let res = r.dispatch("GET", "/secret");
        assert_eq!(res.status, 401);
        assert!(!res.body.contains("SECRET"));
    }

    #[test]
    fn middleware_order_is_outermost_first() {
        let mut r = Router::new();
        r.use_middleware(|req, next| {
            let mut res = next(req);
            res.body.push('A'); // outer: appends last (unwinds last)
            res
        });
        r.use_middleware(|req, next| {
            let mut res = next(req);
            res.body.push('B'); // inner
            res
        });
        r.route("GET", "/", |_| Response::new(200, ""));
        // Handler runs, then inner (B), then outer (A): "BA".
        assert_eq!(r.dispatch("GET", "/").body, "BA");
    }

    #[test]
    fn middleware_sees_every_request_once() {
        let count = Arc::new(AtomicU32::new(0));
        let c = Arc::clone(&count);
        let mut r = Router::new();
        r.use_middleware(move |req, next| {
            c.fetch_add(1, Ordering::Relaxed);
            next(req)
        });
        r.route("GET", "/", |_| Response::new(200, "ok"));
        r.dispatch("GET", "/");
        r.dispatch("GET", "/missing");
        assert_eq!(count.load(Ordering::Relaxed), 2);
    }
}
