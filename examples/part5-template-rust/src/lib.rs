//! A template engine with **autoescaping**: substitute `{{ name }}`
//! placeholders with values from a context, HTML-escaping every value **by
//! default** so untrusted data cannot become markup.
//!
//! This is the output-side mirror of the query-builder lesson. There,
//! parameterized queries kept user values out of SQL syntax; here, autoescaping
//! keeps user values out of HTML syntax. Both defend against injection by making
//! the safe path the default and the unsafe path an explicit, visible opt-in
//! (`| raw`). A template engine that escapes by default turns cross-site
//! scripting (XSS) from "the bug you forgot to prevent" into "the thing you had
//! to deliberately ask for".
//!
//! Filters compose left to right (`{{ name | upper }}`); `raw` is a filter that
//! disables the final escape. Everything is pure string work — no I/O — so the
//! rendered output is directly assertable and byte-identical across languages.

use std::collections::HashMap;

/// HTML-escapes the five characters that are significant in HTML text and
/// attributes. `&` must be replaced first, or the `&` introduced by later
/// replacements would be double-escaped.
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn apply_filter(name: &str, value: String) -> Result<String, String> {
    match name {
        "upper" => Ok(value.to_uppercase()),
        "lower" => Ok(value.to_lowercase()),
        "trim" => Ok(value.trim().to_string()),
        "raw" => Ok(value), // handled specially by the renderer; identity here
        other => Err(format!("unknown filter: {other}")),
    }
}

/// Renders `template` against `context`, substituting each `{{ expr }}` and
/// autoescaping the result unless the expression's filter chain contains `raw`.
///
/// Returns an error for: an unclosed `{{`, an unknown variable, or an unknown
/// filter. Loud failures beat silently rendering a broken page.
pub fn render(template: &str, context: &HashMap<String, String>) -> Result<String, String> {
    let mut out = String::new();
    let bytes = template.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // Find the closing }}.
            let rest = &template[i + 2..];
            let close = rest.find("}}").ok_or_else(|| "unclosed '{{'".to_string())?;
            let expr = rest[..close].trim();
            out.push_str(&render_expr(expr, context)?);
            i += 2 + close + 2;
        } else {
            out.push(template[i..].chars().next().unwrap());
            i += template[i..].chars().next().unwrap().len_utf8();
        }
    }
    Ok(out)
}

/// Renders one `{{ ... }}` expression: `varname` optionally followed by
/// `| filter | filter ...`.
fn render_expr(expr: &str, context: &HashMap<String, String>) -> Result<String, String> {
    let mut parts = expr.split('|').map(str::trim);
    let var = parts.next().unwrap_or("");
    if var.is_empty() {
        return Err("empty expression: {{ }}".to_string());
    }
    let mut value = context
        .get(var)
        .cloned()
        .ok_or_else(|| format!("unknown variable: {var}"))?;

    let filters: Vec<&str> = parts.collect();
    let raw = filters.contains(&"raw");
    for filter in &filters {
        value = apply_filter(filter, value)?;
    }

    Ok(if raw { value } else { escape_html(&value) })
}

/// Convenience for tests/demos: build a context from string pairs.
pub fn context(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_a_variable() {
        let ctx = context(&[("name", "Ana")]);
        assert_eq!(render("Hello {{ name }}!", &ctx).unwrap(), "Hello Ana!");
    }

    #[test]
    fn autoescapes_html_by_default() {
        // The XSS payload becomes inert text, not a live <script> tag.
        let ctx = context(&[("comment", "<script>alert('xss')</script>")]);
        assert_eq!(
            render("<p>{{ comment }}</p>", &ctx).unwrap(),
            "<p>&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;</p>"
        );
    }

    #[test]
    fn raw_filter_opts_out_of_escaping() {
        // Explicit, visible opt-out for trusted HTML.
        let ctx = context(&[("body", "<b>bold</b>")]);
        assert_eq!(render("{{ body | raw }}", &ctx).unwrap(), "<b>bold</b>");
    }

    #[test]
    fn ampersand_is_escaped_first() {
        let ctx = context(&[("x", "a & b < c")]);
        assert_eq!(render("{{ x }}", &ctx).unwrap(), "a &amp; b &lt; c");
    }

    #[test]
    fn filters_compose_then_escape() {
        let ctx = context(&[("name", "  <ana>  ")]);
        // trim then upper, then autoescape the result.
        assert_eq!(
            render("{{ name | trim | upper }}", &ctx).unwrap(),
            "&lt;ANA&gt;"
        );
    }

    #[test]
    fn upper_then_raw_skips_escape() {
        let ctx = context(&[("tag", "<b>")]);
        assert_eq!(render("{{ tag | upper | raw }}", &ctx).unwrap(), "<B>");
    }

    #[test]
    fn unknown_variable_is_an_error() {
        let err = render("{{ missing }}", &context(&[])).unwrap_err();
        assert!(
            err.contains("missing"),
            "error should name the variable: {err}"
        );
    }

    #[test]
    fn unknown_filter_is_an_error() {
        let ctx = context(&[("x", "hi")]);
        let err = render("{{ x | shout }}", &ctx).unwrap_err();
        assert!(err.contains("shout"), "error should name the filter: {err}");
    }

    #[test]
    fn unclosed_delimiter_is_an_error() {
        let ctx = context(&[("x", "hi")]);
        assert!(render("start {{ x ", &ctx)
            .unwrap_err()
            .contains("unclosed"));
    }

    #[test]
    fn literal_text_passes_through_untouched() {
        let ctx = context(&[("n", "1")]);
        assert_eq!(render("a {{ n }} b {{ n }} c", &ctx).unwrap(), "a 1 b 1 c");
    }
}
