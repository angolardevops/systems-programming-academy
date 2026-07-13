//! mininginx — usage: mininginx <docroot> <port> [workers]
//!
//! Serves static files from <docroot> on 127.0.0.1:<port> with a fixed
//! thread pool (default: 8 workers). Port 0 picks an ephemeral port and
//! prints it, so scripts and the load harness can discover it.

use part4_mininginx_rust::serve;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: mininginx <docroot> <port> [workers]");
        exit(2);
    }
    let docroot = PathBuf::from(&args[1]);
    if !docroot.is_dir() {
        eprintln!("error: docroot {} is not a directory", docroot.display());
        exit(2);
    }
    let port: u16 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("error: invalid port {}", args[2]);
        exit(2);
    });
    let workers: usize = args
        .get(3)
        .map(|w| w.parse().expect("numeric workers"))
        .unwrap_or(8);

    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| {
        eprintln!("error: bind failed: {e}");
        exit(1);
    });
    println!("listening on {}", listener.local_addr().unwrap());
    serve(listener, docroot, workers);
}
