use crate::clock::Clock;
use crate::date_time_switch::DateTimeSwitch;
use crate::storage::{Storage, UpdateError};
use crate::types::{GateKey, GateState};
use async_trait::async_trait;
use openapi::models;

#[derive(Debug)]
pub struct Input {
    pub group: String,
    pub service: String,
    pub environment: String,
    pub state: GateState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    GateNotFound,
    Internal(String),
    GateClosed(String),
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
        date_time_switch: &(dyn DateTimeSwitch + Send + Sync),
    ) -> Result<models::Gate, Error>;
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
            state,
        }: Input,
        storage: &(dyn Storage + Send + Sync),
        clock: &(dyn Clock + Send + Sync),
        date_time_switch: &(dyn DateTimeSwitch + Send + Sync),
    ) -> Result<models::Gate, Error> {
        if date_time_switch.is_closed(clock.now()) {
            return Err(Error::GateClosed(
                "Already after business hours - rejecting attempt to change state".to_owned(),
            ));
        }
        Ok(storage
            .update_state_and_last_updated(
                GateKey {
                    group,
                    service,
                    environment,
                },
                state,
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
    use crate::date_time_switch::MockDateTimeSwitch;
    use crate::storage::MockStorage;
    use crate::types::GateState::Open;
    use crate::types::{Gate, GateKey, GateState};

    use super::*;

    #[tokio::test]
    async fn should_open_gate() {
        // given
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let mut mock_date_time_switch = MockDateTimeSwitch::new();

        mock_date_time_switch.expect_is_closed().return_const(false);

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        let gate = some_gate("some group", "some service", "some environment");

        mock_storage
            .expect_update_state_and_last_updated()
            .return_once(move |key, state, last_updated| {
                Ok(Gate {
                    key,
                    state,
                    comments: HashSet::default(),
                    last_updated,
                    display_order: Option::default(),
                })
            });

        // when
        let gate_with_state = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                    state: Open,
                },
                &mock_storage,
                &mock_clock,
                &mock_date_time_switch,
            )
            .await;

        // then
        assert!(gate_with_state.is_ok());
        assert_eq!(
            gate_with_state.expect("here should be a gate for comparison"),
            models::Gate {
                group: gate.key.group,
                service: gate.key.service,
                environment: gate.key.environment,
                state: models::GateState::default(),
                comments: vec![],
                last_updated: now.to_utc().to_string(),
                display_order: Option::default(),
            }
        );
    }

    #[tokio::test]
    async fn should_time_close_gate() {
        // given
        let mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let mut mock_date_time_switch = MockDateTimeSwitch::new();
        let now = DateTime::parse_from_rfc3339("2023-05-28T22:10:57+02:00") //sunday
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);
        mock_date_time_switch.expect_is_closed().return_const(true);

        // when
        let gate_with_state = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                    state: GateState::default(),
                },
                &mock_storage,
                &mock_clock,
                &mock_date_time_switch,
            )
            .await;

        // then
        assert!(gate_with_state.is_err());
        assert_eq!(
            gate_with_state.expect_err("I did not expect a gate here!"),
            Error::GateClosed(
                "Already after business hours - rejecting attempt to change state".to_owned()
            )
        );
    }

    fn some_gate(group: &str, service: &str, environment: &str) -> Gate {
        Gate {
            key: GateKey {
                group: group.to_owned(),
                service: service.to_owned(),
                environment: environment.to_owned(),
            },
            state: Open,
            comments: HashSet::default(),
            last_updated: DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("failed creating date")
                .into(),
            display_order: Option::default(),
        }
    }
}
