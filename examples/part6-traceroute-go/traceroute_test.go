package main

import "testing"

// timeExceeded wraps an echo request in an ICMP time-exceeded message the way a
// router does: outer header [11,0,cksum,unused(4)] then a minimal IP header and
// the original ICMP.
func timeExceeded(inner []byte) []byte {
	pkt := []byte{11, 0, 0, 0, 0, 0, 0, 0}
	ip := make([]byte, 20)
	ip[0] = 0x45 // version 4, IHL 5
	pkt = append(pkt, ip...)
	pkt = append(pkt, inner...)
	ck := Checksum(pkt)
	pkt[2], pkt[3] = byte(ck>>8), byte(ck)
	return pkt
}

func TestBuildsProbeWithValidChecksum(t *testing.T) {
	pkt := BuildEchoRequest(0x1234, 1, []byte("abcd"))
	if pkt[0] != 8 {
		t.Fatalf("type = %d, want 8", pkt[0])
	}
	if Checksum(pkt) != 0 {
		t.Fatal("valid probe should sum back to zero")
	}
}

func TestClassifiesADestinationEchoReply(t *testing.T) {
	reply := BuildEchoRequest(0x00aa, 5, nil)
	reply[0] = 0
	reply[2], reply[3] = 0, 0
	ck := Checksum(reply)
	reply[2], reply[3] = byte(ck>>8), byte(ck)
	got, ok := Classify(reply)
	if !ok || got.Kind != EchoReply || got.ID != 0x00aa || got.Seq != 5 {
		t.Errorf("Classify = %+v, %v; want echo reply {0x00aa 5}", got, ok)
	}
}

func TestClassifiesARouterTimeExceeded(t *testing.T) {
	te := timeExceeded(BuildEchoRequest(0xbeef, 3, nil))
	got, ok := Classify(te)
	if !ok || got.Kind != TimeExceeded || got.ID != 0xbeef || got.Seq != 3 {
		t.Errorf("Classify = %+v, %v; want time-exceeded {0xbeef 3}", got, ok)
	}
}

func TestRejectsCorruptionAndUnknownTypes(t *testing.T) {
	te := timeExceeded(BuildEchoRequest(1, 1, nil))
	if _, ok := Classify(te); !ok {
		t.Fatal("valid time-exceeded should classify")
	}
	te[10] ^= 0xff // corrupt a byte -> checksum fails
	if _, ok := Classify(te); ok {
		t.Error("corrupted message passed the checksum")
	}
	other := []byte{3, 0, 0, 0, 0, 0, 0, 0} // destination unreachable
	ck := Checksum(other)
	other[2], other[3] = byte(ck>>8), byte(ck)
	if _, ok := Classify(other); ok {
		t.Error("type 3 should not classify")
	}
}

func TestRendersTheHeader(t *testing.T) {
	got := RenderHeader("example.com", "93.184.216.34", 30)
	want := "traceroute to example.com (93.184.216.34), 30 hops max"
	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestRendersReplyPartialAndTimeoutHops(t *testing.T) {
	full := RenderHop(1, "192.168.1.1", []Probe{{0.512, true}, {0.489, true}, {0.501, true}})
	if want := " 1  192.168.1.1  0.512 ms  0.489 ms  0.501 ms"; full != want {
		t.Errorf("full hop\n got: %q\nwant: %q", full, want)
	}
	partial := RenderHop(2, "10.0.0.1", []Probe{{4.123, true}, {4.200, true}, {0, false}})
	if want := " 2  10.0.0.1  4.123 ms  4.200 ms  *"; partial != want {
		t.Errorf("partial hop\n got: %q\nwant: %q", partial, want)
	}
	gone := RenderHop(3, "", []Probe{{0, false}, {0, false}, {0, false}})
	if want := " 3  * * *"; gone != want {
		t.Errorf("timeout hop\n got: %q\nwant: %q", gone, want)
	}
}
