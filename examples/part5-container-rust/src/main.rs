//! Demo: wire a small app graph — a singleton config and db, transient
//! request-scoped handlers — and show cycle detection catching a mistake.

use part5_container_rust::Container;

fn main() {
    let mut c = Container::new();

    c.register_singleton("config", |_| Ok("Config(env=prod)".to_string()));
    c.register_singleton("db", |c| Ok(format!("Pool(from {})", c.resolve("config")?)));
    c.register("user_repo", |c| {
        Ok(format!("UserRepo(on {})", c.resolve("db")?))
    });
    c.register("handler", |c| {
        Ok(format!("Handler(with {})", c.resolve("user_repo")?))
    });

    println!("resolve handler:");
    println!("  {}", c.resolve("handler").unwrap());
    println!("resolve handler again (db/config are singletons, reused):");
    println!("  {}", c.resolve("handler").unwrap());

    // Introduce a cycle and watch the container refuse it.
    let mut bad = Container::new();
    bad.register("a", |c| Ok(format!("A({})", c.resolve("b")?)));
    bad.register("b", |c| Ok(format!("B({})", c.resolve("a")?)));
    println!("\nresolving a cyclic graph:");
    match bad.resolve("a") {
        Ok(v) => println!("  unexpectedly built: {v}"),
        Err(e) => println!("  refused: {e}"),
    }
}
