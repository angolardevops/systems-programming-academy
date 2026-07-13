//! Demo: encode a document, then decode it back and re-encode to show the
//! round trip is exact.

use part5_json_rust::{decode, encode, Json};

fn main() {
    let doc = Json::Object(vec![
        ("user".into(), Json::Str("Ana".into())),
        ("age".into(), Json::Int(34)),
        ("admin".into(), Json::Bool(true)),
        (
            "note".into(),
            Json::Str("has \"quotes\" and a\nnewline".into()),
        ),
        (
            "tags".into(),
            Json::Array(vec![Json::Str("rust".into()), Json::Null]),
        ),
    ]);

    let json = encode(&doc);
    println!("encoded:   {json}");

    let reparsed = decode(&json).unwrap();
    println!("re-encoded: {}", encode(&reparsed));
    println!("round-trip exact: {}", encode(&reparsed) == json);
}
