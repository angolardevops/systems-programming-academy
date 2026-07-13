package main

import (
	"strings"
	"testing"
)

func TestCounterAccumulatesPerSeries(t *testing.T) {
	r := NewRegistry()
	r.IncCounter("hits", "Hits.", map[string]string{"path": "/"}, 1)
	r.IncCounter("hits", "Hits.", map[string]string{"path": "/"}, 2)
	r.IncCounter("hits", "Hits.", map[string]string{"path": "/a"}, 5)
	out := r.Render()
	if !strings.Contains(out, `hits{path="/"} 3`+"\n") ||
		!strings.Contains(out, `hits{path="/a"} 5`+"\n") {
		t.Errorf("unexpected render:\n%s", out)
	}
}

func TestGaugeOverwrites(t *testing.T) {
	r := NewRegistry()
	r.SetGauge("depth", "Depth.", nil, 9)
	r.SetGauge("depth", "Depth.", nil, 3)
	if !strings.Contains(r.Render(), "depth 3\n") {
		t.Errorf("gauge did not overwrite:\n%s", r.Render())
	}
}

func TestLabelsRenderSorted(t *testing.T) {
	got := labelString(map[string]string{"z": "1", "a": "2"})
	if got != `{a="2",z="1"}` {
		t.Errorf("labelString = %s", got)
	}
	if labelString(nil) != "" {
		t.Error("empty labels should render empty")
	}
}

func TestValueFormatting(t *testing.T) {
	if formatValue(42) != "42" || formatValue(0.5) != "0.5" {
		t.Errorf("formatValue: %s / %s", formatValue(42), formatValue(0.5))
	}
}

func TestDemoRendersSharedExposition(t *testing.T) {
	out := DemoRegistry().Render()
	if !strings.HasPrefix(out, "# HELP cpu_load 1-minute load average.\n") ||
		!strings.Contains(out, `http_requests_total{method="GET",path="/"} 42`+"\n") ||
		!strings.Contains(out, "queue_depth 3\n") {
		t.Errorf("unexpected demo render:\n%s", out)
	}
}
