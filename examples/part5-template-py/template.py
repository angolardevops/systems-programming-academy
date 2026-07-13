"""A template engine with autoescaping: substitute ``{{ name }}`` placeholders
with values from a context, HTML-escaping every value **by default** so
untrusted data cannot become markup.

This is the output-side mirror of the query-builder lesson. There, parameterized
queries kept user values out of SQL syntax; here, autoescaping keeps user values
out of HTML syntax. Both defend against injection by making the safe path the
default and the unsafe path an explicit opt-in (``| raw``). A template engine
that escapes by default turns XSS from "the bug you forgot to prevent" into "the
thing you had to deliberately ask for".

Filters compose left to right (``{{ name | upper }}``); ``raw`` disables the
final escape. Pure string work — no I/O — so the rendered output is directly
assertable and byte-identical across languages.
"""

from __future__ import annotations


class TemplateError(Exception):
    """Raised for an unclosed delimiter, unknown variable, or unknown filter."""


def escape_html(s: str) -> str:
    """Escape the five characters significant in HTML text and attributes.

    ``&`` must be replaced first, or the ``&`` introduced by later replacements
    would be double-escaped.

    >>> escape_html("a & b < c")
    'a &amp; b &lt; c'
    """
    return (
        s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
    )


def _apply_filter(name: str, value: str) -> str:
    match name:
        case "upper":
            return value.upper()
        case "lower":
            return value.lower()
        case "trim":
            return value.strip()
        case "raw":
            return value  # handled specially by the renderer; identity here
        case _:
            raise TemplateError(f"unknown filter: {name}")


def render(template: str, context: dict[str, str]) -> str:
    """Render ``template`` against ``context``, substituting each ``{{ expr }}``
    and autoescaping the result unless the filter chain contains ``raw``.

    Raises :class:`TemplateError` for an unclosed ``{{``, an unknown variable,
    or an unknown filter — loud failures beat silently rendering a broken page.

    >>> render("Hello {{ name }}!", {"name": "Ana"})
    'Hello Ana!'
    """
    out: list[str] = []
    while True:
        open_at = template.find("{{")
        if open_at == -1:
            out.append(template)
            break
        out.append(template[:open_at])
        rest = template[open_at + 2 :]
        close_at = rest.find("}}")
        if close_at == -1:
            raise TemplateError("unclosed '{{'")
        expr = rest[:close_at].strip()
        out.append(_render_expr(expr, context))
        template = rest[close_at + 2 :]
    return "".join(out)


def _render_expr(expr: str, context: dict[str, str]) -> str:
    parts = [p.strip() for p in expr.split("|")]
    var = parts[0]
    if var == "":
        raise TemplateError("empty expression: {{ }}")
    if var not in context:
        raise TemplateError(f"unknown variable: {var}")
    value = context[var]

    filters = parts[1:]
    raw = "raw" in filters
    for f in filters:
        value = _apply_filter(f, value)

    return value if raw else escape_html(value)


if __name__ == "__main__":
    ctx = {
        "user": "Ana",
        "comment": "<script>steal(document.cookie)</script>",
        "bio": "<em>trusted markup</em>",
    }
    page = (
        "<article>\n"
        "  <h2>{{ user }}</h2>\n"
        "  <p>{{ comment }}</p>\n"
        "  <footer>{{ bio | raw }}</footer>\n"
        "</article>"
    )
    print(render(page, ctx))
    print(
        "\n-- the <script> became inert text; only the trusted bio rendered as HTML --"
    )
