//! Event-driven I/O with no runtime: a single-threaded echo server that
//! multiplexes many connections with the `poll(2)` syscall.
//!
//! This is the mechanism under every async runtime — tokio, asyncio, Go's
//! netpoller: put sockets in non-blocking mode, ask the kernel which are
//! ready, and act only on those. One thread, many waiting connections.

use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::os::fd::AsRawFd;
use std::thread;
use std::time::{Duration, Instant};

/// One entry of the fd set handed to `poll(2)` — mirrors C's `struct pollfd`.
#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}

/// "There is data to read" — from `<poll.h>`.
const POLLIN: i16 = 0x001;

extern "C" {
    /// int poll(struct pollfd *fds, nfds_t nfds, int timeout);
    fn poll(fds: *mut PollFd, nfds: u64, timeout_ms: i32) -> i32;
}

/// Serves newline-terminated messages on `listener`, echoing each one back
/// and closing the connection, until `n` messages have been echoed.
/// Returns the number served (always `n` unless poll fails).
///
/// The whole server is ONE thread: `poll` sleeps until the listener or any
/// connection is ready, and each wake-up does only the work that is ready —
/// partial messages are buffered per connection and finished later.
pub fn serve_n_echoes(listener: &TcpListener, n: usize) -> usize {
    listener
        .set_nonblocking(true)
        .expect("nonblocking listener");
    let mut conns: Vec<(TcpStream, Vec<u8>)> = Vec::new();
    let mut served = 0;

    while served < n {
        // Rebuild the interest set: the listener plus every live connection.
        let mut fds = Vec::with_capacity(1 + conns.len());
        fds.push(PollFd {
            fd: listener.as_raw_fd(),
            events: POLLIN,
            revents: 0,
        });
        for (stream, _) in &conns {
            fds.push(PollFd {
                fd: stream.as_raw_fd(),
                events: POLLIN,
                revents: 0,
            });
        }

        // Sleep until something is readable (5s safety timeout).
        let rc = unsafe { poll(fds.as_mut_ptr(), fds.len() as u64, 5000) };
        assert!(rc >= 0, "poll failed");

        let known = conns.len(); // fds[1..=known] map to conns[0..known]

        // Ready to accept? Drain the accept queue without blocking.
        if fds[0].revents & POLLIN != 0 {
            loop {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream.set_nonblocking(true).expect("nonblocking conn");
                        conns.push((stream, Vec::new()));
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(e) => panic!("accept: {e}"),
                }
            }
        }

        // Read from every connection the kernel marked ready.
        let mut finished: Vec<usize> = Vec::new();
        for i in 0..known {
            if fds[i + 1].revents & POLLIN == 0 {
                continue;
            }
            let (stream, buf) = &mut conns[i];
            let mut chunk = [0u8; 1024];
            match stream.read(&mut chunk) {
                Ok(0) => finished.push(i), // peer closed mid-message
                Ok(k) => {
                    buf.extend_from_slice(&chunk[..k]);
                    if buf.contains(&b'\n') {
                        stream.write_all(buf).expect("echo write");
                        served += 1;
                        finished.push(i);
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(_) => finished.push(i),
            }
        }
        // Remove finished connections back-to-front so indices stay valid.
        for i in finished.into_iter().rev() {
            conns.remove(i);
        }
    }
    served
}

/// Spawns `n` OS threads that each sleep for `pause`, joins them all, and
/// returns the wall-clock elapsed time. The thread-per-wait baseline the
/// event loop competes against.
pub fn thread_sleepers(n: usize, pause: Duration) -> Duration {
    let start = Instant::now();
    let handles: Vec<_> = (0..n)
        .map(|_| thread::spawn(move || thread::sleep(pause)))
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    start.elapsed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;

    fn start_server(n: usize) -> (std::net::SocketAddr, thread::JoinHandle<usize>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || serve_n_echoes(&listener, n));
        (addr, handle)
    }

    fn roundtrip(addr: std::net::SocketAddr, msg: &str) -> String {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        writeln!(stream, "{msg}").unwrap();
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line).unwrap();
        line.trim_end().to_string()
    }

    #[test]
    fn echoes_one_message() {
        let (addr, handle) = start_server(1);
        assert_eq!(roundtrip(addr, "hello"), "hello");
        assert_eq!(handle.join().unwrap(), 1);
    }

    #[test]
    fn echoes_many_clients_on_one_thread() {
        let (addr, handle) = start_server(5);
        for i in 0..5 {
            let msg = format!("client {i}");
            assert_eq!(roundtrip(addr, &msg), msg);
        }
        assert_eq!(handle.join().unwrap(), 5);
    }

    #[test]
    fn interleaves_partial_messages() {
        // Client A sends half a message; client B connects afterwards, sends
        // a full one, and completes FIRST — the single-threaded loop was not
        // stuck waiting for A. Then A finishes and is served too.
        let (addr, handle) = start_server(2);

        let mut a = TcpStream::connect(addr).unwrap();
        a.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        a.write_all(b"first ha").unwrap(); // no newline yet: incomplete

        let b_reply = roundtrip(addr, "second full message");
        assert_eq!(b_reply, "second full message");

        a.write_all(b"lf\n").unwrap(); // now complete A's message
        let mut line = String::new();
        BufReader::new(a).read_line(&mut line).unwrap();
        assert_eq!(line.trim_end(), "first half");

        assert_eq!(handle.join().unwrap(), 2);
    }

    #[test]
    fn thread_sleepers_overlap_their_waits() {
        // 50 threads sleeping 50ms each finish in ~one sleep, not 2.5s.
        let elapsed = thread_sleepers(50, Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(500), "took {elapsed:?}");
    }
}
