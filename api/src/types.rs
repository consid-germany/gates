use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod app_state;
pub mod representation;
pub mod use_cases;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtcTime {
    hour: u8,
    minute: u8,
    second: u8,
}

impl UtcTime {
    fn _new(hour: u8, minute: u8, second: u8) -> Result<Self, &'static str> {
        if (0..=23).contains(&hour) && (0..=59).contains(&minute) && (0..=59).contains(&second) {
            Ok(Self {
                hour,
                minute,
                second,
            })
        } else {
            Err("Invalid time format: must have 0 <= hour <= 23, 0 <= minute <= 59 and 0 <= second <= 59")
        }
    }

    fn _to_string(&self) -> String {
        format!("{:02}:{:02}:{:02}Z", self.hour, self.minute, self.second)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveHours {
    pub start: UtcTime,
    pub end: UtcTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveHoursPerWeek {
    pub monday: Option<ActiveHours>,
    pub tuesday: Option<ActiveHours>,
    pub wednesday: Option<ActiveHours>,
    pub thursday: Option<ActiveHours>,
    pub friday: Option<ActiveHours>,
    pub saturday: Option<ActiveHours>,
    pub sunday: Option<ActiveHours>,
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
