use axum::response::IntoResponse;
use axum::Json;
use openapi::models::ApiInfo;

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
