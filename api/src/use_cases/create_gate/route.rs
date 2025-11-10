use crate::types::app_state::AppState;
use crate::use_cases::create_gate::use_case;
use crate::use_cases::create_gate::use_case::Error;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Payload {
    pub group: String,
    pub service: String,
    pub environment: String,
    pub display_order: Option<u32>,
}

pub async fn handler(
    State(app_state): State<AppState>,
    Json(Payload {
        group,
        service,
        environment,
        display_order,
    }): Json<Payload>,
) -> impl IntoResponse {
    //TODO add display order
    match app_state
        .use_cases
        .create_gate
        .execute(
            use_case::Input {
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
            Error::InvalidInput(error) => (StatusCode::BAD_REQUEST, Json(error)).into_response(),
            Error::GateAlreadyExists => StatusCode::CONFLICT.into_response(),
            Error::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        },
    }
}
