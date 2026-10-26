use std::fmt;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::db::Db;

#[derive(Deserialize)]
pub struct LoadRequest {
    name: String,
}

#[derive(Deserialize)]
pub struct SaveRequest {
    name: String,
    data: Value,
}

pub enum PageError {
    BadName,
    Db(String),
}

impl IntoResponse for PageError {
    fn into_response(self) -> Response {
        let status = match self {
            PageError::BadName => StatusCode::BAD_REQUEST,
            PageError::Db(_) => StatusCode::BAD_GATEWAY,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

impl fmt::Display for PageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PageError::BadName => write!(f, "Bad name."),
            PageError::Db(e) => write!(f, "{e}"),
        }
    }
}

pub async fn load(State(db): State<Arc<Db>>, Query(req): Query<LoadRequest>) -> Result<Json<Value>, PageError> {
    check_name(&req.name)?;
    db.load_page(&req.name).map(Json).map_err(PageError::Db)
}

pub async fn save(State(db): State<Arc<Db>>, Json(req): Json<SaveRequest>) -> Result<Json<()>, PageError> {
    check_name(&req.name)?;
    db.save_page(&req.name, &req.data).map(Json).map_err(PageError::Db)
}

fn check_name(name: &str) -> Result<(), PageError> {
    let ok = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if ok { Ok(()) } else { Err(PageError::BadName) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_name_accepts_letters_digits_and_dash() {
        assert!(check_name("todo-items-2").is_ok());
    }

    #[test]
    fn check_name_rejects_other_names() {
        for name in ["", "todo_items", "todo items", "../db", "todö"] {
            assert!(check_name(name).is_err(), "{name}");
        }
    }
}
