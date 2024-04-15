use crate::storage;
use axum::async_trait;
use itertools::Itertools;
use openapi::models;

use crate::clock::Clock;
use crate::date_time_switch::DateTimeSwitch;
use crate::storage::Storage;
use crate::types::Gate;

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
        storage: &(dyn Storage + Send + Sync),
        clock: &(dyn Clock + Send + Sync),
        date_time_switch: &(dyn DateTimeSwitch + Send + Sync),
    ) -> Result<Vec<models::Group>, Error>;
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
        storage: &(dyn Storage + Send + Sync),
        clock: &(dyn Clock + Send + Sync),
        date_time_switch: &(dyn DateTimeSwitch + Send + Sync),
    ) -> Result<Vec<models::Group>, Error> {
        Ok(ordered_by_group(
            storage
                .find_all()
                .await?
                .into_iter()
                .map(|gate| date_time_switch.close_if_time(clock.now(), gate))
                .collect(),
        ))
    }
}

fn ordered_by_group(gates: Vec<Gate>) -> Vec<models::Group> {
    let mut groups: Vec<models::Group> = Vec::new();
    let group_to_items = gates
        .into_iter()
        .sorted_by_key(|item| item.key.group.clone())
        .group_by(|item| item.key.group.clone());

    for (group, items) in &group_to_items {
        let service_to_items = items
            .into_iter()
            .sorted_by_key(|item| item.key.service.clone())
            .group_by(|item| item.key.service.clone());

        let mut services: Vec<models::Service> = Vec::new();
        for (service, items) in &service_to_items {
            let mut environments: Vec<models::Environment> = Vec::new();
            for item in items {
                environments.push(models::Environment {
                    name: item.key.environment.clone(),
                    gate: Box::new(item.into()),
                });
            }
            environments.sort_by(|a, b| {
                a.gate
                    .display_order
                    .partial_cmp(&b.gate.display_order)
                    .unwrap()
            });
            services.push(models::Service {
                name: service.clone(),
                environments,
            });
        }
        groups.push(models::Group {
            name: group.clone(),
            services,
        });
    }
    groups
}

#[cfg(test)]
mod unit_tests {
    use std::collections::HashSet;

    use crate::storage;
    use chrono::DateTime;
    use mockall::predicate::eq;
    use similar_asserts::assert_eq;

    use crate::clock::MockClock;
    use crate::date_time_switch::MockDateTimeSwitch;
    use crate::storage::MockStorage;
    use crate::types::{Comment, GateKey, GateState};

    use super::*;

    #[tokio::test]
    async fn should_list_gates_of_same_group() {
        // given
        let mut mock_clock = MockClock::new();
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        let mut mock_date_time_switch = MockDateTimeSwitch::new();
        mock_date_time_switch
            .expect_close_if_time()
            .with(
                eq::<chrono::DateTime<chrono::Utc>>(now.into()),
                eq(some_gate(
                    "some group",
                    "1 some service",
                    "some environment",
                )),
            )
            .return_once(|_, gate| Gate {
                key: gate.key,
                state: GateState::Closed,
                comments: gate.comments,
                last_updated: gate.last_updated,
                display_order: gate.display_order,
            });

        mock_date_time_switch
            .expect_close_if_time()
            .with(
                eq::<chrono::DateTime<chrono::Utc>>(now.into()),
                eq(some_gate(
                    "some group",
                    "1 some service",
                    "some other environment",
                )),
            )
            .return_once(|_, gate| gate);

        mock_date_time_switch
            .expect_close_if_time()
            .with(
                eq::<chrono::DateTime<chrono::Utc>>(now.into()),
                eq(some_gate(
                    "some group",
                    "2 some other service",
                    "some environment",
                )),
            )
            .return_once(|_, gate| gate);

        let mut mock_storage = MockStorage::new();

        let gate1 = some_gate("some group", "1 some service", "some environment");
        let gate2 = some_gate("some group", "1 some service", "some other environment");
        let gate3 = some_gate("some group", "2 some other service", "some environment");

        mock_storage
            .expect_find_all()
            .return_once(|| Ok(vec![gate1, gate2, gate3]));

        // when
        let groups = UseCaseImpl {}
            .execute(&mock_storage, &mock_clock, &mock_date_time_switch)
            .await;

        // then
        let gate1 = some_gate("some group", "1 some service", "some environment");
        let gate2 = some_gate("some group", "1 some service", "some other environment");
        let gate3 = some_gate("some group", "2 some other service", "some environment");

        assert_eq!(groups.is_ok(), true);
        assert_eq!(
            groups.expect("no groups found"),
            vec![models::Group {
                name: "some group".to_owned(),
                services: vec![
                    models::Service {
                        name: "1 some service".to_owned(),
                        environments: vec![
                            models::Environment {
                                name: "some environment".to_owned(),
                                gate: Box::new(
                                    Gate {
                                        key: gate1.key,
                                        state: GateState::Closed,
                                        comments: gate1.comments,
                                        last_updated: gate1.last_updated,
                                        display_order: gate1.display_order,
                                    }
                                    .into()
                                )
                            },
                            models::Environment {
                                name: "some other environment".to_owned(),
                                gate: Box::new(gate2.into()),
                            },
                        ],
                    },
                    models::Service {
                        name: "2 some other service".to_owned(),
                        environments: vec![models::Environment {
                            name: "some environment".to_owned(),
                            gate: Box::new(gate3.into()),
                        },],
                    },
                ],
            }],
        );
    }

