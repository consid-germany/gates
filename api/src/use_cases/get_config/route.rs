use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::types::app_state::AppState;

pub async fn handler(State(app_state): State<AppState>) -> impl IntoResponse {
    match app_state
        .use_cases
        .get_config
        .execute(app_state.clock.as_ref(), app_state.active_hours_per_week)
        .await
    {
        Ok(config) => Json(config).into_response(),
        _err => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
