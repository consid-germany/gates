use crate::clock::Clock;
use crate::date_time_switch::DateTimeSwitch;
use crate::storage;
use axum::async_trait;

use crate::storage::Storage;
use crate::types::{representation, GateKey};

#[derive(Debug)]
pub struct Input {
    pub group: String,
    pub service: String,
    pub environment: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
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
    async fn execute<'required_for_mocking>(
        &self,
        input: Input,
        storage: &(dyn Storage + Send + Sync + 'required_for_mocking),
        clock: &(dyn Clock + Send + Sync + 'required_for_mocking),
        date_time_switch: &(dyn DateTimeSwitch + Send + Sync + 'required_for_mocking),
    ) -> Result<Option<representation::Gate>, Error>;
}

pub fn create() -> impl UseCase {
    UseCaseImpl {}
}

#[derive(Clone)]
struct UseCaseImpl {}

#[async_trait]
impl UseCase for UseCaseImpl {
    async fn execute<'required_for_mocking>(
        &self,
        Input {
            group,
            service,
            environment,
        }: Input,
        storage: &(dyn Storage + Send + Sync + 'required_for_mocking),
        clock: &(dyn Clock + Send + Sync + 'required_for_mocking),
        date_time_switch: &(dyn DateTimeSwitch + Send + Sync + 'required_for_mocking),
    ) -> Result<Option<representation::Gate>, Error> {
        let gate = storage
            .find_one(GateKey {
                group,
                service,
                environment,
            })
            .await?
            .map(|gate| date_time_switch.close_if_time(clock.now(), gate))
            .map(Into::into);
        Ok(gate)
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::clock::MockClock;
    use crate::date_time_switch::MockDateTimeSwitch;
    use crate::storage::MockStorage;
    use crate::types::{representation, Gate, GateKey, GateState};
    use crate::use_cases::get_gate::use_case::{Input, UseCase, UseCaseImpl};
    use chrono::{DateTime, Utc};
    use mockall::predicate::eq;
    use std::collections::HashSet;

    #[tokio::test]
    async fn should_get_gate() {
        let group = "com.consid";
        let service = "stargate";
        let environment = "live";

        let mut mock_clock = MockClock::new();
        let now: DateTime<Utc> = DateTime::from(
            DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("failed to parse date"),
        );
        mock_clock.expect_now().return_const(now);

        let mut mock_date_time_switch = MockDateTimeSwitch::new();
        mock_date_time_switch
            .expect_close_if_time()
            .with(
                eq(now),
                eq(Gate {
                    key: GateKey {
                        group: group.to_string(),
                        service: service.to_string(),
                        environment: environment.to_string(),
                    },
                    state: GateState::Open,
                    comments: HashSet::default(),
                    last_updated: DateTime::default(),

                    display_order: Some(5),
                }),
            )
            .return_once(move |_, _| Gate {
                key: GateKey {
                    group: group.to_string(),
                    service: service.to_string(),
                    environment: environment.to_string(),
                },
                state: GateState::Closed,
                comments: HashSet::default(),
                last_updated: DateTime::default(),
                display_order: Some(5),
            });
        let mut mock_storage = MockStorage::new();
        mock_storage
            .expect_find_one()
            .with(eq(GateKey {
                group: group.to_string(),
                service: service.to_string(),
                environment: environment.to_string(),
            }))
            .return_once(move |_| {
                Ok(Some(Gate {
                    key: GateKey {
                        group: group.to_string(),
                        service: service.to_string(),
                        environment: environment.to_string(),
                    },
                    state: GateState::Open,
                    comments: HashSet::default(),
                    last_updated: DateTime::default(),
                    display_order: Some(5),
                }))
            });
        let left = UseCaseImpl {}
            .execute(
                Input {
                    group: group.to_string(),
                    service: service.to_string(),
                    environment: environment.to_string(),
                },
                &mock_storage,
                &mock_clock,
                &mock_date_time_switch,
            )
            .await;
        assert!(left.is_ok());
        let expected_gate = Some(representation::Gate {
            group: group.to_string(),
            service: service.to_string(),
            environment: environment.to_string(),
            state: GateState::Closed,
            comments: vec![],
            last_updated: DateTime::default(),
            display_order: Some(5),
        });
        assert_eq!(left.expect("could not unwrap gate"), expected_gate);
    }
}
