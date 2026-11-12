use serde_json::{Value, json};

use crate::db::*;

#[test]
fn load_page_without_save_returns_null() {
    let db = Db::open(":memory:").unwrap();
    assert_eq!(db.load_page("todo-items").unwrap(), Value::Null);
}

#[test]
fn save_page_replaces_last_save() {
    let db = Db::open(":memory:").unwrap();
    db.save_page("todo-items", &json!({ "items": [{ "text": "a" }] }))
        .unwrap();
    db.save_page("todo-items", &json!({ "items": [] })).unwrap();
    assert_eq!(db.load_page("todo-items").unwrap(), json!({ "items": [] }));
}
