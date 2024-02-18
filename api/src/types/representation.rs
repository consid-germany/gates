use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::types;
use crate::types::GateState;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApiInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub services: Vec<Service>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub environments: Vec<Environment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub gate: Gate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gate {
    pub group: String,
    pub service: String,
    pub environment: String,
    pub state: GateState,
    pub comments: Vec<Comment>,
    pub last_updated: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_order: Option<u32>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub message: String,
    pub created: DateTime<Utc>,
}

impl From<types::Gate> for Gate {
    fn from(value: types::Gate) -> Self {
        Self {
            group: value.key.group,
            service: value.key.service,
            environment: value.key.environment,
            state: value.state,
            comments: value
                .comments
                .into_iter()
                .map_into::<Comment>()
                .sorted_by_key(|comment| comment.created)
                .collect(),
            last_updated: value.last_updated,
            display_order: value.display_order,
        }
    }
}

impl From<types::Comment> for Comment {
    fn from(value: types::Comment) -> Self {
        Self {
            id: value.id,
            message: value.message,
            created: value.created,
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::types;
    use crate::types::representation::{Comment, Gate};
    use chrono::DateTime;
    use std::collections::HashSet;

    #[test]
    fn should_convert_domain_gate_to_representation() {
        let gate = some_gate("com.consid", "stargate", "live");
        let actual: Gate = gate.into();
        let expected = Gate {
            group: "com.consid".to_string(),
            service: "stargate".to_string(),
            environment: "live".to_string(),
            state: types::GateState::Open,
            comments: vec![
                Comment {
                    id: "Comment1".into(),
                    message: "Some comment message".into(),
                    created: DateTime::parse_from_rfc3339("2021-04-12T20:10:57Z")
                        .expect("can not convert date")
                        .into(),
                },
                Comment {
                    id: "Comment2".into(),
                    message: "Some other comment message".into(),
                    created: DateTime::parse_from_rfc3339("2022-04-12T20:10:57Z")
                        .expect("can not convert date")
                        .into(),
                },
            ],
            last_updated: DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("can not convert date")
                .into(),
            display_order: Option::default(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_convert_comment() {
        let actual: Comment = types::Comment {
            id: "1234".to_string(),
            message: "Gate closed because of ticket #63468 ".to_string(),
            created: DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("can not convert date")
                .into(),
        }
        .into();

        let expected = Comment {
            id: "1234".to_string(),
            message: "Gate closed because of ticket #63468 ".to_string(),
            created: DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("can not convert date")
                .into(),
        };
        assert_eq!(actual, expected);
    }

    fn some_gate(group: &str, service: &str, environment: &str) -> types::Gate {
        types::Gate {
            key: types::GateKey {
                group: group.to_owned(),
                service: service.to_owned(),
                environment: environment.to_owned(),
            },
            state: types::GateState::Open,
            comments: HashSet::from([
                types::Comment {
                    id: "Comment1".to_owned(),
                    message: "Some comment message".to_owned(),
                    created: DateTime::parse_from_rfc3339("2021-04-12T22:10:57+02:00")
                        .expect("failed creating date")
                        .into(),
                },
                types::Comment {
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
