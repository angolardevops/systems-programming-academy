// Package query is a SQL query builder: a fluent, chainable API that compiles
// method calls into a parameterized SQL string plus a list of bound values.
//
// Two framework ideas live here. First, the builder pattern: each method
// returns the builder, so calls chain into a sentence —
// Table("users").Where(...).OrderBy(...). Second, and non-negotiable,
// parameterized queries: user values NEVER get formatted into the SQL text.
// They become "?" placeholders and travel in a separate params slice, so a
// value like "'; DROP TABLE users; --" is data, not code — the single most
// important defence against SQL injection.
//
// Compiling to a string means the whole thing is testable with zero database:
// we assert the exact SQL and params.
package query

import (
	"fmt"
	"strings"
)

type condition struct {
	column string
	op     string
	value  string
}

// Query is a query under construction. Build with Table, chain methods, then
// call Build to get (sql, params).
type Query struct {
	table      string
	columns    []string
	conditions []condition
	orderBy    string
	limit      int
	hasLimit   bool
}

// Table starts a query against name, selecting all columns by default.
func Table(name string) *Query {
	return &Query{table: name}
}

// Select restricts the selected columns. With no columns, the query selects *.
func (q *Query) Select(columns ...string) *Query {
	q.columns = columns
	return q
}

// Where adds a "column op ?" condition, binding value as a parameter.
// Multiple calls are joined with AND in call order.
func (q *Query) Where(column, op, value string) *Query {
	q.conditions = append(q.conditions, condition{column, op, value})
	return q
}

// OrderBy sets the ORDER BY column (last call wins).
func (q *Query) OrderBy(column string) *Query {
	q.orderBy = column
	return q
}

// Limit sets the LIMIT (last call wins).
func (q *Query) Limit(n int) *Query {
	q.limit = n
	q.hasLimit = true
	return q
}

// Build compiles the query into (sql, params). The SQL contains only "?"
// placeholders where user values go; the values themselves are in params,
// positionally matching the placeholders left to right.
func (q *Query) Build() (string, []string) {
	cols := "*"
	if len(q.columns) > 0 {
		cols = strings.Join(q.columns, ", ")
	}
	sql := "SELECT " + cols + " FROM " + q.table
	params := []string{}

	if len(q.conditions) > 0 {
		clauses := make([]string, len(q.conditions))
		for i, c := range q.conditions {
			params = append(params, c.value)
			clauses[i] = fmt.Sprintf("%s %s ?", c.column, c.op)
		}
		sql += " WHERE " + strings.Join(clauses, " AND ")
	}

	if q.orderBy != "" {
		sql += " ORDER BY " + q.orderBy
	}
	if q.hasLimit {
		sql += fmt.Sprintf(" LIMIT %d", q.limit)
	}

	return sql, params
}
