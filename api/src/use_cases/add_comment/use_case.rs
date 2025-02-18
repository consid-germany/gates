use crate::clock::Clock;
use crate::id_provider::IdProvider;
use crate::storage::{Storage, UpdateError};
use crate::types::{Comment, GateKey};
use axum::async_trait;
use openapi::models;

#[derive(Debug)]
pub struct Input {
    pub group: String,
    pub service: String,
    pub environment: String,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    GateNotFound,
    InvalidInputMessage(String),
    Internal(String),
}

impl From<UpdateError> for Error {
    fn from(value: UpdateError) -> Self {
        match value {
            UpdateError::ItemToUpdateNotFound(_) => Self::GateNotFound,
            UpdateError::Other(error) => Self::Internal(error),
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
        id_provider: &(dyn IdProvider + Send + Sync),
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
            message,
        }: Input,
        storage: &(dyn Storage + Send + Sync),
        clock: &(dyn Clock + Send + Sync),
        id_provider: &(dyn IdProvider + Send + Sync),
    ) -> Result<models::Gate, Error> {
        let now = clock.now();
        match message.as_str().trim() {
            "" => Err(Error::InvalidInputMessage(
                "cannot add comment without message".to_owned(),
            )),
            _ => Ok(storage
                .update_comment_and_last_updated(
                    GateKey {
                        group,
                        service,
                        environment,
                    },
                    Comment {
                        id: id_provider.get(),
                        message: message.trim().to_owned(),
                        created: now,
                    },
                    now,
                )
                .await?
                .into()),
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use std::collections::HashSet;

    use chrono::DateTime;
    use itertools::concat;
    use similar_asserts::assert_eq;

    use crate::clock::MockClock;
    use crate::id_provider::MockIdProvider;
    use crate::storage::MockStorage;
    use crate::types::{Comment, Gate, GateKey, GateState};

    use super::*;

    #[tokio::test]
    async fn should_add_comment() {
        // given
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let mut mock_id_provider = MockIdProvider::new();

        mock_id_provider.expect_get().return_const("id");

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date")
            .to_utc();
        mock_clock.expect_now().return_const(now);

        let gate = some_gate("some group", "some service", "some environment");

        mock_storage
            .expect_update_comment_and_last_updated()
            .return_once(move |key, comment, last_updated| {
                Ok(Gate {
                    key,
                    state: gate.state.clone(),
                    comments: concat(vec![gate.comments, HashSet::from([comment])]),
                    last_updated,
                    display_order: Option::default(),
                })
            });

        // when
        let gate_with_comment = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                    message: "    some new comment".to_owned(),
                },
                &mock_storage,
                &mock_clock,
                &mock_id_provider,
            )
            .await;

        // then
        assert_eq!(gate_with_comment.is_ok(), true);
        assert_eq!(
            gate_with_comment.expect("no gate found"),
            models::Gate {
                group: gate.key.group,
                service: gate.key.service,
                environment: gate.key.environment,
                state: models::GateState::Open,
                comments: vec![
                    models::Comment {
                        id: "Comment1".to_owned(),
                        message: "Some comment message".to_owned(),
                        created: DateTime::parse_from_rfc3339("2021-04-12T22:10:57+02:00")
                            .expect("failed creating date")
                            .to_utc()
                            .to_rfc3339(),
                    },
                    models::Comment {
                        id: "Comment2".to_owned(),
                        message: "Some other comment message".to_owned(),
                        created: DateTime::parse_from_rfc3339("2022-04-12T22:10:57+02:00")
                            .expect("failed creating date")
                            .to_utc()
                            .to_rfc3339(),
                    },
                    models::Comment {
                        id: "id".to_owned(),
                        message: "some new comment".to_owned(),
                        created: now.to_rfc3339(),
                    },
                ],
                last_updated: now.to_rfc3339(),
                display_order: Option::default(),
            }
        );
    }

