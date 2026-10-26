use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;

const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS page (name TEXT PRIMARY KEY, data TEXT NOT NULL, saved_at INTEGER NOT NULL)";

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("failed to open {path}: {e}"))?;
        conn.execute_batch(SCHEMA).map_err(|e| format!("failed to create table page: {e}"))?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn load_page(&self, name: &str) -> Result<Value, String> {
        let data: Option<String> = self
            .lock()
            .query_row("SELECT data FROM page WHERE name = ?1", [name], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        match data {
            Some(data) => serde_json::from_str(&data).map_err(|e| e.to_string()),
            None => Ok(Value::Null),
        }
    }

    pub fn save_page(&self, name: &str, data: &Value) -> Result<(), String> {
        let saved_at = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        self.lock()
            .execute(
                "INSERT INTO page (name, data, saved_at) VALUES (?1, ?2, ?3) \
                 ON CONFLICT (name) DO UPDATE SET data = ?2, saved_at = ?3",
                params![name, data.to_string(), saved_at],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    // a panic in a different request poisons the lock, but the connection stays usable
    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn load_page_without_save_returns_null() {
        let db = Db::open(":memory:").unwrap();
        assert_eq!(db.load_page("todo-items").unwrap(), Value::Null);
    }

    #[test]
    fn save_page_replaces_last_save() {
        let db = Db::open(":memory:").unwrap();
        db.save_page("todo-items", &json!({ "items": [{ "text": "a" }] })).unwrap();
        db.save_page("todo-items", &json!({ "items": [] })).unwrap();
        assert_eq!(db.load_page("todo-items").unwrap(), json!({ "items": [] }));
    }
}
