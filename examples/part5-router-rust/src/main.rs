//! Demo: wire up a tiny app and dispatch a few requests, showing routing,
//! path params, and a logging + auth middleware chain in action.

use part5_router_rust::{Request, Response, Router};

fn main() {
    let mut app = Router::new();

    // Logging middleware: runs outermost, wraps every request.
    app.use_middleware(|req: &Request, next: &dyn Fn(&Request) -> Response| {
        let res = next(req);
        println!("{} {} -> {}", req.method, req.path, res.status);
        res
    });

    // Auth middleware: reject the admin area (toy check on the path).
    app.use_middleware(|req: &Request, next: &dyn Fn(&Request) -> Response| {
        if req.path.starts_with("/admin") {
            return Response::new(401, "401 Unauthorized");
        }
        next(req)
    });

    app.route("GET", "/", |_| Response::new(200, "home"));
    app.route("GET", "/users/:id", |req| {
        Response::new(200, format!("user profile: {}", req.params["id"]))
    });
    app.route("GET", "/admin", |_| Response::new(200, "admin panel"));

    for (method, path) in [
        ("GET", "/"),
        ("GET", "/users/7"),
        ("GET", "/admin"),
        ("GET", "/nope"),
        ("POST", "/"),
    ] {
        let res = app.dispatch(method, path);
        println!("    => {} {:?}", res.status, res.body);
    }
}
