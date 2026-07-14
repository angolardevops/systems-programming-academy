//! ping — the testable core of an ICMP echo tool.
//!
//! Sending an ICMP packet needs a raw socket, which needs root (see main.rs).
//! But everything *interesting* is pure and needs no privileges: building the
//! echo-request packet, the internet checksum (RFC 1071), parsing an echo
//! reply, and the round-trip-time statistics. Those are what this library
//! provides and what the tests pin down.

/// The 16-bit internet checksum (RFC 1071): the one's-complement sum of the
/// data as big-endian 16-bit words, then complemented. A valid packet — one
/// whose checksum field already holds the right value — sums back to zero, so
/// `checksum(valid_packet) == 0`. That is exactly how a receiver verifies one.
pub fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut chunks = data.chunks_exact(2);
    for c in &mut chunks {
        sum += u16::from_be_bytes([c[0], c[1]]) as u32;
    }
    // An odd trailing byte is padded with a zero low byte.
    if let [last] = chunks.remainder() {
        sum += u16::from_be_bytes([*last, 0]) as u32;
    }
    // Fold the carries back in until the sum fits in 16 bits.
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

/// Build an ICMP echo-request packet (type 8, code 0) with the given
/// identifier, sequence number, and payload, checksum filled in. These are the
/// exact bytes that go on the wire.
pub fn build_echo_request(id: u16, seq: u16, payload: &[u8]) -> Vec<u8> {
    let mut pkt = Vec::with_capacity(8 + payload.len());
    pkt.push(8); // type: echo request
    pkt.push(0); // code
    pkt.extend_from_slice(&[0, 0]); // checksum placeholder
    pkt.extend_from_slice(&id.to_be_bytes());
    pkt.extend_from_slice(&seq.to_be_bytes());
    pkt.extend_from_slice(payload);
    let ck = checksum(&pkt);
    pkt[2..4].copy_from_slice(&ck.to_be_bytes());
    pkt
}

/// The identifier and sequence number recovered from an echo reply.
#[derive(Debug, PartialEq, Eq)]
pub struct EchoReply {
    pub id: u16,
    pub seq: u16,
}

/// Parse an ICMP echo reply (type 0, code 0), returning its id and seq — but
/// only if the checksum verifies. A corrupted packet, a non-reply type, or a
/// runt returns `None`.
pub fn parse_echo_reply(data: &[u8]) -> Option<EchoReply> {
    if data.len() < 8 || data[0] != 0 || data[1] != 0 {
        return None;
    }
    if checksum(data) != 0 {
        return None; // checksum did not verify
    }
    Some(EchoReply {
        id: u16::from_be_bytes([data[4], data[5]]),
        seq: u16::from_be_bytes([data[6], data[7]]),
    })
}

/// Render the closing statistics block, byte-for-byte like `ping(8)`:
///
/// ```text
/// --- example.com ping statistics ---
/// 3 packets transmitted, 3 received, 0% packet loss
/// rtt min/avg/max/mdev = 10.000/20.000/30.000/8.165 ms
/// ```
///
/// `rtts` are the successful round-trip times in milliseconds; `transmitted`
/// is how many requests went out. With zero replies the rtt line is omitted.
pub fn summarize(host: &str, transmitted: usize, rtts: &[f64]) -> String {
    let received = rtts.len();
    let lost = transmitted - received;
    let loss_pct = if transmitted == 0 {
        0
    } else {
        (lost as f64 / transmitted as f64 * 100.0 + 0.5).floor() as u64
    };
    let mut out = format!(
        "--- {host} ping statistics ---\n\
         {transmitted} packets transmitted, {received} received, {loss_pct}% packet loss"
    );
    if received > 0 {
        let n = received as f64;
        let min = rtts.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = rtts.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let avg = rtts.iter().sum::<f64>() / n;
        let sq = rtts.iter().map(|r| r * r).sum::<f64>() / n;
        let mdev = (sq - avg * avg).max(0.0).sqrt();
        out.push_str(&format!(
            "\nrtt min/avg/max/mdev = {min:.3}/{avg:.3}/{max:.3}/{mdev:.3} ms"
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_of_a_known_packet() {
        // type 8, code 0, checksum 0, id 1, seq 1.
        let bytes = [0x08, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01];
        assert_eq!(checksum(&bytes), 0xf7fd);
    }

    #[test]
    fn builds_echo_request_with_correct_checksum() {
        let pkt = build_echo_request(1, 1, &[]);
        assert_eq!(pkt, [0x08, 0x00, 0xf7, 0xfd, 0x00, 0x01, 0x00, 0x01]);
        // A packet with a valid checksum sums back to zero.
        assert_eq!(checksum(&pkt), 0);
    }

    #[test]
    fn build_and_parse_round_trip() {
        // Turn a request into the matching reply by flipping the type to 0 and
        // recomputing the checksum, then parse it back.
        let mut reply = build_echo_request(0x1234, 7, b"payload");
        reply[0] = 0; // echo reply
        reply[2..4].copy_from_slice(&[0, 0]);
        let ck = checksum(&reply);
        reply[2..4].copy_from_slice(&ck.to_be_bytes());
        assert_eq!(
            parse_echo_reply(&reply),
            Some(EchoReply { id: 0x1234, seq: 7 })
        );
    }

    #[test]
    fn rejects_non_replies_runts_and_corruption() {
        // An echo *request* (type 8) is not a reply.
        assert_eq!(parse_echo_reply(&build_echo_request(1, 1, &[])), None);
        // Too short to hold a header.
        assert_eq!(parse_echo_reply(&[0, 0, 0, 0]), None);
        // A valid reply with one byte flipped fails the checksum.
        let mut reply = [0x00, 0x00, 0xff, 0xef, 0x00, 0x07, 0x00, 0x09];
        assert_eq!(parse_echo_reply(&reply), Some(EchoReply { id: 7, seq: 9 }));
        reply[5] ^= 0xff;
        assert_eq!(parse_echo_reply(&reply), None);
    }

    #[test]
    fn summarizes_a_clean_run() {
        let s = summarize("example.com", 3, &[10.0, 20.0, 30.0]);
        assert_eq!(
            s,
            "--- example.com ping statistics ---\n\
             3 packets transmitted, 3 received, 0% packet loss\n\
             rtt min/avg/max/mdev = 10.000/20.000/30.000/8.165 ms"
        );
    }

    #[test]
    fn summarizes_loss_and_total_loss() {
        let half = summarize("example.com", 4, &[10.0, 30.0]);
        assert_eq!(
            half,
            "--- example.com ping statistics ---\n\
             4 packets transmitted, 2 received, 50% packet loss\n\
             rtt min/avg/max/mdev = 10.000/20.000/30.000/10.000 ms"
        );
        let none = summarize("example.com", 3, &[]);
        assert_eq!(
            none,
            "--- example.com ping statistics ---\n\
             3 packets transmitted, 0 received, 100% packet loss"
        );
    }
}
