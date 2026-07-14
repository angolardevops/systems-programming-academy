package main

import (
	"bytes"
	"testing"
)

func TestChecksumOfAKnownPacket(t *testing.T) {
	// type 8, code 0, checksum 0, id 1, seq 1.
	bytes := []byte{0x08, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01}
	if got := Checksum(bytes); got != 0xf7fd {
		t.Errorf("Checksum = %#04x, want 0xf7fd", got)
	}
}

func TestBuildsEchoRequestWithCorrectChecksum(t *testing.T) {
	pkt := BuildEchoRequest(1, 1, nil)
	want := []byte{0x08, 0x00, 0xf7, 0xfd, 0x00, 0x01, 0x00, 0x01}
	if !bytes.Equal(pkt, want) {
		t.Errorf("packet = %#x, want %#x", pkt, want)
	}
	if got := Checksum(pkt); got != 0 { // valid checksum sums back to zero
		t.Errorf("Checksum(valid) = %#04x, want 0", got)
	}
}

func TestBuildAndParseRoundTrip(t *testing.T) {
	// Turn a request into the matching reply by flipping the type to 0 and
	// recomputing the checksum, then parse it back.
	reply := BuildEchoRequest(0x1234, 7, []byte("payload"))
	reply[0] = 0
	reply[2], reply[3] = 0, 0
	ck := Checksum(reply)
	reply[2], reply[3] = byte(ck>>8), byte(ck)
	got, ok := ParseEchoReply(reply)
	if !ok || got.ID != 0x1234 || got.Seq != 7 {
		t.Errorf("ParseEchoReply = %+v, %v; want {0x1234 7}, true", got, ok)
	}
}

func TestRejectsNonRepliesRuntsAndCorruption(t *testing.T) {
	if _, ok := ParseEchoReply(BuildEchoRequest(1, 1, nil)); ok {
		t.Error("echo request (type 8) parsed as a reply")
	}
	if _, ok := ParseEchoReply([]byte{0, 0, 0, 0}); ok {
		t.Error("runt parsed as a reply")
	}
	reply := []byte{0x00, 0x00, 0xff, 0xef, 0x00, 0x07, 0x00, 0x09}
	if got, ok := ParseEchoReply(reply); !ok || got.Seq != 9 {
		t.Errorf("valid reply not parsed: %+v %v", got, ok)
	}
	reply[5] ^= 0xff // corrupt one byte
	if _, ok := ParseEchoReply(reply); ok {
		t.Error("corrupted reply passed the checksum")
	}
}

func TestSummarizesACleanRun(t *testing.T) {
	got := Summarize("example.com", 3, []float64{10.0, 20.0, 30.0})
	want := "--- example.com ping statistics ---\n" +
		"3 packets transmitted, 3 received, 0% packet loss\n" +
		"rtt min/avg/max/mdev = 10.000/20.000/30.000/8.165 ms"
	if got != want {
		t.Errorf("summary\n got:\n%s\nwant:\n%s", got, want)
	}
}

func TestSummarizesLossAndTotalLoss(t *testing.T) {
	half := Summarize("example.com", 4, []float64{10.0, 30.0})
	wantHalf := "--- example.com ping statistics ---\n" +
		"4 packets transmitted, 2 received, 50% packet loss\n" +
		"rtt min/avg/max/mdev = 10.000/20.000/30.000/10.000 ms"
	if half != wantHalf {
		t.Errorf("half-loss summary\n got:\n%s\nwant:\n%s", half, wantHalf)
	}
	none := Summarize("example.com", 3, nil)
	wantNone := "--- example.com ping statistics ---\n" +
		"3 packets transmitted, 0 received, 100% packet loss"
	if none != wantNone {
		t.Errorf("total-loss summary\n got:\n%s\nwant:\n%s", none, wantNone)
	}
}
