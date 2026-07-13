package template

import (
	"strings"
	"testing"
)

func mustRender(t *testing.T, tmpl string, ctx map[string]string) string {
	t.Helper()
	out, err := Render(tmpl, ctx)
	if err != nil {
		t.Fatalf("Render(%q) errored: %v", tmpl, err)
	}
	return out
}

func TestSubstitutesAVariable(t *testing.T) {
	got := mustRender(t, "Hello {{ name }}!", map[string]string{"name": "Ana"})
	if got != "Hello Ana!" {
		t.Fatalf("got %q", got)
	}
}

func TestAutoescapesHTMLByDefault(t *testing.T) {
	ctx := map[string]string{"comment": "<script>alert('xss')</script>"}
	want := "<p>&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;</p>"
	if got := mustRender(t, "<p>{{ comment }}</p>", ctx); got != want {
		t.Fatalf("got  %q\nwant %q", got, want)
	}
}

func TestRawFilterOptsOutOfEscaping(t *testing.T) {
	got := mustRender(t, "{{ body | raw }}", map[string]string{"body": "<b>bold</b>"})
	if got != "<b>bold</b>" {
		t.Fatalf("got %q", got)
	}
}

func TestAmpersandIsEscapedFirst(t *testing.T) {
	got := mustRender(t, "{{ x }}", map[string]string{"x": "a & b < c"})
	if got != "a &amp; b &lt; c" {
		t.Fatalf("got %q", got)
	}
}

func TestFiltersComposeThenEscape(t *testing.T) {
	got := mustRender(t, "{{ name | trim | upper }}", map[string]string{"name": "  <ana>  "})
	if got != "&lt;ANA&gt;" {
		t.Fatalf("got %q", got)
	}
}

func TestUpperThenRawSkipsEscape(t *testing.T) {
	got := mustRender(t, "{{ tag | upper | raw }}", map[string]string{"tag": "<b>"})
	if got != "<B>" {
		t.Fatalf("got %q", got)
	}
}

func TestUnknownVariableIsAnError(t *testing.T) {
	_, err := Render("{{ missing }}", map[string]string{})
	if err == nil || !strings.Contains(err.Error(), "missing") {
		t.Fatalf("expected error naming the variable, got: %v", err)
	}
}

func TestUnknownFilterIsAnError(t *testing.T) {
	_, err := Render("{{ x | shout }}", map[string]string{"x": "hi"})
	if err == nil || !strings.Contains(err.Error(), "shout") {
		t.Fatalf("expected error naming the filter, got: %v", err)
	}
}

func TestUnclosedDelimiterIsAnError(t *testing.T) {
	_, err := Render("start {{ x ", map[string]string{"x": "hi"})
	if err == nil || !strings.Contains(err.Error(), "unclosed") {
		t.Fatalf("expected unclosed error, got: %v", err)
	}
}

func TestLiteralTextPassesThroughUntouched(t *testing.T) {
	got := mustRender(t, "a {{ n }} b {{ n }} c", map[string]string{"n": "1"})
	if got != "a 1 b 1 c" {
		t.Fatalf("got %q", got)
	}
}
