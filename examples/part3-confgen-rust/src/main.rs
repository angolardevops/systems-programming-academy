//! CLI entry: `confgen <spec-file>` (or stdin when no argument).

use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = match std::env::args().nth(1) {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };
    print!("{}", confgen::generate(&input)?);
    Ok(())
}
