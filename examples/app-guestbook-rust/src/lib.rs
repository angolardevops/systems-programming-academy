//! The capstone guestbook, promoted to a real running web app: a fixed
//! thread-pool HTTP server (the Part 4 mini-NGINX model) backed by a real
//! SQLite database via `rusqlite`.
//!
//! The security defences are unchanged from the capstone — parameterized
//! inserts, autoescaped rendering — but now they run over real TCP sockets
//! against a real SQLite engine, so the SQL-injection proof is against actual
//! SQLite, not a stand-in.
//!
//! Routes:
//! * `GET /`         — render the guestbook page.
//! * `POST /comment` — parse the form, validate, insert (parameterized). On
//!   success, 303-redirect to `/`; on validation failure, 400 with the errors.
//!
//! The one dependency (`rusqlite`, bundled) buys a real database. Concurrency:
//! a fixed pool of worker threads shares one SQLite connection behind a
//! `Mutex` — simple and correct for a teaching app; a production server would
//! use a connection pool.

use rusqlite::{params, Connection};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// ---------------------------------------------------------------------------
// Domain logic (validation, escaping, rendering) — same as the capstone.
// ---------------------------------------------------------------------------

/// Returns every validation error at once as `"field: message"` lines.
pub fn validate_submission(author: &str, body: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let author = author.trim();
    let body = body.trim();
    if author.is_empty() {
        errors.push("author: is required".to_string());
    } else if author.chars().count() < 2 {
        errors.push("author: must be at least 2 characters".to_string());
    } else if author.chars().count() > 40 {
        errors.push("author: must be at most 40 characters".to_string());
    }
    if body.is_empty() {
        errors.push("body: is required".to_string());
    } else if body.chars().count() > 500 {
        errors.push("body: must be at most 500 characters".to_string());
    }
    errors
}

/// HTML-escapes text (`&` first, to avoid double-escaping later entities).
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ---------------------------------------------------------------------------
// Real SQLite storage. The insert is parameterized: rusqlite binds the value,
// so a '; DROP TABLE ... payload is stored as data, never executed.
// ---------------------------------------------------------------------------

/// Opens (or creates) the database and ensures the comments table exists.
pub fn open_db(path: &str) -> Connection {
    let conn = Connection::open(path).expect("open sqlite");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS comments (\
             id INTEGER PRIMARY KEY AUTOINCREMENT, author TEXT NOT NULL, body TEXT NOT NULL)",
        [],
    )
    .expect("create table");
    conn
}

/// Runs a parameterized INSERT — the values are bound, never spliced into SQL.
pub fn insert_comment(conn: &Connection, author: &str, body: &str) {
    conn.execute(
        "INSERT INTO comments (author, body) VALUES (?1, ?2)",
        params![author.trim(), body.trim()],
    )
    .expect("insert");
}

/// Returns every comment, oldest first, as `(author, body)` pairs.
pub fn all_comments(conn: &Connection) -> Vec<(String, String)> {
    let mut stmt = conn
        .prepare("SELECT author, body FROM comments ORDER BY id")
        .expect("prepare");
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .expect("query");
    rows.map(|r| r.expect("row")).collect()
}

/// Renders the full HTML page, every value autoescaped.
pub fn render_page(conn: &Connection) -> String {
    let mut items = String::new();
    for (author, body) in all_comments(conn) {
        items.push_str(&format!(
            "  <li><strong>{}</strong>: {}</li>\n",
            escape_html(&author),
            escape_html(&body)
        ));
    }
    format!(
        "<!doctype html>\n<html><head><title>Guestbook</title></head><body>\n\
         <h1>Guestbook</h1>\n\
         <ul class=\"guestbook\">\n{items}</ul>\n\
         <form method=\"post\" action=\"/comment\">\n\
         \x20 <input name=\"author\" placeholder=\"name\">\n\
         \x20 <input name=\"body\" placeholder=\"message\">\n\
         \x20 <button>Post</button>\n\
         </form>\n</body></html>"
    )
}

// ---------------------------------------------------------------------------
// HTTP layer: parse the request head + form body, route, build the response.
// ---------------------------------------------------------------------------

/// Percent-decodes an application/x-www-form-urlencoded token (`+` -> space,
/// `%XX` -> byte).
fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = (bytes[i + 1] as char).to_digit(16);
                let lo = (bytes[i + 2] as char).to_digit(16);
                if let (Some(h), Some(l)) = (hi, lo) {
                    out.push((h * 16 + l) as u8);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Parses a form body into `(author, body)`, url-decoding each value.
pub fn parse_form(body: &str) -> (String, String) {
    let mut author = String::new();
    let mut message = String::new();
    for pair in body.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "author" => author = url_decode(v),
                "body" => message = url_decode(v),
                _ => {}
            }
        }
    }
    (author, message)
}

fn response(status: u16, ctype: &str, body: &str, extra: &str) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        303 => "See Other",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Internal Server Error",
    };
    let mut out = format!(
        "HTTP/1.0 {status} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    out.extend_from_slice(body.as_bytes());
    out
}

/// Routes one request to a response. `GET /` renders; `POST /comment` submits.
pub fn handle_request(conn: &Connection, method: &str, path: &str, body: &str) -> Vec<u8> {
    match (method, path) {
        ("GET", "/") => response(200, "text/html", &render_page(conn), ""),
        ("POST", "/comment") => {
            let (author, message) = parse_form(body);
            let errors = validate_submission(&author, &message);
            if !errors.is_empty() {
                let mut page = String::from("<h1>Errors</h1>\n<ul>\n");
                for e in &errors {
                    page.push_str(&format!("  <li>{}</li>\n", escape_html(e)));
                }
                page.push_str("</ul>");
                return response(400, "text/html", &page, "");
            }
            insert_comment(conn, &author, &message);
            response(303, "text/plain", "", "Location: /\r\n")
        }
        _ => response(404, "text/plain", "404 Not Found\n", ""),
    }
}

