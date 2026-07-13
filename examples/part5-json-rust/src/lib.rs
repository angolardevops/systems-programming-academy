//! A JSON serialization framework: a value tree, a canonical encoder, and a
//! recursive-descent decoder — the round trip that every API speaks.
//!
//! This is the serialization side of the "encoding for a grammar" theme that
//! runs through Part 5. The query builder kept values out of SQL, the template
//! engine kept them out of HTML; here the encoder puts values *into* JSON, which
//! means escaping them for the **JSON** grammar (`"`, `\`, control characters) —
//! a different grammar with different dangerous characters. Reusing an HTML
//! escaper here would be wrong; each output format needs its own encoder.
//!
//! Two framework ideas: a **tagged value tree** (`Json`) that models any JSON
//! document, and **canonical output** (object keys in insertion order, no
//! incidental whitespace) so the encoding is deterministic and byte-identical
//! across languages. Everything is pure string work — no I/O.

use std::fmt::Write as _;

/// Any JSON value. Objects preserve key insertion order so encoding is
/// deterministic.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

/// Escapes a string for a JSON string literal: the two structural characters
/// (`"` and `\`) plus the control characters that JSON forbids raw.
pub fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Encodes a value tree to canonical JSON: no incidental whitespace, object
/// keys in insertion order. The exact bytes are the cross-language contract.
pub fn encode(value: &Json) -> String {
    let mut out = String::new();
    encode_into(value, &mut out);
    out
}

fn encode_into(value: &Json, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Int(n) => {
            let _ = write!(out, "{n}");
        }
        Json::Str(s) => out.push_str(&escape_json_string(s)),
        Json::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                encode_into(item, out);
            }
            out.push(']');
        }
        Json::Object(pairs) => {
            out.push('{');
            for (i, (key, val)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&escape_json_string(key));
                out.push(':');
                encode_into(val, out);
            }
            out.push('}');
        }
    }
}

