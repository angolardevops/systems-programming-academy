package query

import (
	"reflect"
	"testing"
)

func eq(t *testing.T, gotSQL, wantSQL string, gotParams, wantParams []string) {
	t.Helper()
	if gotSQL != wantSQL {
		t.Fatalf("sql\n got:  %q\n want: %q", gotSQL, wantSQL)
	}
	if !reflect.DeepEqual(gotParams, wantParams) {
		t.Fatalf("params\n got:  %v\n want: %v", gotParams, wantParams)
	}
}

func TestSelectAllFromTable(t *testing.T) {
	sql, params := Table("users").Build()
	eq(t, sql, "SELECT * FROM users", params, []string{})
}

func TestSelectSpecificColumns(t *testing.T) {
	sql, _ := Table("users").Select("id", "name").Build()
	if sql != "SELECT id, name FROM users" {
		t.Fatalf("sql = %q", sql)
	}
}

func TestSingleWhereBecomesPlaceholder(t *testing.T) {
	sql, params := Table("users").Where("age", ">", "18").Build()
	eq(t, sql, "SELECT * FROM users WHERE age > ?", params, []string{"18"})
}

func TestMultipleWhereJoinedWithAnd(t *testing.T) {
	sql, params := Table("users").
		Where("age", ">", "18").
		Where("country", "=", "AO").
		Build()
	eq(t, sql, "SELECT * FROM users WHERE age > ? AND country = ?", params, []string{"18", "AO"})
}

func TestFullQueryAllClauses(t *testing.T) {
	sql, params := Table("orders").
		Select("id", "total").
		Where("status", "=", "paid").
		OrderBy("total").
		Limit(10).
		Build()
	eq(t, sql, "SELECT id, total FROM orders WHERE status = ? ORDER BY total LIMIT 10",
		params, []string{"paid"})
}

func TestInjectionAttemptIsAParameterNotSQL(t *testing.T) {
	evil := "'; DROP TABLE users; --"
	sql, params := Table("users").Where("name", "=", evil).Build()
	eq(t, sql, "SELECT * FROM users WHERE name = ?", params, []string{evil})
	if want := "DROP"; contains(sql, want) {
		t.Fatal("payload leaked into SQL")
	}
}

func TestOrderAndLimitOptionalLastWins(t *testing.T) {
	sql, _ := Table("t").Limit(5).Limit(20).Build()
	if sql != "SELECT * FROM t LIMIT 20" {
		t.Fatalf("sql = %q", sql)
	}
}

func contains(s, sub string) bool {
	for i := 0; i+len(sub) <= len(s); i++ {
		if s[i:i+len(sub)] == sub {
			return true
		}
	}
	return false
}
