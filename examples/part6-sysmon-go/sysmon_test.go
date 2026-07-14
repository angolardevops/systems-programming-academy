package main

import "testing"

const (
	statA   = "cpu  1000 0 0 1000 0 0 0 0 0 0\ncpu0 500 0 0 500 0 0 0 0"
	statB   = "cpu  1200 0 0 1200 0 0 0 0 0 0\ncpu0 600 0 0 600 0 0 0 0"
	meminfo = "MemTotal:       16000000 kB\nMemFree:  1000000 kB\nMemAvailable:    4800000 kB\nBuffers: 100 kB"
	netA    = "Inter-|   Receive\n face |bytes\n  eth0: 1000000 10 0 0 0 0 0 0 2000000 20 0 0 0 0 0 0\n    lo: 5 0 0 0 0 0 0 0 5 0"
	netB    = "Inter-|   Receive\n face |bytes\n  eth0: 21000000 99 0 0 0 0 0 0 32000000 99 0 0 0 0 0 0\n    lo: 9 0 0 0 0 0 0 0 9 0"
	diskA   = "   8       0 sda 100 0 200 0 100 0 300 0 0 0 0"
	diskB   = "   8       0 sda 100 0 25000 0 100 0 25000 0 0 0 0"
)

func TestParsesAndComputesCPU(t *testing.T) {
	a, b := ParseCPU(statA), ParseCPU(statB)
	if a != (CpuTimes{Busy: 1000, Total: 2000}) || b != (CpuTimes{Busy: 1200, Total: 2400}) {
		t.Fatalf("cpu parse: %+v %+v", a, b)
	}
	if CPUPercent(a, b) != 50.0 {
		t.Fatalf("cpu %% = %v", CPUPercent(a, b))
	}
}

func TestParsesAndComputesMemory(t *testing.T) {
	used, total := ParseMem(meminfo)
	if used != 11_200_000 || total != 16_000_000 {
		t.Fatalf("mem parse: %d %d", used, total)
	}
	if MemPercent(used, total) != 70.0 {
		t.Fatalf("mem %% = %v", MemPercent(used, total))
	}
}

func TestParsesNetAndComputesRate(t *testing.T) {
	rx1, tx1 := ParseNet(netA, "eth0")
	rx2, tx2 := ParseNet(netB, "eth0")
	if rx1 != 1_000_000 || tx1 != 2_000_000 || rx2 != 21_000_000 || tx2 != 32_000_000 {
		t.Fatalf("net parse: %d %d %d %d", rx1, tx1, rx2, tx2)
	}
	total := RateBps(rx1+tx1, rx2+tx2, 2.0)
	if total != 25_000_000.0 || FormatRate(total) != "25.0 MB/s" {
		t.Fatalf("net rate: %v %q", total, FormatRate(total))
	}
}

func TestParsesDisk(t *testing.T) {
	r1, w1 := ParseDisk(diskA, "sda")
	r2, w2 := ParseDisk(diskB, "sda")
	if r1 != 200 || w1 != 300 || r2 != 25000 || w2 != 25000 {
		t.Fatalf("disk parse: %d %d %d %d", r1, w1, r2, w2)
	}
}

func TestBarFillsProportionallyWithHeatColor(t *testing.T) {
	cases := map[string]string{
		RenderBar(50.0, 20): "\x1b[33m██████████░░░░░░░░░░\x1b[0m",
		RenderBar(10.0, 20): "\x1b[32m██░░░░░░░░░░░░░░░░░░\x1b[0m",
		RenderBar(90.0, 20): "\x1b[31m██████████████████░░\x1b[0m",
	}
	for got, want := range cases {
		if got != want {
			t.Fatalf("bar\n got:  %q\n want: %q", got, want)
		}
	}
}

func TestRendersTheFullDashboardFrame(t *testing.T) {
	frame := RenderDashboard(50.0, 70.0, 25_000_000.0, 12_800_000.0)
	want := " CPU  \x1b[33m██████████░░░░░░░░░░\x1b[0m  50.0%\n" +
		" MEM  \x1b[33m██████████████░░░░░░\x1b[0m  70.0%\n" +
		" NET  \x1b[32m████░░░░░░░░░░░░░░░░\x1b[0m  25.0 MB/s\n" +
		" DISK \x1b[32m██░░░░░░░░░░░░░░░░░░\x1b[0m  12.8 MB/s"
	if frame != want {
		t.Fatalf("frame\n got:\n%q\n want:\n%q", frame, want)
	}
}
