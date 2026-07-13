//! mini-NGINX: a concurrent static-file HTTP server.
//!
//! Concurrency model: a **fixed thread pool** fed by a channel — the worker
//! pool from the Message Passing lesson, now doing real protocol work over
//! blocking sockets. Bounded workers means bounded memory and file
//! descriptors: the capacity dial the fan-out-everything model lacks.
//!
//! The Go and Python twins serve byte-identical responses; the tests and the
//! load harness prove it.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/// A parsed request line: method and URL path (query strings not supported).
pub struct Request {
    pub method: String,
    pub path: String,
}

/// Reads and parses one HTTP request head (request line + headers) from the
/// stream. Returns `Err(400)` on anything malformed.
pub fn parse_request(stream: &mut TcpStream) -> Result<Request, u16> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if reader.read_line(&mut line).map_err(|_| 400u16)? == 0 {
        return Err(400);
    }
    let mut parts = line.split_whitespace();
    let (method, path) = match (parts.next(), parts.next(), parts.next()) {
        (Some(m), Some(p), Some(v)) if v.starts_with("HTTP/") => (m, p),
        _ => return Err(400),
    };
    let request = Request {
        method: method.to_string(),
        path: path.to_string(),
    };
    // Drain the headers; we serve statelessly and ignore them all.
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).map_err(|_| 400u16)? == 0 {
            break;
        }
        if header == "\r\n" || header == "\n" {
            break;
        }
    }
    Ok(request)
}

/// Maps a URL path to a file inside `docroot`, or `None` if the path tries
/// to escape it. Purely lexical: any `..` component is rejected outright —
/// the request never touches the filesystem outside the root.
pub fn resolve(docroot: &Path, url_path: &str) -> Option<PathBuf> {
    let path = if url_path == "/" {
        "/index.html"
    } else {
        url_path
    };
    let mut resolved = docroot.to_path_buf();
    for component in path.split('/') {
        match component {
            "" | "." => continue,
            ".." => return None, // traversal attempt: never leaves docroot
            c if c.contains('\0') => return None,
            c => resolved.push(c),
        }
    }
    Some(resolved)
}

/// Content-Type by file extension — the tiny subset a static site needs.
pub fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("txt") => "text/plain",
        _ => "application/octet-stream",
    }
}

