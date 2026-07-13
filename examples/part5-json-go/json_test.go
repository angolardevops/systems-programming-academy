package json

import "testing"

func TestEncodesPrimitives(t *testing.T) {
	cases := map[string]*Value{
		"null":  Null(),
		"true":  Bool(true),
		"false": Bool(false),
		"-42":   Int(-42),
		`"hi"`:  Str("hi"),
	}
	for want, v := range cases {
		if got := Encode(v); got != want {
			t.Errorf("Encode = %q, want %q", got, want)
		}
	}
}

func TestEncodesNestedStructureCanonically(t *testing.T) {
	doc := Object(
		Member{"name", Str("Ana")},
		Member{"age", Int(30)},
		Member{"tags", Array(Str("a"), Str("b"))},
	)
	want := `{"name":"Ana","age":30,"tags":["a","b"]}`
	if got := Encode(doc); got != want {
		t.Fatalf("Encode\n got:  %s\n want: %s", got, want)
	}
}

func TestEscapesJSONStringGrammarNotHTML(t *testing.T) {
	// Quotes and backslashes get JSON escapes; < and > are NOT touched.
	got := Encode(Str("a\"b\\c\nd<e>"))
	want := `"a\"b\\c\nd<e>"`
	if got != want {
		t.Fatalf("Encode = %s, want %s", got, want)
	}
}

func TestEscapesControlCharactersAsUnicode(t *testing.T) {
	got := Encode(Str("\x01\x1f"))
	want := `"\u0001\u001f"`
	if got != want {
		t.Fatalf("Encode = %s, want %s", got, want)
	}
}

func TestDecodesPrimitives(t *testing.T) {
	if v, _ := Decode("null"); v.Kind != KindNull {
		t.Fatal("null")
	}
	if v, _ := Decode("true"); !v.Bool {
		t.Fatal("true")
	}
	if v, _ := Decode("-42"); v.Int != -42 {
		t.Fatal("-42")
	}
	if v, _ := Decode(`  "hi"  `); v.Str != "hi" {
		t.Fatal("hi")
	}
}

func TestRoundTripsCanonicalJSON(t *testing.T) {
	canonical := `{"id":7,"items":["x","y"],"ok":false,"note":null}`
	v, err := Decode(canonical)
	if err != nil {
		t.Fatalf("decode: %v", err)
	}
	if got := Encode(v); got != canonical {
		t.Fatalf("round trip\n got:  %s\n want: %s", got, canonical)
	}
}

func TestRoundTripsEscapedString(t *testing.T) {
	value := Str("line1\nline2\t\"quoted\"")
	encoded := Encode(value)
	decoded, err := Decode(encoded)
	if err != nil {
		t.Fatalf("decode: %v", err)
	}
	if decoded.Str != value.Str {
		t.Fatalf("round trip lost data: %q vs %q", decoded.Str, value.Str)
	}
}

func TestMalformedInputIsAnError(t *testing.T) {
	for _, bad := range []string{"{", "[1,]", "nul", "true false"} {
		if _, err := Decode(bad); err == nil {
			t.Errorf("Decode(%q) should error", bad)
		}
	}
}
