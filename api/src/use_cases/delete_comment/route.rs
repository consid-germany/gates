use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::types::appstate::AppState;
use crate::use_cases::delete_comment::use_case;
use crate::use_cases::delete_comment::use_case::Error;

#[derive(Serialize, Deserialize)]
pub struct PathParams {
    group: String,
    service: String,
    environment: String,
    comment_id: String,
}

pub async fn handler(
    Path(PathParams {
        group,
        service,
        environment,
        comment_id,
    }): Path<PathParams>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    match app_state
        .use_cases
        .delete_comment
        .execute(
            use_case::Input {
                group,
                service,
                environment,
                comment_id,
            },
            app_state.storage.as_ref(),
            app_state.clock.as_ref(),
        )
        .await
    {
        Ok(gate) => Json(gate).into_response(),
        Err(error) => match error {
            Error::GateOrCommentNotFound => StatusCode::NO_CONTENT.into_response(),
            Error::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        },
    }
}
