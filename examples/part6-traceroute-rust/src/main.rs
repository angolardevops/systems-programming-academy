//! traceroute — trace the path to a host by stepping the IP TTL.
//!
//! Usage: `traceroute <host> [max_hops]`   (needs root or CAP_NET_RAW)
//!
//! The probe building, checksum, ICMP classification, and rendering live in the
//! library and are fully tested without privileges. Only this file needs a raw
//! socket — to set the outgoing TTL and read the ICMP replies. Run with `sudo`,
//! or grant the capability once: `sudo setcap cap_net_raw+ep ./target/debug/traceroute`.

use part6_traceroute_rust::{build_echo_request, classify, render_header, render_hop, Reply};
use std::net::ToSocketAddrs;
use std::process::exit;
use std::time::Instant;

const AF_INET: i32 = 2;
const SOCK_RAW: i32 = 3;
const IPPROTO_IP: i32 = 0;
const IPPROTO_ICMP: i32 = 1;
const SOL_SOCKET: i32 = 1;
const SO_RCVTIMEO: i32 = 20;
const IP_TTL: i32 = 2;
const PROBES_PER_HOP: usize = 3;

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
    let max_hops: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(30);
    if let Err(e) = run(host, max_hops) {
        eprintln!("traceroute: {e}");
        exit(1);
    }
}

fn run(host: &str, max_hops: usize) -> std::io::Result<()> {
    let octets = (host, 0u16)
        .to_socket_addrs()?
        .find_map(|a| match a {
            std::net::SocketAddr::V4(v4) => Some(v4.ip().octets()),
            _ => None,
        })
        .ok_or_else(|| std::io::Error::other("no IPv4 address"))?;
    let ip = fmt_ip(octets);

    let fd = unsafe { socket(AF_INET, SOCK_RAW, IPPROTO_ICMP) };
    if fd < 0 {
        let err = std::io::Error::last_os_error();
        if err.kind() == std::io::ErrorKind::PermissionDenied {
            eprintln!(
                "traceroute: raw sockets need root. Try `sudo traceroute {host}`, or grant\n\
                 the capability once with `sudo setcap cap_net_raw+ep <binary>`."
            );
        }
        return Err(err);
    }

    let tv = timeval_bytes(1, 0);
    unsafe { setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, tv.as_ptr(), tv.len() as u32) };

    let dest = sockaddr_in(octets);
    let id = std::process::id() as u16;
    println!("{}", render_header(host, &ip, max_hops));

    for ttl in 1..=max_hops {
        // The one extra socket option that makes traceroute out of ping.
        let ttl_val = (ttl as u32).to_ne_bytes();
        unsafe { setsockopt(fd, IPPROTO_IP, IP_TTL, ttl_val.as_ptr(), 4) };

        let mut addr: Option<String> = None;
        let mut rtts: Vec<Option<f64>> = Vec::with_capacity(PROBES_PER_HOP);
        let mut reached = false;

        for probe in 0..PROBES_PER_HOP {
            let seq = (ttl * PROBES_PER_HOP + probe) as u16;
            let packet = build_echo_request(id, seq, &[0x42; 32]);
            let sent = Instant::now();
            unsafe {
                sendto(
                    fd,
                    packet.as_ptr(),
                    packet.len(),
                    0,
                    dest.as_ptr(),
                    dest.len() as u32,
                );
            }
            let mut buf = [0u8; 1500];
            let mut src = [0u8; 16];
            let mut slen = src.len() as u32;
            let n = unsafe {
                recvfrom(
                    fd,
                    buf.as_mut_ptr(),
                    buf.len(),
                    0,
                    src.as_mut_ptr(),
                    &mut slen,
                )
            };
            if n < 0 {
                rtts.push(None); // timeout -> '*'
                continue;
            }
            let elapsed = sent.elapsed().as_secs_f64() * 1000.0;
            let datagram = &buf[..n as usize];
            let ihl = ((datagram[0] & 0x0f) as usize) * 4;
            match classify(&datagram[ihl..]) {
                Some(Reply::EchoReply { id: rid, .. }) if rid == id => {
                    addr.get_or_insert_with(|| fmt_ip([src[4], src[5], src[6], src[7]]));
                    rtts.push(Some(elapsed));
                    reached = true;
                }
                Some(Reply::TimeExceeded { id: rid, .. }) if rid == id => {
                    addr.get_or_insert_with(|| fmt_ip([src[4], src[5], src[6], src[7]]));
                    rtts.push(Some(elapsed));
                }
                _ => rtts.push(None),
            }
        }

        println!("{}", render_hop(ttl, addr.as_deref(), &rtts));
        if reached {
            break; // the destination itself answered
        }
    }

    unsafe { close(fd) };
    Ok(())
}

fn fmt_ip(o: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", o[0], o[1], o[2], o[3])
}

fn timeval_bytes(sec: i64, usec: i64) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[0..8].copy_from_slice(&sec.to_ne_bytes());
    b[8..16].copy_from_slice(&usec.to_ne_bytes());
    b
}

fn sockaddr_in(octets: [u8; 4]) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[0..2].copy_from_slice(&(AF_INET as u16).to_ne_bytes());
    b[4..8].copy_from_slice(&octets);
    b
}
