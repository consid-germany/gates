use crate::types::app_state::AppState;
use crate::use_cases::delete_gate::use_case;
use crate::use_cases::delete_gate::use_case::Error;
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
        .delete_gates
        .execute(
            use_case::Input {
                group,
                service,
                environment,
            },
            app_state.storage.as_ref(),
        )
        .await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(error) => match error {
            Error::GateNotFound => StatusCode::NO_CONTENT.into_response(),
            Error::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        },
    }
}
