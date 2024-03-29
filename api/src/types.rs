use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod representation;
pub mod use_cases;
pub mod app_state;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateKey {
    pub group: String,
    pub service: String,
    pub environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub key: GateKey,
    pub state: GateState,
    pub comments: HashSet<Comment>,
    pub last_updated: DateTime<Utc>,
    pub display_order: Option<u32>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Comment {
    pub id: String,
    pub message: String,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum GateState {
    Open,
    #[default]
    Closed,
}

impl TryFrom<String> for GateState {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&format!("\"{value}\"")).map_or_else(
            |serde_error| {
                Err(format!(
                    "can not convert {value} to GateState: {serde_error}"
                ))
            },
            Ok,
        )
    }
}

impl TryFrom<GateState> for String {
    type Error = Self;

    fn try_from(value: GateState) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
            .map(|gate_state| gate_state.replace('\"', ""))
            .map_or_else(
                |serde_error| {
                    Err(format!(
                        "can not convert from GateState to string: {serde_error}"
                    ))
                },
                Ok,
            )
    }
}
