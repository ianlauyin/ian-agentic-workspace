use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::Value;

use crate::AppState;
use crate::page::{Fail, check_name, fail, lock, put};

#[derive(Deserialize)]
pub struct B {
    name: String,
    data: Value,
}

pub async fn handler(State(state): State<Arc<AppState>>, Json(b): Json<B>) -> Result<Json<()>, Fail> {
    check_name(&b.name)?;
    put(&lock(&state.db), &b.name, &b.data).map_err(fail)?;
    Ok(Json(()))
}
