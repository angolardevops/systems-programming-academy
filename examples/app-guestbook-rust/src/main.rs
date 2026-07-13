//! guestbook — usage: guestbook [db-path] [port]
//!
//! Runs the guestbook web app: a thread-pool HTTP server backed by real SQLite.
//! Defaults to guestbook.db on port 8080. Open http://127.0.0.1:8080 and post a
//! comment; try `<script>` or a SQL payload and watch both defences hold.

use app_guestbook_rust::{open_db, serve};
use std::net::TcpListener;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let db_path = args.get(1).map(String::as_str).unwrap_or("guestbook.db");
    let port = args.get(2).map(String::as_str).unwrap_or("8080");

    let conn = open_db(db_path);
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).expect("bind");
    println!(
        "guestbook listening on http://{}",
        listener.local_addr().unwrap()
    );
    serve(listener, conn, 8);
}
