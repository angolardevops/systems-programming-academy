//! Demo: build a few queries and print the SQL + params, including the
//! injection-attempt case that proves the payload stays inert.

use part5_querybuilder_rust::Query;

fn main() {
    let (sql, params) = Query::table("users")
        .select(&["id", "name", "email"])
        .where_("age", ">=", "18")
        .where_("country", "=", "AO")
        .order_by("name")
        .limit(25)
        .build();
    println!("SQL:    {sql}");
    println!("params: {params:?}\n");

    let evil = "'; DROP TABLE users; --";
    let (sql, params) = Query::table("users").where_("name", "=", evil).build();
    println!("Injection attempt as a value:");
    println!("SQL:    {sql}");
    println!("params: {params:?}");
    println!("-> the payload is data, not SQL. The table is safe.");
}