    #[tokio::test]
    async fn should_list_gates_of_different_groups() {
        // given
        let mut mock_clock = MockClock::new();
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        let mut mock_date_time_switch = MockDateTimeSwitch::new();
        mock_date_time_switch
            .expect_close_if_time()
            .with(
                eq::<DateTime<chrono::Utc>>(now.into()),
                eq(some_gate("some group", "some service", "some environment")),
            )
            .return_once(|_, gate| gate);

        mock_date_time_switch
            .expect_close_if_time()
            .with(
                eq::<DateTime<chrono::Utc>>(now.into()),
                eq(some_gate(
                    "some other group",
                    "some other service",
                    "some other environment",
                )),
            )
            .return_once(|_, gate| gate);

        let mut mock_storage = MockStorage::new();
        let gate1 = some_gate("some group", "some service", "some environment");
        let gate2 = some_gate(
            "some other group",
            "some other service",
            "some other environment",
        );

        mock_storage
            .expect_find_all()
            .return_once(|| Ok(vec![gate1, gate2]));

        // when
        let groups = UseCaseImpl {}
            .execute(&mock_storage, &mock_clock, &mock_date_time_switch)
            .await;

        // then
        let gate1 = some_gate("some group", "some service", "some environment");
        let gate2 = some_gate(
            "some other group",
            "some other service",
            "some other environment",
        );

        assert_eq!(groups.is_ok(), true);
        assert_eq!(
            groups.expect("no groups found"),
            vec![
                models::Group {
                    name: "some group".to_owned(),
                    services: vec![models::Service {
                        name: "some service".to_owned(),
                        environments: vec![models::Environment {
                            name: "some environment".to_owned(),
                            gate: Box::new(gate1.into()),
                        },],
                    },],
                },
                models::Group {
                    name: "some other group".to_owned(),
                    services: vec![models::Service {
                        name: "some other service".to_owned(),
                        environments: vec![models::Environment {
                            name: "some other environment".to_owned(),
                            gate: Box::new(gate2.into()),
                        },],
                    },],
                },
            ],
        );
    }

    #[tokio::test]
    async fn should_list_gates_and_alter_with_date_time_switch() {
        // given
        let mut mock_clock = MockClock::new();
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        let mut mock_date_time_switch = MockDateTimeSwitch::new();
        mock_date_time_switch
            .expect_close_if_time()
            .with(
                eq::<DateTime<chrono::Utc>>(now.into()),
                eq(some_gate("some group", "some service", "some environment")),
            )
            .return_once(|_, gate| Gate {
                key: gate.key,
                state: GateState::Closed,
                comments: gate.comments,
                last_updated: gate.last_updated,
                display_order: gate.display_order,
            });

        let mut mock_storage = MockStorage::new();
        let gate1 = some_gate("some group", "some service", "some environment");

        mock_storage
            .expect_find_all()
            .return_once(|| Ok(vec![gate1]));

        // when
        let groups = UseCaseImpl {}
            .execute(&mock_storage, &mock_clock, &mock_date_time_switch)
            .await;

        // then
        let gate = some_gate("some group", "some service", "some environment");

        assert_eq!(groups.is_ok(), true);
        let gate_representation = models::Gate::from(gate);
        assert_eq!(
            groups.expect("no groups found"),
            vec![models::Group {
                name: "some group".to_owned(),
                services: vec![models::Service {
                    name: "some service".to_owned(),
                    environments: vec![models::Environment {
                        name: "some environment".to_owned(),
                        gate: Box::new(models::Gate {
                            group: gate_representation.group,
                            service: gate_representation.service,
                            environment: gate_representation.environment,
                            state: models::GateState::Closed,
                            comments: gate_representation.comments,
                            last_updated: gate_representation.last_updated,
                            display_order: gate_representation.display_order
                        })
                    },],
                },],
            },],
        );
    }

    #[tokio::test]
    async fn should_return_error_if_storage_fails_to_decode() {
        // given
        let mock_clock = MockClock::new();
        let mock_date_time_switch = MockDateTimeSwitch::new();
        let mut mock_storage = MockStorage::new();

        mock_storage.expect_find_all().return_once(|| {
            Err(storage::FindError::ItemCouldNotBeDecoded(
                "some error".to_owned(),
            ))
        });

        // when
        let groups = UseCaseImpl {}
            .execute(&mock_storage, &mock_clock, &mock_date_time_switch)
            .await;

        // then
        assert_eq!(groups.is_err(), true);
        assert_eq!(
            groups.expect_err("unexpected groups"),
            Error::Internal("some error".to_owned())
        );
    }

    #[tokio::test]
    async fn should_return_error_if_storage_fails() {
        // given
        let mock_clock = MockClock::new();
        let mock_date_time_switch = MockDateTimeSwitch::new();
        let mut mock_storage = MockStorage::new();

        mock_storage
            .expect_find_all()
            .return_once(|| Err(storage::FindError::Other("some error".to_owned())));

        // when
        let groups = UseCaseImpl {}
            .execute(&mock_storage, &mock_clock, &mock_date_time_switch)
            .await;

        // then
        assert_eq!(groups.is_err(), true);
        assert_eq!(
            groups.expect_err("unexpected groups"),
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