/// Serializes a full HTTP/1.0 response. The exact bytes here are the
/// cross-language contract: Go and Python emit the identical header block.
pub fn build_response(status: u16, ctype: &str, body: &[u8]) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    };
    let mut response = format!(
        "HTTP/1.0 {status} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

fn error_response(status: u16) -> Vec<u8> {
    let body = match status {
        400 => "400 Bad Request\n",
        404 => "404 Not Found\n",
        405 => "405 Method Not Allowed\n",
        _ => "500 Internal Server Error\n",
    };
    build_response(status, "text/plain", body.as_bytes())
}

/// Handles one connection: parse, resolve, read the file, respond, close.
pub fn handle_connection(mut stream: TcpStream, docroot: &Path) {
    let response = match parse_request(&mut stream) {
        Err(status) => error_response(status),
        Ok(req) if req.method != "GET" => error_response(405),
        Ok(req) => match resolve(docroot, &req.path) {
            None => error_response(404),
            Some(file) => match std::fs::read(&file) {
                Err(_) => error_response(404),
                Ok(body) => build_response(200, content_type(&file), &body),
            },
        },
    };
    let _ = stream.write_all(&response);
}

/// Runs the server: `n_workers` threads pull connections from a channel and
/// serve them. Returns only if the listener fails. This is the Rust Book's
/// final-project shape, built on this Part's lessons.
pub fn serve(listener: TcpListener, docroot: PathBuf, n_workers: usize) {
    assert!(n_workers > 0, "need at least one worker");
    let (tx, rx) = mpsc::channel::<TcpStream>();
    let rx = Arc::new(Mutex::new(rx));
    let docroot = Arc::new(docroot);

    for _ in 0..n_workers {
        let rx = Arc::clone(&rx);
        let docroot = Arc::clone(&docroot);
        thread::spawn(move || {
            // recv() returns Err when the sender is dropped: clean shutdown.
            while let Ok(stream) = { rx.lock().unwrap().recv() } {
                handle_connection(stream, &docroot);
            }
        });
    }

    for stream in listener.incoming().flatten() {
        if tx.send(stream).is_err() {
            break;
        }
    }
}

/// Reads a full HTTP response from raw bytes into (status, headers, body) —
/// shared by the tests and the load harness.
pub fn read_response(stream: &mut TcpStream) -> (u16, String, Vec<u8>) {
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).expect("read response");
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("header terminator");
    let head = String::from_utf8_lossy(&raw[..split]).to_string();
    let status: u16 = head
        .split_whitespace()
        .nth(1)
        .expect("status code")
        .parse()
        .expect("numeric status");
    (status, head, raw[split + 4..].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static DIR_SEQ: AtomicU32 = AtomicU32::new(0);

    /// Creates a fresh docroot in the OS temp dir with index.html and
    /// style.css inside it — and secret.txt one level OUTSIDE it, as the
    /// traversal target that must never be served.
    fn setup_docroot() -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "mininginx-test-{}-{}",
            std::process::id(),
            DIR_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let docroot = base.join("public");
        std::fs::create_dir_all(&docroot).unwrap();
        std::fs::write(docroot.join("index.html"), "<h1>home</h1>\n").unwrap();
        std::fs::write(docroot.join("style.css"), "body{}\n").unwrap();
        std::fs::write(base.join("secret.txt"), "TOP SECRET\n").unwrap();
        docroot
    }

    fn start_server(n_workers: usize) -> std::net::SocketAddr {
        let docroot = setup_docroot();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || serve(listener, docroot, n_workers));
        addr
    }

    fn request(addr: std::net::SocketAddr, raw: &str) -> (u16, String, Vec<u8>) {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(raw.as_bytes()).unwrap();
        read_response(&mut stream)
    }

    #[test]
    fn serves_index_for_root() {
        let addr = start_server(2);
        let (status, head, body) = request(addr, "GET / HTTP/1.0\r\n\r\n");
        assert_eq!(status, 200);
        assert!(head.contains("Content-Type: text/html"));
        assert_eq!(body, b"<h1>home</h1>\n");
    }

    #[test]
    fn serves_css_with_content_type_and_length() {
        let addr = start_server(2);
        let (status, head, body) = request(addr, "GET /style.css HTTP/1.0\r\n\r\n");
        assert_eq!(status, 200);
        assert!(head.contains("Content-Type: text/css"));
        assert!(head.contains(&format!("Content-Length: {}", body.len())));
    }

    #[test]
    fn missing_file_is_404() {
        let addr = start_server(2);
        let (status, _, body) = request(addr, "GET /nope.html HTTP/1.0\r\n\r\n");
        assert_eq!(status, 404);
        assert_eq!(body, b"404 Not Found\n");
    }

    #[test]
    fn post_is_405() {
        let addr = start_server(2);
        let (status, _, _) = request(addr, "POST / HTTP/1.0\r\n\r\n");
        assert_eq!(status, 405);
    }

    #[test]
    fn traversal_never_escapes_docroot() {
        let addr = start_server(2);
        let (status, _, body) = request(addr, "GET /../secret.txt HTTP/1.0\r\n\r\n");
        assert_eq!(status, 404, "traversal must be rejected");
        assert!(!body.windows(6).any(|w| w == b"SECRET"));
    }

    #[test]
    fn garbage_is_400() {
        let addr = start_server(2);
        let (status, _, _) = request(addr, "NOT-HTTP\r\n\r\n");
        assert_eq!(status, 400);
    }

    #[test]
    fn concurrent_clients_all_succeed() {
        let addr = start_server(4);
        let handles: Vec<_> = (0..16)
            .map(|_| {
                thread::spawn(move || {
                    let (status, _, body) = request(addr, "GET / HTTP/1.0\r\n\r\n");
                    (status, body)
                })
            })
            .collect();
        for h in handles {
            let (status, body) = h.join().unwrap();
            assert_eq!(status, 200);
            assert_eq!(body, b"<h1>home</h1>\n");
        }
    }

    #[test]
    fn resolve_rejects_dotdot_and_maps_root() {
        let root = Path::new("/srv/www");
        assert_eq!(
            resolve(root, "/").unwrap(),
            Path::new("/srv/www/index.html")
        );
        assert_eq!(
            resolve(root, "/a/b.txt").unwrap(),
            Path::new("/srv/www/a/b.txt")
        );
        assert!(resolve(root, "/../etc/passwd").is_none());
        assert!(resolve(root, "/a/../../etc/passwd").is_none());
    }
}
