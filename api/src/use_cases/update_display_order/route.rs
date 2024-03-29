use crate::types::app_state::AppState;
use crate::use_cases::update_display_order::use_case::{Error, Input};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use Error::GateNotFound;

#[derive(Serialize, Deserialize)]
pub struct PathParams {
    group: String,
    service: String,
    environment: String,
}

#[derive(Serialize, Deserialize)]
pub struct Payload {
    pub display_order: u32,
}

pub async fn handler(
    Path(PathParams {
        group,
        service,
        environment,
    }): Path<PathParams>,
    State(app_state): State<AppState>,
    Json(Payload { display_order }): Json<Payload>,
) -> impl IntoResponse {
    match app_state
        .use_cases
        .update_display_order
        .execute(
            Input {
                group,
                service,
                environment,
                display_order,
            },
            app_state.storage.as_ref(),
            app_state.clock.as_ref(),
        )
        .await
    {
        Ok(gate) => Json(gate).into_response(),
        Err(error) => match error {
            GateNotFound => StatusCode::NO_CONTENT.into_response(),
            Error::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        },
    }
}
