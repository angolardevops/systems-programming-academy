//! Capstone: a guestbook that composes the Part 5 frameworks end to end and
//! then defeats both classic injection attacks.
//!
//! The request pipeline is: **validate → store (parameterized) → render
//! (autoescaped)**. Each stage is a miniature of a Part 5 lesson:
//!
//! * `validate_submission` — the [validation lesson]: accumulate every error.
//! * `insert_sql` — the [query-builder lesson]: user values become `?`
//!   parameters, never SQL text (no SQL injection).
//! * `render_page` — the [template lesson]: every value is HTML-escaped by
//!   default (no cross-site scripting).
//!
//! The point of the capstone is the two adversarial tests at the bottom: we
//! submit `'; DROP TABLE comments; --` and `<script>alert(1)</script>` as a real
//! comment and prove the store's other rows survive and the script renders as
//! inert text. Input defence and output defence, the same "safe by default"
//! principle on both sides.

// ---------------------------------------------------------------------------
// Validation (from the validation lesson, trimmed to what the guestbook needs)
// ---------------------------------------------------------------------------

/// Validates a guestbook submission, returning every error at once (never
/// bailing on the first) as `"field: message"` lines.
pub fn validate_submission(author: &str, body: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let author = author.trim();
    let body = body.trim();

    if author.is_empty() {
        errors.push("author: is required".to_string());
    } else if author.chars().count() < 2 {
        errors.push("author: must be at least 2 characters".to_string());
    } else if author.chars().count() > 40 {
        errors.push("author: must be at most 40 characters".to_string());
    }

    if body.is_empty() {
        errors.push("body: is required".to_string());
    } else if body.chars().count() > 500 {
        errors.push("body: must be at most 500 characters".to_string());
    }

    errors
}

// ---------------------------------------------------------------------------
// Parameterized query building (from the query-builder lesson)
// ---------------------------------------------------------------------------

/// Builds the parameterized INSERT for a comment: the SQL carries only `?`
/// placeholders; the user values travel separately in the params vector, so
/// they are bound as data and never parsed as SQL.
pub fn insert_sql(author: &str, body: &str) -> (String, Vec<String>) {
    (
        "INSERT INTO comments (author, body) VALUES (?, ?)".to_string(),
        vec![author.to_string(), body.to_string()],
    )
}

// ---------------------------------------------------------------------------
// Autoescaped rendering (from the template lesson)
// ---------------------------------------------------------------------------

/// HTML-escapes text (`&` first, to avoid double-escaping the entities the
/// later replacements introduce).
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ---------------------------------------------------------------------------
// The store: an in-memory table standing in for a real database. It records
// exactly the (sql, params) the app hands it, and inserts the *params* as row
// data — modelling what a real driver does: bind values, never execute them.
// ---------------------------------------------------------------------------

/// One stored comment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    pub author: String,
    pub body: String,
}

/// An in-memory comment store.
#[derive(Default)]
pub struct Store {
    comments: Vec<Comment>,
}

impl Store {
    pub fn new() -> Self {
        Store::default()
    }

    /// Executes a parameterized INSERT. Panics on anything but the exact
    /// two-placeholder comment insert — a deliberately strict stand-in that
    /// *only* accepts the parameterized shape, so a caller cannot smuggle
    /// values into the SQL string.
    pub fn execute_insert(&mut self, sql: &str, params: &[String]) {
        assert_eq!(
            sql, "INSERT INTO comments (author, body) VALUES (?, ?)",
            "store only accepts the parameterized comment insert"
        );
        assert_eq!(params.len(), 2, "expected two bound params");
        // Bind params as DATA. Whatever is in them — including a '; DROP...'
        // string — is stored verbatim as a value; it is never SQL.
        self.comments.push(Comment {
            author: params[0].clone(),
            body: params[1].clone(),
        });
    }

    /// All stored comments, oldest first.
    pub fn all(&self) -> &[Comment] {
        &self.comments
    }
}

