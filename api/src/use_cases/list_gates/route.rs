use crate::types::appstate::AppState;
use crate::use_cases::list_gates::use_case::Error;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

pub async fn handler(State(app_state): State<AppState>) -> impl IntoResponse {
    match app_state
        .use_cases
        .list_gates
        .execute(
            app_state.storage.as_ref(),
            app_state.clock.as_ref(),
            app_state.date_time_switch.as_ref(),
        )
        .await
    {
        Ok(groups) => Json(groups).into_response(),
        Err(error) => match error {
            Error::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        },
    }
}
