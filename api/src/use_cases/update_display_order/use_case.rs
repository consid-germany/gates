use crate::clock::Clock;
use crate::storage::{Storage, UpdateError};
use crate::types::{representation, GateKey};
use axum::async_trait;

#[derive(Debug)]
pub struct Input {
    pub group: String,
    pub service: String,
    pub environment: String,
    pub display_order: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    GateNotFound,
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
    ) -> Result<representation::Gate, Error>;
}

pub fn create() -> impl UseCase {
    UseCaseImpl {}
}

#[derive(Clone)]
struct UseCaseImpl {}

#[async_trait]
impl UseCase for UseCaseImpl {
    async fn execute(
        &self,
        Input {
            group,
            service,
            environment,
            display_order,
        }: Input,
        storage: &(dyn Storage + Send + Sync),
        clock: &(dyn Clock + Send + Sync),
    ) -> Result<representation::Gate, Error> {
        Ok(storage
            .update_display_order_and_last_updated(
                GateKey {
                    group,
                    service,
                    environment,
                },
                display_order,
                clock.now(),
            )
            .await?
            .into())
    }
}

#[cfg(test)]
mod unit_tests {
    use std::collections::HashSet;

    use chrono::DateTime;
    use similar_asserts::assert_eq;

    use crate::clock::MockClock;
    use crate::storage::MockStorage;
    use crate::types::{Gate, GateState};

    use super::*;

    #[tokio::test]
    async fn should_set_display_order() {
        // given
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        let gate = Gate {
            key: GateKey {
                group: "some group".to_owned(),
                service: "some service".to_owned(),
                environment: "some environment".to_owned(),
            },
            state: GateState::Open,
            comments: HashSet::default(),
            last_updated: DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("failed creating date")
                .into(),
            display_order: Option::default(),
        };

        mock_storage
            .expect_update_display_order_and_last_updated()
            .return_once(move |key, display_order, last_updated| {
                Ok(Gate {
                    key,
                    state: GateState::default(),
                    comments: HashSet::default(),
                    last_updated,
                    display_order: Some(display_order),
                })
            });

        // when
        let gate_with_state = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                    display_order: 1,
                },
                &mock_storage,
                &mock_clock,
            )
            .await;

        // then
        assert!(gate_with_state.is_ok());
        assert_eq!(
            gate_with_state.expect("here should be a gate for comparison"),
            representation::Gate {
                group: gate.key.group,
                service: gate.key.service,
                environment: gate.key.environment,
                state: GateState::default(),
                comments: vec![],
                last_updated: now.into(),
                display_order: Some(1),
            }
        );
    }
}
