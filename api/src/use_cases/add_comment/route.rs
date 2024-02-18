use crate::types::appstate::AppState;
use crate::use_cases::add_comment::use_case::{Error, Input};
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

#[derive(Serialize, Deserialize)]
pub struct Payload {
    pub message: String,
}

pub async fn handler(
    Path(PathParams {
        group,
        service,
        environment,
    }): Path<PathParams>,
    State(app_state): State<AppState>,
    Json(Payload { message }): Json<Payload>,
) -> impl IntoResponse {
    match app_state
        .use_cases
        .add_comment
        .execute(
            Input {
                group,
                service,
                environment,
                message,
            },
            app_state.storage.as_ref(),
            app_state.clock.as_ref(),
            app_state.id_provider.as_ref(),
        )
        .await
    {
        Ok(gate) => Json(gate).into_response(),
        Err(error) => match error {
            Error::GateNotFound => StatusCode::NO_CONTENT.into_response(),
            Error::InvalidInputMessage(error) => {
                (StatusCode::BAD_REQUEST, Json(error)).into_response()
            }
            Error::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        },
    }
}
