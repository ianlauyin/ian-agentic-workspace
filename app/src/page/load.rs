use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::Value;

use crate::AppState;
use crate::page::{Fail, check_name, fail, get, lock};

#[derive(Deserialize)]
pub struct B {
    name: String,
}

pub async fn handler(State(state): State<Arc<AppState>>, Query(b): Query<B>) -> Result<Json<Value>, Fail> {
    check_name(&b.name)?;
    Ok(Json(get(&lock(&state.db), &b.name).map_err(fail)?))
}
