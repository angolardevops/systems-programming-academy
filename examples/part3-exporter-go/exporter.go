// Package main is the Go implementation of the Part 3 exporter project. All
// three languages serve byte-identical /metrics for the same registry.
//
//	go test ./...
//	go build && ./exporter 9101   # then: curl localhost:9101/metrics
package main

import (
	"fmt"
	"net/http"
	"os"
	"sort"
	"strconv"
	"strings"
)

// Kind is counter or gauge.
type Kind string

const (
	Counter Kind = "counter"
	Gauge   Kind = "gauge"
)

type metric struct {
	help   string
	kind   Kind
	series map[string]float64 // label-string -> value
}

// Registry accumulates metrics and renders the text exposition format.
type Registry struct {
	metrics map[string]*metric
}

// NewRegistry builds an empty registry.
func NewRegistry() *Registry { return &Registry{metrics: make(map[string]*metric)} }

// labelString renders {k="v",...} with keys sorted; empty -> "".
func labelString(labels map[string]string) string {
	if len(labels) == 0 {
		return ""
	}
	keys := make([]string, 0, len(labels))
	for k := range labels {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	parts := make([]string, 0, len(keys))
	for _, k := range keys {
		parts = append(parts, fmt.Sprintf("%s=%q", k, labels[k]))
	}
	return "{" + strings.Join(parts, ",") + "}"
}

// formatValue: whole numbers as integers, others shortest — keeps the three
// implementations byte-identical.
func formatValue(v float64) string {
	if v == float64(int64(v)) {
		return strconv.FormatInt(int64(v), 10)
	}
	return strconv.FormatFloat(v, 'g', -1, 64)
}

func (r *Registry) metric(name, help string, kind Kind) *metric {
	m, ok := r.metrics[name]
	if !ok {
		m = &metric{help: help, kind: kind, series: make(map[string]float64)}
		r.metrics[name] = m
	}
	return m
}

// IncCounter increments a counter series by delta.
func (r *Registry) IncCounter(name, help string, labels map[string]string, delta float64) {
	r.metric(name, help, Counter).series[labelString(labels)] += delta
}

// SetGauge sets a gauge series to value.
func (r *Registry) SetGauge(name, help string, labels map[string]string, value float64) {
	r.metric(name, help, Gauge).series[labelString(labels)] = value
}

// Render emits the exposition text, fully deterministic (sorted names/series).
func (r *Registry) Render() string {
	names := make([]string, 0, len(r.metrics))
	for n := range r.metrics {
		names = append(names, n)
	}
	sort.Strings(names)

	var b strings.Builder
	for _, name := range names {
		m := r.metrics[name]
		fmt.Fprintf(&b, "# HELP %s %s\n", name, m.help)
		fmt.Fprintf(&b, "# TYPE %s %s\n", name, m.kind)
		keys := make([]string, 0, len(m.series))
		for k := range m.series {
			keys = append(keys, k)
		}
		sort.Strings(keys)
		for _, k := range keys {
			fmt.Fprintf(&b, "%s%s %s\n", name, k, formatValue(m.series[k]))
		}
	}
	return b.String()
}

// DemoRegistry seeds the shared demo data so /metrics can be diffed.
func DemoRegistry() *Registry {
	r := NewRegistry()
	r.IncCounter("http_requests_total", "Total HTTP requests.",
		map[string]string{"method": "GET", "path": "/"}, 42)
	r.IncCounter("http_requests_total", "Total HTTP requests.",
		map[string]string{"method": "POST", "path": "/api"}, 7)
	r.SetGauge("queue_depth", "Jobs waiting in the queue.", nil, 3)
	r.SetGauge("cpu_load", "1-minute load average.", map[string]string{"core": "0"}, 0.5)
	return r
}

func main() {
	port := "9100"
	if len(os.Args) > 1 {
		port = os.Args[1]
	}
	http.HandleFunc("/metrics", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "text/plain; version=0.0.4")
		fmt.Fprint(w, DemoRegistry().Render())
	})
	fmt.Fprintln(os.Stderr, "exporter listening on :"+port)
	if err := http.ListenAndServe("127.0.0.1:"+port, nil); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
