use crate::clock::Clock;
use crate::storage;
use crate::storage::Storage;
use crate::types::GateKey;
use async_trait::async_trait;
use openapi::models;

#[derive(Debug)]
pub struct Input {
    pub group: String,
    pub service: String,
    pub environment: String,
    pub comment_id: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    GateOrCommentNotFound,
    Internal(String),
}

impl From<storage::UpdateError> for Error {
    fn from(value: storage::UpdateError) -> Self {
        match value {
            storage::UpdateError::ItemToUpdateNotFound(_) => Self::GateOrCommentNotFound,
            storage::UpdateError::Other(error) => Self::Internal(error),
        }
    }
}

#[async_trait]
pub trait UseCase {
    async fn execute(
        &self,
        input: Input,
        storage: &(dyn Storage + Send + Sync),
        clock: &(dyn Clock + Send + Sync),
    ) -> Result<models::Gate, Error>;
}

pub fn create() -> impl UseCase {
    UseCaseImpl {}
}

#[derive(Clone)]
struct UseCaseImpl;

#[async_trait]
impl UseCase for UseCaseImpl {
    async fn execute(
        &self,
        Input {
            group,
            service,
            environment,
            comment_id,
        }: Input,
        storage: &(dyn Storage + Send + Sync),
        clock: &(dyn Clock + Send + Sync),
    ) -> Result<models::Gate, Error> {
        Ok(storage
            .delete_comment_by_id_and_update_last_updated(
                GateKey {
                    group,
                    service,
                    environment,
                },
                comment_id,
                clock.now(),
            )
            .await?
            .into())
    }
}

#[cfg(test)]
mod unit_tests {
    use chrono::DateTime;
    use similar_asserts::assert_eq;
    use std::collections::HashSet;

    use crate::clock::MockClock;
    use crate::storage;
    use crate::storage::MockStorage;
    use crate::types::{Gate, GateState};

    use super::*;

    #[tokio::test]
    async fn should_delete_gate_comment() {
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date")
            .to_utc();
        mock_clock.expect_now().return_const(now);

        mock_storage
            .expect_delete_comment_by_id_and_update_last_updated()
            .return_once(|key, _, now| {
                Ok(Gate {
                    key,
                    state: GateState::Open,
                    comments: HashSet::default(),
                    last_updated: now,
                    display_order: Option::default(),
                })
            });

        let left = UseCaseImpl {}
            .execute(
                Input {
                    group: "group".to_string(),
                    service: "service".to_string(),
                    environment: "environment".to_string(),
                    comment_id: "comment_id".to_string(),
                },
                &mock_storage,
                &mock_clock,
            )
            .await;

        let expected = models::Gate {
            group: "group".to_string(),
            service: "service".to_string(),
            environment: "environment".to_string(),
            state: models::GateState::Open,
            comments: vec![],
            last_updated: now.to_rfc3339(),
            display_order: Option::default(),
        };
        assert_eq!(left.unwrap(), expected);
    }

    #[tokio::test]
    async fn should_return_gate_or_comment_not_found_error_if_item_to_delete_is_not_found() {
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        mock_storage
            .expect_delete_comment_by_id_and_update_last_updated()
            .return_once(|_, _, _| {
                Err(storage::UpdateError::ItemToUpdateNotFound(
                    "ConditionalCheckFailedException".to_string(),
                ))
            });

        let left = UseCaseImpl {}
            .execute(
                Input {
                    group: "group".to_string(),
                    service: "service".to_string(),
                    environment: "environment".to_string(),
                    comment_id: "comment_id".to_string(),
                },
                &mock_storage,
                &mock_clock,
            )
            .await;

        assert!(left.is_err());
        assert_eq!(
            left.expect_err("expected error missing"),
            Error::GateOrCommentNotFound
        );
    }

    #[tokio::test]
    async fn should_return_storage_error() {
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        mock_storage
            .expect_delete_comment_by_id_and_update_last_updated()
            .return_once(|_, _, _| Err(storage::UpdateError::Other("some error".to_string())));

        let left = UseCaseImpl {}
            .execute(
                Input {
                    group: "group".to_string(),
                    service: "service".to_string(),
                    environment: "environment".to_string(),
                    comment_id: "comment_id".to_string(),
                },
                &mock_storage,
                &mock_clock,
            )
            .await;

        assert!(left.is_err());
        assert_eq!(
            left.expect_err("expected error missing"),
            Error::Internal("some error".to_string())
        );
    }
}