fn handle_connection(mut stream: TcpStream, conn: &Mutex<Connection>) {
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let mut parts = request_line.split_whitespace();
    let (method, path) = match (parts.next(), parts.next()) {
        (Some(m), Some(p)) => (m.to_string(), p.to_string()),
        _ => return,
    };

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 && reader.read_exact(&mut body).is_err() {
        return;
    }
    let body = String::from_utf8_lossy(&body).into_owned();

    let out = {
        let guard = conn.lock().unwrap();
        handle_request(&guard, &method, &path, &body)
    };
    let _ = stream.write_all(&out);
}

/// Runs the server: `n_workers` threads share one SQLite connection behind a
/// `Mutex`, pulling connections from a channel.
pub fn serve(listener: TcpListener, conn: Connection, n_workers: usize) {
    let conn = Arc::new(Mutex::new(conn));
    let (tx, rx) = mpsc::channel::<TcpStream>();
    let rx = Arc::new(Mutex::new(rx));

    for _ in 0..n_workers {
        let rx = Arc::clone(&rx);
        let conn = Arc::clone(&conn);
        thread::spawn(move || {
            while let Ok(stream) = { rx.lock().unwrap().recv() } {
                handle_connection(stream, &conn);
            }
        });
    }
    for stream in listener.incoming().flatten() {
        if tx.send(stream).is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static SEQ: AtomicU32 = AtomicU32::new(0);

    fn start_server() -> (std::net::SocketAddr, Arc<Mutex<Connection>>) {
        // A unique temp-file DB per test.
        let path = std::env::temp_dir().join(format!(
            "guestbook-app-{}-{}.db",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&path);
        let conn = open_db(path.to_str().unwrap());
        // A second handle to the same DB file for assertions from the test.
        let probe = Arc::new(Mutex::new(open_db(path.to_str().unwrap())));

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || serve(listener, conn, 4));
        (addr, probe)
    }

    fn request(addr: std::net::SocketAddr, raw: &str) -> (u16, String, String) {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(raw.as_bytes()).unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        let (head, body) = resp.split_once("\r\n\r\n").unwrap_or((&resp, ""));
        let status: u16 = head.split_whitespace().nth(1).unwrap().parse().unwrap();
        (status, head.to_string(), body.to_string())
    }

    fn get(addr: std::net::SocketAddr, path: &str) -> (u16, String, String) {
        request(addr, &format!("GET {path} HTTP/1.0\r\n\r\n"))
    }

    fn post(addr: std::net::SocketAddr, path: &str, form: &str) -> (u16, String, String) {
        request(
            addr,
            &format!(
                "POST {path} HTTP/1.0\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{form}",
                form.len()
            ),
        )
    }

    #[test]
    fn get_root_renders_empty_guestbook() {
        let (addr, _probe) = start_server();
        let (status, _, body) = get(addr, "/");
        assert_eq!(status, 200);
        assert!(body.contains("<h1>Guestbook</h1>"));
    }

    #[test]
    fn post_valid_comment_redirects_and_persists() {
        let (addr, probe) = start_server();
        let (status, head, _) = post(addr, "/comment", "author=Ana&body=Hello");
        assert_eq!(status, 303);
        assert!(head.contains("Location: /"));
        assert_eq!(
            all_comments(&probe.lock().unwrap()),
            vec![("Ana".into(), "Hello".into())]
        );
        let (_, _, page) = get(addr, "/");
        assert!(page.contains("<strong>Ana</strong>: Hello"));
    }

    #[test]
    fn post_invalid_comment_is_400_and_persists_nothing() {
        let (addr, probe) = start_server();
        let (status, _, body) = post(addr, "/comment", "author=A&body=");
        assert_eq!(status, 400);
        assert!(body.contains("author: must be at least 2 characters"));
        assert!(body.contains("body: is required"));
        assert!(all_comments(&probe.lock().unwrap()).is_empty());
    }

    #[test]
    fn sql_injection_against_real_sqlite_table_survives() {
        let (addr, probe) = start_server();
        post(addr, "/comment", "author=Alice&body=first");
        // URL-encoded '; DROP TABLE comments; --
        let payload = "%27%3B+DROP+TABLE+comments%3B+--";
        let (status, _, _) = post(addr, "/comment", &format!("author=Mallory&body={payload}"));
        assert_eq!(status, 303);
        // The REAL table survives with both rows.
        let rows = all_comments(&probe.lock().unwrap());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], ("Alice".to_string(), "first".to_string()));
        assert_eq!(rows[1].1, "'; DROP TABLE comments; --");
    }

    #[test]
    fn xss_payload_renders_inert() {
        let (addr, _probe) = start_server();
        post(
            addr,
            "/comment",
            "author=Eve&body=%3Cscript%3Ealert(1)%3C%2Fscript%3E",
        );
        let (_, _, page) = get(addr, "/");
        assert!(page.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!page.contains("<script>alert(1)"));
    }

    #[test]
    fn unknown_route_is_404() {
        let (addr, _probe) = start_server();
        let (status, _, _) = get(addr, "/nope");
        assert_eq!(status, 404);
    }
}
