// Package json is a JSON serialization framework: a value tree, a canonical
// encoder, and a recursive-descent decoder — the round trip every API speaks.
//
// This is the serialization side of the "encoding for a grammar" theme in
// Part 5. The query builder kept values out of SQL, the template engine kept
// them out of HTML; here the encoder puts values INTO JSON, escaping them for
// the JSON grammar (", \, control characters) — a different grammar with
// different dangerous characters. Reusing an HTML escaper here would be wrong.
//
// Two framework ideas: a tagged value tree (Value) modelling any JSON document,
// and canonical output (object keys in insertion order, no incidental
// whitespace) so the encoding is deterministic and byte-identical across
// languages. Pure string work — no I/O, and NOT encoding/json (we build it).
//
// This deliberately reimplements what encoding/json does, to understand it.
package json

import (
	"fmt"
	"strconv"
	"strings"
)

// Kind tags a Value's variant.
type Kind int

const (
	KindNull Kind = iota
	KindBool
	KindInt
	KindStr
	KindArray
	KindObject
)

// Member is one object key/value pair (order preserved).
type Member struct {
	Key   string
	Value *Value
}

// Value is any JSON value.
type Value struct {
	Kind    Kind
	Bool    bool
	Int     int64
	Str     string
	Array   []*Value
	Members []Member
}

// Constructors.
func Null() *Value             { return &Value{Kind: KindNull} }
func Bool(b bool) *Value       { return &Value{Kind: KindBool, Bool: b} }
func Int(n int64) *Value       { return &Value{Kind: KindInt, Int: n} }
func Str(s string) *Value      { return &Value{Kind: KindStr, Str: s} }
func Array(v ...*Value) *Value { return &Value{Kind: KindArray, Array: v} }
func Object(m ...Member) *Value {
	return &Value{Kind: KindObject, Members: m}
}

// EscapeJSONString escapes a string for a JSON string literal: the two
// structural characters (" and \) plus the control characters JSON forbids raw.
func EscapeJSONString(s string) string {
	var b strings.Builder
	b.WriteByte('"')
	for _, r := range s {
		switch r {
		case '"':
			b.WriteString("\\\"")
		case '\\':
			b.WriteString("\\\\")
		case '\n':
			b.WriteString("\\n")
		case '\r':
			b.WriteString("\\r")
		case '\t':
			b.WriteString("\\t")
		default:
			if r < 0x20 {
				fmt.Fprintf(&b, "\\u%04x", r)
			} else {
				b.WriteRune(r)
			}
		}
	}
	b.WriteByte('"')
	return b.String()
}

// Encode turns a value tree into canonical JSON: no incidental whitespace,
// object keys in insertion order. The exact bytes are the cross-language
// contract.
func Encode(v *Value) string {
	var b strings.Builder
	encodeInto(v, &b)
	return b.String()
}

func encodeInto(v *Value, b *strings.Builder) {
	switch v.Kind {
	case KindNull:
		b.WriteString("null")
	case KindBool:
		if v.Bool {
			b.WriteString("true")
		} else {
			b.WriteString("false")
		}
	case KindInt:
		b.WriteString(strconv.FormatInt(v.Int, 10))
	case KindStr:
		b.WriteString(EscapeJSONString(v.Str))
	case KindArray:
		b.WriteByte('[')
		for i, item := range v.Array {
			if i > 0 {
				b.WriteByte(',')
			}
			encodeInto(item, b)
		}
		b.WriteByte(']')
	case KindObject:
		b.WriteByte('{')
		for i, m := range v.Members {
			if i > 0 {
				b.WriteByte(',')
			}
			b.WriteString(EscapeJSONString(m.Key))
			b.WriteByte(':')
			encodeInto(m.Value, b)
		}
		b.WriteByte('}')
	}
}

// Decode parses JSON text into a value tree, or returns an error. A
// hand-written recursive-descent parser — the mirror of the encoder.
func Decode(input string) (*Value, error) {
	p := &parser{runes: []rune(input)}
	p.skipWS()
	v, err := p.parseValue()
	if err != nil {
		return nil, err
	}
	p.skipWS()
	if p.pos != len(p.runes) {
		return nil, fmt.Errorf("trailing characters at position %d", p.pos)
	}
	return v, nil
}

type parser struct {
	runes []rune
	pos   int
}

func (p *parser) peek() (rune, bool) {
	if p.pos < len(p.runes) {
		return p.runes[p.pos], true
	}
	return 0, false
}

func (p *parser) skipWS() {
	for {
		r, ok := p.peek()
		if !ok || (r != ' ' && r != '\t' && r != '\n' && r != '\r') {
			return
		}
		p.pos++
	}
}