// ---------------------------------------------------------------------------
// The pipeline: validate → store (parameterized). Returns the validation
// errors (empty on success).
// ---------------------------------------------------------------------------

/// Handles a guestbook submission: validate, and if clean, store via a
/// parameterized insert. Returns the (possibly empty) list of validation
/// errors. On error the store is left untouched.
pub fn submit(store: &mut Store, author: &str, body: &str) -> Vec<String> {
    let errors = validate_submission(author, body);
    if errors.is_empty() {
        let (sql, params) = insert_sql(author.trim(), body.trim());
        store.execute_insert(&sql, &params);
    }
    errors
}

/// Renders the guestbook page: every stored comment, HTML-escaped. Untrusted
/// content can never become markup.
pub fn render_page(store: &Store) -> String {
    let mut html = String::from("<ul class=\"guestbook\">\n");
    for c in store.all() {
        html.push_str(&format!(
            "  <li><strong>{}</strong>: {}</li>\n",
            escape_html(&c.author),
            escape_html(&c.body)
        ));
    }
    html.push_str("</ul>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_submission_is_stored() {
        let mut store = Store::new();
        let errors = submit(&mut store, "Ana", "Hello, world!");
        assert!(errors.is_empty());
        assert_eq!(store.all().len(), 1);
        assert_eq!(store.all()[0].author, "Ana");
    }

    #[test]
    fn invalid_submission_accumulates_errors_and_stores_nothing() {
        let mut store = Store::new();
        let errors = submit(&mut store, "A", "   ");
        assert_eq!(
            errors,
            vec!["author: must be at least 2 characters", "body: is required",]
        );
        assert!(store.all().is_empty(), "nothing should be stored on error");
    }

    #[test]
    fn insert_is_parameterized_never_interpolated() {
        let evil = "'; DROP TABLE comments; --";
        let (sql, params) = insert_sql("Ana", evil);
        assert_eq!(sql, "INSERT INTO comments (author, body) VALUES (?, ?)");
        assert_eq!(params, vec!["Ana".to_string(), evil.to_string()]);
        assert!(
            !sql.contains("DROP"),
            "payload must never reach the SQL text"
        );
    }

    #[test]
    fn sql_injection_payload_is_stored_as_inert_data_table_survives() {
        let mut store = Store::new();
        submit(&mut store, "Alice", "first comment"); // an existing row
                                                      // The attack: a body that tries to drop the table.
        let errors = submit(&mut store, "Mallory", "'; DROP TABLE comments; --");
        assert!(errors.is_empty());
        // The "existing row" is still there — nothing was dropped.
        assert_eq!(store.all().len(), 2);
        assert_eq!(store.all()[0].body, "first comment");
        // The payload was stored verbatim as a plain string value.
        assert_eq!(store.all()[1].body, "'; DROP TABLE comments; --");
    }

    #[test]
    fn xss_payload_renders_as_inert_text() {
        let mut store = Store::new();
        submit(
            &mut store,
            "Mallory",
            "<script>alert(document.cookie)</script>",
        );
        let page = render_page(&store);
        assert!(
            page.contains("&lt;script&gt;alert(document.cookie)&lt;/script&gt;"),
            "script must be escaped: {page}"
        );
        assert!(
            !page.contains("<script>"),
            "no live script tag may appear: {page}"
        );
    }

    #[test]
    fn end_to_end_both_attacks_defeated() {
        let mut store = Store::new();
        submit(&mut store, "Ana", "Nice site!");
        submit(&mut store, "Mallory", "'; DROP TABLE comments; --");
        submit(&mut store, "Eve", "<script>steal()</script>");

        // All three rows survived (SQLi did not drop anything).
        assert_eq!(store.all().len(), 3);

        let page = render_page(&store);
        // The SQLi text is shown as an escaped literal comment, harmlessly.
        assert!(page.contains("&#39;; DROP TABLE comments; --"));
        // The XSS script is inert text, not a tag.
        assert!(page.contains("&lt;script&gt;steal()&lt;/script&gt;"));
        assert!(!page.contains("<script>"));
    }
}