/// Parses JSON text into a value tree, or returns an error message. A
/// hand-written recursive-descent parser — the mirror of the encoder.
pub fn decode(input: &str) -> Result<Json, String> {
    let mut parser = Parser {
        chars: input.chars().collect(),
        pos: 0,
    };
    parser.skip_ws();
    let value = parser.parse_value()?;
    parser.skip_ws();
    if parser.pos != parser.chars.len() {
        return Err(format!("trailing characters at position {}", parser.pos));
    }
    Ok(value)
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, want: char) -> Result<(), String> {
        if self.peek() == Some(want) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected '{want}' at position {}", self.pos))
        }
    }

    fn parse_value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.peek() {
            Some('n') => self.parse_literal("null", Json::Null),
            Some('t') => self.parse_literal("true", Json::Bool(true)),
            Some('f') => self.parse_literal("false", Json::Bool(false)),
            Some('"') => Ok(Json::Str(self.parse_string()?)),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_int(),
            _ => Err(format!("unexpected input at position {}", self.pos)),
        }
    }

    fn parse_literal(&mut self, text: &str, value: Json) -> Result<Json, String> {
        for want in text.chars() {
            self.expect(want)?;
        }
        Ok(value)
    }

    fn parse_int(&mut self) -> Result<Json, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        text.parse::<i64>()
            .map(Json::Int)
            .map_err(|_| format!("invalid integer '{text}'"))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err("unterminated string".to_string()),
                Some('"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some('\\') => {
                    self.pos += 1;
                    match self.peek() {
                        Some('"') => out.push('"'),
                        Some('\\') => out.push('\\'),
                        Some('/') => out.push('/'),
                        Some('n') => out.push('\n'),
                        Some('r') => out.push('\r'),
                        Some('t') => out.push('\t'),
                        Some('u') => {
                            let hex: String = self
                                .chars
                                .get(self.pos + 1..self.pos + 5)
                                .map(|s| s.iter().collect())
                                .unwrap_or_default();
                            let code = u32::from_str_radix(&hex, 16)
                                .map_err(|_| "invalid \\u escape".to_string())?;
                            out.push(char::from_u32(code).ok_or("invalid code point")?);
                            self.pos += 4;
                        }
                        _ => return Err("invalid escape".to_string()),
                    }
                    self.pos += 1;
                }
                Some(c) => {
                    out.push(c);
                    self.pos += 1;
                }
            }
        }
    }

    fn parse_array(&mut self) -> Result<Json, String> {
        self.expect('[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                }
                Some(']') => {
                    self.pos += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err(format!("expected ',' or ']' at position {}", self.pos)),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Json, String> {
        self.expect('{')?;
        let mut pairs = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(Json::Object(pairs));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(':')?;
            let value = self.parse_value()?;
            pairs.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                }
                Some('}') => {
                    self.pos += 1;
                    return Ok(Json::Object(pairs));
                }
                _ => return Err(format!("expected ',' or '}}' at position {}", self.pos)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(pairs: Vec<(&str, Json)>) -> Json {
        Json::Object(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    #[test]
    fn encodes_primitives() {
        assert_eq!(encode(&Json::Null), "null");
        assert_eq!(encode(&Json::Bool(true)), "true");
        assert_eq!(encode(&Json::Bool(false)), "false");
        assert_eq!(encode(&Json::Int(-42)), "-42");
        assert_eq!(encode(&Json::Str("hi".into())), "\"hi\"");
    }

    #[test]
    fn encodes_nested_structure_canonically() {
        let doc = obj(vec![
            ("name", Json::Str("Ana".into())),
            ("age", Json::Int(30)),
            (
                "tags",
                Json::Array(vec![Json::Str("a".into()), Json::Str("b".into())]),
            ),
        ]);
        assert_eq!(
            encode(&doc),
            "{\"name\":\"Ana\",\"age\":30,\"tags\":[\"a\",\"b\"]}"
        );
    }

    #[test]
    fn escapes_json_string_grammar_not_html() {
        // Quotes and backslashes get JSON escapes; < and > are NOT touched
        // (that would be HTML escaping, the wrong grammar).
        let s = Json::Str("a\"b\\c\nd<e>".into());
        assert_eq!(encode(&s), "\"a\\\"b\\\\c\\nd<e>\"");
    }

    #[test]
    fn escapes_control_characters_as_unicode() {
        let s = Json::Str("\u{0001}\u{001f}".into());
        assert_eq!(encode(&s), "\"\\u0001\\u001f\"");
    }

    #[test]
    fn decodes_primitives() {
        assert_eq!(decode("null").unwrap(), Json::Null);
        assert_eq!(decode("true").unwrap(), Json::Bool(true));
        assert_eq!(decode("-42").unwrap(), Json::Int(-42));
        assert_eq!(decode("  \"hi\"  ").unwrap(), Json::Str("hi".into()));
    }

    #[test]
    fn decodes_nested_structure() {
        let parsed = decode("{\"a\":[1,2],\"b\":{\"c\":true}}").unwrap();
        assert_eq!(
            parsed,
            obj(vec![
                ("a", Json::Array(vec![Json::Int(1), Json::Int(2)])),
                ("b", obj(vec![("c", Json::Bool(true))])),
            ])
        );
    }

    #[test]
    fn round_trips_canonical_json() {
        let canonical = "{\"id\":7,\"items\":[\"x\",\"y\"],\"ok\":false,\"note\":null}";
        assert_eq!(encode(&decode(canonical).unwrap()), canonical);
    }

    #[test]
    fn round_trips_escaped_string() {
        let value = Json::Str("line1\nline2\t\"quoted\"".into());
        assert_eq!(decode(&encode(&value)).unwrap(), value);
    }

    #[test]
    fn malformed_input_is_an_error() {
        assert!(decode("{").is_err());
        assert!(decode("[1,]").is_err());
        assert!(decode("nul").is_err());
        assert!(decode("true false").is_err());
    }
}
