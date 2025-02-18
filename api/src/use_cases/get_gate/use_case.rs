use async_trait::async_trait;
use openapi::models;

use crate::clock::Clock;
use crate::date_time_switch::DateTimeSwitch;
use crate::storage;
use crate::storage::Storage;
use crate::types::GateKey;

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

impl From<storage::FindError> for Error {
    fn from(value: storage::FindError) -> Self {
        match value {
            storage::FindError::ItemCouldNotBeDecoded(error) | storage::FindError::Other(error) => {
                Self::Internal(error)
            }
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
    ) -> Result<Option<models::Gate>, Error>;
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
        }: Input,
        storage: &(dyn Storage + Send + Sync),
        clock: &(dyn Clock + Send + Sync),
        date_time_switch: &(dyn DateTimeSwitch + Send + Sync),
    ) -> Result<Option<models::Gate>, Error> {
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
    use std::collections::HashSet;

    use chrono::{DateTime, Utc};
    use mockall::predicate::eq;
    use openapi::models;

    use crate::clock::MockClock;
    use crate::date_time_switch::MockDateTimeSwitch;
    use crate::storage;
    use crate::storage::MockStorage;
    use crate::types::{Gate, GateKey, GateState};
    use crate::use_cases::get_gate::use_case::{Error, Input, UseCase, UseCaseImpl};
    use similar_asserts::assert_eq;

    #[tokio::test]
    async fn should_get_gate_and_alter_with_date_time_switch() {
        // given
        let group = "some-group";
        let service = "some-service";
        let environment = "some-environment";

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
        let expected_gate = Some(models::Gate {
            group: group.to_string(),
            service: service.to_string(),
            environment: environment.to_string(),
            state: models::GateState::Closed,
            comments: vec![],
            last_updated: DateTime::<Utc>::default().to_string(),
            display_order: Some(f64::from(5)),
        });
        assert_eq!(left.expect("could not unwrap gate"), expected_gate);
    }

    #[tokio::test]
    async fn should_return_error_if_storage_fails_to_decode_item() {
        // given
        let group = "some-group";
        let service = "some-service";
        let environment = "some-environment";

        let mock_clock = MockClock::new();
        let mock_date_time_switch = MockDateTimeSwitch::new();

        let mut mock_storage = MockStorage::new();
        mock_storage
            .expect_find_one()
            .with(eq(GateKey {
                group: group.to_string(),
                service: service.to_string(),
                environment: environment.to_string(),
            }))
            .return_once(|_| {
                Err(storage::FindError::ItemCouldNotBeDecoded(
                    "some error".to_owned(),
                ))
            });

        // when
        let gate = UseCaseImpl {}
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

        // then
        assert!(gate.is_err());
        assert_eq!(
            gate.expect_err("unexpected groups"),
            Error::Internal("some error".to_owned())
        );
    }

    #[tokio::test]
    async fn should_return_storage_error() {
        // given
        let group = "some-group";
        let service = "some-service";
        let environment = "some-environment";

        let mock_clock = MockClock::new();
        let mock_date_time_switch = MockDateTimeSwitch::new();

        let mut mock_storage = MockStorage::new();
        mock_storage
            .expect_find_one()
            .with(eq(GateKey {
                group: group.to_string(),
                service: service.to_string(),
                environment: environment.to_string(),
            }))
            .return_once(|_| Err(storage::FindError::Other("some error".to_owned())));

        // when
        let gate = UseCaseImpl {}
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

        // then
        assert!(gate.is_err());
        assert_eq!(
            gate.expect_err("unexpected groups"),
            Error::Internal("some error".to_owned())
        );
    }
}
