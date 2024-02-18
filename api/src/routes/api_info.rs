use axum::response::IntoResponse;
use axum::Json;

use crate::types::representation::ApiInfo;

#[allow(clippy::unused_async)]
pub async fn handler() -> impl IntoResponse {
    Json(ApiInfo {
        name: option_env!("CARGO_PKG_NAME")
            .unwrap_or("unknown")
            .to_owned(),
        version: option_env!("CARGO_PKG_VERSION")
            .unwrap_or("unknown")
            .to_owned(),
    })
}
