use axum::async_trait;
use std::collections::HashSet;

use crate::clock::Clock;
use crate::storage;
use crate::storage::Storage;
use crate::types::{representation, Gate, GateKey, GateState};

#[derive(Debug)]
pub struct Input {
    pub group: String,
    pub service: String,
    pub environment: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInput(String),
    Internal(String),
}

impl From<storage::Error> for Error {
    fn from(value: storage::Error) -> Self {
        Self::Internal(value.message)
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UseCase {
    async fn execute<'a>(
        &self,
        input: Input,
        storage: &(dyn Storage + Send + Sync + 'a),
        clock: &(dyn Clock + Send + Sync + 'a),
    ) -> Result<representation::Gate, Error>;
}

pub fn create() -> impl UseCase {
    UseCaseImpl {}
}

#[derive(Clone)]
struct UseCaseImpl {}

#[async_trait]
impl UseCase for UseCaseImpl {
    async fn execute<'a>(
        &self,
        Input {
            group,
            service,
            environment,
        }: Input,
        storage: &(dyn Storage + Send + Sync + 'a),
        clock: &(dyn Clock + Send + Sync + 'a),
    ) -> Result<representation::Gate, Error> {
        if group.is_empty() || service.is_empty() || environment.is_empty() {
            return Err(Error::InvalidInput(
                "group, service and environment must not be empty".to_owned(),
            ));
        }

        let gate = Gate {
            key: GateKey {
                group,
                service,
                environment,
            },
            state: GateState::default(),
            comments: HashSet::default(),
            last_updated: clock.now(),
            display_order: Option::default(),
        };

        storage.save(&gate).await?;

        Ok(gate.into())
    }
}

#[cfg(test)]
mod unit_tests {
    use std::collections::HashSet;

    use chrono::DateTime;
    use mockall::predicate::eq;

    use crate::clock::MockClock;
    use crate::storage;
    use crate::storage::MockStorage;
    use crate::types::{Gate, GateKey, GateState};
    use crate::use_cases::create_gate::use_case::{Error, Input, UseCase, UseCaseImpl};

    #[tokio::test]
    async fn should_create_gate() {
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        let gate1 = Gate {
            key: GateKey {
                group: "some group".to_owned(),
                service: "some service".to_owned(),
                environment: "some environment".to_owned(),
            },
            state: GateState::Closed,
            comments: HashSet::default(),
            last_updated: DateTime::from(now),
            display_order: Option::default(),
        };

        mock_storage
            .expect_save()
            .with(eq(gate1))
            .returning(move |_| Ok(()));

        UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                },
                &mock_storage,
                &mock_clock,
            )
            .await
            .expect("There is no error expected here!");
    }

    #[tokio::test]
    async fn should_fail_when_group_is_empty() {
        let mock_storage = MockStorage::new();
        let mock_clock = MockClock::new();

        let result = UseCaseImpl {}
            .execute(
                Input {
                    group: String::default(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                },
                &mock_storage,
                &mock_clock,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Error expected here"),
            Error::InvalidInput("group, service and environment must not be empty".to_owned())
        );
    }

    #[tokio::test]
    async fn should_fail_when_service_is_empty() {
        let mock_storage = MockStorage::new();
        let mock_clock = MockClock::new();

        let result = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: String::default(),
                    environment: "some environment".to_owned(),
                },
                &mock_storage,
                &mock_clock,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Error expected here"),
            Error::InvalidInput("group, service and environment must not be empty".to_owned())
        );
    }

    #[tokio::test]
    async fn should_fail_when_environment_is_empty() {
        let mock_storage = MockStorage::new();
        let mock_clock = MockClock::new();

        let result = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: String::default(),
                },
                &mock_storage,
                &mock_clock,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Error expected here"),
            Error::InvalidInput("group, service and environment must not be empty".to_owned())
        );
    }

    #[tokio::test]
    async fn should_return_storage_error() {
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        let gate1 = Gate {
            key: GateKey {
                group: "some group".to_owned(),
                service: "some service".to_owned(),
                environment: "some environment".to_owned(),
            },
            state: GateState::Closed,
            comments: HashSet::default(),
            last_updated: DateTime::from(now),
            display_order: Option::default(),
        };

        mock_storage
            .expect_save()
            .with(eq(gate1))
            .returning(move |_| {
                Err(storage::Error {
                    message: "can´t save".to_string(),
                })
            });

        let left = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                },
                &mock_storage,
                &mock_clock,
            )
            .await;

        assert!(left.is_err());
        assert_eq!(
            left.expect_err("Error expected here"),
            Error::Internal("can´t save".to_owned())
        );
    }
}
