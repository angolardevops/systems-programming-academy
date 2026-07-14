//! ping — send ICMP echo requests and time the replies.
//!
//! Usage: `ping <host> [count]`   (needs root or CAP_NET_RAW)
//!
//! The packet building, checksum, reply parsing, and statistics live in the
//! library and are fully tested without privileges. Only *this* file needs a
//! raw socket — the one thing the kernel guards behind root, because a raw
//! socket can forge any packet. Run it with `sudo`, or grant the capability
//! once: `sudo setcap cap_net_raw+ep ./target/debug/ping`.

use part6_ping_rust::{build_echo_request, parse_echo_reply, summarize};
use std::net::ToSocketAddrs;
use std::process::exit;
use std::time::{Duration, Instant};

// --- the one syscall family that needs a raw socket -------------------------
const AF_INET: i32 = 2;
const SOCK_RAW: i32 = 3;
const IPPROTO_ICMP: i32 = 1;
const SOL_SOCKET: i32 = 1;
const SO_RCVTIMEO: i32 = 20;

extern "C" {
    fn socket(domain: i32, ty: i32, protocol: i32) -> i32;
    fn setsockopt(fd: i32, level: i32, name: i32, val: *const u8, len: u32) -> i32;
    fn sendto(fd: i32, buf: *const u8, len: usize, flags: i32, addr: *const u8, alen: u32)
        -> isize;
    fn recvfrom(
        fd: i32,
        buf: *mut u8,
        len: usize,
        flags: i32,
        addr: *mut u8,
        alen: *mut u32,
    ) -> isize;
    fn close(fd: i32) -> i32;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let host = args.get(1).map(String::as_str).unwrap_or("127.0.0.1");
    let count: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
    if let Err(e) = run(host, count) {
        eprintln!("ping: {e}");
        exit(1);
    }
}

fn run(host: &str, count: u16) -> std::io::Result<()> {
    // Resolve the host to an IPv4 address using the standard resolver.
    let octets = (host, 0u16)
        .to_socket_addrs()?
        .find_map(|a| match a {
            std::net::SocketAddr::V4(v4) => Some(v4.ip().octets()),
            _ => None,
        })
        .ok_or_else(|| std::io::Error::other("no IPv4 address"))?;
    let ip = format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3]);

    // A raw ICMP socket — this is the privileged line.
    let fd = unsafe { socket(AF_INET, SOCK_RAW, IPPROTO_ICMP) };
    if fd < 0 {
        let err = std::io::Error::last_os_error();
        if err.kind() == std::io::ErrorKind::PermissionDenied {
            eprintln!(
                "ping: raw sockets need root. Try `sudo ping {host}`, or grant the\n\
                 capability once with `sudo setcap cap_net_raw+ep <binary>`."
            );
        }
        return Err(err);
    }

    // A 1-second receive timeout so a lost reply doesn't hang the loop.
    let tv = timeval_bytes(1, 0);
    unsafe { setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, tv.as_ptr(), tv.len() as u32) };

    let dest = sockaddr_in(octets);
    let id = std::process::id() as u16;
    println!("PING {host} ({ip}): 56 data bytes");

    let mut rtts = Vec::new();
    for seq in 1..=count {
        let packet = build_echo_request(id, seq, &[0x42; 56]);
        let sent = Instant::now();
        let n = unsafe {
            sendto(
                fd,
                packet.as_ptr(),
                packet.len(),
                0,
                dest.as_ptr(),
                dest.len() as u32,
            )
        };
        if n < 0 {
            eprintln!("ping: send failed: {}", std::io::Error::last_os_error());
            continue;
        }
        // The kernel hands back the whole IP datagram; skip the IP header.
        let mut buf = [0u8; 1500];
        let got = unsafe {
            recvfrom(
                fd,
                buf.as_mut_ptr(),
                buf.len(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if got < 0 {
            println!("Request timeout for icmp_seq {seq}");
            continue;
        }
        let elapsed = sent.elapsed();
        let datagram = &buf[..got as usize];
        let ihl = ((datagram[0] & 0x0f) as usize) * 4;
        if let Some(reply) = parse_echo_reply(&datagram[ihl..]) {
            if reply.id == id {
                let ms = elapsed.as_secs_f64() * 1000.0;
                rtts.push(ms);
                println!("64 bytes from {ip}: icmp_seq={} time={ms:.3} ms", reply.seq);
            }
        }
        if seq < count {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    unsafe { close(fd) };
    println!("\n{}", summarize(host, count as usize, &rtts));
    Ok(())
}

/// A `struct timeval` { i64 sec; i64 usec } as raw bytes.
fn timeval_bytes(sec: i64, usec: i64) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[0..8].copy_from_slice(&sec.to_ne_bytes());
    b[8..16].copy_from_slice(&usec.to_ne_bytes());
    b
}

/// A `struct sockaddr_in` for the destination as raw bytes: family, port 0,
/// the address in network order, then padding.
fn sockaddr_in(octets: [u8; 4]) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[0..2].copy_from_slice(&(AF_INET as u16).to_ne_bytes());
    // port stays 0; sin_addr is the four octets in network (big-endian) order.
    b[4..8].copy_from_slice(&octets);
    b
}