    #[tokio::test]
    async fn should_not_add_comment_if_message_empty() {
        // given
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let mut mock_id_provider = MockIdProvider::new();

        mock_id_provider.expect_get().return_const("id");

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        let gate = some_gate("some group", "some service", "some environment");

        mock_storage
            .expect_update_comment_and_last_updated()
            .return_once(move |key, comment, last_updated| {
                Ok(Gate {
                    key,
                    state: gate.state.clone(),
                    comments: concat(vec![gate.comments, HashSet::from([comment])]),
                    last_updated,
                    display_order: Option::default(),
                })
            });

        // when
        let gate_with_comment = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                    message: String::default(),
                },
                &mock_storage,
                &mock_clock,
                &mock_id_provider,
            )
            .await;

        // then
        assert_eq!(gate_with_comment.is_err(), true);
        assert_eq!(
            gate_with_comment.expect_err("unexpected gate found"),
            Error::InvalidInputMessage("cannot add comment without message".to_owned())
        );
    }

    #[tokio::test]
    async fn should_not_add_comment_if_message_is_whitespace() {
        // given
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let mut mock_id_provider = MockIdProvider::new();

        mock_id_provider.expect_get().return_const("id");

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        let gate = some_gate("some group", "some service", "some environment");

        mock_storage
            .expect_update_comment_and_last_updated()
            .return_once(move |key, comment, last_updated| {
                Ok(Gate {
                    key,
                    state: gate.state.clone(),
                    comments: concat(vec![gate.comments, HashSet::from([comment])]),
                    last_updated,
                    display_order: Option::default(),
                })
            });

        // when
        let gate_with_comment = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                    message: " ".to_owned(),
                },
                &mock_storage,
                &mock_clock,
                &mock_id_provider,
            )
            .await;

        // then
        assert!(gate_with_comment.is_err());
        assert_eq!(
            gate_with_comment.expect_err("unexpected gate found"),
            Error::InvalidInputMessage("cannot add comment without message".to_owned())
        );
    }

    #[tokio::test]
    async fn should_return_gate_not_found_error_if_storage_could_not_find_item_to_update() {
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let mut mock_id_provider = MockIdProvider::new();

        mock_id_provider.expect_get().return_const("id");

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        mock_storage
            .expect_update_comment_and_last_updated()
            .returning(move |_, _, _| {
                Err(UpdateError::ItemToUpdateNotFound(
                    "ConditionalCheckFailedException".to_owned(),
                ))
            });

        let left = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                    message: "some message".to_owned(),
                },
                &mock_storage,
                &mock_clock,
                &mock_id_provider,
            )
            .await;

        assert!(left.is_err());
        assert_eq!(left.expect_err("Error expected here"), Error::GateNotFound);
    }

    #[tokio::test]
    async fn should_return_storage_error() {
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let mut mock_id_provider = MockIdProvider::new();

        mock_id_provider.expect_get().return_const("id");

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        mock_storage
            .expect_update_comment_and_last_updated()
            .returning(move |_, _, _| Err(UpdateError::Other("some error".to_owned())));

        let left = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                    message: "some message".to_owned(),
                },
                &mock_storage,
                &mock_clock,
                &mock_id_provider,
            )
            .await;

        assert!(left.is_err());
        assert_eq!(
            left.expect_err("Error expected here"),
            Error::Internal("some error".to_owned())
        );
    }

    fn some_gate(group: &str, service: &str, environment: &str) -> Gate {
        Gate {
            key: GateKey {
                group: group.to_owned(),
                service: service.to_owned(),
                environment: environment.to_owned(),
            },
            state: GateState::Open,
            comments: HashSet::from([
                Comment {
                    id: "Comment1".to_owned(),
                    message: "Some comment message".to_owned(),
                    created: DateTime::parse_from_rfc3339("2021-04-12T22:10:57+02:00")
                        .expect("failed creating date")
                        .into(),
                },
                Comment {
                    id: "Comment2".to_owned(),
                    message: "Some other comment message".to_owned(),
                    created: DateTime::parse_from_rfc3339("2022-04-12T22:10:57+02:00")
                        .expect("failed creating date")
                        .into(),
                },
            ]),
            last_updated: DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("failed creating date")
                .into(),
            display_order: Option::default(),
        }
    }
}
