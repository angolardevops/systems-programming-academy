//! traceroute — the testable core of a hop-by-hop path tracer.
//!
//! traceroute is ping with one twist: send echo requests with a deliberately
//! small IP **TTL**, so each router along the path decrements it to zero and
//! reports back an ICMP *time-exceeded* (type 11). Raise the TTL by one each
//! round and you learn the path, hop by hop, until the destination itself
//! answers with an echo reply (type 0).
//!
//! Setting the TTL and reading replies needs a raw socket (root — see main.rs).
//! Everything else is pure: building the probe, the checksum, classifying an
//! ICMP reply, and rendering the report. Those are what this library tests.

/// The 16-bit internet checksum (RFC 1071) — identical to the one in the ping
/// lesson. A valid packet sums back to zero, so `checksum(valid) == 0`.
pub fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut chunks = data.chunks_exact(2);
    for c in &mut chunks {
        sum += u16::from_be_bytes([c[0], c[1]]) as u32;
    }
    if let [last] = chunks.remainder() {
        sum += u16::from_be_bytes([*last, 0]) as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

/// Build an ICMP echo-request probe (type 8) with the checksum filled in — the
/// packet whose TTL we vary. Same shape as ping's request.
pub fn build_echo_request(id: u16, seq: u16, payload: &[u8]) -> Vec<u8> {
    let mut pkt = vec![8, 0, 0, 0];
    pkt.extend_from_slice(&id.to_be_bytes());
    pkt.extend_from_slice(&seq.to_be_bytes());
    pkt.extend_from_slice(payload);
    let ck = checksum(&pkt);
    pkt[2..4].copy_from_slice(&ck.to_be_bytes());
    pkt
}

/// What an incoming ICMP message tells us about a probe.
#[derive(Debug, PartialEq, Eq)]
pub enum Reply {
    /// The destination answered — this hop is the final one.
    EchoReply { id: u16, seq: u16 },
    /// A router on the path reported the TTL expired — an intermediate hop.
    TimeExceeded { id: u16, seq: u16 },
}

/// Classify a received ICMP message, recovering the id and seq of the probe it
/// refers to. Returns `None` for anything else (bad checksum, unknown type,
/// runt).
///
/// The trick is *time-exceeded*: its body carries the IP header **and first 8
/// bytes** of the packet that expired — which is exactly our echo request's
/// header, holding the id and seq. So we reach past the outer ICMP header and
/// the embedded IP header to read them back.
pub fn classify(data: &[u8]) -> Option<Reply> {
    if data.len() < 8 || checksum(data) != 0 {
        return None;
    }
    match (data[0], data[1]) {
        (0, 0) => Some(Reply::EchoReply {
            id: u16::from_be_bytes([data[4], data[5]]),
            seq: u16::from_be_bytes([data[6], data[7]]),
        }),
        (11, _) => {
            // Body: [outer ICMP header 8][embedded IP header][embedded ICMP 8+].
            let embedded = &data[8..];
            let ihl = ((embedded.first()? & 0x0f) as usize) * 4;
            let inner = embedded.get(ihl..)?;
            if inner.len() < 8 {
                return None;
            }
            Some(Reply::TimeExceeded {
                id: u16::from_be_bytes([inner[4], inner[5]]),
                seq: u16::from_be_bytes([inner[6], inner[7]]),
            })
        }
        _ => None,
    }
}

/// The opening line, byte-for-byte like `traceroute(8)`.
pub fn render_header(host: &str, ip: &str, max_hops: usize) -> String {
    format!("traceroute to {host} ({ip}), {max_hops} hops max")
}

/// Render one hop line. `addr` is the responding router (None if no probe
/// answered), and `rtts` holds one entry per probe: `Some(ms)` for a reply,
/// `None` for a timeout — printed as `*`.
///
/// ```text
///  1  192.168.1.1  0.512 ms  0.489 ms  0.501 ms
///  2  10.0.0.1  4.123 ms  4.200 ms  *
///  3  * * *
/// ```
pub fn render_hop(ttl: usize, addr: Option<&str>, rtts: &[Option<f64>]) -> String {
    let mut line = format!("{ttl:2}  ");
    match addr {
        None => {
            let stars: Vec<&str> = rtts.iter().map(|_| "*").collect();
            line.push_str(&stars.join(" "));
        }
        Some(a) => {
            line.push_str(a);
            for r in rtts {
                match r {
                    Some(ms) => line.push_str(&format!("  {ms:.3} ms")),
                    None => line.push_str("  *"),
                }
            }
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wrap an echo request in an ICMP time-exceeded message the way a router
    /// does: outer header [11,0,cksum,unused(4)] then the original IP header
    /// and the original ICMP.
    fn time_exceeded(inner_request: &[u8]) -> Vec<u8> {
        let mut pkt = vec![11, 0, 0, 0, 0, 0, 0, 0];
        // A minimal 20-byte IPv4 header: version 4, IHL 5 (0x45), rest zeros.
        let mut ip = vec![0x45u8; 1];
        ip.extend_from_slice(&[0u8; 19]);
        pkt.extend_from_slice(&ip);
        pkt.extend_from_slice(inner_request);
        let ck = checksum(&pkt);
        pkt[2..4].copy_from_slice(&ck.to_be_bytes());
        pkt
    }

    #[test]
    fn builds_probe_with_valid_checksum() {
        let pkt = build_echo_request(0x1234, 1, b"abcd");
        assert_eq!(pkt[0], 8);
        assert_eq!(checksum(&pkt), 0);
    }

    #[test]
    fn classifies_a_destination_echo_reply() {
        let mut reply = build_echo_request(0x00aa, 5, &[]);
        reply[0] = 0; // echo reply
        reply[2..4].copy_from_slice(&[0, 0]);
        let ck = checksum(&reply);
        reply[2..4].copy_from_slice(&ck.to_be_bytes());
        assert_eq!(
            classify(&reply),
            Some(Reply::EchoReply { id: 0x00aa, seq: 5 })
        );
    }

    #[test]
    fn classifies_a_router_time_exceeded() {
        // A router quotes our probe back inside a time-exceeded message.
        let probe = build_echo_request(0xbeef, 3, &[]);
        let te = time_exceeded(&probe);
        assert_eq!(
            classify(&te),
            Some(Reply::TimeExceeded { id: 0xbeef, seq: 3 })
        );
    }

    #[test]
    fn rejects_corruption_and_unknown_types() {
        let mut te = time_exceeded(&build_echo_request(1, 1, &[]));
        assert!(classify(&te).is_some());
        te[10] ^= 0xff; // corrupt a byte -> checksum fails
        assert_eq!(classify(&te), None);
        // A destination-unreachable (type 3) is not something we track here.
        let mut other = vec![3u8, 0, 0, 0, 0, 0, 0, 0];
        let ck = checksum(&other);
        other[2..4].copy_from_slice(&ck.to_be_bytes());
        assert_eq!(classify(&other), None);
    }

    #[test]
    fn renders_the_header() {
        assert_eq!(
            render_header("example.com", "93.184.216.34", 30),
            "traceroute to example.com (93.184.216.34), 30 hops max"
        );
    }

    #[test]
    fn renders_reply_partial_and_timeout_hops() {
        let full = render_hop(
            1,
            Some("192.168.1.1"),
            &[Some(0.512), Some(0.489), Some(0.501)],
        );
        assert_eq!(full, " 1  192.168.1.1  0.512 ms  0.489 ms  0.501 ms");
        let partial = render_hop(2, Some("10.0.0.1"), &[Some(4.123), Some(4.200), None]);
        assert_eq!(partial, " 2  10.0.0.1  4.123 ms  4.200 ms  *");
        let gone = render_hop(3, None, &[None, None, None]);
        assert_eq!(gone, " 3  * * *");
    }
}
