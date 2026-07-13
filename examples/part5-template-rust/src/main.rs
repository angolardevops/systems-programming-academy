//! Demo: render a comment with an XSS payload and watch autoescaping
//! neutralize it — then show the explicit `raw` opt-out for trusted HTML.

use part5_template_rust::{context, render};

fn main() {
    let ctx = context(&[
        ("user", "Ana"),
        ("comment", "<script>steal(document.cookie)</script>"),
        ("bio", "<em>trusted markup</em>"),
    ]);

    let page = "<article>\n  <h2>{{ user }}</h2>\n  <p>{{ comment }}</p>\n  <footer>{{ bio | raw }}</footer>\n</article>";

    println!("{}", render(page, &ctx).unwrap());
    println!("\n-- the <script> became inert text; only the trusted bio rendered as HTML --");
}
