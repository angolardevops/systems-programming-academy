//! A SQL query builder: a fluent, chainable API that compiles method calls
//! into a parameterized SQL string plus a list of bound values.
//!
//! Two framework ideas live here. First, the **builder pattern**: each method
//! returns `self`, so calls chain into a sentence — `.table("users").where_(...)
//! .order_by(...)`. Second, and non-negotiable, **parameterized queries**:
//! user values NEVER get formatted into the SQL text. They become `?`
//! placeholders and travel in a separate params vector, so a value like
//! `"'; DROP TABLE users; --"` is data, not code. This is the single most
//! important defence against SQL injection, and the builder makes it the only
//! thing you *can* do.
//!
//! Compiling to a string means the whole thing is testable with zero database:
//! we assert the exact SQL and params.

/// A comparison in a WHERE clause: column, operator, and the bound value.
struct Condition {
    column: String,
    op: String,
    value: String,
}

/// A query under construction. Build with [`Query::table`], chain methods,
/// then call [`Query::build`] to get `(sql, params)`.
pub struct Query {
    table: String,
    columns: Vec<String>,
    conditions: Vec<Condition>,
    order_by: Option<String>,
    limit: Option<u64>,
}

impl Query {
    /// Starts a query against `table`, selecting all columns by default.
    pub fn table(name: &str) -> Self {
        Query {
            table: name.to_string(),
            columns: Vec::new(),
            conditions: Vec::new(),
            order_by: None,
            limit: None,
        }
    }

    /// Restricts the selected columns. Called with no names (or never), the
    /// query selects `*`.
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|c| c.to_string()).collect();
        self
    }

    /// Adds a `column op ?` condition, binding `value` as a parameter.
    /// Multiple calls are joined with `AND` in call order.
    pub fn where_(mut self, column: &str, op: &str, value: &str) -> Self {
        self.conditions.push(Condition {
            column: column.to_string(),
            op: op.to_string(),
            value: value.to_string(),
        });
        self
    }

    /// Sets the ORDER BY column (last call wins).
    pub fn order_by(mut self, column: &str) -> Self {
        self.order_by = Some(column.to_string());
        self
    }

    /// Sets the LIMIT (last call wins).
    pub fn limit(mut self, n: u64) -> Self {
        self.limit = Some(n);
        self
    }

    /// Compiles the query into `(sql, params)`. The SQL contains only `?`
    /// placeholders where user values go; the values themselves are in
    /// `params`, positionally matching the placeholders left to right.
    pub fn build(&self) -> (String, Vec<String>) {
        let cols = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };
        let mut sql = format!("SELECT {cols} FROM {}", self.table);
        let mut params = Vec::new();

        if !self.conditions.is_empty() {
            let clauses: Vec<String> = self
                .conditions
                .iter()
                .map(|c| {
                    params.push(c.value.clone());
                    format!("{} {} ?", c.column, c.op)
                })
                .collect();
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }

        if let Some(col) = &self.order_by {
            sql.push_str(&format!(" ORDER BY {col}"));
        }
        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {n}"));
        }

        (sql, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_all_from_table() {
        let (sql, params) = Query::table("users").build();
        assert_eq!(sql, "SELECT * FROM users");
        assert!(params.is_empty());
    }

    #[test]
    fn select_specific_columns() {
        let (sql, _) = Query::table("users").select(&["id", "name"]).build();
        assert_eq!(sql, "SELECT id, name FROM users");
    }

    #[test]
    fn single_where_becomes_placeholder() {
        let (sql, params) = Query::table("users").where_("age", ">", "18").build();
        assert_eq!(sql, "SELECT * FROM users WHERE age > ?");
        assert_eq!(params, vec!["18"]);
    }

    #[test]
    fn multiple_where_joined_with_and() {
        let (sql, params) = Query::table("users")
            .where_("age", ">", "18")
            .where_("country", "=", "AO")
            .build();
        assert_eq!(sql, "SELECT * FROM users WHERE age > ? AND country = ?");
        assert_eq!(params, vec!["18", "AO"]);
    }

    #[test]
    fn full_query_all_clauses() {
        let (sql, params) = Query::table("orders")
            .select(&["id", "total"])
            .where_("status", "=", "paid")
            .order_by("total")
            .limit(10)
            .build();
        assert_eq!(
            sql,
            "SELECT id, total FROM orders WHERE status = ? ORDER BY total LIMIT 10"
        );
        assert_eq!(params, vec!["paid"]);
    }

    #[test]
    fn injection_attempt_is_a_parameter_not_sql() {
        // The classic attack string must end up as DATA in params, never in
        // the SQL text. The SQL keeps a single '?' and the payload is inert.
        let evil = "'; DROP TABLE users; --";
        let (sql, params) = Query::table("users").where_("name", "=", evil).build();
        assert_eq!(sql, "SELECT * FROM users WHERE name = ?");
        assert_eq!(params, vec![evil]);
        assert!(!sql.contains("DROP"), "payload leaked into SQL");
    }

    #[test]
    fn order_and_limit_are_optional_and_last_wins() {
        let (sql, _) = Query::table("t").limit(5).limit(20).build();
        assert_eq!(sql, "SELECT * FROM t LIMIT 20");
    }
}
