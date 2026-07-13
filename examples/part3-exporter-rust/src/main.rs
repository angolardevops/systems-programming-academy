//! Hand-rolled HTTP: accept a connection, read the request head, answer
//! `/metrics` with the exposition text. No dependencies — a taste of the
//! Part 4 mini-NGINX.

use std::io::{Read, Write};
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let port = std::env::args().nth(1).unwrap_or_else(|| "9100".into());
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))?;
    eprintln!("exporter listening on :{port}");

    for stream in listener.incoming() {
        let mut stream = stream?;
        // Read (up to) the request head; we only need the first line.
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).unwrap_or(0);
        let head = String::from_utf8_lossy(&buf[..n]);
        let ok = head.starts_with("GET /metrics");

        let (status, body) = if ok {
            ("200 OK", exporter::demo_registry().render())
        } else {
            ("404 Not Found", "not found\n".to_string())
        };
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
    }
    Ok(())
}
