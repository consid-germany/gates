use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod app_state;
pub mod representation;
pub mod use_cases;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActiveHours {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActiveHoursPerWeek {
    pub monday: Option<ActiveHours>,
    pub tuesday: Option<ActiveHours>,
    pub wednesday: Option<ActiveHours>,
    pub thursday: Option<ActiveHours>,
    pub friday: Option<ActiveHours>,
    pub saturday: Option<ActiveHours>,
    pub sunday: Option<ActiveHours>,
}

impl ActiveHoursPerWeek {
    pub fn default() -> Self {
        Self {
            monday: Some(ActiveHours {
                start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            }),
            tuesday: Some(crate::types::ActiveHours {
                start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            }),
            wednesday: Some(crate::types::ActiveHours {
                start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            }),
            thursday: Some(crate::types::ActiveHours {
                start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            }),
            friday: Some(crate::types::ActiveHours {
                start: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            }),
            saturday: None,
            sunday: None,
        }
    }
}

#[derive(Debug, serde::Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub system_time: DateTime<Utc>,
    pub active_hours_per_week: ActiveHoursPerWeek,
}

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
