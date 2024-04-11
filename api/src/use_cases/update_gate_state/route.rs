use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::types::app_state::AppState;
use crate::types::GateState;
use crate::use_cases::update_gate_state::use_case;
use crate::use_cases::update_gate_state::use_case::Error;

#[derive(Serialize, Deserialize)]
pub struct PathParams {
    group: String,
    service: String,
    environment: String,
}

#[derive(Serialize, Deserialize)]
pub struct Payload {
    pub state: GateState,
}

pub async fn handler(
    Path(PathParams {
        group,
        service,
        environment,
    }): Path<PathParams>,
    State(app_state): State<AppState>,
    Json(Payload { state }): Json<Payload>,
) -> impl IntoResponse {
    match app_state
        .use_cases
        .update_gate_state
        .execute(
            use_case::Input {
                group,
                service,
                environment,
                state,
            },
            app_state.storage.as_ref(),
            app_state.clock.as_ref(),
            app_state.date_time_switch.as_ref(),
        )
        .await
    {
        Ok(gate) => Json(gate).into_response(),
        Err(error) => match error {
            Error::GateClosed(error) => (StatusCode::CONFLICT, Json(error)).into_response(),
            Error::GateNotFound => StatusCode::NO_CONTENT.into_response(),
            Error::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        },
    }
}
