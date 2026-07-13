// Package template is a template engine with autoescaping: substitute
// {{ name }} placeholders with values from a context, HTML-escaping every value
// by default so untrusted data cannot become markup.
//
// This is the output-side mirror of the query-builder lesson. There,
// parameterized queries kept user values out of SQL syntax; here, autoescaping
// keeps user values out of HTML syntax. Both defend against injection by making
// the safe path the default and the unsafe path an explicit opt-in ("| raw").
// A template engine that escapes by default turns XSS from "the bug you forgot
// to prevent" into "the thing you had to deliberately ask for".
//
// Filters compose left to right ({{ name | upper }}); raw disables the final
// escape. Pure string work — no I/O — so the rendered output is directly
// assertable and byte-identical across languages.
package template

import (
	"fmt"
	"strings"
)

// EscapeHTML escapes the five characters significant in HTML text and
// attributes. "&" must be replaced first, or the "&" introduced by later
// replacements would be double-escaped.
func EscapeHTML(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, `"`, "&quot;")
	s = strings.ReplaceAll(s, "'", "&#39;")
	return s
}

func applyFilter(name, value string) (string, error) {
	switch name {
	case "upper":
		return strings.ToUpper(value), nil
	case "lower":
		return strings.ToLower(value), nil
	case "trim":
		return strings.TrimSpace(value), nil
	case "raw":
		return value, nil // handled specially by the renderer; identity here
	default:
		return "", fmt.Errorf("unknown filter: %s", name)
	}
}

// Render renders template against context, substituting each {{ expr }} and
// autoescaping the result unless the expression's filter chain contains raw.
//
// Returns an error for an unclosed "{{", an unknown variable, or an unknown
// filter. Loud failures beat silently rendering a broken page.
func Render(template string, context map[string]string) (string, error) {
	var out strings.Builder
	for {
		open := strings.Index(template, "{{")
		if open == -1 {
			out.WriteString(template)
			break
		}
		out.WriteString(template[:open])
		rest := template[open+2:]
		close := strings.Index(rest, "}}")
		if close == -1 {
			return "", fmt.Errorf("unclosed '{{'")
		}
		expr := strings.TrimSpace(rest[:close])
		rendered, err := renderExpr(expr, context)
		if err != nil {
			return "", err
		}
		out.WriteString(rendered)
		template = rest[close+2:]
	}
	return out.String(), nil
}

func renderExpr(expr string, context map[string]string) (string, error) {
	parts := strings.Split(expr, "|")
	for i := range parts {
		parts[i] = strings.TrimSpace(parts[i])
	}
	varName := parts[0]
	if varName == "" {
		return "", fmt.Errorf("empty expression: {{ }}")
	}
	value, ok := context[varName]
	if !ok {
		return "", fmt.Errorf("unknown variable: %s", varName)
	}

	filters := parts[1:]
	raw := false
	for _, f := range filters {
		if f == "raw" {
			raw = true
		}
	}
	for _, f := range filters {
		var err error
		value, err = applyFilter(f, value)
		if err != nil {
			return "", err
		}
	}

	if raw {
		return value, nil
	}
	return EscapeHTML(value), nil
}