func (p *parser) expect(want rune) error {
	if r, ok := p.peek(); ok && r == want {
		p.pos++
		return nil
	}
	return fmt.Errorf("expected %q at position %d", want, p.pos)
}

func (p *parser) parseValue() (*Value, error) {
	p.skipWS()
	r, ok := p.peek()
	if !ok {
		return nil, fmt.Errorf("unexpected end of input")
	}
	switch {
	case r == 'n':
		return p.parseLiteral("null", Null())
	case r == 't':
		return p.parseLiteral("true", Bool(true))
	case r == 'f':
		return p.parseLiteral("false", Bool(false))
	case r == '"':
		s, err := p.parseString()
		if err != nil {
			return nil, err
		}
		return Str(s), nil
	case r == '[':
		return p.parseArray()
	case r == '{':
		return p.parseObject()
	case r == '-' || (r >= '0' && r <= '9'):
		return p.parseInt()
	default:
		return nil, fmt.Errorf("unexpected input at position %d", p.pos)
	}
}

func (p *parser) parseLiteral(text string, v *Value) (*Value, error) {
	for _, want := range text {
		if err := p.expect(want); err != nil {
			return nil, err
		}
	}
	return v, nil
}

func (p *parser) parseInt() (*Value, error) {
	start := p.pos
	if r, ok := p.peek(); ok && r == '-' {
		p.pos++
	}
	for {
		r, ok := p.peek()
		if !ok || r < '0' || r > '9' {
			break
		}
		p.pos++
	}
	text := string(p.runes[start:p.pos])
	n, err := strconv.ParseInt(text, 10, 64)
	if err != nil {
		return nil, fmt.Errorf("invalid integer %q", text)
	}
	return Int(n), nil
}

func (p *parser) parseString() (string, error) {
	if err := p.expect('"'); err != nil {
		return "", err
	}
	var b strings.Builder
	for {
		r, ok := p.peek()
		if !ok {
			return "", fmt.Errorf("unterminated string")
		}
		switch r {
		case '"':
			p.pos++
			return b.String(), nil
		case '\\':
			p.pos++
			esc, ok := p.peek()
			if !ok {
				return "", fmt.Errorf("invalid escape")
			}
			switch esc {
			case '"':
				b.WriteRune('"')
			case '\\':
				b.WriteRune('\\')
			case '/':
				b.WriteRune('/')
			case 'n':
				b.WriteRune('\n')
			case 'r':
				b.WriteRune('\r')
			case 't':
				b.WriteRune('\t')
			case 'u':
				if p.pos+4 >= len(p.runes) {
					return "", fmt.Errorf("invalid \\u escape")
				}
				hex := string(p.runes[p.pos+1 : p.pos+5])
				code, err := strconv.ParseInt(hex, 16, 32)
				if err != nil {
					return "", fmt.Errorf("invalid \\u escape")
				}
				b.WriteRune(rune(code))
				p.pos += 4
			default:
				return "", fmt.Errorf("invalid escape")
			}
			p.pos++
		default:
			b.WriteRune(r)
			p.pos++
		}
	}
}

func (p *parser) parseArray() (*Value, error) {
	if err := p.expect('['); err != nil {
		return nil, err
	}
	items := []*Value{}
	p.skipWS()
	if r, ok := p.peek(); ok && r == ']' {
		p.pos++
		return Array(items...), nil
	}
	for {
		v, err := p.parseValue()
		if err != nil {
			return nil, err
		}
		items = append(items, v)
		p.skipWS()
		r, ok := p.peek()
		if !ok {
			return nil, fmt.Errorf("unterminated array")
		}
		switch r {
		case ',':
			p.pos++
		case ']':
			p.pos++
			return Array(items...), nil
		default:
			return nil, fmt.Errorf("expected ',' or ']' at position %d", p.pos)
		}
	}
}

func (p *parser) parseObject() (*Value, error) {
	if err := p.expect('{'); err != nil {
		return nil, err
	}
	members := []Member{}
	p.skipWS()
	if r, ok := p.peek(); ok && r == '}' {
		p.pos++
		return Object(members...), nil
	}
	for {
		p.skipWS()
		key, err := p.parseString()
		if err != nil {
			return nil, err
		}
		p.skipWS()
		if err := p.expect(':'); err != nil {
			return nil, err
		}
		v, err := p.parseValue()
		if err != nil {
			return nil, err
		}
		members = append(members, Member{Key: key, Value: v})
		p.skipWS()
		r, ok := p.peek()
		if !ok {
			return nil, fmt.Errorf("unterminated object")
		}
		switch r {
		case ',':
			p.pos++
		case '}':
			p.pos++
			return Object(members...), nil
		default:
			return nil, fmt.Errorf("expected ',' or '}' at position %d", p.pos)
		}
	}
}
