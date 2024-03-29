use crate::types::app_state::AppState;
use crate::use_cases::get_gate::use_case;
use crate::use_cases::get_gate::use_case::Error;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PathParams {
    group: String,
    service: String,
    environment: String,
}

pub async fn handler(
    Path(PathParams {
        group,
        service,
        environment,
    }): Path<PathParams>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    match app_state
        .use_cases
        .get_gate
        .execute(
            use_case::Input {
                group,
                service,
                environment,
            },
            app_state.storage.as_ref(),
            app_state.clock.as_ref(),
            app_state.date_time_switch.as_ref(),
        )
        .await
    {
        Ok(Some(gate)) => Json(gate).into_response(),
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => match error {
            Error::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        },
    }
}
